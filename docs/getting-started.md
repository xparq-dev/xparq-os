# Getting Started with XPARQ OS

This guide provides step-by-step instructions for setting up the development environment and building XPARQ OS.

> Reality note: this repository is currently foundation/prototype stage.
> Check `docs/implementation-status.md` and `docs/project-structure-plan.md` before starting new subsystem work.

## Prerequisites

### System Requirements
- **OS**: Ubuntu 24.04 LTS (recommended) or other Linux distribution
- **Windows**: Supported through the PowerShell build scripts in `tools/`
- **CPU**: x86-64 processor with virtualization support
- **RAM**: 8GB minimum, 16GB recommended
- **Storage**: 20GB free disk space
- **Internet**: For downloading dependencies

### Required Software
- **Rust**: 1.70.0 or newer
- **QEMU**: 7.0 or newer
- **Git**: For source code management
- **Make**: Build system
- **Python 3**: For some build scripts

## Installation

### 1. Install Rust

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Install QEMU

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install qemu-system-arm qemu-system-x86 qemu-utils

# Verify installation
qemu-system-aarch64 --version
qemu-system-x86_64 --version
```

### 3. Install Development Tools

```bash
# Install additional tools
sudo apt install build-essential git make python3 python3-pip

# Install Rust targets for cross-compilation
rustup target add aarch64-unknown-none
rustup target add x86_64-unknown-none

# Install cargo tools
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

## Getting the Source Code

```bash
# Clone the repository
git clone https://github.com/xparq/xparq-os.git
cd xparq-os

# Verify the structure
ls -la
```

You should see the following structure:
```
xparq-os/
|-- .gitignore
|-- Cargo.toml
|-- Makefile
|-- README.md
|-- docs/
|-- kernel/
|-- hal/
|-- interfaces/fidl/
|-- bootloader/
|-- tools/
`-- .cargo/
```

## Building XPARQ OS

Build artifact source of truth: `docs/build-contract.md`

### Using the Makefile

The Makefile provides convenient targets for building and running XPARQ OS:

```bash
# Install all dependencies
make deps

# Build all targets
make all

# Build specific targets
make arm64      # Build ARM64 target
make x86_64     # Build x86-64 target

# Clean build artifacts
make clean

# Show help
make help
```

### Using PowerShell on Windows

Canonical Windows build scripts:

```powershell
# Build and boot test x86-64
powershell -ExecutionPolicy Bypass -File .\tools\build-x86_64.ps1

# Build and boot test ARM64
powershell -ExecutionPolicy Bypass -File .\tools\build-arm64.ps1

# Build only, skip QEMU
powershell -ExecutionPolicy Bypass -File .\tools\build-x86_64.ps1 --no-test
powershell -ExecutionPolicy Bypass -File .\tools\build-arm64.ps1 --no-test
```

### Manual Build Process

#### ARM64 Target

```bash
# Canonical script output:
# build/arm64/kernel.bin and build/arm64/bootloader.bin
powershell -ExecutionPolicy Bypass -File .\tools\build-arm64.ps1 --no-test
```

#### x86-64 Target

```bash
# Canonical script output:
# build/x86-64/kernel.bin and build/x86-64/bootloader.bin
powershell -ExecutionPolicy Bypass -File .\tools\build-x86_64.ps1 --no-test
```

## Running on QEMU

### ARM64 Emulation

```bash
# Using Makefile
make run-arm64

# Manual command
qemu-system-aarch64 \
  -machine virt \
  -cpu cortex-a72 \
  -m 512M \
  -nographic \
  -kernel build/arm64/kernel.bin
```

Expected output:
```
[XPARQ OS] Booting on AArch64...
[XPARQ OS] Kernel initialized.
```

### x86-64 Emulation

```bash
# Using Makefile
make run-x86_64

# Manual command
# Windows canonical flow uses the generated disk image:
qemu-system-x86_64 \
  -drive format=raw,file=build/x86-64/disk.img \
  -boot order=c \
  -nographic \
  -no-reboot \
  -m 128M
```

Expected output:
```
[XPARQ OS] Booting on x86-64...
[XPARQ OS] Kernel initialized.
```

## Development Workflow

### 1. Make Changes

Edit the source files in the appropriate directories:
- `kernel/src/` - Core kernel code
- `kernel/arch/arm64/` - ARM64-specific code
- `kernel/arch/x86_64/` - x86-64-specific code
- `hal/` - Hardware abstraction layer
- `interfaces/fidl/` - FIDL interface definitions

### 2. Build and Test

```bash
# Build your changes
make arm64      # or make x86_64

# Test on QEMU
make run-arm64  # or make run-x86_64
```

### Windows (PowerShell) helpers

Canonical Windows build scripts:
- `tools/build-arm64.ps1`
- `tools/build-x86_64.ps1`

Prototype raw-boot helpers remain in `tools/windows/`.

### 3. Debug

#### Using GDB

```bash
# Terminal 1: Start QEMU with GDB server
qemu-system-aarch64 -machine virt -cpu cortex-a72 -m 512M -s -S -kernel build/arm64/kernel.bin

# Terminal 2: Connect GDB
rust-gdb target/aarch64-unknown-none/release/xparq_kernel
(gdb) target remote localhost:1234
(gdb) break kernel_main
(gdb) continue
```

#### Using LLDB (for ARM64)

```bash
# Terminal 1: Start QEMU
qemu-system-aarch64 -machine virt -cpu cortex-a72 -m 512M -s -S -kernel build/arm64/kernel.bin

# Terminal 2: Connect LLDB
lldb target/aarch64-unknown-none/release/xparq_kernel
(lldb) target create --arch arm64 target/aarch64-unknown-none/release/xparq_kernel
(lldb) process connect connect://localhost:1234
(lldb) breakpoint set --name kernel_main
(lldb) continue
```

## Architecture-Specific Development

### ARM64 Development

#### Key Files
- `kernel/arch/arm64/mod.rs` - ARM64 module entry
- `kernel/arch/arm64/boot.rs` - ARM bootloader entry point
- `kernel/arch/arm64/mmu.rs` - ARM page table management
- `kernel/arch/arm64/uart.rs` - PL011 UART driver

#### ARM64 Features
- Exception Levels (EL0-EL3)
- TrustZone security
- LPAE page tables
- GICv3 interrupt controller

### x86-64 Development

#### Key Files
- `kernel/arch/x86_64/mod.rs` - x86-64 module entry
- `kernel/arch/x86_64/boot.rs` - UEFI bootloader entry point
- `kernel/arch/x86_64/mmu.rs` - x86 page table management
- `kernel/arch/x86_64/serial.rs` - COM1 serial driver

#### x86-64 Features
- Protection Rings (Ring 0-3)
- UEFI boot process
- PML4 page tables
- ACPI power management

## Troubleshooting

### Common Issues

#### Build Errors
```bash
# Clean and rebuild
make clean
make deps
make all
```

#### QEMU Errors
```bash
# Check QEMU installation
qemu-system-aarch64 --version
qemu-system-x86_64 --version

# Install missing packages
sudo apt install qemu-system-arm qemu-system-x86
```

#### Target Not Found
```bash
# Reinstall Rust targets
rustup target add aarch64-unknown-none
rustup target add x86_64-unknown-none
rustup target list --installed
```

### Getting Help

1. **Check the logs**: QEMU output often shows useful error messages
2. **Review the code**: Check the architecture-specific files
3. **Consult documentation**: Read the architecture docs in `docs/`
4. **Ask the community**: Join the development discussions

### Debug Tips

1. **Enable debug output**: Add `println!` statements for debugging
2. **Use GDB/LLDB**: Step through code with a debugger
3. **Check registers**: Verify CPU state in critical sections
4. **Memory inspection**: Use debugger to examine memory contents

## Advanced Topics

### Cross-Compilation

The project uses Rust's cross-compilation capabilities:

```toml
# In .cargo/config.toml
[target.aarch64-unknown-none]
rustflags = ["-C", "link-arg=-nostartfiles"]

[target.x86_64-unknown-none]
rustflags = ["-C", "link-arg=-nostartfiles"]
```

### Build Optimization

For release builds:

```bash
# Build optimized release
cargo build --release --target aarch64-unknown-none
cargo build --release --target x86_64-unknown-none

# Run release build
qemu-system-aarch64 -machine virt -cpu cortex-a72 -m 512M -nographic -kernel build/arm64/kernel.bin
```

### Custom QEMU Configurations

#### ARM64 with More Memory
```bash
qemu-system-aarch64 -machine virt -cpu cortex-a72 -m 1G -nographic -kernel build/arm64/kernel.bin
```

#### x86-64 with KVM
```bash
qemu-system-x86_64 -enable-kvm -nographic -kernel build/x86-64/kernel.bin
```

## Contributing

When contributing to XPARQ OS:

1. **Follow the coding standards**: Each file must have phase annotations
2. **Test on both architectures**: Ensure changes work on ARM64 and x86-64
3. **Document changes**: Update relevant documentation
4. **Use proper git workflow**: Create feature branches for changes

### Code Style

```rust
// XPARQ OS - Phase XX: <phase name>
#![no_std]

/// Function description
pub fn example_function() {
    // Implementation
}
```

## Next Steps

After getting the basic build working:

1. **Explore the codebase**: Read through the kernel source
2. **Study the architecture**: Understand the capability model
3. **Experiment with drivers**: Try adding simple drivers
4. **Contribute to FIDL**: Define new service interfaces
5. **Join the community**: Participate in development discussions

For more detailed information, see:
- [Architecture Guide](architecture.md)
- [Development Roadmap](roadmap.md)
- [API Documentation](https://docs.xparq-os.org)
