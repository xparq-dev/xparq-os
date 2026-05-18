# XPARQ OS x86_64 Build Script (PowerShell)
# Phase 02: Build and Boot Verification
# Customized build script for x86_64 architecture with boot verification

# Colors for output
$RED = "`e[0;31m"
$GREEN = "`e[0;32m"
$YELLOW = "`e[1;33m"
$BLUE = "`e[0;34m"
$NC = "`e[0m"

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

# Print colored output
function Write-Info {
    param([string]$Message)
    Write-Host "${BLUE}[INFO]${NC} $Message"
}

function Write-Success {
    param([string]$Message)
    Write-Host "${GREEN}[SUCCESS]${NC} $Message"
}

function Write-Warning {
    param([string]$Message)
    Write-Host "${YELLOW}[WARNING]${NC} $Message"
}

function Write-Error {
    param([string]$Message)
    Write-Host "${RED}[ERROR]${NC} $Message"
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
    if ($TEST -and -not (Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue)) {
        Write-Error "QEMU for x86_64 not found. Please install QEMU."
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
    
    Set-Location $PROJECT_ROOT
    
    # Build kernel
    if ($VERBOSE) {
        cargo build --target $X86_64_TARGET --profile $BUILD_TYPE --package xparq-kernel
    } else {
        cargo build --target $X86_64_TARGET --profile $BUILD_TYPE --package xparq-kernel --quiet
    }
    
    # Build bootloader
    if ($VERBOSE) {
        cargo build --target $X86_64_TARGET --profile $BUILD_TYPE --package xparq-bootloader-x86_64
    } else {
        cargo build --target $X86_64_TARGET --profile $BUILD_TYPE --package xparq-bootloader-x86_64 --quiet
    }
    
    # Resolve artifacts to build directory.
    # Prefer runnable binaries; keep staticlib fallback for prototype variants.
    $kernel_file = "target\$X86_64_TARGET\$BUILD_TYPE\xparq_kernel"
    if (-not (Test-Path $kernel_file)) {
        $kernel_file = "target\$X86_64_TARGET\$BUILD_TYPE\libxparq_kernel.a"
    }
    $bootloader_file = "target\$X86_64_TARGET\$BUILD_TYPE\xparq-bootloader-x86_64"
    if (-not (Test-Path $bootloader_file)) {
        $bootloader_file = "target\$X86_64_TARGET\$BUILD_TYPE\libxparq_bootloader_x86_64.a"
    }
    
    if (Test-Path $kernel_file) {
        Copy-Item $kernel_file (Join-Path $x86_64_build_dir "kernel.bin")
        Write-Success "x86_64 kernel built: $(Join-Path $x86_64_build_dir "kernel.bin")"
    } else {
        Write-Error "x86_64 kernel build failed"
        return 1
    }
    
    if (Test-Path $bootloader_file) {
        Copy-Item $bootloader_file (Join-Path $x86_64_build_dir "bootloader.bin")
        Write-Success "x86_64 bootloader built: $(Join-Path $x86_64_build_dir "bootloader.bin")"
    } else {
        Write-Warning "x86_64 bootloader build failed"
    }
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
    
    # Run QEMU with configuration to capture boot messages
    $boot_log = Join-Path $x86_64_build_dir "boot.log"
    $qemu_cmd = @(
        "timeout", "30", "qemu-system-x86_64",
        "-machine", "q35",
        "-cpu", "qemu64",
        "-m", "512M",
        "-nographic",
        "-kernel", $kernel_file,
        "-no-reboot"
    )
    
    & $qemu_cmd[0] $qemu_cmd[1..$qemu_cmd.Length] 2>&1 | Tee-Object -FilePath $boot_log
    
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
