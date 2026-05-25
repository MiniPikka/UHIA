pub mod display;
pub mod parser;
pub mod types;

use uefi::prelude::*;
use uefi::table::cfg::{SMBIOS3_GUID, SMBIOS_GUID};

use self::parser::parse_smbios_table;
use self::types::SmbiosData;

/// Locate and parse the SMBIOS tables from the UEFI configuration table
pub fn collect(system_table: &SystemTable<Boot>) -> Result<SmbiosData, &'static str> {
    let config_tables = system_table.config_table();

    // Try SMBIOS 3.0 first, then fall back to 2.1
    for entry in config_tables {
        if entry.guid == SMBIOS3_GUID {
            // entry.address points to the SMBIOS 3.0 entry point (anchor "_SM3_")
            let ep = entry.address as *const u8;
            let anchor = unsafe { core::slice::from_raw_parts(ep, 5) };
            if anchor != b"_SM3_" {
                continue;
            }
            // Table address at entry point + 0x10 (8 bytes), length at +0x0C (4 bytes)
            let table_addr = unsafe {
                u64::from_le_bytes(
                    core::slice::from_raw_parts(ep.add(0x10), 8)
                        .try_into()
                        .unwrap(),
                )
            };
            let table_len = unsafe {
                u32::from_le_bytes(
                    core::slice::from_raw_parts(ep.add(0x0C), 4)
                        .try_into()
                        .unwrap(),
                )
            };
            if table_addr == 0 || table_len == 0 {
                continue;
            }
            let table_data = unsafe {
                core::slice::from_raw_parts(table_addr as *const u8, table_len as usize)
            };
            return Ok(parse_smbios_table(table_data));
        }

        if entry.guid == SMBIOS_GUID {
            // entry.address points to the SMBIOS 2.1 entry point (anchor "_SM_")
            let ep = entry.address as *const u8;
            let anchor = unsafe { core::slice::from_raw_parts(ep, 4) };
            if anchor != b"_SM_" {
                continue;
            }
            // Intermediate anchor "_DMI_" is at entry point + 0x10
            // Table address at +0x18 (4 bytes), length at +0x16 (2 bytes)
            let table_addr = unsafe {
                u32::from_le_bytes(
                    core::slice::from_raw_parts(ep.add(0x18), 4)
                        .try_into()
                        .unwrap(),
                )
            };
            let table_len = unsafe {
                u16::from_le_bytes(
                    core::slice::from_raw_parts(ep.add(0x16), 2)
                        .try_into()
                        .unwrap(),
                )
            };
            if table_addr == 0 || table_len == 0 {
                continue;
            }
            let table_data = unsafe {
                core::slice::from_raw_parts(table_addr as *const u8, table_len as usize)
            };
            return Ok(parse_smbios_table(table_data));
        }
    }

    Err("SMBIOS table not found in configuration table")
}
