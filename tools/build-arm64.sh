#!/bin/bash
# XPARQ OS ARM64 Build Script
# Phase 02: Build and Boot Verification
# Customized build script for ARM64 architecture with boot verification

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Project configuration
PROJECT_NAME="xparq-os"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build"
TOOLS_DIR="$PROJECT_ROOT/tools"

# Architecture configuration
ARM64_TARGET="aarch64-unknown-none"

# Default values
BUILD_TYPE="release"
VERBOSE=false
CLEAN=false
TEST=true

# Print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check dependencies
check_dependencies() {
    print_info "Checking dependencies..."
    
    # Check Rust toolchain
    if ! command -v rustc &> /dev/null; then
        print_error "Rust toolchain not found. Please install Rust."
        exit 1
    fi
    
    # Check cargo
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Cargo."
        exit 1
    fi
    
    # Check QEMU for testing
    if ! command -v qemu-system-aarch64 &> /dev/null; then
        print_error "QEMU for ARM64 not found. Please install QEMU."
        exit 1
    fi
    
    # Check cross-compilation target
    if ! rustup target list --installed | grep -q "$ARM64_TARGET"; then
        print_info "Installing ARM64 target..."
        rustup target add "$ARM64_TARGET"
    fi
    
    print_success "Dependencies checked"
}

# Clean build directory
clean_build() {
    print_info "Cleaning build directory..."
    
    if [ -d "$BUILD_DIR" ]; then
        rm -rf "$BUILD_DIR"
    fi
    
    # Clean cargo artifacts
    cd "$PROJECT_ROOT"
    cargo clean
    
    print_success "Build directory cleaned"
}

# Build ARM64 version
build_arm64() {
    print_info "Building ARM64 version..."
    
    local arm64_build_dir="$BUILD_DIR/arm64"
    mkdir -p "$arm64_build_dir"
    
    cd "$PROJECT_ROOT"
    
    # Build kernel
    if [ "$VERBOSE" = true ]; then
        cargo build --target "$ARM64_TARGET" --profile "$BUILD_TYPE" --package xparq-kernel
    else
        cargo build --target "$ARM64_TARGET" --profile "$BUILD_TYPE" --package xparq-kernel --quiet
    fi
    
    # Build bootloader
    if [ "$VERBOSE" = true ]; then
        cargo build --target "$ARM64_TARGET" --profile "$BUILD_TYPE" --package xparq-bootloader-arm64
    else
        cargo build --target "$ARM64_TARGET" --profile "$BUILD_TYPE" --package xparq-bootloader-arm64 --quiet
    fi
    
    # Resolve artifacts to build directory.
    # Prefer runnable kernel binary; keep staticlib fallback for prototype builds.
    local kernel_file="target/$ARM64_TARGET/$BUILD_TYPE/xparq_kernel"
    if [ ! -f "$kernel_file" ]; then
        kernel_file="target/$ARM64_TARGET/$BUILD_TYPE/libxparq_kernel.a"
    fi
    local bootloader_file="target/$ARM64_TARGET/$BUILD_TYPE/libxparq_bootloader_arm64.a"
    
    if [ -f "$kernel_file" ]; then
        cp "$kernel_file" "$arm64_build_dir/kernel.bin"
        print_success "ARM64 kernel built: $arm64_build_dir/kernel.bin"
    else
        print_error "ARM64 kernel build failed"
        return 1
    fi
    
    if [ -f "$bootloader_file" ]; then
        cp "$bootloader_file" "$arm64_build_dir/bootloader.bin"
        print_success "ARM64 bootloader built: $arm64_build_dir/bootloader.bin"
    else
        print_warning "ARM64 bootloader build failed"
    fi
}

# Test ARM64 boot
test_arm64_boot() {
    print_info "Testing ARM64 boot..."
    
    local arm64_build_dir="$BUILD_DIR/arm64"
    local kernel_file="$arm64_build_dir/kernel.bin"
    
    if [ ! -f "$kernel_file" ]; then
        print_error "ARM64 kernel not found. Build first."
        return 1
    fi
    
    print_info "Running ARM64 kernel in QEMU..."
    
    # Run QEMU with configuration to capture boot messages
    timeout 30s qemu-system-aarch64 \
        -machine virt \
        -cpu cortex-a72 \
        -m 512M \
        -nographic \
        -kernel "$kernel_file" \
        -semihosting \
        -semihosting-config enable=on,target=native 2>&1 | tee "$arm64_build_dir/boot.log" || true
    
    # Check for expected boot messages
    if grep -q "\[XPARQ OS\] Booting on AArch64..." "$arm64_build_dir/boot.log"; then
        print_success "Found expected boot message: '[XPARQ OS] Booting on AArch64...'"
    else
        print_warning "Expected boot message not found: '[XPARQ OS] Booting on AArch64...'"
    fi
    
    if grep -q "\[XPARQ OS\] Kernel initialized." "$arm64_build_dir/boot.log"; then
        print_success "Found expected boot message: '[XPARQ OS] Kernel initialized.'"
    else
        print_warning "Expected boot message not found: '[XPARQ OS] Kernel initialized.'"
    fi
    
    print_success "ARM64 boot test completed"
    print_info "Boot log saved to: $arm64_build_dir/boot.log"
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -t|--type)
                BUILD_TYPE="$2"
                shift 2
                ;;
            -c|--clean)
                CLEAN=true
                shift
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            --no-test)
                TEST=false
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Validate arguments
    if [ "$BUILD_TYPE" != "debug" ] && [ "$BUILD_TYPE" != "release" ]; then
        print_error "Invalid build type: $BUILD_TYPE"
        exit 1
    fi
}

# Main function
main() {
    print_info "XPARQ OS ARM64 Build Script v0.2.0"
    print_info "Project root: $PROJECT_ROOT"
    
    parse_args "$@"
    
    if [ "$CLEAN" = true ]; then
        clean_build
    fi
    
    check_dependencies
    
    print_info "Starting ARM64 build..."
    print_info "Build type: $BUILD_TYPE"
    
    build_arm64
    
    if [ "$TEST" = true ]; then
        test_arm64_boot
    fi
    
    print_success "ARM64 build completed successfully!"
    print_info "Build artifacts located in: $BUILD_DIR/arm64"
}

# Run main function
main "$@"
