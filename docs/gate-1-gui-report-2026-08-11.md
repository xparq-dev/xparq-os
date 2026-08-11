# Gate 1 Cooperative GUI Evidence — 2026-08-11

## Result

The Windows PowerShell/x86-64 QEMU reference path passed the interactive GUI
scenario 10/10:

```powershell
.\tools\build-x86_64.ps1 --scenario gate1-gui --repeat 10 --timeout-seconds 30
```

Each run reached `XPARQ_GUI:RUNNING`, Ring 3 `INIT_READY`, mouse movement,
terminal drag, keyboard delivery, and terminal redraw markers. QMP screenshots
before and after interaction differed. Evidence is retained in
`build/x86-64/boot.gate1-gui.log`,
`build/x86-64/build-manifest.gate1-gui.json`, and the per-run
`gate1-gui.{before,after}.run-XX.png` files.

## Safety and reproducibility

- Targeted kernel, HAL, and `init` checks run with `static_mut_refs` denied.
- The Roboto Mono 8x16 alpha atlas regenerated twice to SHA-256
  `51d23d2a2f9d10f491a895f932eac6fff83ccfccdd116e11cc969e404d0f28c5`.
- PS/2 drivers initialize in static storage rather than moving their queues on
  the bootstrap stack.
- Ring 3 syscalls and privilege-transition IRQs use a dedicated 16 KB
  frame-allocated kernel stack, and the syscall return path preserves its ABI.

## Boundary

This evidence covers the cooperative profile while the Ring 3 shell is in its
read/yield loop. Generic timer-driven Ring 3 preemption is not enabled or
claimed; user workloads that never yield can still delay desktop processing.
