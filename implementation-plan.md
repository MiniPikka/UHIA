# IronAnchor Implementation Plan — Phases 1-7

## Overview

This plan covers the complete implementation of the IronAnchor UEFI Hardware Identity Agent, from "Hello World" to a production-ready DXE driver with full hardware identity collection capabilities.

**Target**: Produce both an EFI application and DXE driver that run in QEMU+OVMF, collect comprehensive hardware/firmware information, generate stable device identity hashes, and persist identity across reboots.

## Status

- [x] Phase 1: UEFI Hello World
- [x] Phase 2: SMBIOS Reading
- [x] Phase 3: Identity Engine
- [x] Phase 4: EFI Variable Persistence
- [x] Phase 5: TPM Integration
- [x] Phase 6: Network Reporting
- [x] Phase 7: DXE Driver

---

## Phase 1: UEFI Hello World

**Goal**: Get a minimal Rust EFI application booting in QEMU with console output.

### 1.1 Project Setup

Create the Rust project structure:

```
ironanchor/
├── Cargo.toml
├── rust-toolchain.toml
├── src/
│   └── main.rs
├── esp/
│   └── EFI/
│       └── IronAnchor/
│           └── (IronAnchor.efi goes here after build)
├── scripts/
│   ├── build.sh
│   └── run-qemu.sh
└── OVMF.fd (download separately)
```

**Cargo.toml** key configuration:
```toml
[package]
name = "ironanchor"
version = "0.1.0"
edition = "2021"

[dependencies]
uefi = { version = "0.28", features = ["alloc"] }
uefi-services = "0.25"

[profile.release]
panic = "abort"
lto = true
codegen-units = 1
```

**rust-toolchain.toml**:
```toml
[toolchain]
channel = "nightly"
targets = ["x86_64-unknown-uefi"]
components = ["rust-src", "rustfmt", "clippy"]
```

### 1.2 Minimal EFI Application

**src/main.rs** — Phase 1 skeleton:

```rust
#![no_std]
#![no_main]

extern crate alloc;

use uefi::prelude::*;

#[entry]
fn main(image: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    let stdout = system_table.stdout();
    stdout.clear().unwrap();
    stdout.write_str("IronAnchor v0.1.0 — UEFI Hardware Identity Agent\r\n").unwrap();
    stdout.write_str("Phase 1: Hello World — System initialized.\r\n").unwrap();

    // Boot services available, OS not yet loaded
    Status::SUCCESS
}
```

### 1.3 Build & Run Scripts

**scripts/build.sh**:
```bash
#!/bin/bash
set -e

echo "[*] Building IronAnchor EFI application..."
cargo build --release --target x86_64-unknown-uefi

mkdir -p esp/EFI/IronAnchor
cp target/x86_64-unknown-uefi/release/ironanchor.efi esp/EFI/IronAnchor/IronAnchor.efi

echo "[+] Built: esp/EFI/IronAnchor/IronAnchor.efi"
```

**scripts/run-qemu.sh**:
```bash
#!/bin/bash
set -e

OVMF_PATH="${OVMF_PATH:-./OVMF.fd}"
ESP_PATH="${ESP_PATH:-./esp}"

qemu-system-x86_64 \
    -bios "$OVMF_PATH" \
    -drive format=raw,file=fat:rw:"$ESP_PATH" \
    -serial stdio \
    -no-reboot \
    -display none
```

### 1.4 Deliverables

- [ ] Rust project compiles to `x86_64-unknown-uefi`
- [ ] `IronAnchor.efi` boots in QEMU+OVMF
- [ ] "Hello World" message appears on serial output
- [ ] Build and run scripts work

---

## Phase 2: SMBIOS Reading

**Goal**: Parse SMBIOS tables to extract BIOS, motherboard, and memory information.

### 2.1 SMBIOS Module Structure

```
src/
├── main.rs
└── smbios/
    ├── mod.rs          — public API
    ├── parser.rs       — SMBIOS table iterator
    ├── types.rs        — SMBIOS structure type definitions
    └── display.rs      — formatting for console output
```

### 2.2 SMBIOS Access via UEFI

The `uefi` crate provides `Smbios` access through `SystemTable`:

```rust
use uefi::table::system::SMBIOS_ANCHOR_GUID;

// Locate SMBIOS entry point from UEFI configuration table
fn find_smbios_table(system_table: &SystemTable<Boot>) -> Option<&[u8]> {
    system_table
        .configuration_table()
        .iter()
        .find(|entry| entry.guid == SMBIOS_ANCHOR_GUID)
        .map(|entry| {
            let ptr = entry.vendor_specific as *const u8;
            // Parse SMBIOS 3.0 or 2.1 entry point to find table address
            unsafe { /* read entry point, locate table */ }
        })
}
```

### 2.3 Key SMBIOS Structures to Parse

| Type | Structure | Fields of Interest |
|------|-----------|-------------------|
| 0 | BIOS Information | Vendor, Version, Release Date |
| 1 | System Information | Manufacturer, Product Name, UUID, Serial |
| 2 | Baseboard Information | Manufacturer, Product, Serial |
| 4 | Processor Information | Socket, Family, Manufacturer, Signature |
| 17 | Memory Device | Size, Manufacturer, Serial, Part Number |

### 2.4 SMBIOS Table Iterator

**src/smbios/parser.rs** — core parsing logic:

```rust
/// Walk the SMBIOS structure table, yielding each structure header + data
pub struct SmbiosIterator<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for SmbiosIterator<'a> {
    type Item = SmbiosStructure<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Each structure has:
        //   byte 0: type
        //   byte 1: length (header + fixed fields)
        //   byte 2-3: handle
        //   followed by strings (terminated by double null)
        // Navigate to next structure by scanning for double null after length bytes
    }
}
```

### 2.5 Data Extraction Types

**src/smbios/types.rs**:

```rust
pub struct BiosInfo {
    pub vendor: String<64>,      // String from SMBIOS string set
    pub version: String<64>,
    pub release_date: String<64>,
}

pub struct SystemInfo {
    pub manufacturer: String<64>,
    pub product_name: String<64>,
    pub uuid: [u8; 16],
    pub serial: String<64>,
}

pub struct BaseboardInfo {
    pub manufacturer: String<64>,
    pub product: String<64>,
    pub serial: String<64>,
}

pub struct ProcessorInfo {
    pub socket: String<64>,
    pub manufacturer: String<64>,
    pub signature: u32,          // CPUID signature
}
```

### 2.6 Console Display

**src/smbios/display.rs** — pretty-print collected info:

```rust
pub fn display_smbios_info(stdout: &mut Output, info: &SmbiosData) {
    let _ = stdout.write_str("=== BIOS Information ===\r\n");
    let _ = write!(stdout, "  Vendor:  {}\r\n", info.bios.vendor);
    let _ = write!(stdout, "  Version: {}\r\n", info.bios.version);
    // ... system, baseboard, processor, memory
}
```

### 2.7 Integration in main.rs

```rust
mod smbios;

#[entry]
fn main(image: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let stdout = system_table.stdout();

    stdout.write_str("IronAnchor v0.2.0 — SMBIOS Collection\r\n").unwrap();

    match smbios::collect(&system_table) {
        Ok(data) => {
            smbios::display::display_smbios_info(stdout, &data);
        }
        Err(e) => {
            let _ = write!(stdout, "SMBIOS error: {:?}\r\n", e);
        }
    }

    Status::SUCCESS
}
```

### 2.8 QEMU SMBIOS Injection

QEMU allows injecting SMBIOS data for testing:

```bash
qemu-system-x86_64 \
    -bios OVMF.fd \
    -drive format=raw,file=fat:rw:esp/ \
    -serial stdio \
    -smbios type=1,manufacturer=TestCorp,product=IronAnchor-Test,serial=SN12345 \
    -smbios type=2,manufacturer=TestCorp,product=Board-X1 \
    -display none
```

### 2.9 Deliverables

- [ ] SMBIOS module with table iterator
- [ ] Parsing for types 0, 1, 2, 4, 17
- [ ] Console output showing collected hardware info
- [ ] Works with real OVMF and with QEMU-injected SMBIOS data
- [ ] Unit tests for SMBIOS structure parsing (run with `cargo test` on host via feature flag)

---

## Phase 3: Identity Engine

**Goal**: Generate a stable SHA256 device identity hash from collected hardware + firmware inputs.

### 3.1 Identity Module Structure

```
src/
├── main.rs
├── smbios/
│   └── ...
└── identity/
    ├── mod.rs          — public API
    ├── fingerprint.rs  — raw material collection
    ├── hash.rs         — SHA256 computation
    └── display.rs      — identity output
```

### 3.2 Identity Material Collection

**src/identity/fingerprint.rs** — gather raw identity inputs:

```rust
use crate::smbios::SmbiosData;

/// Raw identity material before hashing
pub struct IdentityMaterial {
    pub bios_uuid: [u8; 16],           // SMBIOS Type 1 UUID
    pub board_serial: [u8; 64],        // SMBIOS Type 2 Serial
    pub cpu_signature: u32,            // SMBIOS Type 4 CPUID sig
    pub bios_vendor: [u8; 64],         // SMBIOS Type 0 Vendor
    pub bios_version: [u8; 64],        // SMBIOS Type 0 Version
    // Phase 5 additions:
    // pub tpm_ek: Vec<u8>,            // TPM Endorsement Key
    // pub secure_boot: bool,          // Secure Boot state
}

impl IdentityMaterial {
    pub fn from_smbios(smbios: &SmbiosData) -> Self {
        // Extract and normalize fields from SMBIOS data
        // Normalize = lowercase, trim, fixed-length byte arrays
    }

    /// Serialize all material into a contiguous byte buffer for hashing
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.bios_uuid);
        buf.extend_from_slice(&self.board_serial);
        buf.extend_from_slice(&self.cpu_signature.to_le_bytes());
        buf.extend_from_slice(&self.bios_vendor);
        buf.extend_from_slice(&self.bios_version);
        buf
    }
}
```

### 3.3 SHA256 Hashing

**src/identity/hash.rs**:

```rust
use sha2::{Sha256, Digest};

pub type DeviceHash = [u8; 32];

pub fn compute_device_identity(material: &IdentityMaterial) -> DeviceHash {
    let mut hasher = Sha256::new();
    hasher.update(material.to_bytes());
    let result = hasher.finalize();

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

pub fn hash_to_hex(hash: &DeviceHash) -> [u8; 64] {
    // Convert 32-byte hash to 64-char hex string
}
```

### 3.4 Identity Display

**src/identity/display.rs**:

```rust
pub fn display_identity(stdout: &mut Output, hash: &DeviceHash) {
    let _ = stdout.write_str("\r\n=== Device Identity ===\r\n");
    let _ = stdout.write_str("  SHA256: ");
    // Print hex bytes
    for byte in hash {
        let _ = write!(stdout, "{:02x}", byte);
    }
    let _ = stdout.write_str("\r\n");
}
```

### 3.5 Main Integration

```rust
mod smbios;
mod identity;

#[entry]
fn main(image: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let stdout = system_table.stdout();

    stdout.write_str("IronAnchor v0.3.0 — Identity Engine\r\n").unwrap();

    // Step 1: Collect SMBIOS data
    let smbios_data = match smbios::collect(&system_table) {
        Ok(data) => data,
        Err(e) => {
            let _ = write!(stdout, "FATAL: SMBIOS collection failed: {:?}\r\n", e);
            return Status::UNSUPPORTED;
        }
    };

    // Step 2: Display collected info
    smbios::display::display_smbios_info(stdout, &smbios_data);

    // Step 3: Build identity material
    let material = identity::fingerprint::IdentityMaterial::from_smbios(&smbios_data);

    // Step 4: Compute identity hash
    let device_hash = identity::hash::compute_device_identity(&material);

    // Step 5: Display identity
    identity::display::display_identity(stdout, &device_hash);

    Status::SUCCESS
}
```

### 3.6 Stability Requirements

The identity hash must be **deterministic and stable** across:
- Reboots (same hardware → same hash)
- OS reinstalls (no OS dependency)

The hash **will change** when:
- BIOS is updated (vendor/version change)
- Motherboard is replaced (serial/UUID change)
- CPU is replaced (signature change)

### 3.7 Testing Strategy

**Host-side tests** (run with `cargo test` on Linux, using a feature flag):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_hash() {
        let material = IdentityMaterial {
            bios_uuid: [1, 2, 3, /* ... */],
            board_serial: /* ... */,
            cpu_signature: 0x000906EA,
            bios_vendor: /* ... */,
            bios_version: /* ... */,
        };
        let hash1 = compute_device_identity(&material);
        let hash2 = compute_device_identity(&material);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_hardware_different_hash() {
        let mut material1 = make_test_material();
        let mut material2 = make_test_material();
        material2.board_serial[0] = 0xFF; // different serial
        assert_ne!(
            compute_device_identity(&material1),
            compute_device_identity(&material2)
        );
    }
}
```

### 3.8 Deliverables

- [ ] `identity` module with fingerprint collection and SHA256 hashing
- [ ] Device identity hash displayed on QEMU boot
- [ ] Hash is deterministic (same inputs → same output)
- [ ] Hash changes when hardware inputs change
- [ ] Host-side unit tests pass

---

## Build Order & Dependencies

```
Phase 1 ──→ Phase 2 ──→ Phase 3
(Hello)     (SMBIOS)     (Identity)

Phase 1 delivers:
  - Project scaffolding
  - cargo build --release --target x86_64-unknown-uefi works
  - QEMU boot confirmed

Phase 2 depends on Phase 1:
  - Need working EFI app to test SMBIOS parsing
  - SMBIOS module is standalone, testable independently

Phase 3 depends on Phase 2:
  - Identity engine uses SMBIOS data as input
  - sha2 crate added to Cargo.toml
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| SMBIOS table not found | Fallback to CPUID-only fingerprint, log warning |
| UUID is all zeros (common in VMs) | Document as known limitation, combine with other fields |
| sha2 doesn't work in UEFI | sha2 is pure Rust, no_std compatible — should work. Test early. |
| OVMF not available | Download from Arch `edk2-ovmf` package or build from source |

---

## File Summary After Phase 3

```
ironanchor/
├── Cargo.toml
├── rust-toolchain.toml
├── src/
│   ├── main.rs              — entry point, orchestration
│   ├── smbios/
│   │   ├── mod.rs           — SMBIOS public API
│   │   ├── parser.rs        — table iterator
│   │   ├── types.rs         — structure definitions
│   │   └── display.rs       — console output
│   └── identity/
│       ├── mod.rs           — identity public API
│       ├── fingerprint.rs   — raw material collection
│       ├── hash.rs          — SHA256 computation
│       └── display.rs       — identity output
├── esp/
│   └── EFI/
│       └── IronAnchor/
├── scripts/
│   ├── build.sh
│   └── run-qemu.sh
└── tests/
    └── unit/                — host-side tests
```

---

## Phase 4: EFI Variable Persistence

**Goal**: Generate and persist a device GUID in EFI Variable storage so the identity survives reboots.

### 4.1 EFI Variable Module Structure

```
src/
├── main.rs
├── smbios/
│   └── ...
├── identity/
│   └── ...
└── efivars/
    ├── mod.rs          — public API
    └── types.rs        — variable names, vendor GUID
```

### 4.2 Key Design Decisions

- **Vendor GUID**: Custom GUID namespace for IronAnchor variables: `a1b2c3d4-e5f6-7890-abcd-ef1234567890`
- **Variable name**: `IronAnchorDeviceGuid`
- **Attributes**: `NON_VOLATILE | BOOTSERVICE_ACCESS | RUNTIME_ACCESS`
- **GUID generation**: Use CPU timestamp counter (`rdtsc`) as entropy source, mixed with project identifier, formatted as RFC 4122 v4 UUID

### 4.3 Implementation

**src/efivars/types.rs**:
```rust
pub const IRONANCHOR_VENDOR_GUID: uefi::Guid = uefi::guid!("a1b2c3d4-e5f6-7890-abcd-ef1234567890");
```

**src/efivars/mod.rs**:
```rust
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
            // Generate new GUID and store it
            let guid = generate_hardware_guid();
            let _ = runtime.set_variable(var_name, &vendor, attributes, &guid);
            guid
        }
    }
}
```

### 4.4 Integration with Identity Engine

The device GUID is included in the identity material:

```rust
pub struct IdentityMaterial {
    // ... existing fields from Phase 3 ...
    pub device_guid: [u8; 16], // From EFI Variable (persistent)
}

pub fn to_bytes(&self) -> [u8; 212] {
    // ... existing fields ...
    buf[196..212].copy_from_slice(&self.device_guid);
    buf
}
```

### 4.5 QEMU Testing Notes

- QEMU's default VARS file is temporary — GUID changes each run unless persistent VARS is configured
- To test persistence: `qemu-system-x86_64 -bios OVMF.fd -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd -drive if=pflash,format=raw,file=OVMF_VARS.fd ...`
- The `rdtsc` instruction may return similar values in QEMU's virtualized environment

### 4.6 Deliverables

- [x] `efivars` module with GUID read/write
- [x] Device GUID generated on first run and displayed
- [x] GUID included in identity material hash
- [x] Build succeeds with no warnings

---

## File Summary After Phase 4

```
ironanchor/
├── Cargo.toml                  — 42K PE32+ binary
├── rust-toolchain.toml
├── src/
│   ├── main.rs                 — entry point, orchestration
│   ├── smbios/
│   │   ├── mod.rs              — SMBIOS table location
│   │   ├── parser.rs           — structure iterator + string extraction
│   │   ├── types.rs            — structure definitions
│   │   └── display.rs          — console output
│   ├── identity/
│   │   ├── mod.rs
│   │   ├── fingerprint.rs      — raw material collection (includes EFI GUID)
│   │   ├── hash.rs             — SHA256 computation
│   │   └── display.rs          — identity output
│   └── efivars/
│       ├── mod.rs              — EFI Variable read/write
│       └── types.rs            — vendor GUID constant
├── esp/EFI/BOOT/BOOTX64.EFI
└── scripts/
    ├── build.sh
    └── run-qemu.sh
```

---

## Phase 5: TPM Integration

**Goal**: Read TPM capabilities and PCR values via EFI_TCG2_PROTOCOL.

### 5.1 TPM Module Structure

```
src/
├── main.rs
├── smbios/
│   └── ...
├── identity/
│   └── ...
├── efivars/
│   └── ...
└── tpm/
    ├── mod.rs          — public API (collect, read_pcrs)
    ├── types.rs        — TpmInfo struct, constants
    └── display.rs      — console output
```

### 5.2 Key Design Decisions

- **Protocol**: Use `uefi::proto::tcg::v2::Tcg` (EFI_TCG2_PROTOCOL)
- **TPM detection**: `Tcg::get_capability()` returns `BootServiceCapability::tpm_present()`
- **Raw commands**: Use `Tcg::submit_command()` for TPM2_GetCapability and TPM2_PCR_Read
- **PCR bank**: SHA256 (algorithm ID 0x000B)
- **PCR range**: PCRs 0-7 (first 8 PCRs)

### 5.3 Implementation

**src/tpm/types.rs**:
```rust
#[derive(Debug, Clone, Default)]
pub struct TpmInfo {
    pub present: bool,
    pub manufacturer: u32,
    pub firmware_version: u64,
    pub active_pcr_banks: u32,
    pub pcr_count: u32,
}
```

**src/tpm/mod.rs**:
```rust
pub fn collect(boot_services: &BootServices) -> Result<TpmInfo, &'static str> {
    let handle = boot_services.get_handle_for_protocol::<Tcg>()?;
    let mut tcg = boot_services.open_protocol_exclusive(handle)?;

    // Get capability
    let cap = tcg.get_capability()?;
    let present = cap.tpm_present();

    // Get active PCR banks
    let banks = tcg.get_active_pcr_banks()?;

    // Send raw TPM2_GetCapability for manufacturer/fw version
    let (manufacturer, fw_version) = get_tpm_properties(&mut tcg);

    Ok(TpmInfo { present, manufacturer, fw_version, ... })
}

pub fn read_pcrs(boot_services: &BootServices) -> Result<[[u8; 32]; 8], &'static str> {
    // Send TPM2_PCR_Read command for SHA256 bank, PCRs 0-7
    // Parse response to extract 32-byte digests
}
```

### 5.4 Raw TPM2 Command Structure

**TPM2_GetCapability**:
- Tag: 0x8001 (TPM_ST_NO_SESSIONS)
- CommandCode: 0x0000017A
- Capability: 0x00000006 (TPM_CAP_TPM_PROPERTIES)
- Property: 0x00000100 (TPM2_PT_MANUFACTURER)
- PropertyCount: 3

**TPM2_PCR_Read**:
- Tag: 0x8001
- CommandCode: 0x0000017E
- PCR Selection: SHA256 (0x000B), PCRs 0-7 (0xFF, 0x00, 0x00)

### 5.5 QEMU TPM Testing

```bash
# Start swtpm emulator
swtpm socket --tpmstate dir=/tmp/swtpm --ctrl type=unixio,path=/tmp/swtpm.sock --tpm2

# Run QEMU with TPM
qemu-system-x86_64 \
    -bios /usr/share/edk2/x64/OVMF.4m.fd \
    -drive format=raw,file=fat:rw:esp/ \
    -chardev socket,id=chrtpm,path=/tmp/swtpm.sock \
    -tpmdev emulator,id=tpm0,chardev=chrtpm \
    -device tpm-tis,tpmdev=tpm0 \
    -serial stdio \
    -display none
```

### 5.6 Graceful Degradation

If TCG2 protocol is not available (no TPM):
- `tpm::collect()` returns `Err("No TCG2 protocol handle found")`
- Main function displays error message and continues
- Identity hash computed without TPM data

### 5.7 Deliverables

- [x] `tpm` module with TCG2 protocol access
- [x] TPM capability detection (present, manufacturer, firmware version)
- [x] PCR value reading (SHA256 bank, PCRs 0-7)
- [x] Graceful handling when TPM is not present
- [x] Build succeeds with no warnings

---

## File Summary After Phase 5

```
ironanchor/
├── Cargo.toml                  — 47K PE32+ binary
├── rust-toolchain.toml
├── src/
│   ├── main.rs                 — entry point, orchestration
│   ├── smbios/
│   │   ├── mod.rs              — SMBIOS table location
│   │   ├── parser.rs           — structure iterator + string extraction
│   │   ├── types.rs            — structure definitions
│   │   └── display.rs          — console output
│   ├── identity/
│   │   ├── mod.rs
│   │   ├── fingerprint.rs      — raw material collection (includes EFI GUID)
│   │   ├── hash.rs             — SHA256 computation
│   │   └── display.rs          — identity output
│   ├── efivars/
│   │   ├── mod.rs              — EFI Variable read/write
│   │   └── types.rs            — vendor GUID constant
│   └── tpm/
│       ├── mod.rs              — TCG2 protocol access, PCR reading
│       ├── types.rs            — TpmInfo struct
│       └── display.rs          — console output
├── esp/EFI/BOOT/BOOTX64.EFI
└── scripts/
    ├── build.sh
    └── run-qemu.sh
```

---

## Phase 6: Network Reporting

**Goal**: Detect network interfaces and read MAC address via EFI_SIMPLE_NETWORK_PROTOCOL.

### 6.1 Network Module Structure

```
src/
├── main.rs
├── smbios/
│   └── ...
├── identity/
│   └── ...
├── efivars/
│   └── ...
├── tpm/
│   └── ...
└── network/
    ├── mod.rs          — public API (collect)
    ├── types.rs        — NetworkInfo struct
    └── display.rs      — console output
```

### 6.2 Key Design Decisions

- **Protocol**: Use `uefi::proto::network::snp::SimpleNetwork` (EFI_SIMPLE_NETWORK_PROTOCOL)
- **MAC address**: Read from `NetworkMode::current_address` (first 6 bytes of 32-byte MacAddress struct)
- **Network state**: Map from `NetworkState` enum (STOPPED, STARTED, INITIALIZED)
- **No HTTP yet**: Full HTTP/TCP implementation deferred — this phase focuses on interface detection

### 6.3 Implementation

**src/network/types.rs**:
```rust
#[derive(Debug, Clone, Default)]
pub struct NetworkInfo {
    pub present: bool,
    pub mac_address: [u8; 6],
    pub media_header_size: u32,
    pub max_packet_size: u32,
    pub state: NetworkState,
}
```

**src/network/mod.rs**:
```rust
pub fn collect(boot_services: &BootServices) -> Result<NetworkInfo, &'static str> {
    let handle = boot_services.get_handle_for_protocol::<SimpleNetwork>()?;
    let snp = boot_services.open_protocol_exclusive(handle)?;

    let mode = snp.mode();
    let mac = mode.current_address.0;
    let mut info = NetworkInfo::default();
    info.mac_address.copy_from_slice(&mac[..6]);
    info.media_header_size = mode.media_header_size;
    info.max_packet_size = mode.max_packet_size;
    info.present = true;

    Ok(info)
}
```

### 6.4 QEMU Network Testing

```bash
# Run QEMU with network interface
qemu-system-x86_64 \
    -bios /usr/share/edk2/x64/OVMF.4m.fd \
    -drive format=raw,file=fat:rw:esp/ \
    -nic user,model=e1000 \
    -serial stdio \
    -display none
```

### 6.5 Graceful Degradation

If SimpleNetwork protocol is not available:
- `network::collect()` returns `Err("No SimpleNetwork protocol handle found")`
- Main function displays error message and continues

### 6.6 Deliverables

- [x] `network` module with SimpleNetwork protocol access
- [x] MAC address reading
- [x] Network state detection
- [x] Graceful handling when network is not present
- [x] Build succeeds with no errors

---

## File Summary After Phase 6

```
ironanchor/
├── Cargo.toml                  — 50K PE32+ binary
├── rust-toolchain.toml
├── src/
│   ├── main.rs                 — entry point, orchestration
│   ├── smbios/
│   │   ├── mod.rs              — SMBIOS table location
│   │   ├── parser.rs           — structure iterator + string extraction
│   │   ├── types.rs            — structure definitions
│   │   └── display.rs          — console output
│   ├── identity/
│   │   ├── mod.rs
│   │   ├── fingerprint.rs      — raw material collection (includes EFI GUID)
│   │   ├── hash.rs             — SHA256 computation
│   │   └── display.rs          — identity output
│   ├── efivars/
│   │   ├── mod.rs              — EFI Variable read/write
│   │   └── types.rs            — vendor GUID constant
│   ├── tpm/
│   │   ├── mod.rs              — TCG2 protocol access, PCR reading
│   │   ├── types.rs            — TpmInfo struct
│   │   └── display.rs          — console output
│   └── network/
│       ├── mod.rs              — SimpleNetwork protocol access
│       ├── types.rs            — NetworkInfo struct
│       └── display.rs          — console output
├── esp/EFI/BOOT/BOOTX64.EFI
└── scripts/
    ├── build.sh
    └── run-qemu.sh
```

---

## Phase 7: DXE Driver

**Goal**: Convert the EFI application into a DXE driver that can be loaded by the firmware during boot.

### 7.1 DXE Driver vs EFI Application

| Aspect | EFI Application | DXE Driver |
|--------|-----------------|------------|
| Loading | Loaded manually or by boot manager | Loaded by firmware during DXE phase |
| Lifetime | Runs and exits | Persists in memory |
| Entry point | `efi_main` returns Status | `efi_main` returns Status, can register protocols |
| Use case | One-shot execution | System service, persistent agent |

### 7.2 Project Structure for Dual Builds

```
ironanchor/
├── Cargo.toml              — library + 2 binaries
├── src/
│   ├── lib.rs              — shared modules (library crate)
│   ├── main.rs             — EFI Application entry point
│   └── bin/
│       └── ironanchor_dxe.rs — DXE Driver entry point
```

**Cargo.toml** configuration:
```toml
[lib]
name = "ironanchor"
path = "src/lib.rs"

[[bin]]
name = "ironanchor"
path = "src/main.rs"

[[bin]]
name = "ironanchor_dxe"
path = "src/bin/ironanchor_dxe.rs"
```

### 7.3 DXE Driver Implementation

**src/bin/ironanchor_dxe.rs**:
```rust
#![no_std]
#![no_main]

extern crate alloc;

use core::fmt::Write;
use uefi::prelude::*;
use ironanchor::{efivars, identity, network, smbios, tpm};

#[entry]
fn efi_main(_image: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).unwrap();

    // Same identity collection logic as EFI Application
    // ...

    // Return SUCCESS to indicate driver is loaded and ready
    Status::SUCCESS
}
```

### 7.4 Build Script Updates

**scripts/build.sh**:
```bash
#!/bin/bash
set -e

echo "[*] Building IronAnchor..."
cargo build --release --target x86_64-unknown-uefi

mkdir -p esp/EFI/IronAnchor esp/EFI/BOOT

# EFI Application
cp target/x86_64-unknown-uefi/release/ironanchor.efi esp/EFI/IronAnchor/IronAnchor.efi
cp target/x86_64-unknown-uefi/release/ironanchor.efi esp/EFI/BOOT/BOOTX64.EFI

# DXE Driver
cp target/x86_64-unknown-uefi/release/ironanchor_dxe.efi esp/EFI/IronAnchor/IronAnchorDxe.efi

echo "[+] Built: esp/EFI/IronAnchor/IronAnchor.efi (EFI Application)"
echo "[+] Built: esp/EFI/BOOT/BOOTX64.EFI (Fallback Boot)"
echo "[+] Built: esp/EFI/IronAnchor/IronAnchorDxe.efi (DXE Driver)"
```

### 7.5 Run Script Updates

**scripts/run-qemu.sh**:
```bash
#!/bin/bash
set -e

OVMF_PATH="${OVMF_PATH:-/usr/share/edk2/x64/OVMF.4m.fd}"
ESP_PATH="${ESP_PATH:-./esp}"

# Default QEMU args
QEMU_ARGS=(
    -bios "$OVMF_PATH"
    -drive "format=raw,file=fat:rw:$ESP_PATH"
    -serial stdio
    -no-reboot
    -display none
)

# Add network if requested
if [ "${ENABLE_NETWORK:-0}" = "1" ]; then
    QEMU_ARGS+=(-nic user,model=e1000)
fi

# Add TPM if swtpm socket exists
if [ -S "${TPM_SOCK:-}" ]; then
    QEMU_ARGS+=(
        -chardev "socket,id=chrtpm,path=$TPM_SOCK"
        -tpmdev emulator,id=tpm0,chardev=chrtpm
        -device tpm-tis,tpmdev=tpm0
    )
fi

echo "[*] Running IronAnchor in QEMU..."
echo "[*] OVMF: $OVMF_PATH"
echo "[*] ESP:  $ESP_PATH"

if [ "${USE_DXE:-0}" = "1" ]; then
    echo "[*] Mode: DXE Driver"
    cp "$ESP_PATH/EFI/IronAnchor/IronAnchorDxe.efi" "$ESP_PATH/EFI/BOOT/BOOTX64.EFI"
else
    echo "[*] Mode: EFI Application"
    cp "$ESP_PATH/EFI/IronAnchor/IronAnchor.efi" "$ESP_PATH/EFI/BOOT/BOOTX64.EFI"
fi

qemu-system-x86_64 "${QEMU_ARGS[@]}"
```

### 7.6 Testing

```bash
# Build both binaries
./scripts/build.sh

# Run as EFI Application (default)
./scripts/run-qemu.sh

# Run as DXE Driver
USE_DXE=1 ./scripts/run-qemu.sh

# Run with network
ENABLE_NETWORK=1 ./scripts/run-qemu.sh

# Run with TPM
swtpm socket --tpmstate dir=/tmp/swtpm --ctrl type=unixio,path=/tmp/swtpm.sock --tpm2 &
TPM_SOCK=/tmp/swtpm.sock ./scripts/run-qemu.sh
```

### 7.7 Deliverables

- [x] `lib.rs` with shared modules
- [x] DXE driver binary (`ironanchor_dxe.efi`)
- [x] EFI application binary (`ironanchor.efi`)
- [x] Updated build script with dual binary support
- [x] Updated run script with configuration options
- [x] Both binaries tested in QEMU

---

## File Summary After Phase 7

```
ironanchor/
├── Cargo.toml                  — 49K PE32+ binaries (app + DXE)
├── rust-toolchain.toml
├── src/
│   ├── lib.rs                  — shared modules (library crate)
│   ├── main.rs                 — EFI Application entry point
│   ├── bin/
│   │   └── ironanchor_dxe.rs  — DXE Driver entry point
│   ├── smbios/
│   │   ├── mod.rs              — SMBIOS table location
│   │   ├── parser.rs           — structure iterator + string extraction
│   │   ├── types.rs            — structure definitions
│   │   └── display.rs          — console output
│   ├── identity/
│   │   ├── mod.rs
│   │   ├── fingerprint.rs      — raw material collection (includes EFI GUID)
│   │   ├── hash.rs             — SHA256 computation
│   │   └── display.rs          — identity output
│   ├── efivars/
│   │   ├── mod.rs              — EFI Variable read/write
│   │   └── types.rs            — vendor GUID constant
│   ├── tpm/
│   │   ├── mod.rs              — TCG2 protocol access, PCR reading
│   │   ├── types.rs            — TpmInfo struct
│   │   └── display.rs          — console output
│   └── network/
│       ├── mod.rs              — SimpleNetwork protocol access
│       ├── types.rs            — NetworkInfo struct
│       └── display.rs          — console output
├── esp/EFI/
│   ├── IronAnchor/
│   │   ├── IronAnchor.efi      — EFI Application
│   │   └── IronAnchorDxe.efi   — DXE Driver
│   └── BOOT/
│       └── BOOTX64.EFI         — Fallback boot
└── scripts/
    ├── build.sh                — dual binary build
    └── run-qemu.sh             — configurable QEMU launcher
```
