# 🛡️ Project Ghost: The Kernel Core

**Universal Multilingual Overlay for Linux Kernel Events**

Experimental kprobe-based kernel module for real-time kernel event monitoring and diagnostic purposes.

---

## ⚡ Performance Architecture

[!NOTE]  
Ghost Shell prioritizes design-level efficiency over runtime optimization.

**Core Design:**

- **Zero-Cost Abstractions**: Compile-time optimization
- **No Heap Allocation**: Fixed 64KB memory pool (`no_std`)
- **Direct Kernel Access**: C FFI bridge at native privilege
- **Constant-Time Filtering**: O(1) event processing path

**Expected Characteristics:**

- Overhead: Designed to approach native C modules
- Memory: Fixed 64KB (no runtime growth)
- Latency: O(1) in critical path

*Formal benchmarking planned for v2.0*  
*Current claims based on architectural analysis*

---

## ⚠️ Compatibility Warning

[!CAUTION]  
**NOT SUPPORTED: WSL2 (Windows Subsystem for Linux)**

This module uses ELF relocation type `R_X86_64_GOTPCREL` and Rust-for-Linux features not implemented in WSL2 kernel.

**Supported:**

- ✅ Native Ubuntu 22.04/24.04 LTS
- ✅ VMware / VirtualBox VM
- ✅ Bare-metal Linux

---

## 📸 Screenshots

**ASCII Banner & System Online & System Call Capture:**

<img width="1718" height="920" alt="ghost01" src="https://github.com/user-attachments/assets/540f396e-33eb-421f-a639-ee8c10c1ea7c" />

**Installation & System Messages:**

<img width="1718" height="920" alt="ghost02" src="https://github.com/user-attachments/assets/73fced4a-9591-4422-8171-89bc9cde22a8" />

**Module Unload:**

<img width="1718" height="920" alt="ghost03" src="https://github.com/user-attachments/assets/2189dcbe-b59d-499d-af12-c5e8efe3fe01" />

*Running on Ubuntu 24.04 LTS (VMware)*

---

## 🚀 Quick Start

### Prerequisites

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y \
    linux-headers-$(uname -r) \
    build-essential \
    clang llvm
```

**Rust Nightly:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf \
    https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Set nightly toolchain
rustup default nightly

# Install kernel development components (REQUIRED)
rustup component add rust-src
rustup target add x86_64-unknown-none

# Verify installation
rustup component list --installed | grep -E "rust-src|x86_64-unknown-none"
```

### Build & Load
```bash
# Build
make clean
make

# Load module
sudo insmod ghost_driver.ko

# Monitor output
dmesg -w | grep GHOST

# Expected output:
# [GHOST] __x64_sys_write Captured. System Online
# [GHOST] Arch: x86_64 | Mem: 64KB
```

### Unload
```bash
sudo rmmod ghost_driver
```

---

## ✨ Features

- **Zero Heap Allocation** — Fixed 64KB memory pool
- **kprobe-based Hooking** — Non-invasive syscall monitoring
- **Real-time Logging** — Live kernel event capture
- **ANSI-Aware Filtering** — Binary data/escape sequence handling
- **Architectural Efficiency** — O(1) filtering, no dynamic allocation

---

## 📊 Project Status

**Current Phase:** Proof of Concept (v1.0)

- [x] Core Rust-to-Kernel FFI bridge
- [x] kprobe syscall interception (`__x64_sys_write`)
- [x] Early-return filtering logic
- [x] Real-time event logging
- [x] Reverse Engineering: Successfully identified 8-byte Docker signature patterns (05 00 00 01...)
- [ ] Multi-language translation (planned v2.0)
- [ ] Performance benchmarking (planned v2.0)

⏸️ v2.0 Docker Alternative (On Hold) - Exploring alternative proxy layers.

🛠️ v2.0 Ubuntu Ghost-Shell Integration - **Under Review:** Evaluating Dynamic JSON Loader vs. Performance tradeoffs.

---

## 📌 Current Version: PoC v1.0

### What This Does NOW

This is a **read-only kernel monitoring tool** that:

✅ Monitors kernel syscalls via kprobe  
✅ Displays intercepted events in real-time (`dmesg`)  
✅ Requires `sudo` / root privileges  
✅ Acts as a diagnostic probe tool

### What It Does NOT Do

❌ Modify kernel behavior or syscall results  
❌ Translate kernel messages (planned v2.0)  
❌ Enforce security policies  
❌ Optimize system performance

### Typical Usage
```bash
# Install
sudo insmod ghost_driver.ko

# Monitor
dmesg -w | grep GHOST

# Output example:
# [GHOST] SUCCESS
# [GHOST] __x64_sys_write Captured. System Online
# [GHOST] Scanning System Call Entry
# [GHOST] Kernel active: sys_call_table online

# Unload
sudo rmmod ghost_driver
```

**Use Cases:**

- System diagnosis and debugging
- Kernel event tracing and analysis
- Educational/research purposes
- Security monitoring (read-only)

---

## 🔧 Troubleshooting

**Error: "Invalid module format"**
```bash
# Rebuild kernel headers
sudo apt install --reinstall linux-headers-$(uname -r)
make clean && make
```

**Error: "Unknown symbol in module"**
```bash
# Check kernel config
cat /boot/config-$(uname -r) | grep KPROBES
# Should show: CONFIG_KPROBES=y

# If missing, recompile kernel with kprobes enabled
```

**Error: "Operation not permitted"**
```bash
# Ensure you're using sudo
sudo insmod ghost_driver.ko

# Check secure boot status (may block unsigned modules)
mokutil --sb-state
```

**WSL2 Users:**

❌ This module cannot run on WSL2.  
✅ Use VMware/VirtualBox with native Ubuntu instead.

---

## 🌍 Future: Multi-language Support (v2.0)

**Vision:**  
Kernel event translation layer with expandable language mappings.

| Language | Status | Contributor |
|----------|--------|-------------|
| 🇰🇷 Korean | 🔜 Planned | - |
| 🇨🇳 Chinese | 🔜 Planned | - |
| 🇯🇵 Japanese | 🔜 Planned | - |
| 🇷🇺 Russian | 🔜 Planned | - |

*Translation framework and contribution guide coming in v2.0*

---

## 🛠️ Technical Details

**Architecture:**

- **Language:** Rust (`no_std`) + C (FFI wrapper)
- **Hook Method:** Linux kprobes API
- **Target:** `__x64_sys_write` syscall entry point
- **Memory:** Fixed 64KB pool (zero dynamic allocation)
- **Compatibility:** Linux Kernel 5.15+ (tested on 6.8)

**File Structure:**
```
ghost-shell/
├── Doc/
│   ├── KERNEL_MAP.md
│   └── RESEARCH.md
├── workspace/
│   └── src/
│       └── chrono-kernel-core/
│           ├── src/
│           │   ├── lib.rs
│           │   └── wrapper.c
│           ├── .gitignore
│           ├── Cargo.lock
│           ├── Cargo.toml
│           ├── Makefile
│           ├── build.rs
│           ├── rust-toolchain.toml
│           ├── shim.c
│           └── wrapper.h
├── DEVLOG.md
├── LICENSE
└── README.md
```

---

## 📝 License

MIT License

---

## 🚨 Disclaimer

This is an **experimental kernel module**.

- Requires root/sudo access
- May cause system instability if misused
- **NOT** intended for production environments
- Use in VM or test systems only

**Author assumes NO liability for:**

- System crashes or data loss
- Security vulnerabilities
- Any damage resulting from use

Educational and research purposes only.

---

## 🙏 Acknowledgments

Built by a Korean "Underdog", "Mad Scientist" exploring the boundaries between hardware, OS, and human language.

Inspired by the philosophy of systems architecture and the poetry of low-level programming.

*"The core trusts English; the shell speaks your mother tongue."*

---
## The Poetry of Systems

*Architect: hallispio*

**The Infernal Translator (Rust)**  
Language is the conduit of data. Within the colossal system of the OS, where English is calcified into 0s and 1s, I use the cold, precise blade of Rust to refine reality. In the safest manner possible, this infernal translator shifts the machine's tongue into human language—without a single error.

**The Glitch in Perception**  
Even in the most perfect systems, a gap exists. That split second after hardware spits out bits, but just before the OS perceives them. That void is my battlefield. I do not destroy the system; I merely pierce the gap. While the core trusts the English, the shell speaks your mother tongue.

**The Great Shift**  
I tear down the illusion of language barriers. Inside, the cold, hard logic of the machine remains. Outside, the warmth of human history and emotion flows. This is the essence of the interpreter I have designed

---

