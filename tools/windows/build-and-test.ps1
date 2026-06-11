# Build and test script
$ErrorActionPreference = "Continue"

Write-Host "=== XPARQ OS Build and Test ==="

# Resolve repository root from script location
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $RepoRoot

# First build the bootloader with NASM
Write-Host "Building bootloader..."
$nasm = "$RepoRoot\third_party\nasm\nasm-2.16.03\nasm.exe"
& $nasm -f bin "$RepoRoot\bootloader\x86_64\src\boot.asm" -o "$RepoRoot\build\x86-64\bootloader.bin"
if ($LASTEXITCODE -ne 0) {
    throw "NASM failed to build bootloader"
}

# Now build the kernel and convert to flat binary
Write-Host "Building kernel and converting to flat binary..."
& "$RepoRoot\tools\windows\make-flat-kernel.ps1"

# Create disk image padded to 32KB (64 sectors) for BIOS compatibility
$bootloader = [System.IO.File]::ReadAllBytes("build\x86-64\bootloader.bin")
$kernel = [System.IO.File]::ReadAllBytes("build\x86-64\kernel.bin")
$targetSize = 32768
$combined = New-Object byte[] $targetSize
[System.Array]::Copy($bootloader, 0, $combined, 0, $bootloader.Length)
[System.Array]::Copy($kernel, 0, $combined, $bootloader.Length, $kernel.Length)
[System.IO.File]::WriteAllBytes("build\x86-64\disk.img", $combined)

Write-Host "Kernel binary size: $($kernel.Length) bytes"
Write-Host "Created disk.img: $($combined.Length) bytes ($($combined.Length/512) sectors)"
Write-Host "=== Build Complete ==="
