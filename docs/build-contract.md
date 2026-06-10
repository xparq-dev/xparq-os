# XPARQ OS Build Artifact Contract (Phase B)

This document is the source of truth for build outputs used by scripts, Make targets, and run commands.

## Contract Scope

- Workspace root builds only (`Cargo.toml` at repo root).
- Active kernel crate: `xparq-kernel`.
- Active bootloader crates: `xparq-bootloader-arm64`, `xparq-bootloader-x86_64`.

## Canonical Outputs

| Architecture | Canonical Output | Preferred Cargo Artifact | Fallback Artifact |
|---|---|---|---|
| ARM64 | `build/arm64/kernel.bin` | `target/aarch64-unknown-none/<profile>/xparq_kernel` | `target/aarch64-unknown-none/<profile>/libxparq_kernel.a` |
| ARM64 | `build/arm64/bootloader.bin` | `target/aarch64-unknown-none/<profile>/libxparq_bootloader_arm64.a` | none |
| x86-64 | `build/x86-64/kernel.bin` | `target/x86_64-unknown-none/<profile>/xparq_kernel` | `target/x86_64-unknown-none/<profile>/libxparq_kernel.a` |
| x86-64 | `build/x86-64/bootloader.bin` | `target/x86_64-unknown-none/<profile>/xparq-bootloader-x86_64` | `target/x86_64-unknown-none/<profile>/libxparq_bootloader_x86_64.a` |
| x86-64 | `build/x86-64/disk.img` | derived from `build/x86-64/bootloader.bin` + padded `build/x86-64/kernel.bin` | none |

`<profile>` is `debug` or `release`.

## Canonical Entrypoints

- `make arm64`, `make x86_64`, `make run-arm64`, `make run-x86_64`
- `tools/build-arm64.sh`, `tools/build-x86_64.sh`, `tools/build.sh`
- `tools/build-arm64.ps1`, `tools/build-x86_64.ps1`

`make run-arm64` must run QEMU with the canonical `build/arm64/kernel.bin` output.
`make run-x86_64` must run QEMU with the canonical `build/x86-64/disk.img` output.

## Windows Legacy Helpers

Scripts under `tools/windows/` may still build raw disk images for prototype boot flows, but kernel conversion must read from:

- `target/x86_64-unknown-none/release/xparq_kernel`

Never read kernel artifacts from `kernel-simple/` in active workflows.

## Change Rule

If output names or crate entrypoints change, update this file and all canonical scripts in the same change set.
