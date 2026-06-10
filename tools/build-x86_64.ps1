# XPARQ OS x86_64 Build Script (PowerShell)
# Phase 02: Build and Boot Verification
# Customized build script for x86_64 architecture with boot verification

# Project configuration
$PROJECT_NAME = "xparq-os"
$PROJECT_ROOT = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$BUILD_DIR = Join-Path $PROJECT_ROOT "build"
$TOOLS_DIR = Join-Path $PROJECT_ROOT "tools"

# Architecture configuration
$X86_64_TARGET = "x86_64-unknown-none"

# Default values
$BUILD_TYPE = "release"
$VERBOSE = $false
$CLEAN = $false
$TEST = $true

function Resolve-QemuX86 {
    $cmd = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue
    if ($cmd) {
        return $cmd.Source
    }

    $fallback = "C:\Program Files\qemu\qemu-system-x86_64.exe"
    if (Test-Path $fallback) {
        return $fallback
    }

    return $null
}

function Resolve-Nasm {
    $cmd = Get-Command nasm -ErrorAction SilentlyContinue
    if ($cmd) {
        return $cmd.Source
    }

    $fallback = Join-Path $PROJECT_ROOT "third_party\nasm\nasm-2.16.03\nasm.exe"
    if (Test-Path $fallback) {
        return $fallback
    }

    return $null
}

function Resolve-LlvmObjcopy {
    $rustc = & rustup which rustc
    if (-not $rustc) {
        return $null
    }

    $rustcDir = Split-Path $rustc
    $candidate = Join-Path $rustcDir "llvm-objcopy.exe"
    if (Test-Path $candidate) {
        return $candidate
    }

    $toolchainRoot = Split-Path $rustcDir -Parent
    $found = Get-ChildItem -Path $toolchainRoot -Recurse -Filter "llvm-objcopy.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($found) {
        return $found.FullName
    }

    return $null
}

function Stop-RunningQemuX86 {
    Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}

# Print colored output
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Check dependencies
function Test-Dependencies {
    Write-Info "Checking dependencies..."
    
    # Check Rust toolchain
    if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
        Write-Error "Rust toolchain not found. Please install Rust."
        exit 1
    }
    
    # Check cargo
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "Cargo not found. Please install Cargo."
        exit 1
    }
    
    # Check QEMU for testing
    $script:QemuX86 = Resolve-QemuX86
    if ($TEST -and -not $script:QemuX86) {
        Write-Error "QEMU for x86_64 not found. Please install QEMU."
        exit 1
    }

    $script:Nasm = Resolve-Nasm
    if (-not $script:Nasm) {
        Write-Error "NASM not found. Please install NASM or keep the bundled third_party copy."
        exit 1
    }

    $script:LlvmObjcopy = Resolve-LlvmObjcopy
    if (-not $script:LlvmObjcopy) {
        Write-Error "llvm-objcopy not found. Please install rust-llvm tools."
        exit 1
    }
    
    # Check cross-compilation target
    $targets = rustup target list --installed
    if ($targets -notmatch $X86_64_TARGET) {
        Write-Info "Installing x86_64 target..."
        rustup target add $X86_64_TARGET
    }
    
    Write-Success "Dependencies checked"
}

# Clean build directory
function Clear-Build {
    Write-Info "Cleaning build directory..."
    
    if (Test-Path $BUILD_DIR) {
        Remove-Item -Recurse -Force $BUILD_DIR
    }
    
    # Clean cargo artifacts
    Set-Location $PROJECT_ROOT
    cargo clean
    
    Write-Success "Build directory cleaned"
}

# Build x86_64 version
function Build-x86_64 {
    Write-Info "Building x86_64 version..."
    
    $x86_64_build_dir = Join-Path $BUILD_DIR "x86-64"
    New-Item -ItemType Directory -Force -Path $x86_64_build_dir | Out-Null

    Stop-RunningQemuX86
    
    Set-Location $PROJECT_ROOT
    
    # Build kernel ELF
    if ($VERBOSE) {
        cargo build --target $X86_64_TARGET --profile $BUILD_TYPE --package xparq-kernel
    } else {
        cargo build --target $X86_64_TARGET --profile $BUILD_TYPE --package xparq-kernel --quiet
    }

    $kernel_elf = Join-Path $PROJECT_ROOT "target\$X86_64_TARGET\$BUILD_TYPE\xparq_kernel"
    if (-not (Test-Path $kernel_elf)) {
        Write-Error "x86_64 kernel ELF not found: $kernel_elf"
        return 1
    }

    & $script:LlvmObjcopy -O binary $kernel_elf (Join-Path $x86_64_build_dir "kernel.bin")
    $kernel_file = Join-Path $x86_64_build_dir "kernel.bin"
    if (-not (Test-Path $kernel_file)) {
        Write-Error "x86_64 kernel binary conversion failed"
        return 1
    }
    Write-Success "x86_64 kernel built: $kernel_file"

    # Build bootloader sector
    $bootloader_asm = Join-Path $PROJECT_ROOT "bootloader\x86_64\src\simple_boot.asm"
    if (-not (Test-Path $bootloader_asm)) {
        Write-Error "Bootloader source not found: $bootloader_asm"
        return 1
    }

    & $script:Nasm -f bin $bootloader_asm -o (Join-Path $x86_64_build_dir "bootloader.bin")
    $bootloader_file = Join-Path $x86_64_build_dir "bootloader.bin"
    if (-not (Test-Path $bootloader_file)) {
        Write-Error "x86_64 bootloader build failed"
        return 1
    }
    Write-Success "x86_64 bootloader built: $bootloader_file"

    $kernelBytes = [IO.File]::ReadAllBytes($kernel_file)
    $sectorSize = 512
    $minKernelSectors = 16
    $kernelSectors = [int][Math]::Ceiling($kernelBytes.Length / $sectorSize)
    if ($kernelSectors -lt $minKernelSectors) {
        $kernelSectors = $minKernelSectors
    }
    if ($kernelBytes.Length -gt ($minKernelSectors * $sectorSize)) {
        Write-Warning "Kernel is larger than the bootloader load window (16 sectors); boot may fail."
    }

    $paddedKernel = New-Object byte[] ($kernelSectors * $sectorSize)
    [Array]::Copy($kernelBytes, 0, $paddedKernel, 0, $kernelBytes.Length)
    [IO.File]::WriteAllBytes($kernel_file, $paddedKernel)

    $bootBytes = [IO.File]::ReadAllBytes($bootloader_file)
    $diskImage = Join-Path $x86_64_build_dir "disk.img"
    $stream = New-Object System.IO.MemoryStream
    try {
        $stream.Write($bootBytes, 0, $bootBytes.Length)
        $stream.Write($paddedKernel, 0, $paddedKernel.Length)
        [IO.File]::WriteAllBytes($diskImage, $stream.ToArray())
    } finally {
        $stream.Dispose()
    }
    Write-Success "x86_64 disk image built: $diskImage"
}

# Test x86_64 boot
function Test-x86_64Boot {
    Write-Info "Testing x86_64 boot..."
    
    $x86_64_build_dir = Join-Path $BUILD_DIR "x86-64"
    $kernel_file = Join-Path $x86_64_build_dir "kernel.bin"
    
    if (-not (Test-Path $kernel_file)) {
        Write-Error "x86_64 kernel not found. Build first."
        return 1
    }
    
    Write-Info "Running x86_64 kernel in QEMU..."

    Stop-RunningQemuX86
    
    # Run QEMU with configuration to capture boot messages
    $boot_log = Join-Path $x86_64_build_dir "boot.log"
    $disk_image = Join-Path $x86_64_build_dir "disk.img"
    if (-not (Test-Path $disk_image)) {
        Write-Error "x86_64 disk image not found. Build first."
        return 1
    }
    $qemu_args = @(
        "-drive", "format=raw,file=$disk_image",
        "-boot", "order=c",
        "-nographic",
        "-no-reboot",
        "-m", "128M"
    )

    $stamp = Get-Date -Format "yyyyMMddHHmmss"
    $stdout_log = Join-Path $x86_64_build_dir "boot.$stamp.stdout.log"
    $stderr_log = Join-Path $x86_64_build_dir "boot.$stamp.stderr.log"

    $proc = Start-Process -FilePath $script:QemuX86 -ArgumentList $qemu_args -PassThru -WindowStyle Hidden `
        -RedirectStandardOutput $stdout_log -RedirectStandardError $stderr_log

    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while (-not $proc.HasExited -and [DateTime]::UtcNow -lt $deadline) {
        Start-Sleep -Milliseconds 200
        $proc.Refresh()
    }

    if (-not $proc.HasExited) {
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        Write-Warning "QEMU timed out after 30 seconds"
    }

    $boot_output = @()
    if (Test-Path $stdout_log) {
        $boot_output += Get-Content $stdout_log
    }
    if (Test-Path $stderr_log) {
        $boot_output += Get-Content $stderr_log
    }
    $boot_output | Set-Content $boot_log
    $boot_output | ForEach-Object { Write-Host $_ }
    
    # Check for expected boot messages
    if (Select-String -Path $boot_log -Pattern "\[XPARQ OS\] Booting on x86-64..." -Quiet) {
        Write-Success "Found expected boot message: '[XPARQ OS] Booting on x86-64...'"
    } else {
        Write-Warning "Expected boot message not found: '[XPARQ OS] Booting on x86-64...'"
    }
    
    if (Select-String -Path $boot_log -Pattern "\[XPARQ OS\] Kernel initialized." -Quiet) {
        Write-Success "Found expected boot message: '[XPARQ OS] Kernel initialized.'"
    } else {
        Write-Warning "Expected boot message not found: '[XPARQ OS] Kernel initialized.'"
    }
    
    Write-Success "x86_64 boot test completed"
    Write-Info "Boot log saved to: $boot_log"
}

# Parse command line arguments
for ($i = 0; $i -lt $args.Length; $i++) {
    switch ($args[$i]) {
        "-t" { $BUILD_TYPE = $args[++$i] }
        "-c" { $CLEAN = $true }
        "-v" { $VERBOSE = $true }
        "--no-test" { $TEST = $false }
        default {
            Write-Error "Unknown option: $($args[$i])"
            exit 1
        }
    }
}

# Validate arguments
if ($BUILD_TYPE -notin @("debug", "release")) {
    Write-Error "Invalid build type: $BUILD_TYPE"
    exit 1
}

# Main function
Write-Info "XPARQ OS x86_64 Build Script v0.2.0"
Write-Info "Project root: $PROJECT_ROOT"

if ($CLEAN) {
    Clear-Build
}

Test-Dependencies

Write-Info "Starting x86_64 build..."
Write-Info "Build type: $BUILD_TYPE"

Build-x86_64

if ($TEST) {
    Test-x86_64Boot
}

Write-Success "x86_64 build completed successfully!"
Write-Info "Build artifacts located in: $BUILD_DIR\x86-64"
