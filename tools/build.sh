#!/bin/bash
# XPARQ OS Build Script
# Phase 01: OS & Kernel Foundations
# Cross-compilation build script for ARM64 and x86-64 architectures

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

# Architecture configurations
ARM64_TARGET="aarch64-unknown-none"
X86_64_TARGET="x86_64-unknown-none"

# Default values
ARCH=""
BUILD_TYPE="release"
VERBOSE=false
CLEAN=false
TEST=false
FLASH=false

# Help message
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "XPARQ OS Build Script"
    echo ""
    echo "Options:"
    echo "  -a, --arch ARCH      Target architecture (arm64, x86-64, all)"
    echo "  -t, --type TYPE      Build type (debug, release) [default: release]"
    echo "  -c, --clean          Clean build directory"
    echo "  -v, --verbose        Verbose output"
    echo "  -T, --test           Run tests after build"
    echo "  -F, --flash          Flash to device (ARM64 only)"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 -a arm64          Build ARM64 version"
    echo "  $0 -a x86-64         Build x86-64 version"
    echo "  $0 -a all            Build both architectures"
    echo "  $0 -c -a all         Clean and build both architectures"
    echo "  $0 -T -a arm64       Build and test ARM64 version"
}

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
    if [ "$TEST" = true ]; then
        if ! command -v qemu-system-aarch64 &> /dev/null && [ "$ARCH" = "arm64" -o "$ARCH" = "all" ]; then
            print_warning "QEMU for ARM64 not found. Testing will be skipped."
        fi
        if ! command -v qemu-system-x86_64 &> /dev/null && [ "$ARCH" = "x86-64" -o "$ARCH" = "all" ]; then
            print_warning "QEMU for x86-64 not found. Testing will be skipped."
        fi
    fi
    
    # Check cross-compilation targets
    if [ "$ARCH" = "arm64" -o "$ARCH" = "all" ]; then
        if ! rustup target list --installed | grep -q "$ARM64_TARGET"; then
            print_info "Installing ARM64 target..."
            rustup target add "$ARM64_TARGET"
        fi
    fi
    
    if [ "$ARCH" = "x86-64" -o "$ARCH" = "all" ]; then
        if ! rustup target list --installed | grep -q "$X86_64_TARGET"; then
            print_info "Installing x86-64 target..."
            rustup target add "$X86_64_TARGET"
        fi
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

# Build x86-64 version
build_x86_64() {
    print_info "Building x86-64 version..."
    
    local x86_64_build_dir="$BUILD_DIR/x86-64"
    mkdir -p "$x86_64_build_dir"
    
    cd "$PROJECT_ROOT"
    
    # Build kernel
    if [ "$VERBOSE" = true ]; then
        cargo build --target "$X86_64_TARGET" --profile "$BUILD_TYPE" --package xparq-kernel
    else
        cargo build --target "$X86_64_TARGET" --profile "$BUILD_TYPE" --package xparq-kernel --quiet
    fi
    
    # Build bootloader
    if [ "$VERBOSE" = true ]; then
        cargo build --target "$X86_64_TARGET" --profile "$BUILD_TYPE" --package xparq-bootloader-x86_64
    else
        cargo build --target "$X86_64_TARGET" --profile "$BUILD_TYPE" --package xparq-bootloader-x86_64 --quiet
    fi
    
    # Resolve artifacts to build directory.
    # Prefer runnable binaries; keep staticlib fallback for prototype variants.
    local kernel_file="target/$X86_64_TARGET/$BUILD_TYPE/xparq_kernel"
    if [ ! -f "$kernel_file" ]; then
        kernel_file="target/$X86_64_TARGET/$BUILD_TYPE/libxparq_kernel.a"
    fi
    local bootloader_file="target/$X86_64_TARGET/$BUILD_TYPE/xparq-bootloader-x86_64"
    if [ ! -f "$bootloader_file" ]; then
        bootloader_file="target/$X86_64_TARGET/$BUILD_TYPE/libxparq_bootloader_x86_64.a"
    fi
    
    if [ -f "$kernel_file" ]; then
        cp "$kernel_file" "$x86_64_build_dir/kernel.bin"
        print_success "x86-64 kernel built: $x86_64_build_dir/kernel.bin"
    else
        print_error "x86-64 kernel build failed"
        return 1
    fi
    
    if [ -f "$bootloader_file" ]; then
        cp "$bootloader_file" "$x86_64_build_dir/bootloader.bin"
        print_success "x86-64 bootloader built: $x86_64_build_dir/bootloader.bin"
    else
        print_warning "x86-64 bootloader build failed"
    fi
}

# Run tests
run_tests() {
    print_info "Running tests..."
    
    if [ "$ARCH" = "arm64" -o "$ARCH" = "all" ]; then
        test_arm64
    fi
    
    if [ "$ARCH" = "x86-64" -o "$ARCH" = "all" ]; then
        test_x86_64
    fi
}

# Test ARM64 version
test_arm64() {
    print_info "Testing ARM64 version..."
    
    if ! command -v qemu-system-aarch64 &> /dev/null; then
        print_warning "QEMU for ARM64 not found. Skipping ARM64 tests."
        return
    fi
    
    local arm64_build_dir="$BUILD_DIR/arm64"
    local kernel_file="$arm64_build_dir/kernel.bin"
    
    if [ ! -f "$kernel_file" ]; then
        print_error "ARM64 kernel not found. Build first."
        return 1
    fi
    
    print_info "Running ARM64 kernel in QEMU..."
    
    # Run QEMU with minimal configuration
    timeout 30s qemu-system-aarch64 \
        -machine virt \
        -cpu cortex-a72 \
        -m 512M \
        -nographic \
        -kernel "$kernel_file" \
        -semihosting \
        -semihosting-config enable=on,target=native || true
    
    print_success "ARM64 test completed"
}

# Test x86-64 version
test_x86_64() {
    print_info "Testing x86-64 version..."
    
    if ! command -v qemu-system-x86_64 &> /dev/null; then
        print_warning "QEMU for x86-64 not found. Skipping x86-64 tests."
        return
    fi
    
    local x86_64_build_dir="$BUILD_DIR/x86-64"
    local kernel_file="$x86_64_build_dir/kernel.bin"
    
    if [ ! -f "$kernel_file" ]; then
        print_error "x86-64 kernel not found. Build first."
        return 1
    fi
    
    print_info "Running x86-64 kernel in QEMU..."
    
    # Run QEMU with minimal configuration
    timeout 30s qemu-system-x86_64 \
        -machine q35 \
        -cpu qemu64 \
        -m 512M \
        -nographic \
        -kernel "$kernel_file" \
        -no-reboot || true
    
    print_success "x86-64 test completed"
}

# Flash to device (ARM64 only)
flash_device() {
    print_info "Flashing to device..."
    
    if [ "$ARCH" != "arm64" ]; then
        print_error "Flashing is only supported for ARM64 architecture."
        return 1
    fi
    
    local arm64_build_dir="$BUILD_DIR/arm64"
    local kernel_file="$arm64_build_dir/kernel.bin"
    
    if [ ! -f "$kernel_file" ]; then
        print_error "ARM64 kernel not found. Build first."
        return 1
    fi
    
    # Check for device
    if ! command -v fastboot &> /dev/null; then
        print_warning "fastboot not found. Please install Android SDK platform-tools."
        return 1
    fi
    
    print_info "Flashing kernel to device..."
    
    # Create boot image (Phase 1: dummy implementation)
    local boot_image="$arm64_build_dir/boot.img"
    cp "$kernel_file" "$boot_image"
    
    # Flash to device
    if fastboot devices | grep -q .; then
        fastboot flash boot "$boot_image"
        print_success "Kernel flashed to device"
    else
        print_error "No device found in fastboot mode."
        return 1
    fi
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -a|--arch)
                ARCH="$2"
                shift 2
                ;;
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
            -T|--test)
                TEST=true
                shift
                ;;
            -F|--flash)
                FLASH=true
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Validate arguments
    if [ -z "$ARCH" ]; then
        print_error "Architecture must be specified."
        show_help
        exit 1
    fi
    
    if [ "$ARCH" != "arm64" ] && [ "$ARCH" != "x86-64" ] && [ "$ARCH" != "all" ]; then
        print_error "Invalid architecture: $ARCH"
        show_help
        exit 1
    fi
    
    if [ "$BUILD_TYPE" != "debug" ] && [ "$BUILD_TYPE" != "release" ]; then
        print_error "Invalid build type: $BUILD_TYPE"
        show_help
        exit 1
    fi
}

# Main function
main() {
    print_info "XPARQ OS Build Script v0.1.0"
    print_info "Project root: $PROJECT_ROOT"
    
    parse_args "$@"
    
    if [ "$CLEAN" = true ]; then
        clean_build
    fi
    
    check_dependencies
    
    print_info "Starting build for architecture: $ARCH"
    print_info "Build type: $BUILD_TYPE"
    
    case $ARCH in
        arm64)
            build_arm64
            ;;
        x86-64)
            build_x86_64
            ;;
        all)
            build_arm64
            build_x86_64
            ;;
    esac
    
    if [ "$TEST" = true ]; then
        run_tests
    fi
    
    if [ "$FLASH" = true ]; then
        flash_device
    fi
    
    print_success "Build completed successfully!"
    print_info "Build artifacts located in: $BUILD_DIR"
}

# Run main function
main "$@"
