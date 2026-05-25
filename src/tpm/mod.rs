pub mod display;
pub mod types;

use uefi::prelude::*;
use uefi::proto::tcg::v2::Tcg;
use uefi::table::boot::ScopedProtocol;

use self::types::*;

/// Collect TPM information via TCG2 protocol
pub fn collect(boot_services: &BootServices) -> Result<TpmInfo, &'static str> {
    // Try to locate TCG2 protocol
    let handle = boot_services
        .get_handle_for_protocol::<Tcg>()
        .map_err(|_| "No TCG2 protocol handle found")?;

    let mut tcg: ScopedProtocol<Tcg> = boot_services
        .open_protocol_exclusive(handle)
        .map_err(|_| "Failed to open TCG2 protocol")?;

    let mut info = TpmInfo::default();

    // Get capability to check if TPM is present
    match tcg.get_capability() {
        Ok(cap) => {
            info.present = cap.tpm_present();
            info.pcr_count = 24; // Standard TPM2 has 24 PCRs
        }
        Err(_) => {
            info.present = false;
            return Ok(info);
        }
    }

    // Get active PCR banks
    match tcg.get_active_pcr_banks() {
        Ok(banks) => {
            info.active_pcr_banks = banks.bits();
        }
        Err(_) => {}
    }

    // Send raw TPM2_GetCapability command to get manufacturer and firmware version
    if let Some((manufacturer, fw_version)) = get_tpm_properties(&mut tcg) {
        info.manufacturer = manufacturer;
        info.firmware_version = fw_version;
    }

    Ok(info)
}

/// Send TPM2_GetCapability command to read TPM properties
fn get_tpm_properties(tcg: &mut Tcg) -> Option<(u32, u64)> {
    // TPM2_GetCapability command structure (big-endian)
    // Tag: 0x8001 (TPM_ST_NO_SESSIONS)
    // CommandSize: varies
    // CommandCode: 0x0000017A (TPM2_GetCapability)
    // Capability: 0x00000006 (TPM_CAP_TPM_PROPERTIES)
    // Property: 0x00000100 (TPM2_PT_MANUFACTURER)
    // PropertyCount: 3

    let mut cmd = [0u8; 22];
    // Tag
    cmd[0] = 0x80;
    cmd[1] = 0x01;
    // CommandSize (22 bytes)
    cmd[2] = 0x00;
    cmd[3] = 0x00;
    cmd[4] = 0x00;
    cmd[5] = 0x16;
    // CommandCode
    cmd[6] = 0x00;
    cmd[7] = 0x00;
    cmd[8] = 0x01;
    cmd[9] = 0x7A;
    // Capability (TPM_CAP_TPM_PROPERTIES)
    cmd[10] = 0x00;
    cmd[11] = 0x00;
    cmd[12] = 0x00;
    cmd[13] = 0x06;
    // Property (TPM2_PT_MANUFACTURER)
    cmd[14] = 0x00;
    cmd[15] = 0x00;
    cmd[16] = 0x01;
    cmd[17] = 0x00;
    // PropertyCount (3: manufacturer + fw1 + fw2)
    cmd[18] = 0x00;
    cmd[19] = 0x00;
    cmd[20] = 0x00;
    cmd[21] = 0x03;

    let mut response = [0u8; 256];
    match tcg.submit_command(&cmd, &mut response) {
        Ok(_) => {
            // Parse response
            // ResponseCode at offset 6-9
            let rc = u32::from_be_bytes([response[6], response[7], response[8], response[9]]);
            if rc != TPM2_RC_SUCCESS {
                return None;
            }

            // Parse capability data
            // MoreData (1 byte) at offset 10
            // Capability (4 bytes) at offset 11-14
            // TPMU_CAPABILITIES starts at offset 15
            // For TPM_CAP_TPM_PROPERTIES: TPML_TAGGED_PROPERTY
            //   count (4 bytes) at offset 15-18
            //   TPMT_TAGGED_PROPERTY[] starts at offset 19
            //     property (4 bytes) + value (4 bytes) each

            if response.len() < 27 {
                return None;
            }

            let count = u32::from_be_bytes([response[15], response[16], response[17], response[18]]);
            if count < 1 {
                return None;
            }

            // First property: manufacturer
            let manufacturer = u32::from_be_bytes([response[23], response[24], response[25], response[26]]);

            // Firmware version (if available)
            let mut fw_version: u64 = 0;
            if count >= 2 && response.len() >= 35 {
                let fw1 = u32::from_be_bytes([response[31], response[32], response[33], response[34]]);
                fw_version = (fw1 as u64) << 32;
            }
            if count >= 3 && response.len() >= 43 {
                let fw2 = u32::from_be_bytes([response[39], response[40], response[41], response[42]]);
                fw_version |= fw2 as u64;
            }

            Some((manufacturer, fw_version))
        }
        Err(_) => None,
    }
}

/// Read PCR values (SHA256 bank) for PCRs 0-7
pub fn read_pcrs(boot_services: &BootServices) -> Result<[[u8; 32]; 8], &'static str> {
    let handle = boot_services
        .get_handle_for_protocol::<Tcg>()
        .map_err(|_| "No TCG2 protocol handle found")?;

    let mut tcg: ScopedProtocol<Tcg> = boot_services
        .open_protocol_exclusive(handle)
        .map_err(|_| "Failed to open TCG2 protocol")?;

    let mut pcrs = [[0u8; 32]; 8];

    // TPM2_PCR_Read command (20 bytes total)
    // Tag: 0x8001 (2)
    // CommandSize: 0x00000014 (4) = 20 bytes
    // CommandCode: 0x0000017E (4)
    // TPML_PCR_SELECTION:
    //   count: 0x00000001 (4)
    //   TPMS_PCR_SELECTION:
    //     hashAlg: 0x000B (2) = SHA256
    //     sizeofSelect: 0x03 (1)
    //     pcrSelect: 0xFF 0x00 0x00 (3) = PCRs 0-7

    let cmd: [u8; 20] = [
        0x80, 0x01,                         // Tag: TPM_ST_NO_SESSIONS
        0x00, 0x00, 0x00, 0x14,             // CommandSize: 20
        0x00, 0x00, 0x01, 0x7E,             // CommandCode: TPM2_PCR_Read
        0x00, 0x00, 0x00, 0x01,             // count: 1
        0x00, 0x0B,                         // hashAlg: SHA256
        0x03,                               // sizeofSelect: 3
        0xFF, 0x00, 0x00,                   // pcrSelect: PCRs 0-7
    ];

    let mut response = [0u8; 512];
    match tcg.submit_command(&cmd, &mut response) {
        Ok(_) => {
            // Parse response
            let rc = u32::from_be_bytes([response[6], response[7], response[8], response[9]]);
            if rc != TPM2_RC_SUCCESS {
                return Err("TPM2_PCR_Read failed");
            }

            // Response structure:
            // Tag (2) + ResponseSize (4) + ResponseCode (4) = 10 bytes header
            // pcrUpdateCounter (4)
            // TPML_PCR_SELECTION: count(4) + selections(6) = 10 bytes
            // TPML_DIGEST: count(4) + digests[]
            // Each digest: TPM2B_DIGEST { size(2) + buffer(32) }

            // Header (10) + pcrUpdateCounter (4) + TPML_PCR_SELECTION (10) = 24
            // Then TPML_DIGEST count at offset 24
            let digest_count = u32::from_be_bytes([response[24], response[25], response[26], response[27]]);
            if digest_count < 8 {
                return Err("Not enough PCR digests in response");
            }

            // Each digest is TPM2B_DIGEST: size(2) + buffer(32) = 34 bytes
            let mut offset = 28;
            for i in 0..8 {
                if offset + 34 > response.len() {
                    break;
                }
                // Skip size field (2 bytes)
                offset += 2;
                // Copy 32 bytes of digest
                pcrs[i].copy_from_slice(&response[offset..offset + 32]);
                offset += 32;
            }

            Ok(pcrs)
        }
        Err(_) => Err("Failed to submit TPM2_PCR_Read command"),
    }
}
