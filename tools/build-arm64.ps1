# XPARQ OS ARM64 Build Script (PowerShell)
# Phase 02: Build and Boot Verification
# Customized build script for ARM64 architecture with boot verification

# Project configuration
$PROJECT_NAME = "xparq-os"
$PROJECT_ROOT = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$BUILD_DIR = Join-Path $PROJECT_ROOT "build"
$TOOLS_DIR = Join-Path $PROJECT_ROOT "tools"

# Architecture configuration
$ARM64_TARGET = "aarch64-unknown-none"

# Default values
$BUILD_TYPE = "release"
$VERBOSE = $false
$CLEAN = $false
$TEST = $true

function Resolve-QemuArm64 {
    $cmd = Get-Command qemu-system-aarch64 -ErrorAction SilentlyContinue
    if ($cmd) {
        return $cmd.Source
    }

    $fallback = "C:\Program Files\qemu\qemu-system-aarch64.exe"
    if (Test-Path $fallback) {
        return $fallback
    }

    return $null
}

function Stop-RunningQemuArm64 {
    Get-Process qemu-system-aarch64 -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
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
    $script:QemuArm64 = Resolve-QemuArm64
    if ($TEST -and -not $script:QemuArm64) {
        Write-Error "QEMU for ARM64 not found. Please install QEMU."
        exit 1
    }
    
    # Check cross-compilation target
    $targets = rustup target list --installed
    if ($targets -notmatch $ARM64_TARGET) {
        Write-Info "Installing ARM64 target..."
        rustup target add $ARM64_TARGET
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

# Build ARM64 version
function Build-ARM64 {
    Write-Info "Building ARM64 version..."
    
    $arm64_build_dir = Join-Path $BUILD_DIR "arm64"
    New-Item -ItemType Directory -Force -Path $arm64_build_dir | Out-Null

    Stop-RunningQemuArm64
    
    Set-Location $PROJECT_ROOT
    
    # Build kernel
    if ($VERBOSE) {
        cargo build --target $ARM64_TARGET --profile $BUILD_TYPE --package xparq-kernel
    } else {
        cargo build --target $ARM64_TARGET --profile $BUILD_TYPE --package xparq-kernel --quiet
    }
    
    # Build bootloader
    if ($VERBOSE) {
        cargo build --target $ARM64_TARGET --profile $BUILD_TYPE --package xparq-bootloader-arm64
    } else {
        cargo build --target $ARM64_TARGET --profile $BUILD_TYPE --package xparq-bootloader-arm64 --quiet
    }
    
    # Resolve artifacts to build directory.
    # Prefer runnable kernel binary; keep staticlib fallback for prototype builds.
    $kernel_file = "target\$ARM64_TARGET\$BUILD_TYPE\xparq_kernel"
    if (-not (Test-Path $kernel_file)) {
        $kernel_file = "target\$ARM64_TARGET\$BUILD_TYPE\libxparq_kernel.a"
    }
    $bootloader_file = "target\$ARM64_TARGET\$BUILD_TYPE\libxparq_bootloader_arm64.a"
    
    if (Test-Path $kernel_file) {
        Copy-Item $kernel_file (Join-Path $arm64_build_dir "kernel.bin")
        Write-Success "ARM64 kernel built: $(Join-Path $arm64_build_dir "kernel.bin")"
    } else {
        Write-Error "ARM64 kernel build failed"
        return 1
    }
    
    if (Test-Path $bootloader_file) {
        Copy-Item $bootloader_file (Join-Path $arm64_build_dir "bootloader.bin")
        Write-Success "ARM64 bootloader built: $(Join-Path $arm64_build_dir "bootloader.bin")"
    } else {
        Write-Warning "ARM64 bootloader build failed"
    }
}

# Test ARM64 boot
function Test-ARM64Boot {
    Write-Info "Testing ARM64 boot..."
    
    $arm64_build_dir = Join-Path $BUILD_DIR "arm64"
    $kernel_file = Join-Path $arm64_build_dir "kernel.bin"
    
    if (-not (Test-Path $kernel_file)) {
        Write-Error "ARM64 kernel not found. Build first."
        return 1
    }
    
    Write-Info "Running ARM64 kernel in QEMU..."

    Stop-RunningQemuArm64
    
    # Run QEMU with configuration to capture boot messages
    $boot_log = Join-Path $arm64_build_dir "boot.log"

    $stamp = Get-Date -Format "yyyyMMddHHmmss"
    $stdout_log = Join-Path $arm64_build_dir "boot.$stamp.stdout.log"
    $stderr_log = Join-Path $arm64_build_dir "boot.$stamp.stderr.log"

    $qemu_cmdline = "-machine virt -cpu cortex-a72 -m 512M -nographic -kernel `"$kernel_file`""

    $proc = Start-Process -FilePath $script:QemuArm64 -ArgumentList $qemu_cmdline -PassThru -WindowStyle Hidden `
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
    if (Select-String -Path $boot_log -Pattern "\[XPARQ OS\] Booting on AArch64..." -Quiet) {
        Write-Success "Found expected boot message: '[XPARQ OS] Booting on AArch64...'"
    } else {
        Write-Warning "Expected boot message not found: '[XPARQ OS] Booting on AArch64...'"
    }
    
    if (Select-String -Path $boot_log -Pattern "\[XPARQ OS\] Kernel initialized." -Quiet) {
        Write-Success "Found expected boot message: '[XPARQ OS] Kernel initialized.'"
    } else {
        Write-Warning "Expected boot message not found: '[XPARQ OS] Kernel initialized.'"
    }
    
    Write-Success "ARM64 boot test completed"
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
Write-Info "XPARQ OS ARM64 Build Script v0.2.0"
Write-Info "Project root: $PROJECT_ROOT"

if ($CLEAN) {
    Clear-Build
}

Test-Dependencies

Write-Info "Starting ARM64 build..."
Write-Info "Build type: $BUILD_TYPE"

Build-ARM64

if ($TEST) {
    Test-ARM64Boot
}

Write-Success "ARM64 build completed successfully!"
Write-Info "Build artifacts located in: $BUILD_DIR\arm64"
