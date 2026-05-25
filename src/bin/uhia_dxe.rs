#![no_std]
#![no_main]

extern crate alloc;

use core::fmt::Write;
use uefi::prelude::*;

use uhia::{efivars, identity, network, smbios, tpm};

/// DXE Driver entry point
///
/// Unlike an EFI Application that runs and exits, a DXE Driver
/// is loaded by the firmware and can provide services.
/// This driver performs identity collection and then signals readiness.
#[entry]
fn efi_main(_image: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).unwrap();

    // Collect all data before borrowing stdout
    let smbios_result = smbios::collect(&system_table);
    let runtime = system_table.runtime_services();
    let device_guid = efivars::get_or_create_device_guid(runtime);
    let boot_services = system_table.boot_services();
    let tpm_result = tpm::collect(boot_services);
    let pcr_result = match &tpm_result {
        Ok(info) if info.present => tpm::read_pcrs(boot_services),
        _ => Err("TPM not present"),
    };
    let network_result = network::collect(boot_services);

    let stdout = system_table.stdout();
    stdout.clear().unwrap();
    let _ = stdout.write_str("UHIA v0.1.0 — UEFI Hardware Identity Agent (DXE)\r\n");

    // Display device GUID
    let _ = stdout.write_str("=== Device GUID (Persistent) ===\r\n  ");
    for (i, b) in device_guid.iter().enumerate() {
        if i == 4 || i == 6 || i == 8 || i == 10 {
            let _ = stdout.write_char('-');
        }
        let _ = write!(stdout, "{:02x}", b);
    }
    let _ = stdout.write_str("\r\n");

    // Display network info
    match &network_result {
        Ok(net_info) => {
            network::display::display_network_info(stdout, net_info);
        }
        Err(e) => {
            let _ = write!(stdout, "Network error: {}\r\n", e);
        }
    }

    // Display TPM info
    match &tpm_result {
        Ok(tpm_info) => {
            tpm::display::display_tpm_info(stdout, tpm_info);

            if tpm_info.present {
                match &pcr_result {
                    Ok(pcrs) => {
                        tpm::display::display_pcrs(stdout, pcrs);
                    }
                    Err(e) => {
                        let _ = write!(stdout, "PCR read error: {}\r\n", e);
                    }
                }
            }
        }
        Err(e) => {
            let _ = write!(stdout, "TPM error: {}\r\n", e);
        }
    }

    // Display SMBIOS info and compute identity
    match smbios_result {
        Ok(data) => {
            smbios::display::display_smbios_info(stdout, &data);

            let material = identity::fingerprint::IdentityMaterial::from_smbios(&data, device_guid);
            let device_hash = identity::hash::compute_device_identity(&material);
            identity::display::display_identity(stdout, &device_hash);
        }
        Err(e) => {
            let _ = write!(stdout, "SMBIOS error: {}\r\n", e);
        }
    }

    let _ = stdout.write_str("\r\n=== DXE Driver Loaded ===\r\n");

    // Return SUCCESS to indicate driver is loaded and ready
    // In a real DXE driver, this would register protocols and services
    Status::SUCCESS
}
