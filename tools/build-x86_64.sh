#!/bin/bash
# XPARQ OS x86_64 Build Script
# Phase 02: Build and Boot Verification
# Customized build script for x86_64 architecture with boot verification

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
X86_64_TARGET="x86_64-unknown-none"

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

    # Check NASM
    if ! command -v nasm &> /dev/null; then
        print_error "NASM not found. Please install NASM."
        exit 1
    fi

    # Check llvm-objcopy
    if ! command -v llvm-objcopy &> /dev/null; then
        print_error "llvm-objcopy not found. Please install rust-llvm tools."
        exit 1
    fi

    if ! command -v python3 &> /dev/null; then
        print_error "python3 not found. Please install Python 3."
        exit 1
    fi
    
    # Check QEMU for testing
    if ! command -v qemu-system-x86_64 &> /dev/null; then
        print_error "QEMU for x86_64 not found. Please install QEMU."
        exit 1
    fi
    
    # Check cross-compilation target
    if ! rustup target list --installed | grep -q "$X86_64_TARGET"; then
        print_info "Installing x86_64 target..."
        rustup target add "$X86_64_TARGET"
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

# Build x86_64 version
build_x86_64() {
    print_info "Building x86_64 version..."
    
    local x86_64_build_dir="$BUILD_DIR/x86-64"
    mkdir -p "$x86_64_build_dir"
    
    cd "$PROJECT_ROOT"
    
    # Build kernel
    if [ "$VERBOSE" = true ]; then
        cargo build --target "$X86_64_TARGET" --profile "$BUILD_TYPE" --package xparq-kernel
    else
        cargo build --target "$X86_64_TARGET" --profile "$BUILD_TYPE" --package xparq-kernel --quiet
    fi
    
    # Resolve kernel artifact and convert to raw binary.
    local kernel_elf="target/$X86_64_TARGET/$BUILD_TYPE/xparq_kernel"
    if [ ! -f "$kernel_elf" ]; then
        kernel_elf="target/$X86_64_TARGET/$BUILD_TYPE/libxparq_kernel.a"
    fi
    if [ ! -f "$kernel_elf" ]; then
        print_error "x86_64 kernel artifact not found"
        return 1
    fi

    local bootloader_asm="$PROJECT_ROOT/bootloader/x86_64/src/simple_boot.asm"
    if [ ! -f "$bootloader_asm" ]; then
        print_error "Bootloader source not found: $bootloader_asm"
        return 1
    fi

    llvm-objcopy -O binary "$kernel_elf" "$x86_64_build_dir/kernel.bin"
    nasm -f bin "$bootloader_asm" -o "$x86_64_build_dir/bootloader.bin"

    local kernel_file="$x86_64_build_dir/kernel.bin"
    local bootloader_file="$x86_64_build_dir/bootloader.bin"

    if [ -f "$kernel_file" ]; then
        print_success "x86_64 kernel built: $x86_64_build_dir/kernel.bin"
    else
        print_error "x86_64 kernel build failed"
        return 1
    fi

    if [ -f "$bootloader_file" ]; then
        print_success "x86_64 bootloader built: $x86_64_build_dir/bootloader.bin"
    else
        print_error "x86_64 bootloader build failed"
        return 1
    fi

    # Build init
    print_info "Building init (User Space Shell)..."
    cd "$PROJECT_ROOT/init"
    cargo build --target "$X86_64_TARGET" --release
    local init_elf="$PROJECT_ROOT/target/$X86_64_TARGET/release/init"
    cd "$PROJECT_ROOT"

    # Build fat32-injector
    print_info "Building fat32-injector..."
    cd "$PROJECT_ROOT/tools/fat32-injector"
    cargo build --release
    local fat32_injector="$PROJECT_ROOT/target/release/fat32-injector"
    cd "$PROJECT_ROOT"

    # Create FAT32 image and inject init.elf
    local fat32_img="$x86_64_build_dir/fat32.img"
    rm -f "$fat32_img"
    "$fat32_injector" "$fat32_img" "$init_elf" "INIT.ELF"

    local disk_img="$x86_64_build_dir/disk.img"
    
    # Python script to assemble the entire disk image with an MBR partition table
    python3 - "$bootloader_file" "$kernel_file" "$fat32_img" "$disk_img" <<'PY'
import sys, pathlib, struct

bootloader = pathlib.Path(sys.argv[1]).read_bytes()
kernel = pathlib.Path(sys.argv[2]).read_bytes()
fat32 = pathlib.Path(sys.argv[3]).read_bytes()

# Pad bootloader to exactly 512 bytes
if len(bootloader) > 512:
    print("Bootloader is too large!")
    sys.exit(1)
bootloader += b'\x00' * (512 - len(bootloader))

# MBR Partition Table is at offset 446
# Partition 1: FAT32 starting at LBA 2048 (1MB offset)
start_lba = 2048
sector_count = len(fat32) // 512

part1 = struct.pack('<B3sB3sII',
    0x00,               # status (non-bootable)
    b'\x00\x00\x00',    # start CHS (ignored)
    0x0C,               # type (FAT32 LBA)
    b'\x00\x00\x00',    # end CHS (ignored)
    start_lba,          # start LBA
    sector_count        # sector count
)

# Insert Partition 1 into Bootloader
bootloader = bytearray(bootloader)
bootloader[446:446+16] = part1
bootloader[510:512] = b'\x55\xaa'

# Assemble disk
disk = bytearray()
disk.extend(bootloader)

# Kernel at LBA 1
kernel_padded_size = start_lba * 512 - 512
if len(kernel) > kernel_padded_size:
    print(f"Kernel too large! Size: {len(kernel)}")
    sys.exit(1)

disk.extend(kernel)
disk.extend(b'\x00' * (kernel_padded_size - len(kernel)))

# FAT32 at LBA 2048
disk.extend(fat32)

pathlib.Path(sys.argv[4]).write_bytes(disk)
PY

    print_success "x86-64 disk image built: $disk_img"
}

# Test x86_64 boot
test_x86_64_boot() {
    print_info "Testing x86_64 boot..."
    
    local x86_64_build_dir="$BUILD_DIR/x86-64"
    local disk_image="$x86_64_build_dir/disk.img"
    
    if [ ! -f "$disk_image" ]; then
        print_error "x86_64 disk image not found. Build first."
        return 1
    fi
    
    print_info "Running x86_64 kernel in QEMU..."
    
    # Run QEMU with configuration to capture boot messages
    timeout 30s qemu-system-x86_64 \
        -drive format=raw,file="$disk_image" \
        -boot order=c \
        -nographic \
        -no-reboot \
        -m 128M 2>&1 | tee "$x86_64_build_dir/boot.log" || true
    
    # Check for expected boot messages
    if grep -q "\[XPARQ OS\] Booting on x86-64..." "$x86_64_build_dir/boot.log"; then
        print_success "Found expected boot message: '[XPARQ OS] Booting on x86-64...'"
    else
        print_warning "Expected boot message not found: '[XPARQ OS] Booting on x86-64...'"
    fi
    
    if grep -q "\[XPARQ OS\] Kernel initialized." "$x86_64_build_dir/boot.log"; then
        print_success "Found expected boot message: '[XPARQ OS] Kernel initialized.'"
    else
        print_warning "Expected boot message not found: '[XPARQ OS] Kernel initialized.'"
    fi
    
    print_success "x86_64 boot test completed"
    print_info "Boot log saved to: $x86_64_build_dir/boot.log"
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
    print_info "XPARQ OS x86_64 Build Script v0.2.0"
    print_info "Project root: $PROJECT_ROOT"
    
    parse_args "$@"
    
    if [ "$CLEAN" = true ]; then
        clean_build
    fi
    
    check_dependencies
    
    print_info "Starting x86_64 build..."
    print_info "Build type: $BUILD_TYPE"
    
    build_x86_64
    
    if [ "$TEST" = true ]; then
        test_x86_64_boot
    fi
    
    print_success "x86_64 build completed successfully!"
    print_info "Build artifacts located in: $BUILD_DIR/x86-64"
}

# Run main function
main "$@"
