# Run XPARQ OS disk.img in QEMU (x86_64)
param(
    [switch]$Debug = $false
)

$ErrorActionPreference = "Continue"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $RepoRoot

# Build first
Write-Host "Running build..."
& powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\build-and-test.ps1"
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed, cannot boot"
    exit 1
}

Write-Host "Booting in QEMU x86_64..."

# Find QEMU (check common installation locations)
$qemuPaths = @(
    "C:\Program Files\qemu\qemu-system-x86_64.exe",
    "C:\Program Files (x86)\qemu\qemu-system-x86_64.exe"
)
$qemu = $null
foreach ($p in $qemuPaths) {
    if (Test-Path $p) {
        $qemu = $p
        break
    }
}
if (-not $qemu) {
    $qemu = Get-Command "qemu-system-x86_64" -ErrorAction SilentlyContinue
    if ($qemu) {
        $qemu = $qemu.Source
    }
}
if (-not $qemu) {
    Write-Host "ERROR: qemu-system-x86_64.exe not found!"
    Write-Host "       Please install QEMU and add it to your PATH."
    Write-Host "       Download: https://www.qemu.org/download/"
    exit 1
}

$diskImg = "$RepoRoot\build\x86-64\disk.img"

# Build QEMU args
$qemuArgs = @(
    "-drive", "file=$diskImg,format=raw,index=0,media=disk"
    "-m", "128"
    "-serial", "stdio"
    "-vga", "std"
    "-no-reboot"
)

if ($Debug) {
    $qemuArgs += @("-S", "-s")
    Write-Host "Waiting for GDB to connect (port 1234)"
}

Write-Host "Running: & '$qemu' $($qemuArgs -join ' ')"

& $qemu @qemuArgs
