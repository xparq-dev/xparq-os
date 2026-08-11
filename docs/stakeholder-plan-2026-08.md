# XPARQ OS Stakeholder Delivery Plan

**Planning baseline:** 2026-08-07  
**Current stage:** x86-64 kernel prototype; Phase 3 stabilization with an early Phase 3.5 desktop demonstrator

> Current evidence (2026-08-11): Gate 0 and Gate 1 core pass 10/10, automated QMP keyboard/mouse delivery passes 10/10, and the cooperative interactive desktop passes 10/10 through cursor movement, terminal drag, Ring 3 typing, command output, and redraw. Generic timer-driven Ring 3 preemption remains outside the verified boundary.

## Executive Summary

XPARQ OS has moved beyond scaffolding. The active x86-64 path contains memory management, tasks and scheduling, syscalls, IPC, FAT32/VFS integration, networking, hardware abstraction, and a framebuffer desktop demonstrator. Kernel, HAL, and `init` currently compile for the bare-metal x86-64 target.

It is not yet a reliable operating-system baseline. The latest disk image and QEMU evidence predate the current commit, automated tests are absent, ARM64 parity is unverified, and several drivers remain partial. The immediate objective is to make the x86-64 prototype reproducible and testable before expanding features.

## Evidence Baseline

| Evidence | Status on 2026-08-07 | Interpretation |
|---|---|---|
| Repository | `main` clean and aligned with `origin/main` | No uncommitted delivery work found |
| x86-64 compile | Kernel, HAL, and `init` pass targeted `cargo check` | Type-checking passes; this is not runtime proof |
| Runtime image | Current HEAD produces a validated single disk image | Kernel is inside its load window; FAT32 begins at LBA 2048 |
| QEMU evidence | Current HEAD reaches `XPARQ_TEST:INIT_READY` in 10/10 runs | Gate 0 runtime and stability criteria pass |
| Automated tests | Three MBR/FAT32 host tests plus Gate 0/Gate 1 QEMU scenarios | Core-path confidence improved; broad coverage remains low |
| Code health | HAL has many warnings, including Rust 2024 `static mut` warnings | Safety and maintainability work is required |

## Capability Assessment

- **Verified:** checked during the 2026-08-07 review.
- **Implemented, unverified:** connected to the active path but not runtime-tested at current HEAD.
- **Partial:** meaningful code exists, but the end-to-end capability is incomplete.
- **Roadmap:** no dependable end-to-end implementation yet.

| Capability | Status | Main gap |
|---|---|---|
| x86-64 kernel compile | Verified | Runtime proof still required |
| Current x86-64 boot | Verified | Marker-based stability test passes 10/10 |
| Memory, tasks, scheduler, syscalls, IPC | Verified slice | Scheduler-backed sleep and broader concurrency remain open |
| FAT32/VFS and user `init` loading | Verified slice | Read-only known-file scenario passes; mutation is untested |
| VBE desktop and PS/2 input | Verified cooperative GUI slice | Automated cursor movement, terminal drag, Ring 3 typing and terminal redraw pass 10/10; CPU-bound user workloads may still delay pumping |
| Desktop/window demonstrator | Partial | Not yet a production shell |
| PCI/APIC/time/SMP | Partial | Coverage and hardware validation |
| ATA/AHCI/NVMe storage | Partial | Reliable read/write tests |
| E1000 and IPv4 networking | Partial | End-to-end connectivity test |
| USB/xHCI | Partial | HID reports and mass storage |
| Audio, GPU acceleration, Wi-Fi/Bluetooth | Roadmap | No end-to-end hardware capability |
| ARM64 | Unverified prototype | Rebuild and boot evidence |

## Delivery Gates

Calendar commitments should be made after Gate 0 measures the toolchain and runtime environment. Every gate produces a reproducible artifact, retained evidence, and an explicit stakeholder decision.

### Gate 0 — Reproducible Baseline

**Target window:** 1–2 weeks after resourcing and tool access are confirmed.

Outcomes:

1. Pin and document Rust, NASM, and QEMU versions.
2. Build `build/x86-64/disk.img` from current HEAD with one canonical command.
3. Add a headless QEMU smoke test with serial markers and a timeout.
4. Confirm HAL initialization, filesystem mount, and `init` start, or record the exact failure boundary.
5. Fix Rust 2024 `static mut` safety warnings on the exercised path.
6. Reconcile README, build contract, status ledger, and scripts with observed behavior.

Exit criteria:

- A clean checkout produces the documented image.
- A repeatable runner reports boot pass/fail and retains logs.
- Current-commit boot evidence exists.
- No known undefined-behavior warning remains on the exercised path.

**Exit decision:** approve Gate 1, or pause feature work to repair the baseline.

Implementation update (2026-08-07): Gate 0 is complete. Canonical image validation, machine-readable markers, current-HEAD boot to user `init`, deterministic clean builds, manifest generation, the `static_mut_refs` safety gate, and the 10/10 stability run pass. Evidence is recorded in `docs/gate-0-report-2026-08-07.md`.

### Gate 1 — Usable x86-64 Kernel Slice

**Planning estimate:** 4–8 weeks after Gate 0; re-estimate using Gate 0 evidence.

Outcomes:

1. Exercise timer interrupts and repeated keyboard/mouse input without hangs.
2. Run user-mode `init` and validate output, sleep, file open/read, exit, and IPC send/receive.
3. Mount a deterministic FAT32 image and verify a known file by content.
4. Add host-side tests for parsers/pure logic and QEMU integration scenarios.
5. Define diagnostics for page faults, invalid syscalls, missing devices, and filesystem errors.

Exit criteria:

- Ten consecutive headless boots pass one acceptance scenario.
- User mode starts, reads a known file, completes an IPC round trip, and idles predictably.
- Critical paths have automated regression evidence.

**Exit decision:** select the next product proof.

Implementation update (2026-08-07): the bounded Gate 1 acceptance slice is green. The core scenario passes 10/10 and verifies user-mode write, scheduler-queue timed sleep, errno behavior, deterministic FAT32 open/read/close, self-IPC, and exit entry. Automated keyboard/mouse delivery passes 10/10 through IRQ1/IRQ12 and both PS/2 drivers. The isolated page-fault scenario records CR2 and the hardware error code, and host MBR/FAT32 tests pass 3/3. The next scheduler-hardening item is a privilege-frame-compatible preemptive context switch; this is not claimed by Gate 1 evidence. Details are in `docs/gate-1-report-2026-08-07.md`.

### Gate 2 — Product Demonstrator

**Planning estimate:** 6–12 weeks after Gate 1, dependent on the selected proof.

Recommended default: an x86-64 QEMU desktop/network demonstrator, because it reuses the most mature existing code.

Outcomes:

1. Stable keyboard and mouse interaction with the framebuffer desktop.
2. E1000 networking with ARP and ICMP, followed by one bounded UDP or TCP scenario.
3. Deterministic storage access without test-image corruption.
4. Baselines for boot time, input latency, memory use, and network reliability.
5. A scripted demo with known limitations and recovery steps.

Exit criteria:

- A non-developer can run the demo from documented prerequisites.
- The demo repeatedly passes in the agreed QEMU configuration.
- No roadmap-only capability is presented as complete.

**Exit decision:** fund hardware enablement, ARM64 parity, or a higher-level runtime track.

## Deferred Until the Baseline Is Stable

- Full GPU acceleration and DRM/KMS-class work
- Wi-Fi, Bluetooth, UWB, and broad device support
- Complete ACPI sleep/resume and advanced power tuning
- Linux application compatibility
- On-device AI, cross-device sync, app store, and public beta
- 120 fps, sub-8 ms, adoption, partner, and user-count targets

These remain strategic options in `docs/roadmap.md`, not current commitments.

## Risks and Controls

| Risk | Near-term control |
|---|---|
| Documentation overstates capabilities | Require evidence labels and gate reports |
| No fresh boot proof | Make current-HEAD smoke testing the first Gate 0 deliverable |
| Unsafe global state and low coverage | Remove active-path safety warnings and add tests |
| Too many parallel subsystems | Limit active work to one gate and one product proof |
| Real-hardware scope expands too early | Standardize one QEMU profile first |
| ARM64 claims exceed evidence | Treat ARM64 as a separate gated workstream |

## Stakeholder Decisions Requested

1. Approve reliability-first stabilization through Gate 1.
2. Approve x86-64 QEMU as the initial acceptance environment.
3. Choose Gate 2 proof: desktop/network (recommended), storage, or ARM64 parity.
4. Agree that compile success alone is insufficient; boot artifacts and scenarios are required.
5. Assign accountable owners for kernel/runtime, HAL/drivers, and build/test infrastructure.
6. Select one real hardware target only after Gate 1 passes.

## Reporting and Definition of Done

- Weekly: report each gate criterion as passed, failed, or not attempted.
- Per change: run targeted compile checks and the smallest relevant scenario.
- Per gate: retain the artifact, logs, risk update, and decision record.
- Monthly: reconcile this plan, `implementation-status.md`, and the long-range roadmap.

A capability is done only when its active code path is identified, it builds from a clean checkout, a repeatable acceptance scenario passes, evidence is retained, failure behavior is documented, and the status ledger is updated in the same change set.
