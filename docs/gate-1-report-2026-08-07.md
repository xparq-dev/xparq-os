# Gate 1 Core Evidence Report — 2026-08-07

## Result

**BOUNDED GATE 1 ACCEPTANCE SLICE PASSES.** On the current working tree, the canonical core scenario passes 10/10, automated QMP-driven keyboard/mouse delivery passes 10/10 through the guest PS/2 drivers, and the isolated page-fault scenario passes 1/1. Generic timer-driven Ring 3 preemption remains explicitly outside the verified boundary.

## Command

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\build-x86_64.ps1 --scenario gate1 -c --repeat 10 --timeout-seconds 30
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\build-x86_64.ps1 --scenario gate1-input --repeat 10 --timeout-seconds 30
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\build-x86_64.ps1 --scenario gate1-fault --timeout-seconds 30
```

The runner performs targeted bare-metal checks with `static_mut_refs` denied, runs the HAL host tests, builds and validates the image, then requires every Gate 1 marker in each QEMU run.

## Reference Environment

- Commit: `177afa347dafe9fc5b4fb8286f1422a146a0878a` plus the working-tree Gate 0/Gate 1 changes
- Rust/Cargo: 1.96.1
- NASM: 2.16.03
- QEMU: 11.0.50
- Target/configuration: `x86_64-unknown-none`, release

## Reproducible Artifacts

Two consecutive Gate 1 builds produced identical SHA-256 values.

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `bootloader.bin` | 512 | `a04763399b9ade01d40ade76f3ba47f6c12578f054c9b7801056d61646be1fe8` |
| `kernel.bin` | 295,736 | `71bcf9512dfbe4a4774461956e78adba558f1f6b003ec4ed12fe25b9d2b6ef00` |
| `fat32.img` | 35,651,584 | `59271c1024b6fdb96ea5bc4c2463dab9889f74c23fff93bb07ab80e14623b4aa` |
| `disk.img` | 36,700,160 | `a66faf3b36dee688aa75f30d3cc817311db52328ffed14941b2e7463002ddd82` |

## Verified Behavior

- Host MBR/FAT32 tests pass 3/3, including invalid signature and empty-partition cases.
- `init` enters Ring 3 and validates write return values.
- Ring 3 sleep enters the scheduler sleep queue, advances through the registered timer vector, wakes at its tick deadline, and preserves the in-flight syscall frame.
- Missing file, invalid descriptor, closed descriptor, and unknown syscall return stable negative errno values.
- `GATE1.TXT` is opened from the MBR-discovered FAT32 partition, read by exact file size, content-checked, and closed.
- A user task sends and receives a fixed-layout IPC message to itself.
- The exit syscall is entered after all assertions pass.
- Gate 0 was rerun after the sleep/fault/PS2 changes and passed its regression run; the earlier Gate 0 stability evidence remains 10/10.
- A controlled unmapped user read emits `XPARQ_TEST:FAULT:PAGE_FAULT`, CR2, and the hardware page-fault error code, then halts predictably.
- QMP injects four keyboard press/release pairs and four mouse packets per run; IRQ1/IRQ12 delivery and both driver markers are observed before Ring 3 `INIT_READY`, for 10/10 runs.
- Scenario-specific evidence is retained as `build/x86-64/build-manifest.<scenario>.json` and `build/x86-64/boot.<scenario>.log`.

## Remaining Limitations

- The timer interrupt performs EOI, tick accounting, timer processing, and wakeups, but does not invoke the existing kernel-only context switcher. Generic preemptive movement of Ring 3 or in-flight syscall frames remains unsafe and is not claimed complete; the verified user path is cooperative.
- The warning baseline remains large outside the enforced `static_mut_refs` safety rule.
- Evidence applies to the x86-64 QEMU reference profile, not ARM64 or physical hardware.
