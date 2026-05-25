# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**IronAnchor** is a firmware-level trusted identity agent that runs in the UEFI environment before the OS boots. It collects hardware/firmware information, generates stable device identities, and supports remote attestation — all without depending on any operating system.

- Codename: IronAnchor
- Language: Rust (`#![no_std]`)
- Target: `x86_64-unknown-uefi`
- Development environment: QEMU + OVMF

## Build & Run

```bash
cd ironanchor

# Build both binaries (EFI Application + DXE Driver)
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

# Or build manually
cargo build --release --target x86_64-unknown-uefi

# Lint
cargo clippy --target x86_64-unknown-uefi

# Format
cargo fmt
```

## Architecture

The agent runs between UEFI firmware and the bootloader in the boot chain:

```
Hardware → UEFI Firmware → Boot Services → IronAnchor Agent → Bootloader → OS
```

### Core Modules (implemented)

| Module | Status | Responsibility |
|---|---|---|
| `main.rs` | Done | Entry point, orchestration |
| `smbios/` | Done | SMBIOS table parsing (board, BIOS, memory info) |
| `identity/` | Done | Machine identity generation (SHA256 of hardware fingerprint) |
| `efivars/` | Done | EFI Variable persistence (device GUID) |
| `tpm/` | Done | TPM2 access via EFI_TCG2_PROTOCOL (capabilities, PCR reading) |
| `network/` | Done | SimpleNetwork protocol (MAC address, network state) |

### Identity System

Three-layer identity hierarchy:
1. **TPM Identity** — Endorsement Key as root device identity (most trusted)
2. **Hardware Fingerprint** — BIOS UUID + board SN + CPU signature + Secure Boot state
3. **EFI Persistent GUID** — generated on first run, stored in EFI Variable

### Key UEFI Protocols Used

- `EFI_SMBIOS_PROTOCOL` — hardware info
- `EFI_TCG2_PROTOCOL` — TPM2 access
- `GetVariable()`/`SetVariable()` — EFI Variable persistence
- `EFI_SIMPLE_NETWORK_PROTOCOL` — MAC address, network
- `EFI_BLOCK_IO_PROTOCOL` — disk info

## Critical Constraints

- **no_std environment**: No libc, no OS, no filesystem (except ESP via UEFI). Use `uefi` crate + `uefi-services`.
- **Memory**: Use `uefi-services` allocator during development. Custom allocator later for DXE driver.
- **Cryptography**: Use `sha2` crate only. Never self-implement crypto or use OpenSSL (too large for no_std).
- **TPM**: Cannot use `tss-esapi` (requires Linux userspace). Must wrap `EFI_TCG2_PROTOCOL` directly.
- **Secure Boot**: Disabled during development. Requires self-signed certs + shim for production.
- **Debugging**: Serial port output is essential — UEFI console can freeze. Use `-serial stdio` with QEMU.

## Development Phases

- [x] Phase 1: UEFI Hello World (EFI app boot + console output)
- [x] Phase 2: SMBIOS reading (BIOS, board, processor, memory info)
- [x] Phase 3: Identity Engine (fingerprint generation + SHA256)
- [x] Phase 4: EFI Variable persistence (device GUID)
- [x] Phase 5: TPM integration (TCG2 protocol, PCR reading)
- [x] Phase 6: Network reporting (SimpleNetwork, MAC address)
- [x] Phase 7: DXE Driver (dual binary: EFI app + DXE driver)

## Rust Configuration

Required in `Cargo.toml`:
```toml
[profile.release]
panic = "abort"
lto = true
codegen-units = 1
```

## References

- `design-document.md` — full system design and architecture
- `tech-stack.md` — technology choices and rationale
- `implementation-plan.md` — phased implementation plan with code examples
