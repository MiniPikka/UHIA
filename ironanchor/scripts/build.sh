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
