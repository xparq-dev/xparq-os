# Build full bootloader with Protected Mode
$ErrorActionPreference = "Continue"

Write-Host "=== Building Full Bootloader (with Protected Mode) ==="

# Resolve repository root from script location
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

# 1. Build bootloader
$nasm = "$RepoRoot\third_party\nasm\nasm-2.16.03\nasm.exe"
& $nasm -f bin "$RepoRoot\bootloader\x86_64\src\boot.asm" -o "$RepoRoot\build\x86-64\bootloader.bin"
if ($LASTEXITCODE -ne 0) {
    Write-Host "NASM failed!" -ForegroundColor Red
    exit 1
}

# Check bootloader
$bl = [IO.File]::ReadAllBytes("$RepoRoot\build\x86-64\bootloader.bin")
Write-Host "Bootloader: $($bl.Length) bytes"
Write-Host "Signature: 0x$($bl[510].ToString('X2')) 0x$($bl[511].ToString('X2'))"

# 2. Convert kernel to binary
$rustc = & rustup which rustc
$rustcDir = Split-Path $rustc
$objcopy = Join-Path $rustcDir "llvm-objcopy.exe"
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
& $objcopy -O binary $kernelElf "$RepoRoot\build\x86-64\kernel.bin"

# Check kernel
$kr = [IO.File]::ReadAllBytes("$RepoRoot\build\x86-64\kernel.bin")
Write-Host "Kernel: $($kr.Length) bytes"
Write-Host "Kernel first 4 bytes: $($kr[0..3] | ForEach-Object { '0x'+$_.ToString('X2') })" -Separator " "

# 3. Create disk image
$combined = New-Object byte[] ($bl.Length + $kr.Length)
[Array]::Copy($bl, 0, $combined, 0, $bl.Length)
[Array]::Copy($kr, 0, $combined, $bl.Length, $kr.Length)
[IO.File]::WriteAllBytes("$RepoRoot\build\x86-64\disk.img", $combined)
Write-Host "Disk image: $($combined.Length) bytes ($($combined.Length/512) sectors)"

Write-Host "=== Build Complete ==="
