# uefi-agent-design-document.md

````markdown id="q7v2lm"
# UEFI Hardware Identity Agent 设计文档

## 项目代号

IronAnchor

---

# 1. 项目目标

开发一个运行于 UEFI 环境中的硬件身份代理（UEFI Agent）。

该代理：

- 在操作系统启动前运行
- 不依赖 Linux / Windows
- 不受系统重装影响
- 可直接读取硬件与固件信息
- 可生成稳定设备身份
- 可实现远程可信上报
- 可扩展为 Secure Boot / Remote Attestation 平台

本项目本质上属于：

```text
Pre-OS Runtime Identity System
````

---

# 2. 系统定位

该系统位于：

```text
Hardware
↓
UEFI Firmware
↓
UEFI Agent   ← 本项目
↓
Bootloader
↓
Operating System
```

因此：

* 不依赖用户态
* 不依赖内核
* 不依赖文件系统
* 不依赖操作系统API

---

# 3. 核心能力

---

## 3.1 硬件信息采集

采集：

| 类别          | 来源                    |
| ----------- | --------------------- |
| CPU         | CPUID                 |
| 主板          | SMBIOS                |
| BIOS        | SMBIOS                |
| 内存          | SMBIOS                |
| TPM         | EFI_TCG2_PROTOCOL     |
| Secure Boot | EFI Variable          |
| 磁盘          | EFI_BLOCK_IO_PROTOCOL |
| MAC地址       | SNP协议                 |

---

## 3.2 固件信息采集

读取：

* BIOS Vendor
* BIOS Version
* UEFI Revision
* Secure Boot 状态
* PK/KEK/db
* TPM Capability

---

## 3.3 设备身份生成

生成：

```text
Machine Identity
```

输入：

```text
TPM EK
+
Board Serial
+
BIOS UUID
+
CPU Signature
+
SecureBoot State
```

输出：

```text
SHA256(identity_material)
```

---

## 3.4 持久化

支持：

| 方案             | 是否推荐 |
| -------------- | ---- |
| EFI Variable   | 推荐   |
| TPM NV Storage | 强烈推荐 |
| ESP隐藏文件        | 备用   |

---

## 3.5 网络上报（后期）

支持：

* UEFI TCP/IP Stack
* HTTP/HTTPS
* MQTT（后期）

---

# 4. 技术架构

---

## 4.1 总体结构

```text
+----------------------+
| UEFI Firmware        |
+----------------------+
            ↓
+----------------------+
| Boot Services        |
+----------------------+
            ↓
+----------------------+
| IronAnchor Agent     |
+----------------------+
|  SMBIOS Parser       |
|  TPM Manager         |
|  EFI Var Manager     |
|  Identity Engine     |
|  Network Reporter    |
+----------------------+
            ↓
+----------------------+
| Bootloader           |
+----------------------+
            ↓
+----------------------+
| OS                   |
+----------------------+
```

---

# 5. 启动方式设计

---

## 5.1 EFI Application

第一阶段采用：

```text
Standalone EFI Application
```

启动方式：

```text
EFI Shell
↓
IronAnchor.efi
```

优点：

* 易开发
* 易调试
* 不破坏Boot链

---

## 5.2 DXE Driver（后期）

后期升级：

```text
DXE Runtime Driver
```

直接由固件加载。

优势：

* 完全OS无关
* 更早运行
* 更隐蔽

缺点：

* 极高复杂度
* 不同主板兼容问题严重

---

# 6. 开发技术栈

---

# 6.1 推荐语言

## Rust（强烈推荐）

原因：

* 内存安全
* 无需 libc
* 非常适合 UEFI
* 社区已有成熟 crate

---

## 关键库

| 功能            | Rust Crate    |
| ------------- | ------------- |
| UEFI Runtime  | uefi          |
| UEFI Services | uefi-services |
| GUID          | guid          |
| Hash          | sha2          |
| TPM           | 自行封装          |
| 无堆分配支持        | alloc         |

---

# 7. UEFI 核心协议

---

## 7.1 SMBIOS

协议：

```text
EFI_SMBIOS_PROTOCOL
```

用于：

* 主板信息
* BIOS信息
* 内存信息

---

## 7.2 TCG2 Protocol

协议：

```text
EFI_TCG2_PROTOCOL
```

用于：

* TPM2访问
* PCR读取
* Quote
* Event Log

---

## 7.3 Variable Services

API：

```text
GetVariable()
SetVariable()
```

用于：

* 持久化身份
* Secure Boot状态读取

---

## 7.4 Simple Network Protocol

协议：

```text
EFI_SIMPLE_NETWORK_PROTOCOL
```

用于：

* MAC地址读取
* 网络通信

---

# 8. 身份系统设计

---

# 8.1 核心原则

身份必须：

* 不依赖磁盘
* 不依赖OS
* 尽量抗硬件变更
* 尽量难以伪造

---

# 8.2 身份层次

---

## Layer 1：TPM Identity

最可信。

读取：

```text
Endorsement Key
```

作为：

```text
Root Device Identity
```

---

## Layer 2：Hardware Fingerprint

包括：

* BIOS UUID
* 主板SN
* CPU Signature
* Secure Boot状态

---

## Layer 3：EFI Persistent GUID

首次运行：

生成：

```text
Device GUID
```

写入：

```text
EFI Variable
```

---

# 9. TPM设计

---

# 9.1 TPM初始化

检测：

```text
EFI_TCG2_PROTOCOL
```

若不存在：

进入：

```text
Fallback Fingerprint Mode
```

---

# 9.2 TPM持久化

创建：

```text
NV Index
```

存储：

```text
Device Secret
```

特点：

* 不随重装消失
* 不随硬盘更换消失

---

# 9.3 TPM Quote（后期）

支持：

```text
PCR Quote
```

用于：

* 验证 Secure Boot
* 验证Boot链
* Remote Attestation

---

# 10. EFI Variable设计

---

## Variable Name

```text
IronAnchorDeviceGuid
```

---

## GUID Namespace

独立Vendor GUID。

---

## 属性

```text
EFI_VARIABLE_NON_VOLATILE
EFI_VARIABLE_BOOTSERVICE_ACCESS
EFI_VARIABLE_RUNTIME_ACCESS
```

---

# 11. 网络设计（后期）

---

# 11.1 UEFI Network Stack

使用：

```text
EFI_TCP4_PROTOCOL
```

或者：

```text
EFI_HTTP_PROTOCOL
```

---

# 11.2 上传内容

```json
{
  "device_id": "...",
  "bios": "...",
  "secure_boot": true,
  "tpm": true,
  "timestamp": "..."
}
```

---

# 12. Secure Boot兼容

---

## 开发阶段

关闭：

```text
Secure Boot
```

---

## 正式阶段

需要：

* 自签名证书
* shim
* Microsoft KEK（极难）

---

# 13. 调试方案

---

# 13.1 QEMU + OVMF

推荐：

```text
QEMU
+
OVMF
```

作为主要开发环境。

---

## 启动：

```bash
qemu-system-x86_64 \
  -bios OVMF.fd \
  -drive format=raw,file=fat:rw:esp/
```

---

# 13.2 日志输出

使用：

```text
Serial Port
```

输出日志。

---

# 14. 项目目录结构

```text
ironanchor/
├── agent/
├── smbios/
├── tpm/
├── identity/
├── efivars/
├── network/
├── common/
├── tools/
└── docs/
```

---

# 15. 开发阶段规划

---

# Phase 1：UEFI Hello World

实现：

* EFI App启动
* 控制台输出

---

# Phase 2：SMBIOS读取

实现：

* BIOS读取
* 主板信息读取

---

# Phase 3：Identity Engine

实现：

* 指纹生成
* SHA256

---

# Phase 4：EFI Variable

实现：

* GUID持久化

---

# Phase 5：TPM

实现：

* TCG2
* NV Storage

---

# Phase 6：网络

实现：

* TCP/IP
* HTTPS上传

---

# Phase 7：DXE Driver

实现：

* 固件级自动运行

---

# 16. 风险与挑战

| 问题            | 难度 |
| ------------- | -- |
| Secure Boot   | 极高 |
| TPM兼容         | 高  |
| 主板兼容          | 高  |
| 网络协议          | 高  |
| HTTPS证书       | 高  |
| DXE Driver稳定性 | 极高 |

---

# 17. 最终目标

最终构建：

```text
OS-independent
Firmware-level
Trusted Identity Agent
```

实现：

* 不依赖操作系统
* 不依赖磁盘
* 不怕重装
* 具备可信身份
* 支持远程证明

的平台级硬件身份系统。

```
```
