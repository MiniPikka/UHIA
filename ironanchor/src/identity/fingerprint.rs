use crate::smbios::types::SmbiosData;

/// Raw identity material before hashing
pub struct IdentityMaterial {
    pub bios_uuid: [u8; 16],
    pub board_serial: [u8; 64],
    pub cpu_signature: u32,
    pub bios_vendor: [u8; 64],
    pub bios_version: [u8; 48],
    pub device_guid: [u8; 16], // From EFI Variable (persistent)
}

impl IdentityMaterial {
    pub fn from_smbios(smbios: &SmbiosData, device_guid: [u8; 16]) -> Self {
        let mut bios_uuid = [0u8; 16];
        bios_uuid.copy_from_slice(&smbios.system.uuid);

        let mut board_serial = [0u8; 64];
        copy_str_to_bytes(&smbios.baseboard.serial, &mut board_serial);

        let mut bios_vendor = [0u8; 64];
        copy_str_to_bytes(&smbios.bios.vendor, &mut bios_vendor);

        let mut bios_version = [0u8; 48];
        copy_str_to_bytes(&smbios.bios.version, &mut bios_version);

        Self {
            bios_uuid,
            board_serial,
            cpu_signature: smbios.processor.signature,
            bios_vendor,
            bios_version,
            device_guid,
        }
    }

    /// Serialize all material into a contiguous byte buffer for hashing
    pub fn to_bytes(&self) -> [u8; 212] {
        let mut buf = [0u8; 212];
        buf[0..16].copy_from_slice(&self.bios_uuid);
        buf[16..80].copy_from_slice(&self.board_serial);
        buf[80..84].copy_from_slice(&self.cpu_signature.to_le_bytes());
        buf[84..148].copy_from_slice(&self.bios_vendor);
        buf[148..196].copy_from_slice(&self.bios_version);
        buf[196..212].copy_from_slice(&self.device_guid);
        buf
    }
}

fn copy_str_to_bytes(s: &str, buf: &mut [u8]) {
    let bytes = s.as_bytes();
    let len = core::cmp::min(bytes.len(), buf.len());
    buf[..len].copy_from_slice(&bytes[..len]);
}
