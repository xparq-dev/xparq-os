# Test Extended LBA bootloader
Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 1

$env:PATH = "C:\Program Files\qemu;" + $env:PATH

# Run QEMU and capture output
$output = & qemu-system-x86_64 -drive format=raw,file=build/x86-64/disk.img -nographic -no-reboot -m 128M 2>&1

Write-Host "=== QEMU OUTPUT ==="
$output | ForEach-Object { Write-Host $_ }
