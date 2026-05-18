# XPARQ OS Project Structure Plan (Step-by-Step)

This plan reorganizes the repository safely in small continuous steps.
Focus: avoid future maintenance risk from duplicate ownership and unclear entrypoints.

## Principles

- Keep one source of truth per subsystem.
- Separate production paths from prototype paths.
- Make each phase executable with measurable outcomes.
- Prefer safe migration with compatibility bridges over big-bang moves.

## Phase A (Week 1): Stabilize and Label

### Goals

- Freeze current ownership and reduce ambiguity without moving major files.

### Tasks

1. Publish authoritative paths (done in `docs/implementation-status.md`).
2. Add clear labels for prototype files and legacy files.
3. Align root docs so new contributors understand actual status.

### Done Criteria

- New contributor can identify where to start in under 10 minutes.

## Phase B (Week 2): Build Contract Alignment

### Goals

- Make build and run commands consistent with actual crate outputs.

### Tasks

1. Align `Makefile` target names with Cargo outputs.
2. Align PowerShell scripts in root and `tools/` with the same artifact names.
3. Update `docs/getting-started.md` command examples to match real outputs.

### Done Criteria

- `make arm64` and `make x86_64` map to reproducible, documented outputs.

## Phase C (Week 3): Isolate Experiments and Legacy

### Goals

- Reduce noise and prevent accidental dependency on prototype code.

### Tasks

1. Move experimental assets under a dedicated area (suggested: `experiments/`).
2. Move legacy files (`*_old.rs`) into an archive path.
3. Keep temporary wrappers where needed to avoid breakage.

### Done Criteria

- Active production path and prototype path are clearly separated.

## Phase D (Week 4-5): Unify Architecture Ownership

### Goals

- Remove duplicate `arch` ownership and keep a single layering model.

### Tasks

1. Choose one canonical owner:
   - Option 1: keep `kernel/arch/*` and retire top-level `arch/*`, or
   - Option 2: keep top-level `arch/*` crates and adapt kernel integration.
2. Implement compatibility bridge first.
3. Remove duplicate implementation only after green checks.

### Done Criteria

- Exactly one architecture ownership model remains.

## Phase E (Week 6+): Subsystem Hardening

### Goals

- Move core modules from placeholder-heavy to usable baseline.

### Tasks

1. Memory: fix VMAR consistency and error variants.
2. Syscalls: replace placeholder returns with real validation flow.
3. IPC/Scheduler: integrate with active entrypoint and smoke tests.
4. Bootloaders: keep one primary implementation per architecture.

### Done Criteria

- Core boot path + minimal kernel services run with coherent module boundaries.

## Execution Rhythm (Continuous)

- Weekly: update status ledger.
- Every change set: include one structural cleanup + one runnable validation.
- No phase overloading: complete small slices end-to-end before adding scope.

## Immediate Next Steps (This Week)

1. Keep docs and onboarding aligned with current reality.
2. Add a small "authoritative map" section to root README.
3. Start build-contract audit (`Makefile` and scripts) before any file moves.
