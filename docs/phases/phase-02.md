# Phase 2: Dev Environment Multi-Arch

**Duration**: 3-6 months  
**Focus**: Cross-compilation pipeline, bootloaders, FIDL protocols

## Overview

Phase 2 focuses on building a complete development environment for XPARQ OS that supports both ARM64 and x86-64 architectures. This includes setting up the build system, creating architecture-specific bootloaders, and defining the core FIDL protocols that will form the foundation of the system's IPC.

## Tasks

### 2.1 Fork and Customize Fuchsia Source
- Clone Fuchsia repository and understand build system
- Rebrand build configuration to XPARQ OS
- Customize GN build files for our needs
- Set up workspace structure for multi-arch development

### 2.2 Zircon Kernel Internals Study
- Deep dive into scheduler implementation
- Study IPC endpoint and channel mechanisms
- Understand VMO system implementation details
- Analyze kernel object lifecycle management

### 2.3 Create XPARQ Component Framework
- Design component model based on FIDL
- Define component lifecycle management
- Implement component discovery and loading
- Create component sandboxing mechanisms

### 2.4 Boot on ARM Hardware
- Flash XPARQ OS to ARM development boards
- Test on Khadas VIM3 or Rock Pi hardware
- Validate ARM-specific boot sequence
- Debug hardware-specific issues

### 2.5 Boot on x86-64 PC (UEFI)
- Create UEFI bootloader for x86-64
- Implement GRUB/systemd-boot configuration
- Set up UEFI application entry point
- Parse ACPI tables for hardware discovery
- Test boot on real PC hardware or QEMU with OVMF

### 2.6 Multi-Architecture Build Pipeline
- Configure GN build targets for both architectures
- Set up cross-compilation toolchains
- Create CI/CD pipeline for automated testing
- Implement shared vs arch-specific source organization

## Master Prompts

### Multi-Arch Build System
> "I'm forking Fuchsia to XPARQ OS and need to support both aarch64 and x86-64. Explain Fuchsia GN + Ninja build system architecture for multi-architecture: toolchain definitions per-arch, board config layering, shared vs arch-specific source sets, and how to structure BUILD.gn to keep architecture-agnostic code in common/ and arch-specific code in arch/arm64/ and arch/x86/ without duplicating logic."

### UEFI Bootloader for x86-64
> "Design XPARQ OS UEFI bootloader for x86-64: EFI application entry point in Rust, ExitBootServices() transition, identity-mapped page tables setup before jump to Zircon kernel, ACPI RSDP parsing for hardware discovery, and Secure Boot chain of trust compatible with Windows Secure Boot keys - compare approach with Fuchsia's existing x86 boot path."

### FIDL Protocol Design
> "Design FIDL protocol for XPARQ OS core services: [1] xparq.system.identity - user authentication [2] xparq.sync.engine - cross-device state sync [3] xparq.display.compositor - UI rendering pipeline. Write FIDL definitions with explanation of every design decision."

## Tools and Environment

### Development Tools
- **Fuchsia SDK / ffx**: Fuchsia development tools adapted for XPARQ
- **GN + Ninja**: Google's build system for multi-arch compilation
- **Zircon GDB script**: Debugging scripts for kernel debugging
- **ARM dev board**: Physical hardware for ARM testing
- **QEMU AArch64 + x86-64**: Emulation for development
- **OVMF (UEFI firmware)**: UEFI firmware for x86-64 emulation

### Build System Components
- **GN**: Meta-build system for generating Ninja files
- **Ninja**: Fast build system for compilation
- **Cargo**: Rust package manager for workspace management
- **Cross-compilation toolchains**: aarch64-unknown-none and x86_64-unknown-none

## Implementation Details

### Build System Architecture
```
xparq-os/
|-- BUILD.gn                 # Root build configuration
|-- kernel/
|   |-- BUILD.gn             # Kernel build rules
|   |-- src/                 # Architecture-agnostic source
|   `-- arch/
|       |-- arm64/BUILD.gn    # ARM64-specific build
|       `-- x86_64/BUILD.gn   # x86-64-specific build
`-- tools/
    |-- build-arm64.sh       # ARM64 build script
    `-- build-x86_64.sh      # x86-64 build script
```

### FIDL Protocol Definitions
```fidl
// xparq.system.identity
library xparq.system.identity;

@discoverable
protocol Identity {
    Authenticate(struct {
        string username;
        string password;
        AuthMethod method;
    }) -> (struct {
        bool success;
        UserHandle? user;
        AuthToken token;
    });
    
    GetUserInfo(UserHandle user) -> (struct {
        UserInfo info;
    });
};
```

### Bootloader Architecture
- **ARM64**: Assembly entry point, stack setup, MMU enablement
- **x86-64**: UEFI application, ExitBootServices(), page table setup
- **Common**: Kernel entry point, boot information passing

## Milestone Requirements

### Technical Milestone
- **Multi-Boot Success**: XPARQ OS boots on QEMU ARM64, QEMU x86-64, and ARM dev board
- **FIDL Implementation**: Core service protocols defined and working
- **CI/CD Pipeline**: Automated build and test for both architectures

### Code Milestone
- Working bootloaders for both architectures
- Complete FIDL protocol definitions
- Automated build system
- Hardware validation on ARM boards

## Success Criteria

### Build System
- [ ] GN build system configured for both architectures
- [ ] Cross-compilation working without errors
- [ ] CI/CD pipeline building and testing automatically
- [ ] Incremental builds working efficiently

### Boot Process
- [ ] ARM64 bootloader boots kernel successfully
- [ ] x86-64 UEFI bootloader boots kernel successfully
- [ ] Both architectures boot on QEMU
- [ ] ARM64 boots on physical hardware

### FIDL System
- [ ] Core service protocols defined
- [ ] FIDL compilation working
- [ ] Basic IPC channels functional
- [ ] Protocol versioning system in place

## Challenges and Solutions

### Challenge 1: Multi-Arch Build Complexity
**Problem**: Managing different architectures in same codebase
**Solution**: Use GN's conditional compilation, separate arch-specific directories

### Challenge 2: UEFI Bootloader Complexity
**Problem**: UEFI specification is complex and x86-specific
**Solution**: Study existing Fuchsia x86 boot, use uefi crate for Rust

### Challenge 3: FIDL Protocol Design
**Problem**: Designing protocols that work across architectures
**Solution**: Focus on architecture-agnostic interfaces, use standard types

### Challenge 4: Hardware Validation
**Problem**: Testing on physical ARM hardware
**Solution**: Start with QEMU, then move to development boards

## Development Workflow

### 1. Environment Setup
```bash
# Install dependencies
make deps

# Configure build system
./tools/configure-build.sh
```

### 2. Daily Development
```bash
# Build both architectures
make all

# Test on QEMU
make run-arm64
make run-x86_64

# Run tests
make test
```

### 3. Hardware Testing
```bash
# Flash to ARM device
make flash-arm64

# Test boot sequence
make test-hardware
```

## Quality Assurance

### Automated Testing
- Unit tests for kernel components
- Integration tests for FIDL protocols
- Boot tests on both architectures
- Performance benchmarks

### Code Review
- Architecture-specific code review
- FIDL protocol design review
- Build system configuration review
- Security model validation

## Next Phase Preparation

Phase 2 prepares for Phase 3 by:
- Establishing complete development environment
- Creating working bootloaders for both architectures
- Defining core system interfaces via FIDL
- Setting up automated build and test pipeline

## Resources

### Documentation
- [Fuchsia Build System Guide](https://fuchsia.dev/fuchsia-src/development/build)
- [UEFI Specification](https://uefi.org/specifications)
- [GN Reference](https://gn.googlesource.com/gn/+/main/docs/reference.md)
- [FIDL Language Guide](https://fuchsia.dev/fuchsia-src/development/languages/fidl)

### Tools
- [ffx tool](https://fuchsia.dev/fuchsia-src/development/sdk/ffx)
- [QEMU documentation](https://www.qemu.org/docs/master/)
- [ARM development boards](https://www.khadas.com/vim3)

### Examples
- Fuchsia source code (build/, zircon/kernel/)
- UEFI application examples
- Multi-architecture Rust projects

This phase establishes the complete development infrastructure needed for building and testing XPARQ OS across multiple architectures, setting the foundation for hardware driver development in Phase 3.
