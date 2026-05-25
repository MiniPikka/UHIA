use core::fmt::Write;
use uefi::proto::console::text::Output;

use super::types::TpmInfo;

pub fn display_tpm_info(stdout: &mut Output, info: &TpmInfo) {
    let _ = stdout.write_str("\r\n=== TPM Information ===\r\n");
    let _ = write!(stdout, "  Present:            {}\r\n", info.present);

    if !info.present {
        return;
    }

    // Decode manufacturer ID (4 bytes, may be ASCII or vendor-specific)
    let mfr_bytes = info.manufacturer.to_be_bytes();
    let is_ascii = mfr_bytes.iter().all(|&b| b >= 0x20 && b < 0x7F);
    if is_ascii {
        let mfr_str = core::str::from_utf8(&mfr_bytes).unwrap_or("????");
        let _ = write!(stdout, "  Manufacturer:       {} (0x{:08X})\r\n", mfr_str, info.manufacturer);
    } else {
        let _ = write!(stdout, "  Manufacturer:       0x{:08X}\r\n", info.manufacturer);
    }

    // Firmware version
    let fw_major = (info.firmware_version >> 32) as u32;
    let fw_minor = info.firmware_version as u32;
    let _ = write!(stdout, "  Firmware Version:   {}.{}\r\n", fw_major, fw_minor);

    // Active PCR banks
    let _ = write!(stdout, "  Active PCR Banks:   0x{:08X}\r\n", info.active_pcr_banks);
    let _ = write!(stdout, "  PCR Count:          {}\r\n", info.pcr_count);
}

pub fn display_pcrs(stdout: &mut Output, pcrs: &[[u8; 32]; 8]) {
    let _ = stdout.write_str("\r\n=== PCR Values (SHA256, PCRs 0-7) ===\r\n");
    for (i, pcr) in pcrs.iter().enumerate() {
        let _ = write!(stdout, "  PCR[{}]: ", i);
        for byte in pcr {
            let _ = write!(stdout, "{:02x}", byte);
        }
        let _ = stdout.write_str("\r\n");
    }
}
