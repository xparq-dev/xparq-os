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
| x86-64 | `build/x86-64/fat32.img` | FAT32 volume containing `INIT.ELF` | none |
| x86-64 | `build/x86-64/disk.img` | boot sector + raw kernel + FAT32 partition | none |
| x86-64 | `build/x86-64/build-manifest.json` | hashes, sizes, tool versions, and runtime evidence | none |

The runner also preserves the latest evidence for each scenario as
`build-manifest.<scenario>.json` and `boot.<scenario>.log`; the unqualified files
remain compatibility aliases for the most recently executed scenario.

`<profile>` is `debug` or `release`.

## Canonical x86-64 Entrypoint

- Windows/reference: `powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\build-x86_64.ps1`
- Build-only validation: append `--no-test`
- Stability run: append `--repeat 10`
- Gate 1 acceptance: append `--scenario gate1 --repeat 10`; this also runs the MBR/FAT32 host parser tests and builds `init` with `gate1-test`
- Automated PS/2 input: append `--scenario gate1-input --repeat 10`; the runner starts QEMU paused, connects to a per-run QMP port, injects four keyboard press/release pairs and four mouse packets, and requires both driver markers plus `INIT_READY`
- Interactive desktop: append `--scenario gate1-gui --repeat 10`; QMP moves the cursor, drags the terminal, types `help` into the Ring 3 shell, requires all four `XPARQ_TEST:GUI_*` markers, and retains before/after PNGs with hashes in the scenario manifest
- Controlled page-fault diagnostics: append `--scenario gate1-fault`; success requires the armed marker, page-fault marker, CR2, and error code
- Compatibility options: `-t debug|release`, `-c`, `-v`, `--timeout-seconds N`
- Make wrappers call this PowerShell entrypoint; set `POWERSHELL=pwsh` where PowerShell 7 is installed under that name.

The x86-64 image layout is fixed: boot sector at LBA 0, raw kernel at LBA 1 in a 960-sector load window, and FAT32 at LBA 2048. `kernel.bin` remains unpadded. The runner validates this layout before QEMU starts.

Gate 0 runtime success requires `XPARQ_TEST:INIT_READY`. Gate 1 requires all markers listed in the manifest, ending with `XPARQ_TEST:GATE1_PASS` and `XPARQ_TEST:GATE1:EXIT_ENTERED`. `gate1-input` requires `INPUT_INJECTION_READY`, repeated keyboard and mouse processing markers, and `INIT_READY`. `gate1-gui` additionally requires `XPARQ_GUI:RUNNING`, mouse, drag, keyboard, and terminal-redraw markers plus different before/after screenshots. `gate1-fault` treats the expected diagnostic halt after `XPARQ_TEST:FAULT:PAGE_FAULT` as success. Exit codes are 0 for scenario success, 1 for dependency/build/layout/host-test failure, 2 for timeout, and 3 for a failure marker or early QEMU exit.

## Windows Legacy Helpers

Scripts under `tools/windows/` and `tools/build-x86_64.sh` are legacy/prototype helpers. They are not allowed to define release artifacts or verification status. Never read kernel artifacts from `kernel-simple/` in active workflows.

## Change Rule

If output names or crate entrypoints change, update this file and all canonical scripts in the same change set.
