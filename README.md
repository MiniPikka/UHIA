# UHIA

UEFI Hardware Identity Agent — 一个运行在 UEFI 环境中的硬件身份代理。

在操作系统启动之前，采集硬件/固件信息，生成稳定的设备身份标识，支持远程证明。

## 功能

- SMBIOS 硬件信息采集（BIOS、主板、CPU、内存）
- SHA256 设备身份指纹生成
- EFI Variable 持久化设备 GUID
- TPM2 集成（能力查询、PCR 读取）
- 网络接口检测（MAC 地址）
- 双二进制输出：EFI Application + DXE Driver

## 构建

```bash
# 安装 Rust nightly + UEFI target
rustup toolchain install nightly
rustup target add x86_64-unknown-uefi --toolchain nightly

# 构建
./scripts/build.sh

# 或手动构建
cargo build --release --target x86_64-unknown-uefi
```

## 运行

需要 QEMU + OVMF：

```bash
# Arch Linux
pacman -S qemu-full edk2-ovmf

# 运行 EFI Application
./scripts/run-qemu.sh

# 运行 DXE Driver
USE_DXE=1 ./scripts/run-qemu.sh

# 带网络
ENABLE_NETWORK=1 ./scripts/run-qemu.sh

# 带 TPM
swtpm socket --tpmstate dir=/tmp/swtpm --ctrl type=unixio,path=/tmp/swtpm.sock --tpm2 &
TPM_SOCK=/tmp/swtpm.sock ./scripts/run-qemu.sh
```

## 项目结构

```
UHIA/
├── Cargo.toml
├── rust-toolchain.toml
├── src/
│   ├── lib.rs              # 共享模块（库）
│   ├── main.rs             # EFI Application 入口
│   ├── bin/
│   │   └── uhia_dxe.rs     # DXE Driver 入口
│   ├── smbios/             # SMBIOS 表解析
│   ├── identity/           # 身份指纹生成（SHA256）
│   ├── efivars/            # EFI Variable 持久化
│   ├── tpm/                # TPM2 协议（TCG2）
│   └── network/            # 网络接口（SimpleNetwork）
├── scripts/
│   ├── build.sh
│   └── run-qemu.sh
└── esp/                    # EFI System Partition（构建后生成）
```

## 身份系统

三层身份层级：

1. **TPM Identity** — Endorsement Key 作为根设备身份（最可信）
2. **Hardware Fingerprint** — BIOS UUID + 主板序列号 + CPU 签名 + Secure Boot 状态
3. **EFI Persistent GUID** — 首次运行时生成，存储在 EFI Variable 中

身份哈希公式：

```
SHA256(bios_uuid || board_serial || cpu_signature || bios_vendor || bios_version || device_guid)
```

## 技术栈

| 组件 | 技术 |
|------|------|
| 语言 | Rust (`#![no_std]`) |
| 目标 | `x86_64-unknown-uefi` |
| UEFI | `uefi` crate 0.28 |
| 哈希 | `sha2` crate |
| 固件 | OVMF + QEMU |

## 开发约束

- **no_std 环境**：无 libc、无 OS、无文件系统
- **密码学**：仅使用 `sha2` crate，不自行实现
- **TPM**：直接封装 `EFI_TCG2_PROTOCOL`，不使用 `tss-esapi`
- **调试**：串口输出是主要调试手段

## 文档

- `design-document.md` — 系统设计与架构
- `tech-stack.md` — 技术选型与依据
- `implementation-plan.md` — 分阶段实现计划
