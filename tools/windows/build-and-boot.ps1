# Build and Boot XPARQ OS x86_64
Write-Host "=== XPARQ OS Build & Boot ===" -ForegroundColor Cyan

# Resolve repository root from script location
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

# 1. Build bootloader
Write-Host "Building bootloader..." -ForegroundColor Yellow
& "$RepoRoot\third_party\nasm\nasm-2.16.03\nasm.exe" -f bin "$RepoRoot\bootloader\x86_64\src\simple_boot.asm" -o "$RepoRoot\build\x86-64\bootloader.bin"
if (-not (Test-Path "$RepoRoot\build\x86-64\bootloader.bin")) {
    Write-Host "Bootloader build FAILED" -ForegroundColor Red
    exit 1
}
$bl = Get-Item "$RepoRoot\build\x86-64\bootloader.bin"
Write-Host "  Bootloader: $($bl.Length) bytes" -ForegroundColor Green
if ($bl.Length -gt 512) {
    Write-Host "  WARNING: Bootloader > 512 bytes!" -ForegroundColor Red
}

# 2. Convert kernel ELF to binary
Write-Host "Converting kernel..." -ForegroundColor Yellow
$rustc = & rustup which rustc
$rustcDir = Split-Path $rustc
$llvmObjcopy = Join-Path $rustcDir "llvm-objcopy.exe"
if (-not (Test-Path $llvmObjcopy)) {
    Write-Host "llvm-objcopy not found at $llvmObjcopy" -ForegroundColor Red
    exit 1
}
$kernelElf = "$RepoRoot\target\x86_64-unknown-none\release\xparq_kernel"
if (-not (Test-Path $kernelElf)) {
    Write-Host "Kernel ELF not found. Building xparq-kernel..." -ForegroundColor Yellow
    Push-Location $RepoRoot
    cargo build --target x86_64-unknown-none --release --package xparq-kernel
    Pop-Location
}
if (-not (Test-Path $kernelElf)) {
    Write-Host "Kernel ELF still missing: $kernelElf" -ForegroundColor Red
    exit 1
}
& $llvmObjcopy -O binary $kernelElf "$RepoRoot\build\x86-64\kernel.bin"
if (-not (Test-Path "$RepoRoot\build\x86-64\kernel.bin")) {
    Write-Host "Kernel conversion FAILED" -ForegroundColor Red
    exit 1
}
$kr = Get-Item "$RepoRoot\build\x86-64\kernel.bin"
Write-Host "  Kernel binary: $($kr.Length) bytes" -ForegroundColor Green

# 3. Create disk image (bootloader + kernel)
Write-Host "Creating disk image..." -ForegroundColor Yellow
[IO.File]::WriteAllBytes("$RepoRoot\build\x86-64\disk.img",
    [IO.File]::ReadAllBytes("$RepoRoot\build\x86-64\bootloader.bin") +
    [IO.File]::ReadAllBytes("$RepoRoot\build\x86-64\kernel.bin"))
$di = Get-Item "$RepoRoot\build\x86-64\disk.img"
Write-Host "  Disk image: $($di.Length) bytes ($([math]::Ceiling($di.Length/512)) sectors)" -ForegroundColor Green

# 4. Kill any running QEMU
Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 1

# 5. Run QEMU
Write-Host "`nBooting QEMU..." -ForegroundColor Yellow
Write-Host "=== QEMU OUTPUT ===" -ForegroundColor Cyan
$env:PATH = "C:\Program Files\qemu;" + $env:PATH
qemu-system-x86_64 -drive format=raw,file="$RepoRoot/build/x86-64/disk.img" -boot order=c -nographic -no-reboot -m 128M
Write-Host "`n=== QEMU EXITED ===" -ForegroundColor Cyan
