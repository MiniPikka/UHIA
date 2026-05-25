# tech-stack.md

````markdown id="bhk5vi"
# IronAnchor 技术栈设计（Tech Stack）

## 项目定位

IronAnchor 是：

```text
Firmware-Level Trusted Identity Agent
````

目标：

* 运行于 UEFI 环境
* 不依赖 Linux / Windows
* 不依赖磁盘
* 不受系统重装影响
* 提供可信硬件身份
* 支持 TPM 与 Secure Boot
* 后续支持 Remote Attestation

因此：

技术栈必须：

* 极其稳定
* 极低运行时依赖
* 适合裸机/Pre-OS
* 易于跨平台
* 能处理低层协议

---

# 1. 总体技术选型

| 层级           | 技术                    |
| ------------ | --------------------- |
| 主语言          | Rust                  |
| UEFI Runtime | rust-osdev/uefi       |
| 编译目标         | x86_64-unknown-uefi   |
| Hash         | sha2                  |
| TPM          | 自封装 TCG2              |
| 内存分配         | alloc + 自定义 allocator |
| 日志           | serial + uefi logger  |
| 构建系统         | cargo + rust-lld      |
| 固件环境         | OVMF + QEMU           |
| 调试           | GDB + QEMU Monitor    |
| 镜像生成         | cargo-make / just     |
| CI           | GitHub Actions        |

---

# 2. 为什么必须用 Rust

---

# 2.1 为什么不用 C

虽然：

UEFI 官方生态几乎全是：

```text
C + EDK2
```

但：

EDK2：

* 极其庞大
* 编译链地狱
* 宏污染严重
* Debug体验极差
* 内存安全极差

对于个人项目：

维护成本会迅速爆炸。

---

# 2.2 Rust 的核心优势

Rust 在 UEFI 场景下：

几乎是“降维打击”。

---

## 内存安全

避免：

* UAF
* Double Free
* Stack Corruption

而这些：

在固件环境里会直接：

```text
死机
```

且难以调试。

---

## 无 libc 运行

Rust 支持：

```rust
#![no_std]
```

天然适合：

* UEFI
* Bootloader
* Kernel
* Firmware

---

## 极强跨平台性

后期：

你甚至可以扩展：

| 平台           | 支持可能 |
| ------------ | ---- |
| x86_64 UEFI  | 完全支持 |
| AArch64 UEFI | 可支持  |
| RISC-V       | 后期   |

---

## Cargo 生态极强

相比 EDK2：

Rust：

```text
cargo build
```

即可。

开发效率不是一个量级。

---

# 3. 编译目标

---

# 3.1 Target Triple

使用：

```text
x86_64-unknown-uefi
```

原因：

* 官方支持
* PE32+ EFI 输出
* 不依赖 ELF
* 兼容 OVMF

---

# 3.2 编译模式

必须：

```toml
[profile.release]
panic = "abort"
lto = true
codegen-units = 1
```

原因：

* 减小 EFI 体积
* 提高稳定性
* 减少 unwinding

---

# 4. UEFI Runtime 库

---

# 4.1 uefi crate

核心库：

```text
rust-osdev/uefi
```

作用：

* Boot Services
* Runtime Services
* Protocol封装
* Console
* Memory map

这是：

当前 Rust UEFI 生态事实标准。

---

# 4.2 uefi-services

用于：

* logger
* allocator
* panic handler

开发阶段极其重要。

---

# 4.3 为什么不用 GNU-EFI

GNU-EFI：

* 很老
* C生态
* API原始
* 类型安全差

只适合：

非常传统的EFI项目。

---

# 5. TPM 技术栈

---

# 5.1 为什么不用 tss-esapi

因为：

```text
tss-esapi
```

依赖：

* Linux userspace
* tpm2-tss daemon

而你现在：

运行于：

```text
Pre-OS
```

根本不存在：

* daemon
* userspace
* POSIX

所以：

必须：

```text
直接调用 TCG2 Protocol
```

---

# 5.2 正确方案

使用：

```text
EFI_TCG2_PROTOCOL
```

自己封装：

* PCR读取
* Event Log
* TPM Capability
* NV Storage

---

# 5.3 TPM 模块设计

```text
tpm/
├── protocol.rs
├── pcr.rs
├── nvram.rs
├── quote.rs
├── ek.rs
└── error.rs
```

---

# 6. Hash 与密码学

---

# 6.1 Hash

使用：

```text
sha2
```

实现：

```text
SHA256
SHA384
```

避免：

* 自实现密码学
* OpenSSL

---

# 6.2 为什么不用 OpenSSL

OpenSSL：

* 巨大
* 不适合 no_std
* EFI环境移植困难

---

# 6.3 后期可加入

| 功能   | 库    |
| ---- | ---- |
| ECC  | p256 |
| RSA  | rsa  |
| HMAC | hmac |

---

# 7. 内存管理

---

# 7.1 为什么需要 allocator

UEFI：

只有：

```text
AllocatePool()
```

Rust：

很多结构：

* Vec
* String
* Box

需要 allocator。

---

# 7.2 推荐方案

开发阶段：

```text
uefi-services allocator
```

后期：

自定义 allocator。

---

# 7.3 为什么后期要自定义

因为：

你后面：

可能：

* DXE Driver
* Runtime Driver
* 长驻内存

必须：

更可控。

---

# 8. 日志系统

---

# 8.1 开发阶段

同时输出：

* UEFI Console
* Serial Port

---

# 8.2 串口日志

极其关键。

原因：

UEFI GUI 经常：

```text
直接卡死
```

串口才是真神。

---

# 8.3 推荐方案

QEMU：

```bash
-serial stdio
```

---

# 9. 构建系统

---

# 9.1 Cargo

必须：

```text
cargo
```

不要：

* Makefile地狱
* EDK2 BaseTools

---

# 9.2 辅助工具

推荐：

| 工具         | 用途          |
| ---------- | ----------- |
| cargo-make | Task Runner |
| just       | 简洁命令        |
| mold/lld   | 快速链接        |

---

# 10. 开发环境

---

# 10.1 QEMU + OVMF（强制）

这是：

唯一正确开发姿势。

---

# 10.2 为什么不能先上真机

UEFI开发：

非常容易：

* 黑屏
* 卡死
* Boot Loop
* NVRAM损坏

真机调试成本极高。

---

# 10.3 推荐环境

```text
QEMU
+
OVMF
+
GDB
+
Serial
```

---

# 11. OVMF

---

# 11.1 固件

使用：

```text
OVMF.fd
```

来自：

```text
edk2-ovmf
```

---

# 11.2 原因

OVMF：

* 最稳定
* 最标准
* 最适合调试

---

# 12. 文件系统

---

# 12.1 ESP

EFI程序位于：

```text
EFI System Partition
```

格式：

```text
FAT32
```

---

# 12.2 推荐目录

```text
esp/
└── EFI/
    └── IronAnchor/
        └── IronAnchor.efi
```

---

# 13. 调试技术栈

---

# 13.1 GDB

QEMU支持：

```text
-gdb tcp::1234
```

---

# 13.2 objdump

用于：

* PE分析
* 符号检查

---

# 13.3 UEFI Shell

用于：

* 手动加载EFI
* 查看变量
* 调试协议

---

# 14. 网络技术栈（后期）

---

# 14.1 第一阶段

不要做 HTTPS。

原因：

UEFI网络协议：

复杂度爆炸。

---

# 14.2 正确路线

Phase 1：

```text
HTTP
```

Phase 2：

```text
TLS
```

Phase 3：

```text
Remote Attestation
```

---

# 14.3 推荐协议

后期：

优先：

```text
EFI_HTTP_PROTOCOL
```

而不是：

手写 TCP。

---

# 15. Secure Boot 技术栈

---

# 15.1 开发阶段

关闭：

```text
Secure Boot
```

---

# 15.2 后期

使用：

| 组件     | 用途    |
| ------ | ----- |
| sbsign | EFI签名 |
| shim   | 第三方加载 |
| pesign | PE签名  |

---

# 16. CI/CD

---

# 16.1 GitHub Actions

自动：

* build
* lint
* clippy
* image generation

---

# 16.2 自动测试

QEMU 自动启动：

```text
headless boot test
```

---

# 17. 项目结构（最终版）

```text
ironanchor/
├── agent/
├── identity/
├── smbios/
├── tpm/
├── efivars/
├── protocols/
├── crypto/
├── network/
├── boot/
├── alloc/
├── logger/
├── tools/
├── tests/
├── docs/
└── scripts/
```

---

# 18. 不推荐技术

---

# 不要用 C++

原因：

* RTTI
* 异常
* ABI复杂

UEFI里：

容易炸。

---

# 不要用 OpenSSL

原因：

* 太大
* 不适合 no_std

---

# 不要直接上 DXE Driver

先：

```text
EFI Application
```

再：

```text
DXE
```

否则：

你会直接掉进固件地狱。

---

# 19. 最终推荐路线

---

# 第一阶段（必须）

实现：

* Rust EFI App
* SMBIOS读取
* SHA256身份生成

---

# 第二阶段

实现：

* EFI Variable
* TPM读取

---

# 第三阶段

实现：

* TPM NV Storage
* PCR读取

---

# 第四阶段

实现：

* 网络上传
* HTTP

---

# 第五阶段

实现：

* Secure Boot兼容
* 签名EFI

---

# 第六阶段

实现：

* Remote Attestation
* Quote
* Measured Boot

---

# 20. 核心理念

IronAnchor 的核心：

不是：

```text
“读取硬件信息”
```

而是：

```text
可信设备身份（Trusted Device Identity）
```

这是：

EDR
+
TPM
+
UEFI
+
Remote Attestation
+
Measured Boot

的交叉领域。

```
```
