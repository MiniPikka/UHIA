use core::fmt::Write;
use uefi::proto::console::text::Output;

use super::types::SmbiosData;

pub fn display_smbios_info(stdout: &mut Output, data: &SmbiosData) {
    let _ = stdout.write_str("\r\n=== BIOS Information ===\r\n");
    let _ = write!(stdout, "  Vendor:        {}\r\n", data.bios.vendor);
    let _ = write!(stdout, "  Version:       {}\r\n", data.bios.version);
    let _ = write!(stdout, "  Release Date:  {}\r\n", data.bios.release_date);

    let _ = stdout.write_str("\r\n=== System Information ===\r\n");
    let _ = write!(stdout, "  Manufacturer:  {}\r\n", data.system.manufacturer);
    let _ = write!(stdout, "  Product Name:  {}\r\n", data.system.product_name);
    let _ = write!(stdout, "  Serial:        {}\r\n", data.system.serial);
    let _ = stdout.write_str("  UUID:          ");
    for (i, b) in data.system.uuid.iter().enumerate() {
        if i == 4 || i == 6 || i == 8 || i == 10 {
            let _ = stdout.write_char('-');
        }
        let _ = write!(stdout, "{:02x}", b);
    }
    let _ = stdout.write_str("\r\n");

    let _ = stdout.write_str("\r\n=== Baseboard Information ===\r\n");
    let _ = write!(stdout, "  Manufacturer:  {}\r\n", data.baseboard.manufacturer);
    let _ = write!(stdout, "  Product:       {}\r\n", data.baseboard.product);
    let _ = write!(stdout, "  Serial:        {}\r\n", data.baseboard.serial);

    let _ = stdout.write_str("\r\n=== Processor Information ===\r\n");
    let _ = write!(stdout, "  Socket:        {}\r\n", data.processor.socket);
    let _ = write!(stdout, "  Manufacturer:  {}\r\n", data.processor.manufacturer);
    let _ = write!(
        stdout,
        "  CPUID Sig:     0x{:08X}\r\n",
        data.processor.signature
    );

    if !data.memory_devices.is_empty() {
        let _ = stdout.write_str("\r\n=== Memory Devices ===\r\n");
        for (i, dev) in data.memory_devices.iter().enumerate() {
            let _ = write!(stdout, "  [{}]\r\n", i);
            let _ = write!(
                stdout,
                "    Size:        {} MB\r\n",
                dev.size_kb / 1024
            );
            let _ = write!(stdout, "    Manufacturer:{}\r\n", dev.manufacturer);
            let _ = write!(stdout, "    Serial:      {}\r\n", dev.serial);
            let _ = write!(stdout, "    Part Number: {}\r\n", dev.part_number);
        }
    }
}
