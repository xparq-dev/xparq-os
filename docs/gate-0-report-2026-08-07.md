# Gate 0 Evidence Report — 2026-08-07

## Result

**PASS.** The canonical Windows x86-64 flow builds a validated single disk image and reaches `XPARQ_TEST:INIT_READY` in 10 of 10 QEMU runs.

## Commands

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\build-x86_64.ps1 -c --no-test
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\build-x86_64.ps1 --timeout-seconds 30 --repeat 10
```

Two independent clean build-only runs produced identical hashes. The final runtime run generated `build/x86-64/build-manifest.json` with `runtime.verified=true`, `passedRuns=10`, and `requestedRuns=10`.

## Reference Environment

- Commit: `177afa347dafe9fc5b4fb8286f1422a146a0878a` plus the working-tree Gate 0 changes documented by this report
- Rust/Cargo: 1.96.1
- NASM: 2.16.03
- QEMU: 11.0.50
- Target/configuration: `x86_64-unknown-none`, release

## Reproducible Artifacts

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `bootloader.bin` | 512 | `a04763399b9ade01d40ade76f3ba47f6c12578f054c9b7801056d61646be1fe8` |
| `kernel.bin` | 293,488 | `6e4b853c1e4719579cbbab32779eb6e4325553e93b601f960359eeecdcf16950` |
| `fat32.img` | 35,651,584 | `8474d3c84d33709a738d3f8940705bd20e7382bbea772aa9e2fa63f8ae320690` |
| `disk.img` | 36,700,160 | `5416d2e047d9c7235d0a4da7279eec45e55fec17a15911afe3f8d26a2c3f8a60` |

## Validated Behavior

- Targeted kernel, HAL, and init checks pass with `static_mut_refs` denied.
- Boot sector is exactly 512 bytes with signature `55 AA` and no code/data overlap with the MBR partition table.
- Raw kernel remains unpadded and fits the 960-sector bootloader window.
- FAT32 is embedded at LBA 2048 and contains deterministic `INIT.ELF` metadata.
- Kernel initializes HAL, mounts the MBR-discovered FAT32 partition, loads `INIT.ELF`, enters Ring 3, and emits the success marker.
- The runner stops only the QEMU process it created and distinguishes build failure, timeout, and runtime failure.

## Remaining Limitations

- Numerous non-safety compile warnings remain and should be reduced incrementally.
- Gate 0 is a QEMU reference-platform result, not real-hardware or ARM64 evidence.
- Subsystem unit and integration coverage beyond the boot scenario remains Gate 1 work.
