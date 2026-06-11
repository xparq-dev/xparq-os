# XPARQ OS Phase 3 Detailed Project Plan
## Hardware Abstraction Layer & Driver System

---

## Table of Contents
1. [Current State Summary](#current-state-summary)
2. [Key Terminology](#key-terminology)
3. [Workstreams Overview](#workstreams-overview)
4. [Detailed Implementation Plan](#detailed-implementation-plan)
   4.1 [Priority 1: Interrupt & Foundation Refinement](#priority-1-interrupt--foundation-refinement)
   4.2 [Priority 2: Storage Stack](#priority-2-storage-stack)
   4.3 [Priority 3: USB Host Controller & Input](#priority-3-usb-host-controller--input)
   4.4 [Priority 4: Power Management (ACPI)](#priority-4-power-management-acpi)
   4.5 [Priority 5: Connectivity (Ethernet/WiFi/Bluetooth)](#priority-5-connectivity-ethernetwifibluetooth)
   4.6 [Priority 6: Graphics & Display Pipeline](#priority-6-graphics--display-pipeline)
   4.7 [Priority 7: ARM64 HAL](#priority-7-arm64-hal)
5. [Testing Strategy](#testing-strategy)
6. [Milestone Schedule](#milestone-schedule)
7. [Risk Register](#risk-register)
8. [Definitions of Done (DoD)](#definitions-of-done-dod)

---

## Current State Summary
### What's Already Built
✅ **x86‑64 Specific Architecture Support:**
- Bootloader (NASM, real→protected→long mode with VBE support)
- PCI Express (PCIe) ECAM-based bus enumeration (`hal/src/x86_64/pci.rs`)
- 8259 PIC driver (for complete disabling, `hal/src/x86_64/pic.rs`)
- Local and I/O APIC driver (`hal/src/x86_64/apic.rs`)
- IDT with exception handlers (div by zero, page fault, etc.) (`hal/src/x86_64/idt.rs`)
- VGA text mode & VBE framebuffer display with VGA bitmap font (`hal/src/x86_64/display.rs`)
- Storage driver with RAM disk (64 MB) and ATA/IDE support (`hal/src/x86_64/storage.rs`)
- Power driver with QEMU shutdown/reboot support (`hal/src/x86_64/power.rs`)
- ACPI driver with RSDP/RSDT/XSDT/MADT parsing (`hal/src/x86_64/acpi.rs`)
- PS/2 keyboard driver with shift modifiers and Caps Lock (`hal/src/x86_64/keyboard.rs`)
- PS/2 mouse driver (`hal/src/x86_64/mouse.rs`)

✅ **Unified HAL Architecture (`hal/`):**
- Trait‑based driver framework
- Static driver instances for all core subsystems
- Display, Input, Power, Storage, Connectivity, USB, Audio, Sensors subsystems
- `DeviceManager` for driver registration/lookup
- Full `no‑std` and `no‑alloc` compatibility

✅ **Kernel Integration (`kernel/`):**
- Uses the HAL initialization path (`kernel/src/main_simple.rs`)
- PCI device enumeration displayed on VGA/VBE console and serial output
- Build script (`tools/windows/build-and-test.ps1`) that builds bootloader, kernel, and disk image for QEMU testing
- Fully bootable build produces `build/x86-64/disk.img` that runs in QEMU

### What's Missing (High Level)
⚠️ **Missing Core Interrupt Features:**
- Actual IRQ handlers for hardware interrupts (PS/2, ATA, etc.)
- Interrupt controller (IOAPIC) redirection setup
- Interrupt priority and masking management
- LAPIC timer for timekeeping

⚠️ **Missing Storage Features:**
- Real NVMe driver
- AHCI/SATA driver
- Partition table parsing (MBR/GPT)
- Simple filesystem (FAT32) read/write support

⚠️ **Missing USB Features:**
- xHCI (USB 3.x) host controller driver
- EHCI (USB 2.0) host controller driver
- UHCI (USB 1.x) host controller driver
- USB HID driver for keyboards/mice
- USB mass storage driver

⚠️ **Missing Power Management Features:**
- Complete ACPI implementation (DSDT/SSDT parsing)
- ACPI S‑states (S0–S5)
- PCIe ASPM (Active State Power Management)
- Battery and thermal sensor monitoring
- CPU C/P states

⚠️ **Missing Connectivity Features:**
- Ethernet driver (Intel I225, Realtek RTL8169)
- WiFi driver (Intel AX210)
- Bluetooth 5.3 support (LE Audio)
- Network stack

⚠️ **Missing Graphics Features:**
- Intel/AMD GPU driver
- DRM/KMS integration
- Compositor backend

⚠️ **Missing ARM64 Support:**
- Full arm64 HAL implementation
- ARM GIC (Generic Interrupt Controller)
- Mali GPU driver

---

## Key Terminology
Let’s define key terms we’ll use in this plan:
- **ECAM**: Enhanced Configuration Access Mechanism (for PCIe config space access)
- **APIC**: Advanced Programmable Interrupt Controller
- **LAPIC**: Local APIC (per‑CPU interrupt controller)
- **IOAPIC**: I/O APIC (per‑system interrupt controller for external devices)
- **GSI**: Global System Interrupt (ACPI’s abstraction for IRQs)
- **AHCI**: Advanced Host Controller Interface (for SATA drives)
- **NVMe**: Non‑Volatile Memory Express (for PCIe SSDs)
- **xHCI**: eXtensible Host Controller Interface (for USB 3.x)
- **ACPI**: Advanced Configuration and Power Interface
- **FADT**: Fixed ACPI Description Table
- **DSDT**: Differentiated System Description Table (contains AML)
- **SSDT**: Secondary System Description Tables (additional AML tables)

---

## Workstreams Overview
Phase 3 work is split into **7 workstreams**, each with clear priorities, owners, and timelines:

| Workstream | Focus | Priority | Timeline Estimate |
|---|---|---|---|
| **1. Interrupt & Foundation** | Interrupt handling, timekeeping, PCIe device init | 1 (Critical) | 2–3 months |
| **2. Storage Stack** | AHCI, NVMe, MBR/GPT, FAT32 | 2 (Critical) | 3–4 months |
| **3. USB Host & Input** | xHCI/EHCI, USB HID, USB mass storage | 3 (High) | 3–4 months |
| **4. ACPI & Power Management** | Full ACPI, S‑states, C‑states, battery/thermal | 4 (High) | 3–4 months |
| **5. Connectivity** | Ethernet, WiFi, Bluetooth, network stack | 5 (Medium) | 4–5 months |
| **6. Graphics Pipeline** | Intel/AMD GPU, DRM/KMS, compositor backend | 6 (Medium) | 5–6 months |
| **7. ARM64 HAL** | Full ARM64 driver suite (APIC equivalent is GIC, etc.) |7 (Medium)| 5–6 months |

---

## Detailed Implementation Plan

---

### Priority 1: Interrupt & Foundation Refinement
**Goal**: Have fully functional interrupt handling, timekeeping, PCIe device initialization, and a robust driver binding mechanism.

#### 1.1 Add Real IRQ Handlers
- **Subtasks**:
  - Update the IDT to map vectors 32–255 (user‑defined) for hardware IRQs
  - Add a generic IRQ handler that can dispatch to registered device drivers
  - Implement PS/2 keyboard (IRQ 1) and mouse (IRQ 12) handlers
  - Implement ATA IDE IRQ handler (IRQ 14/15 for primary/secondary channels)
  - Call `apic.eoi()` at the end of every hardware IRQ handler
  - Test in QEMU!
- **DoD**:
  - PS/2 keyboard input works in the VGA/VESA console
  - PS/2 mouse movement is detected
  - IRQ dispatching mechanism is functional
  - All code builds and tests pass in QEMU

#### 1.2 Add LAPIC Timer for Timekeeping
- **Subtasks**:
  - Initialize LAPIC timer in periodic or one‑shot mode
  - Implement a system tick counter
  - Expose timekeeping API via the HAL (`hal/power.rs` or new `hal/time.rs`)
- **DoD**:
  - Kernel can measure elapsed time with ~1 ms precision
  - Timer interrupts fire correctly and don't crash the system

#### 1.3 Improve PCIe Enumeration & Driver Binding
- **Subtasks**:
  - Add a `pci::PciDevice` struct that holds full config space and BARs
  - Add a `pci::register_driver` function that takes a vendor/product ID and a driver `impl PciDriver`
  - Auto‑bind drivers to PCI devices as they are enumerated
  - Implement a basic xHCI driver skeleton that uses this binding mechanism
- **DoD**:
  - PCI device enumeration shows more details (BARs, capabilities, etc.)
  - Driver binding mechanism can be tested with a dummy driver
  - All builds pass

---

### Priority 2: Storage Stack
**Goal**: Full storage support, including SATA/AHCI, NVMe, MBR/GPT, and FAT32 read/write.

#### 2.1 Implement AHCI/SATA Driver
- **Subtasks**:
  - Probe PCI for AHCI host controllers (`class code 0x010601`)
  - Map BARs for AHCI registers
  - Initialize HBA (Host Bus Adapter) and ports
  - Send ATA commands (IDENTIFY, READ DMA, WRITE DMA)
  - Add AHCI devices to the StorageDriver
  - Test with SATA disks in QEMU
- **DoD**:
  - QEMU SATA disks are detected and can be read/written
  - All storage driver methods work for AHCI devices
  - All unit tests (and QEMU tests) pass

#### 2.2 Implement Full NVMe Driver
- **Subtasks**:
  - Probe PCI for NVMe controllers (`class code 0x010802`)
  - Map BAR0 (MMIO space for NVMe registers)
  - Initialize NVMe controller, create admin queue, create I/O queues
  - Identify namespace, read/write logical blocks
  - Add NVMe devices to the StorageDriver
  - Test with NVMe disks in QEMU
- **DoD**:
  - QEMU NVMe disks are detected and can be read/written at full speed
  - All storage driver methods work for NVMe devices
  - All builds and tests pass

#### 2.3 Add Partition Table Parsing (MBR/GPT)
- **Subtasks**:
  - Create `hal/storage/partition.rs` module
  - Implement MBR partition table parsing
  - Implement GPT partition table parsing
  - Expose partitions as separate block devices
  - Test with MBR and GPT formatted disk images
- **DoD**:
  - Kernel can detect partitions on a disk image (both MBR and GPT)
  - Partition start/length/type are correctly reported
  - All unit tests pass

#### 2.4 Add FAT32 Read/Write Support
- **Subtasks**:
  - Create `hal/storage/fat.rs` module
  - Implement FAT32 BPB (BIOS Parameter Block) parsing
  - Implement FAT entry lookup
  - Implement directory entry traversal
  - Implement basic file read/write
  - Test with a FAT32 disk image containing known files
- **DoD**:
  - Kernel can list files in a FAT32 partition's root directory
  - Kernel can read a known file and verify its contents
  - Kernel can write a small file and read it back correctly
  - All builds and tests pass

---

### Priority 3: USB Host Controller & Input
**Goal**: Support USB keyboards/mice and USB mass storage devices.

#### 3.1 Implement xHCI Host Controller Driver (USB 3.x)
- **Subtasks**:
  - Probe PCI for xHCI host controllers (`class code 0x0c0330`)
  - Map xHCI registers
  - Initialize xHCI controller, setup device context base address array, port initialization
  - Implement simple control transfers
  - Test USB enumeration in QEMU with xHCI enabled
- **DoD**:
  - xHCI controller is detected
  - Basic control transfers work
  - All builds pass

#### 3.2 Add USB HID Driver for Keyboard and Mouse
- **Subtasks**:
  - Implement USB HID report descriptor parsing
  - Implement USB interrupt transfers for receiving HID reports
  - Integrate USB keyboard and mouse with the Input subsystem
  - Test USB keyboard/mouse in QEMU
- **DoD**:
  - USB keyboard works exactly like PS/2 keyboard in the VGA/VESA console
  - USB mouse works exactly like PS/2 mouse
  - All builds pass

#### 3.3 Add USB Mass Storage Driver (Bulk‑Only Transport)
- **Subtasks**:
  - Implement USB BOT (Bulk‑Only Transport)
  - Implement SCSI commands like TEST UNIT READY, READ CAPACITY, READ 10, WRITE 10
  - Add USB mass storage devices to the StorageDriver
  - Test in QEMU
- **DoD**:
  - USB mass storage devices are detected and can be read/written
  - All builds pass

---

### Priority 4: Power Management (ACPI)
**Goal**: Full ACPI support, including DSDT/SSDT parsing, power states, battery and thermal monitoring.

#### 4.1 Parse More ACPI Tables
- **Subtasks**:
  - Extend the ACPI driver to find and parse the FADT (Fixed ACPI Description Table)
  - Extend the ACPI driver to parse the HPET (High Precision Event Timer)
  - Extend the ACPI driver to find DSDT and SSDTs
- **DoD**:
  - FADT and HPET tables are found and parsed
  - DSDT/SSDT pointers are correctly read
  - All builds pass

#### 4.2 Add AML (ACPI Machine Language) Interpreter (Skeleton)
- **Subtasks**:
  - Create `hal/acpi/aml.rs` module
  - Implement basic AML parsing (just enough to parse _OSC/_PRT/_CRS methods)
  - Focus on PCI interrupt routing via _PRT at first
- **DoD**:
  - AML interpreter can parse simple ACPI objects
  - PCI interrupt routing (_PRT) can be parsed
  - All builds pass

#### 4.3 Add S‑State (Sleep/Wake) Support
- **Subtasks**:
  - Implement a `power_sleep` HAL function that enters S3 if possible
  - Implement a `power_wake` handler for waking up
  - Use QEMU's ACPI wake‑up support for testing
- **DoD**:
  - Kernel can enter S3 (suspend to RAM) and wake back up
  - All storage, display, and input devices resume correctly
  - All builds pass

---

### Priority 5: Connectivity (Ethernet/WiFi/Bluetooth)
**Goal**: Have working Ethernet (and eventually WiFi/Bluetooth) for network connectivity.

#### 5.1 Implement Intel I225 Ethernet Driver
- **Subtasks**:
  - Probe PCI for I225 Ethernet controllers (`class code 0x020000`)
  - Map BARs for I225 registers
  - Initialize PHY and MAC
  - Implement simple packet send/receive
  - Test with QEMU and the e1000e or virtio‑net device as a reference
- **DoD**:
  - Kernel can send an ARP packet
  - Kernel can receive packets
  - All builds pass

#### 5.2 Implement Network Stack Skeleton
- **Subtasks**:
  - Create `hal/network.rs` module
  - Implement Ethernet (Layer 2) frame handling
  - Implement ARP (Address Resolution Protocol)
  - Implement simple IPv4 with ICMP (ping support)
- **DoD**:
  - Kernel can answer pings (ICMP Echo Request)
  - All builds pass

---

### Priority 6: Graphics & Display Pipeline
**Goal**: GPU‑accelerated display, with DRM/KMS support and a compositor backend.

#### 6.1 Implement Intel GPU Driver (Basic, i915‑inspired)
- **Subtasks**:
  - Probe PCI for Intel GPU (`vendor 0x8086`, class 0x030000)
  - Map BARs for GPU registers and framebuffer
  - Set up a simple display mode (even if not accelerated yet)
- **DoD**:
  - Intel GPU is detected and initialized
  - Framebuffer is working and matches what we had before
  - All builds pass

#### 6.2 Implement DRM/KMS‑Like Abstraction
- **Subtasks**:
  - Create `hal/graphics/kms.rs` module
  - Implement connector/encoder/crtc/plane abstractions
  - Implement modesetting API
- **DoD**:
  - DRM/KMS‑style modesetting API works
  - Can change display modes at runtime
  - All builds pass

---

### Priority 7: ARM64 HAL
**Goal**: Full ARM64 implementation of the HAL, including display, input, storage, power, etc.

#### 7.1 ARM64 Bootloader and Kernel Support
- **Subtasks**:
  - Verify the existing arm64 bootloader (`bootloader/arm64`) builds correctly
  - Build an arm64 kernel target and verify it runs in QEMU ARM64
  - Test arm64 kernel entry point
- **DoD**:
  - arm64 kernel builds and boots into a simple test output in QEMU
  - All builds pass

#### 7.2 ARM64 Core Driver Implementations
- **Subtasks**:
  - Add `hal/src/arm64/display.rs` (simple framebuffer driver)
  - Add `hal/src/arm64/gic.rs` (Generic Interrupt Controller driver)
  - Add `hal/src/arm64/storage.rs` (eMMC/SDIO skeleton)
  - Add `hal/src/arm64/power.rs` (PSCI support for poweroff/reboot)
  - Update `hal/src/lib.rs` to build arm64 HAL when `arch="arm64"`
- **DoD**:
  - arm64 kernel uses the arm64 HAL and has working display/power
  - All builds pass

---

## Testing Strategy
### Testing Tiers
- **Tier 1 (Unit Tests)**: Tests individual components in isolation (in `hal/src/*/tests.rs`, `kernel/src/*/tests.rs`), using `#[cfg(test)]`.
- **Tier 2 (Integration Tests)**: Tests multiple components together (e.g., storage driver with FAT32 on NVMe) in QEMU.
- **Tier 3 (User Acceptance Tests)**: Manual or semi‑automated tests of real hardware use cases (e.g., boot to shell, play sound, connect to network, etc.).
- **Tier 4 (Performance Tests)**: Tests for latency, throughput, power consumption, and frame rate (e.g., storage read/write speed, display compositor FPS).

### Automated Testing
- **QEMU Integration Tests**: Every commit to `main` runs all tier 1 and tier 2 tests in QEMU via a GitHub Actions workflow (we can add a simple workflow in `.github/workflows`).
- **Test Scripts**: Extend `tools/windows/build-and-test.ps1` with test cases that automate common use cases (e.g., "read a test file from disk.img").

---

## Milestone Schedule
### Timeline Overview
All work will be split into **6‑month "sprints"** (Phase 3 is 1‑2 years total):
- **Months 1–6 (H1)**: Priorities 1–3 (Interrupts, Storage, USB)
- **Months 7–12 (H2)**: Priorities 4–7 (ACPI, Connectivity, Graphics, ARM64)

### Milestones
1. **M1 (Month 3)**: Interrupts are fully working, storage (RAM + ATA + AHCI) is fully functional
2. **M2 (Month 6)**: NVMe driver is functional, MBR/GPT + FAT32 are functional, USB keyboards/mice/mass storage are functional
3. **M3 (Month 9)**: Full ACPI (with S‑states) and Ethernet/network stack are functional
4. **M4 (Month 12)**: Basic GPU driver, ARM64 HAL, and WiFi/Bluetooth skeletons are functional

---

## Risk Register
| Risk | Likelihood | Impact | Mitigation Strategy |
|---|---|---|---|
| **GPU driver complexity is too high** | High | High | Start with simple framebuffer driver, use Linux driver source as reference, accept that full GPU acceleration may be Phase 3.5 |
| **WiFi/Bluetooth drivers are extremely complex** | High | High | Start with Ethernet first; use open‑source drivers (Linux iwlwifi) as a reference |
| **ACPI AML interpreter is extremely complex** | Medium | High | Start with only the parts of AML we need (_PRT for PCI interrupt routing, _S3 for suspend, etc.); use ACPICA as a reference, or use a small existing no‑std AML parser |
| **ARM64 testing is difficult without real hardware** | Medium | Medium | Use QEMU ARM64 extensively, and target a cheap, widely available dev board like the Raspberry Pi 4 or Khadas VIM3 |

---

## Definitions of Done (DoD)
### For All Driver Subtasks
- **Buildability**: All code compiles cleanly with no errors or warnings (with `--deny warnings`)
- **Testability**: Tier 1 unit tests pass, tier 2 integration tests pass
- **Readability**: Code follows project style (consistent formatting, comments for complex parts)
- **Documentation**: All public APIs are documented with doc comments (`///`)
- **No Regressions**: All existing test cases still pass

### For Each Subsystem
- **API Completeness**: Implements all methods required by the HAL trait (e.g., StorageDriver's read, write, etc.)
- **QEMU Tested**: The subsystem is tested in QEMU and works as expected
- **Error Handling**: Gracefully handles common errors (device not found, out of memory, etc.)

---

## Next Steps to Start This Plan
1. **Pick a priority 1 subtask**: Start with adding real IRQ handlers (1.1)
2. **Create a branch**: For each subtask, create a branch off `main`, implement the subtask, then open a PR for review
3. **Update this plan**: As we complete subtasks, mark them as complete!

