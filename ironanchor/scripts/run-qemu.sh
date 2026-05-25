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
