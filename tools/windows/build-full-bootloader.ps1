# Build full bootloader with Protected Mode
$ErrorActionPreference = "Continue"

Write-Host "=== Building Full Bootloader (with Protected Mode) ==="

# Resolve repository root from script location
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

# 0. Kill existing QEMU process to release lock on disk.img
Write-Host "Killing any running QEMU processes..."
Stop-Process -Name qemu-system-x86_64 -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

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

# 2. Build kernel and convert to binary
Write-Host "Building kernel with Rust nightly..."
Push-Location $RepoRoot
cargo +nightly build --target x86_64-unknown-none --release --package xparq-kernel
Pop-Location
if ($LASTEXITCODE -ne 0) {
    Write-Host "Cargo build failed!" -ForegroundColor Red
    exit 1
}

$rustc = & rustup which --toolchain nightly rustc
$rustcDir = Split-Path $rustc
$toolchainDir = Split-Path $rustcDir
$objcopy = (Get-ChildItem -Path $toolchainDir -Filter "llvm-objcopy.exe" -Recurse | Select-Object -First 1).FullName
$kernelElf = "$RepoRoot\target\x86_64-unknown-none\release\xparq_kernel"

if (-not (Test-Path $kernelElf)) {
    Write-Host "Kernel ELF missing: $kernelElf" -ForegroundColor Red
    exit 1
}
& $objcopy -O binary $kernelElf "$RepoRoot\build\x86-64\kernel.bin"
if ($LASTEXITCODE -ne 0) {
    Write-Host "objcopy failed!" -ForegroundColor Red
    exit 1
}

# Check kernel
$kr = [IO.File]::ReadAllBytes("$RepoRoot\build\x86-64\kernel.bin")
Write-Host "Kernel: $($kr.Length) bytes"
Write-Host "Kernel first 4 bytes: $($kr[0..3] | ForEach-Object { '0x'+$_.ToString('X2') })" -Separator " "

# 3. Create disk image padded to 32 sectors (16384 bytes) to ensure BIOS reads succeed
$targetSize = 16384
$combined = New-Object byte[] $targetSize
[Array]::Copy($bl, 0, $combined, 0, $bl.Length)
[Array]::Copy($kr, 0, $combined, $bl.Length, $kr.Length)
[IO.File]::WriteAllBytes("$RepoRoot\build\x86-64\disk.img", $combined)
Write-Host "Disk image: $($combined.Length) bytes ($($combined.Length/512) sectors)"

Write-Host "=== Build Complete ==="

# 4. Auto-run in QEMU to verify
Write-Host "Running QEMU to verify boot (5 second timeout)..."
if (-not (Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue)) {
    $env:PATH += ";C:\Program Files\qemu"
}
$qemuProcess = Start-Process qemu-system-x86_64 -ArgumentList "-drive format=raw,file=$RepoRoot\build\x86-64\disk.img -nographic -no-reboot -m 128M" -PassThru -NoNewWindow
Start-Sleep -Seconds 5
Stop-Process -Id $qemuProcess.Id -Force -ErrorAction SilentlyContinue
