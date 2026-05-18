# Phase Audit - 2026-04-27

This audit records current compile and structure blockers after reviewing core files.

## Scope Reviewed

- `README.md`
- `docs/roadmap.md`
- `docs/architecture.md`
- `docs/getting-started.md`
- `kernel/` core modules
- `bootloader/` crates
- `hal/` and `interfaces/fidl/`
- Build scripts: `Makefile`, `tools/build-arm64.sh`, `tools/build-x86_64.sh`

## Verification Performed

- Command run: `cargo check -p xparq-kernel`
- Result: failed with unresolved module contracts and internal API mismatches

## High-Priority Compile Blockers

1. Memory export/API mismatch in kernel:
   - `kernel/src/syscalls.rs` imports `Vmo`, `Vmar`, `VmoFlags` from `crate::memory` but only submodule exports are available.
2. Capability module mismatch in scheduler:
   - `kernel/src/scheduler/mod.rs` references `crate::capability::*` not present in active crate root wiring.
3. Dual object/memory model conflicts:
   - `kernel/src/lib.rs` expects symbols not exported by current modules.
4. VMAR manager field mismatch:
   - `kernel/src/memory/vmar.rs` uses `self.vmos` while struct has `vmars`.
5. Error enum mismatch:
   - `MemoryError::ResourceExhausted` used but not defined in `kernel/src/memory/mod.rs`.
6. Borrow checker issues in VMO resize path:
   - mutable/immutable borrow conflicts in `kernel/src/memory/vmo.rs`.
7. Handle rights macro API drift:
   - `bits` used as field not method in `kernel/src/objects.rs`.

## Structural Issues Confirmed

- Multiple kernel entry candidates (`main_simple`, `main`, `lib`) create unclear authority.
- Bootloader implementation variants are not yet consolidated per architecture.
- Documentation previously assumed artifact name `xparq-os` while active kernel bin is `xparq_kernel`.

## Actions Completed in This Audit Cycle

1. Added status source-of-truth:
   - `docs/implementation-status.md`
2. Added step-by-step structure execution plan:
   - `docs/project-structure-plan.md`
3. Updated onboarding docs to match current reality:
   - `README.md`
   - `docs/getting-started.md`
4. Updated run/build naming consistency:
   - `Makefile` QEMU kernel artifact paths
   - `tools/build-arm64.sh` artifact resolution
   - `tools/build-x86_64.sh` artifact resolution

## Next Execution Slice (Small, Finishable)

1. Pick one active kernel entry model and lock module exports to it.
2. Fix VMAR/MemoryError symbol mismatches first (high leverage).
3. Resolve scheduler capability import contract.
4. Re-run `cargo check -p xparq-kernel` and capture delta.
