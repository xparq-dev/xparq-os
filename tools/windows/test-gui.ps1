# Quick GUI test
Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 1

$env:PATH += ";C:\Program Files\qemu"

qemu-system-x86_64 -drive format=raw,file=build/x86-64/disk.img -no-reboot -m 128M
