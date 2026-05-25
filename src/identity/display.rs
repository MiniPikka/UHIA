use core::fmt::Write;
use uefi::proto::console::text::Output;

use super::hash::DeviceHash;

pub fn display_identity(stdout: &mut Output, hash: &DeviceHash) {
    let _ = stdout.write_str("\r\n=== Device Identity ===\r\n");
    let _ = stdout.write_str("  SHA256: ");
    for byte in hash {
        let _ = write!(stdout, "{:02x}", byte);
    }
    let _ = stdout.write_str("\r\n");
}
