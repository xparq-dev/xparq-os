# Build and test script
$ErrorActionPreference = "Continue"

Write-Host "=== XPARQ OS Build and Test ==="

# Resolve repository root from script location
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $RepoRoot

# Convert kernel to binary
$rustc = & rustup which rustc
$dir = Split-Path $rustc
$objcopy = Join-Path $dir "llvm-objcopy.exe"

Write-Host "Converting kernel using: $objcopy"
$kernelElf = "target\x86_64-unknown-none\release\xparq_kernel"
if (-not (Test-Path $kernelElf)) {
    Write-Host "Kernel ELF not found. Building xparq-kernel..."
    cargo build --target x86_64-unknown-none --release --package xparq-kernel
}
if (-not (Test-Path $kernelElf)) {
    throw "Kernel ELF still missing: $kernelElf"
}
& $objcopy -O binary $kernelElf "build\x86-64\kernel.bin"

# Check result
$kernel = [System.IO.File]::ReadAllBytes("build\x86-64\kernel.bin")
Write-Host "Kernel binary size: $($kernel.Length) bytes"
Write-Host "First 8 bytes: $($kernel[0..7] | ForEach-Object { '0x' + $_.ToString('X2') })" -Separator " "

# Create disk image
$bootloader = [System.IO.File]::ReadAllBytes("build\x86-64\bootloader.bin")
$combined = New-Object byte[] ($bootloader.Length + $kernel.Length)
[System.Array]::Copy($bootloader, 0, $combined, 0, $bootloader.Length)
[System.Array]::Copy($kernel, 0, $combined, $bootloader.Length, $kernel.Length)
[System.IO.File]::WriteAllBytes("build\x86-64\disk.img", $combined)
Write-Host "Created disk.img: $($combined.Length) bytes"

Write-Host "=== Build Complete ==="
