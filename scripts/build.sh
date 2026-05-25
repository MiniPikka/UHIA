#!/bin/bash
set -e

echo "[*] Building UHIA..."
cargo build --release --target x86_64-unknown-uefi

mkdir -p esp/EFI/UHIA esp/EFI/BOOT

# EFI Application
cp target/x86_64-unknown-uefi/release/uhia.efi esp/EFI/UHIA/UHIA.efi
cp target/x86_64-unknown-uefi/release/uhia.efi esp/EFI/BOOT/BOOTX64.EFI

# DXE Driver
cp target/x86_64-unknown-uefi/release/uhia_dxe.efi esp/EFI/UHIA/UHIA_Dxe.efi

echo "[+] Built: esp/EFI/UHIA/UHIA.efi (EFI Application)"
echo "[+] Built: esp/EFI/BOOT/BOOTX64.EFI (Fallback Boot)"
echo "[+] Built: esp/EFI/UHIA/UHIA_Dxe.efi (DXE Driver)"
