use core::fmt::Write;
use uefi::proto::console::text::Output;

use super::types::NetworkInfo;

pub fn display_network_info(stdout: &mut Output, info: &NetworkInfo) {
    let _ = stdout.write_str("\r\n=== Network Information ===\r\n");
    let _ = write!(stdout, "  Present:            {}\r\n", info.present);

    if !info.present {
        return;
    }

    let _ = write!(stdout, "  State:              {:?}\r\n", info.state);
    let _ = stdout.write_str("  MAC Address:        ");
    for (i, b) in info.mac_address.iter().enumerate() {
        if i > 0 {
            let _ = stdout.write_char(':');
        }
        let _ = write!(stdout, "{:02X}", b);
    }
    let _ = stdout.write_str("\r\n");
    let _ = write!(stdout, "  Media Header Size:  {}\r\n", info.media_header_size);
    let _ = write!(stdout, "  Max Packet Size:    {}\r\n", info.max_packet_size);
}
