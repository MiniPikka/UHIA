pub mod types;

use uefi::prelude::*;
use uefi::table::runtime::{VariableAttributes, VariableVendor};

use self::types::IRONANCHOR_VENDOR_GUID;

/// Read the persistent device GUID from EFI Variable, or generate and store a new one
pub fn get_or_create_device_guid(runtime: &RuntimeServices) -> [u8; 16] {
    let var_name = cstr16!("IronAnchorDeviceGuid");
    let vendor = VariableVendor(IRONANCHOR_VENDOR_GUID);

    // Try to read existing GUID
    match runtime.get_variable(var_name, &vendor, &mut [0u8; 16]) {
        Ok((data, _)) if data.len() == 16 => {
            let mut guid = [0u8; 16];
            guid.copy_from_slice(data);
            guid
        }
        _ => {
            // Generate new GUID using hardware entropy
            let guid = generate_hardware_guid();
            // Store in EFI Variable (best effort)
            let _ = runtime.set_variable(
                var_name,
                &vendor,
                VariableAttributes::NON_VOLATILE
                    | VariableAttributes::BOOTSERVICE_ACCESS
                    | VariableAttributes::RUNTIME_ACCESS,
                &guid,
            );
            guid
        }
    }
}

/// Generate a GUID from hardware-specific entropy (no RNG dependency)
fn generate_hardware_guid() -> [u8; 16] {
    let mut guid = [0u8; 16];

    // Use CPU timestamp counter as entropy source
    let mut seed: u64;
    unsafe {
        core::arch::asm!("rdtsc", out("rax") seed, out("rdx") _);
    }

    // Mix in a fixed project identifier
    let project_id: u64 = 0x49726F6E416E6368; // "IronAnch" in ASCII

    // Simple mixing function
    seed ^= project_id;
    seed = seed.wrapping_mul(6364136223846793005);
    seed ^= seed >> 33;

    // Fill GUID bytes
    for i in 0..8 {
        guid[i] = ((seed >> (i * 8)) & 0xFF) as u8;
    }

    // Second round with different seed
    seed = seed.wrapping_mul(6364136223846793005);
    seed ^= seed >> 33;
    for i in 0..8 {
        guid[8 + i] = ((seed >> (i * 8)) & 0xFF) as u8;
    }

    // Set version (4) and variant (10xx) bits per RFC 4122
    guid[6] = (guid[6] & 0x0F) | 0x40; // version 4
    guid[8] = (guid[8] & 0x3F) | 0x80; // variant 1

    guid
}
