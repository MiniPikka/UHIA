use super::types::*;

/// Iterator over SMBIOS structures in the raw table
pub struct SmbiosIterator<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> SmbiosIterator<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }
}

impl<'a> Iterator for SmbiosIterator<'a> {
    type Item = (StructureHeader, &'a [u8], &'a [u8]);

    /// Returns (header, fixed_data, string_section)
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + 4 > self.data.len() {
            return None;
        }

        let header = StructureHeader {
            typ: self.data[self.offset],
            length: self.data[self.offset + 1],
        };

        // End-of-table marker
        if header.typ == 127 {
            return None;
        }

        let start = self.offset;
        let end = start + header.length as usize;

        if end > self.data.len() {
            return None;
        }

        let fixed_data = &self.data[start..end];

        // Scan for double null terminator after the fixed-length portion
        // The string section starts right after fixed data and contains
        // null-separated strings, terminated by double null (0x00 0x00)
        let mut pos = end;

        while pos + 1 < self.data.len() {
            if self.data[pos] == 0x00 && self.data[pos + 1] == 0x00 {
                self.offset = pos + 2;
                // Return entire string section from end of fixed data to double-null
                let string_section = &self.data[end..pos];
                return Some((header, fixed_data, string_section));
            }
            pos += 1;
        }

        // If we reach here without finding double null, advance past this structure
        self.offset = self.data.len();
        Some((header, fixed_data, &[]))
    }
}

/// Extract a string from the SMBIOS string set by index (1-based)
pub fn get_string(strings: &[u8], index: u8) -> heapless::String<64> {
    if index == 0 {
        return heapless::String::new();
    }

    let mut current: u8 = 1;
    let mut start = 0;

    for (i, &byte) in strings.iter().enumerate() {
        if byte == 0x00 {
            if current == index {
                // Extract the string, stopping at any null/whitespace padding
                let raw = &strings[start..i];
                let trimmed = raw
                    .iter()
                    .position(|&b| b == 0x00)
                    .map(|p| &raw[..p])
                    .unwrap_or(raw);
                let s = core::str::from_utf8(trimmed).unwrap_or("");
                return heapless::String::try_from(s).unwrap_or_default();
            }
            current += 1;
            start = i + 1;
        }
    }

    heapless::String::new()
}

/// Parse SMBIOS data from a raw byte slice (the table pointed to by the entry point)
pub fn parse_smbios_table(table_data: &[u8]) -> SmbiosData {
    let mut data = SmbiosData::default();
    let iter = SmbiosIterator::new(table_data);

    for (header, fixed, strings) in iter {
        match header.typ {
            BIOS_INFO_TYPE if header.length >= 0x12 => {
                let vendor_idx = fixed[0x04];
                let version_idx = fixed[0x05];
                let date_idx = fixed[0x08];
                data.bios.vendor = get_string(strings, vendor_idx);
                data.bios.version = get_string(strings, version_idx);
                data.bios.release_date = get_string(strings, date_idx);
            }
            SYSTEM_INFO_TYPE if header.length >= 0x19 => {
                let mfr_idx = fixed[0x04];
                let product_idx = fixed[0x05];
                let serial_idx = fixed[0x07];
                data.system.manufacturer = get_string(strings, mfr_idx);
                data.system.product_name = get_string(strings, product_idx);
                data.system.serial = get_string(strings, serial_idx);
                // UUID is at offset 0x08, 16 bytes
                if header.length >= 0x18 {
                    data.system.uuid.copy_from_slice(&fixed[0x08..0x18]);
                }
            }
            BASEBOARD_INFO_TYPE if header.length >= 0x08 => {
                let mfr_idx = fixed[0x04];
                let product_idx = fixed[0x05];
                let serial_idx = fixed[0x07];
                data.baseboard.manufacturer = get_string(strings, mfr_idx);
                data.baseboard.product = get_string(strings, product_idx);
                data.baseboard.serial = get_string(strings, serial_idx);
            }
            PROCESSOR_INFO_TYPE if header.length >= 0x20 => {
                let socket_idx = fixed[0x04];
                let mfr_idx = fixed[0x07];
                data.processor.socket = get_string(strings, socket_idx);
                data.processor.manufacturer = get_string(strings, mfr_idx);
                // CPUID signature at offset 0x08 (4 bytes)
                data.processor.signature =
                    u32::from_le_bytes([fixed[0x08], fixed[0x09], fixed[0x0A], fixed[0x0B]]);
            }
            MEMORY_DEVICE_TYPE if header.length >= 0x15 => {
                let size_raw = u16::from_le_bytes([fixed[0x0C], fixed[0x0D]]);
                let size_kb = match size_raw {
                    0 => 0,
                    0xFFFF => 0, // unknown
                    v => (v as u32) * 1024, // SMBIOS reports in MB for values < 32768
                };
                let mfr_idx = fixed[0x17];
                let serial_idx = fixed[0x18];
                let part_idx = fixed[0x1A];

                let dev = MemoryDevice {
                    size_kb,
                    manufacturer: get_string(strings, mfr_idx),
                    serial: get_string(strings, serial_idx),
                    part_number: get_string(strings, part_idx),
                };
                let _ = data.memory_devices.push(dev);
            }
            _ => {}
        }
    }

    data
}
