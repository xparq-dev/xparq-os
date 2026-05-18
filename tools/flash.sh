#!/bin/bash
# XPARQ OS Flash Script
# Phase 01: OS & Kernel Foundations
# Flash script for ARM64 devices using fastboot

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

# Default values
ARCH="arm64"
DEVICE_TYPE="generic"
PARTITION="boot"
VERBOSE=false

# Help message
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "XPARQ OS Flash Script"
    echo ""
    echo "Options:"
    echo "  -a, --arch ARCH      Target architecture (arm64) [default: arm64]"
    echo "  -d, --device TYPE   Device type (generic, pixel, pinephone) [default: generic]"
    echo "  -p, --partition PART Partition to flash (boot, system, recovery) [default: boot]"
    echo "  -v, --verbose        Verbose output"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                    Flash to device"
    echo "  $0 -d pixel          Flash to Pixel device"
    echo "  $0 -p system          Flash system partition"
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
    
    # Check fastboot
    if ! command -v fastboot &> /dev/null; then
        print_error "fastboot not found. Please install Android SDK platform-tools."
        exit 1
    fi
    
    # Check adb
    if ! command -v adb &> /dev/null; then
        print_warning "adb not found. Some features may not work."
    fi
    
    print_success "Dependencies checked"
}

# Check device connection
check_device() {
    print_info "Checking device connection..."
    
    if ! fastboot devices | grep -q .; then
        print_error "No device found in fastboot mode."
        print_info "Please ensure your device is:"
        print_info "1. Connected via USB"
        print_info "2. In fastboot mode (reboot to bootloader)"
        print_info "3. USB debugging enabled"
        exit 1
    fi
    
    local device_count=$(fastboot devices | wc -l)
    print_info "Found $device_count device(s) in fastboot mode"
    
    # Show device info
    local device_info=$(fastboot getvar product 2>/dev/null | head -n 1)
    print_info "Device: $device_info"
}

# Build flash image
build_flash_image() {
    print_info "Building flash image..."
    
    local arch_build_dir="$BUILD_DIR/$ARCH"
    local kernel_file="$arch_build_dir/kernel.bin"
    local bootloader_file="$arch_build_dir/bootloader.bin"
    
    if [ ! -f "$kernel_file" ]; then
        print_error "Kernel not found. Please build first: ./tools/build.sh -a $ARCH"
        exit 1
    fi
    
    # Create flash image directory
    local flash_dir="$arch_build_dir/flash"
    mkdir -p "$flash_dir"
    
    # Create boot image (Phase 1: simple concatenation)
    local boot_image="$flash_dir/boot.img"
    
    if [ -f "$bootloader_file" ]; then
        print_info "Creating boot image with bootloader..."
        cat "$bootloader_file" "$kernel_file" > "$boot_image"
    else
        print_warning "Bootloader not found, using kernel only..."
        cp "$kernel_file" "$boot_image"
    fi
    
    print_success "Flash image created: $boot_image"
}

# Flash to device
flash_device() {
    print_info "Flashing to device..."
    
    local arch_build_dir="$BUILD_DIR/$ARCH"
    local flash_dir="$arch_build_dir/flash"
    local boot_image="$flash_dir/boot.img"
    
    if [ ! -f "$boot_image" ]; then
        print_error "Flash image not found. Build first."
        exit 1
    fi
    
    case $PARTITION in
        boot)
            print_info "Flashing boot partition..."
            fastboot flash boot "$boot_image"
            ;;
        system)
            print_info "Flashing system partition..."
            fastboot flash system "$boot_image"
            ;;
        recovery)
            print_info "Flashing recovery partition..."
            fastboot flash recovery "$boot_image"
            ;;
        *)
            print_error "Unknown partition: $PARTITION"
            exit 1
            ;;
    esac
    
    print_success "Flash completed"
}

# Reboot device
reboot_device() {
    print_info "Rebooting device..."
    
    fastboot reboot
    
    print_success "Device rebooted"
}

# Device-specific configurations
configure_device() {
    case $DEVICE_TYPE in
        pixel)
            configure_pixel_device
            ;;
        pinephone)
            configure_pinephone_device
            ;;
        generic)
            configure_generic_device
            ;;
        *)
            print_error "Unknown device type: $DEVICE_TYPE"
            exit 1
            ;;
    esac
}

# Configure Pixel device
configure_pixel_device() {
    print_info "Configuring for Pixel device..."
    
    # Pixel-specific fastboot commands
    fastboot set_active a
    fastboot flash vbmeta "$BUILD_DIR/arm64/flash/vbmeta.img" 2>/dev/null || true
    
    print_success "Pixel device configured"
}

# Configure PinePhone device
configure_pinephone_device() {
    print_info "Configuring for PinePhone device..."
    
    # PinePhone-specific fastboot commands
    fastboot flash dtb "$BUILD_DIR/arm64/flash/dtb.img" 2>/dev/null || true
    
    print_success "PinePhone device configured"
}

# Configure generic device
configure_generic_device() {
    print_info "Configuring for generic device..."
    
    # Generic device configuration
    # No device-specific commands
    
    print_success "Generic device configured"
}

# Show device status
show_device_status() {
    print_info "Device status:"
    
    # Show fastboot variables
    print_info "Fastboot variables:"
    fastboot getvar all 2>/dev/null | head -20
    
    # Show partition layout
    print_info "Partition layout:"
    fastboot getvar partition-type 2>/dev/null || true
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -a|--arch)
                ARCH="$2"
                shift 2
                ;;
            -d|--device)
                DEVICE_TYPE="$2"
                shift 2
                ;;
            -p|--partition)
                PARTITION="$2"
                shift 2
                ;;
            -v|--verbose)
                VERBOSE=true
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
    if [ "$ARCH" != "arm64" ]; then
        print_error "Flash script only supports ARM64 architecture."
        exit 1
    fi
    
    if [ "$DEVICE_TYPE" != "generic" ] && [ "$DEVICE_TYPE" != "pixel" ] && [ "$DEVICE_TYPE" != "pinephone" ]; then
        print_error "Invalid device type: $DEVICE_TYPE"
        show_help
        exit 1
    fi
    
    if [ "$PARTITION" != "boot" ] && [ "$PARTITION" != "system" ] && [ "$PARTITION" != "recovery" ]; then
        print_error "Invalid partition: $PARTITION"
        show_help
        exit 1
    fi
}

# Main function
main() {
    print_info "XPARQ OS Flash Script v0.1.0"
    print_info "Project root: $PROJECT_ROOT"
    
    parse_args "$@"
    
    print_info "Configuration:"
    print_info "  Architecture: $ARCH"
    print_info "  Device type: $DEVICE_TYPE"
    print_info "  Partition: $PARTITION"
    
    check_dependencies
    check_device
    
    build_flash_image
    configure_device
    flash_device
    show_device_status
    reboot_device
    
    print_success "Flash completed successfully!"
    print_info "Your device should now boot into XPARQ OS"
}

# Run main function
main "$@"
