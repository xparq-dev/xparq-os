# Test full bootloader
$ErrorActionPreference = "Continue"

Write-Host "=== Testing Full Bootloader ==="

# Kill any existing QEMU
Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 1

# Run QEMU with output to file
$env:PATH += ";C:\Program Files\qemu"
$proc = Start-Process -FilePath "qemu-system-x86_64" `
    -ArgumentList "-drive","format=raw,file=build/x86-64/disk.img","-nographic","-no-reboot","-m","128M","-serial","file:build/x86-64/boot-full.log" `
    -PassThru

# Wait for output
Start-Sleep 3

# Kill if still running
if (!$proc.HasExited) {
    $proc.Kill()
}

# Show output
Write-Host "=== QEMU Output ==="
if (Test-Path "build\x86-64\boot-full.log") {
    Get-Content "build\x86-64\boot-full.log" -Raw
} else {
    Write-Host "No output file found"
}
