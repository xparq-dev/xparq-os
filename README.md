# XPARQ OS

Multi-architecture operating system built on Zircon kernel with ARM and x86-64 support.

## Start Here

This repository is currently in a foundation/prototype stage. The current source of truth is:

- Workspace root: `Cargo.toml`
- Kernel entrypoint: `kernel/src/main_simple.rs`
- Kernel crate: `kernel/`
- HAL crate: `hal/`
- FIDL interfaces: `interfaces/fidl/`
- Bootloader crates: `bootloader/arm64/`, `bootloader/x86_64/`
- Build artifact contract: `docs/build-contract.md`
- Execution and structure plan: `docs/project-structure-plan.md`
- Current status ledger: `docs/implementation-status.md`
- AI handoff note: `docs/ai-handoff.md`

If you are an AI agent taking over this repository, read `docs/ai-handoff.md` first. It is the shortest path to the current truth and links to the detailed status docs.

If you are on Windows, use the PowerShell build scripts in `tools/`:

- `tools/build-arm64.ps1`
- `tools/build-x86_64.ps1`

If you are on a Unix-like environment, `make` targets are available as wrappers around the same build contract.

## Overview

XPARQ OS is a modern operating system that combines the security and capabilities of Zircon kernel with cross-device synchronization and premium user experience.

Current implementation status is tracked in `docs/implementation-status.md`.

### Key Features

- **Multi-architecture**: ARM64 (mobile/embedded) and x86-64 (PC/workstation)
- **Capability-based security**: Object-capability model from Zircon
- **Cross-device sync**: Seamless state synchronization across devices
- **Premium UI**: 120fps smooth animations with adaptive layouts
- **AI integration**: On-device intelligence with privacy preservation

## Project Structure

```
xparq-os/
|-- kernel/           # Core kernel implementation
|-- hal/              # Hardware abstraction layer
|-- interfaces/fidl/  # FIDL interface definitions
|-- bootloader/       # Architecture-specific bootloaders
|-- third_party/      # External tools and vendor binaries
|-- logs/             # Runtime and build logs
|-- tools/            # Build and development tools
|-- docs/             # Documentation
```

Authoritative interface path in this repository is `interfaces/fidl/`.

## Quick Start

### Prerequisites

- Rust 1.70+
- QEMU for emulation
- Cross-compilation toolchains

### Install Dependencies

```bash
make deps
```

### Windows helper scripts

Canonical PowerShell build scripts:

- `tools/build-arm64.ps1`
- `tools/build-x86_64.ps1`

Prototype raw-boot helpers (non-canonical for kernel build contract) remain in `tools/windows/`.

### Build and Run

#### Windows

```bash
powershell -ExecutionPolicy Bypass -File .\tools\build-x86_64.ps1
```

#### ARM64 (PowerShell)

```bash
powershell -ExecutionPolicy Bypass -File .\tools\build-arm64.ps1
```

#### Unix-like environments

```bash
make run-arm64
make run-x86_64
```

For x86-64, the runtime image is `build/x86-64/disk.img`.

### Build Targets

- `make all` - Build all targets
- `make arm64` - Build ARM64 target only
- `make x86_64` - Build x86-64 target only
- `make clean` - Clean build artifacts
- `make docs` - Build documentation

## Development Phases

### Phase 1: OS & Kernel Foundations ✅
- Zircon kernel study and customization
- Object-capability security model implementation
- Basic multi-architecture support
- Display pipeline (VGA text/VBE framebuffer on x86-64)
- Input subsystem (PS/2 keyboard/mouse)
- Power management (reboot/shutdown)
- Storage (RAM disk + ATA/IDE PIO)
- PCI enumeration
- Audio (simple)

### Phase 2: Dev Environment Multi-Arch ✅
- Cross-compilation pipeline
- Architecture-specific bootloaders (NASM x86-64)
- FIDL protocol definitions
- Full IRQ and interrupt controller support (8259 PIC + LAPIC/IOAPIC)
- LAPIC timer support
- PCI driver binding system
- Storage stack enhancements (AHCI driver skeleton, NVMe skeleton)
- Filesystem support (MBR, FAT32)

### Phase 3: Hardware Abstraction Layer (In Progress)
- Display pipeline (Mali/DRM on ARM, Intel/AMD on x86)
- Input subsystem
- Connectivity stack
- Power management
- USB host support
- More filesystems (EXT2, NTFS, etc.)
- Full GPT support

### Phase 3.5: PC Platform Layer (x86-64 only)
- Desktop shell mode
- GPU compute support
- Audio stack
- Security (TDX/SEV)
- Linux app compatibility

### Phase 4: UI & Experience System
- Design language and motion engine
- Shell implementation
- On-device AI integration

### Phase 5: Ecosystem & Sync Engine
- CRDT-based sync protocol
- Developer platform
- Cross-platform bridge

### Phase 6: Public Launch
- Beta program
- Developer documentation
- Open source community
- Hardware partnerships

## Authoritative Paths (Current)

- Workspace root: `Cargo.toml`
- Kernel entry crate: `kernel/`
- HAL crate: `hal/`
- Interface crate: `interfaces/fidl/`
- Bootloader crates: `bootloader/arm64/`, `bootloader/x86_64/`
- Build artifact contract: `docs/build-contract.md`
- Structure and execution plan: `docs/project-structure-plan.md`

## Architecture

### Kernel
- Based on Zircon microkernel
- Object-capability security model
- Virtual Memory Objects (VMO) and VMAR regions
- FIDL-based IPC

### Security
- Capability-based access control
- ARM TrustZone / x86 TDX integration
- Hardware-backed key storage
- Secure boot chain

### Multi-Architecture
- Shared kernel codebase
- Architecture-specific optimizations
- Unified FIDL interfaces
- Cross-compilation pipeline

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement changes following the coding standards
4. Test on both ARM64 and x86-64
5. Submit a pull request

## License

[License information to be added]

## Roadmap

See [docs/roadmap.md](docs/roadmap.md) for detailed development phases and milestones.

## Getting Started

See [docs/getting-started.md](docs/getting-started.md) for detailed setup and development instructions.
