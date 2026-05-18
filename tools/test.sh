#!/bin/bash
# XPARQ OS Test Script
# Phase 01: OS & Kernel Foundations
# Test script for ARM64 and x86-64 architectures using QEMU

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
ARCH=""
TEST_TYPE="basic"
TIMEOUT=30
VERBOSE=false
GDB=false
NETWORK=false

# Help message
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "XPARQ OS Test Script"
    echo ""
    echo "Options:"
    echo "  -a, --arch ARCH      Target architecture (arm64, x86-64, all)"
    echo "  -t, --type TYPE      Test type (basic, boot, network, stress) [default: basic]"
    echo "  -T, --timeout SEC    Test timeout in seconds [default: 30]"
    echo "  -v, --verbose        Verbose output"
    echo "  -g, --gdb            Enable GDB debugging"
    echo "  -n, --network        Enable network support"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 -a arm64          Test ARM64 version"
    echo "  $0 -a x86-64         Test x86-64 version"
    echo "  $0 -a all            Test both architectures"
    echo "  $0 -t boot -a arm64  Boot test for ARM64"
    echo "  $0 -g -a x86-64      Debug x86-64 with GDB"
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
    
    # Check QEMU for testing
    if [ "$ARCH" = "arm64" -o "$ARCH" = "all" ]; then
        if ! command -v qemu-system-aarch64 &> /dev/null; then
            print_error "QEMU for ARM64 not found. Please install qemu-system-aarch64."
            exit 1
        fi
    fi
    
    if [ "$ARCH" = "x86-64" -o "$ARCH" = "all" ]; then
        if ! command -v qemu-system-x86_64 &> /dev/null; then
            print_error "QEMU for x86-64 not found. Please install qemu-system-x86_64."
            exit 1
        fi
    fi
    
    # Check GDB if debugging enabled
    if [ "$GDB" = true ]; then
        if ! command -v gdb-multiarch &> /dev/null && ! command -v aarch64-linux-gnu-gdb &> /dev/null && ! command -v gdb &> /dev/null; then
            print_error "GDB not found. Please install gdb-multiarch or architecture-specific GDB."
            exit 1
        fi
    fi
    
    print_success "Dependencies checked"
}

# Check build artifacts
check_build_artifacts() {
    print_info "Checking build artifacts..."
    
    if [ "$ARCH" = "arm64" -o "$ARCH" = "all" ]; then
        local arm64_kernel="$BUILD_DIR/arm64/kernel.bin"
        if [ ! -f "$arm64_kernel" ]; then
            print_error "ARM64 kernel not found. Build first: ./tools/build.sh -a arm64"
            exit 1
        fi
    fi
    
    if [ "$ARCH" = "x86-64" -o "$ARCH" = "all" ]; then
        local x86_64_kernel="$BUILD_DIR/x86-64/kernel.bin"
        if [ ! -f "$x86_64_kernel" ]; then
            print_error "x86-64 kernel not found. Build first: ./tools/build.sh -a x86-64"
            exit 1
        fi
    fi
    
    print_success "Build artifacts checked"
}

# Test ARM64 version
test_arm64() {
    print_info "Testing ARM64 version..."
    
    local arm64_kernel="$BUILD_DIR/arm64/kernel.bin"
    local test_log="$BUILD_DIR/arm64/test.log"
    
    print_info "Running ARM64 test: $TEST_TYPE"
    
    case $TEST_TYPE in
        basic)
            test_arm64_basic "$arm64_kernel" "$test_log"
            ;;
        boot)
            test_arm64_boot "$arm64_kernel" "$test_log"
            ;;
        network)
            test_arm64_network "$arm64_kernel" "$test_log"
            ;;
        stress)
            test_arm64_stress "$arm64_kernel" "$test_log"
            ;;
        *)
            print_error "Unknown test type: $TEST_TYPE"
            exit 1
            ;;
    esac
    
    analyze_test_results "$test_log" "ARM64"
}

# Test ARM64 basic functionality
test_arm64_basic() {
    local kernel="$1"
    local log="$2"
    
    print_info "Running ARM64 basic test..."
    
    local qemu_cmd="qemu-system-aarch64"
    qemu_cmd+=" -machine virt"
    qemu_cmd+=" -cpu cortex-a72"
    qemu_cmd+=" -m 512M"
    qemu_cmd+=" -nographic"
    qemu_cmd+=" -kernel $kernel"
    qemu_cmd+=" -semihosting"
    qemu_cmd+=" -semihosting-config enable=on,target=native"
    
    if [ "$VERBOSE" = true ]; then
        qemu_cmd+=" -d cpu,int,exec"
    fi
    
    if [ "$GDB" = true ]; then
        qemu_cmd+=" -s -S"
        start_gdb_server "arm64" &
        local gdb_pid=$!
        trap "kill $gdb_pid 2>/dev/null || true" EXIT
    fi
    
    # Run QEMU and capture output
    timeout "$TIMEOUT"s $qemu_cmd 2>&1 | tee "$log" || true
    
    if [ "$GDB" = true ]; then
        wait_for_gdb
    fi
}

# Test ARM64 boot
test_arm64_boot() {
    local kernel="$1"
    local log="$2"
    
    print_info "Running ARM64 boot test..."
    
    local qemu_cmd="qemu-system-aarch64"
    qemu_cmd+=" -machine virt"
    qemu_cmd+=" -cpu cortex-a72"
    qemu_cmd+=" -m 1G"
    qemu_cmd+=" -nographic"
    qemu_cmd+=" -kernel $kernel"
    qemu_cmd+=" -append \"console=tty0 debug\""
    
    # Add boot-specific options
    qemu_cmd+=" -monitor none"
    qemu_cmd+=" -serial stdio"
    
    timeout "$TIMEOUT"s $qemu_cmd 2>&1 | tee "$log" || true
}

# Test ARM64 network
test_arm64_network() {
    local kernel="$1"
    local log="$2"
    
    print_info "Running ARM64 network test..."
    
    local qemu_cmd="qemu-system-aarch64"
    qemu_cmd+=" -machine virt"
    qemu_cmd+=" -cpu cortex-a72"
    qemu_cmd+=" -m 512M"
    qemu_cmd+=" -nographic"
    qemu_cmd+=" -kernel $kernel"
    qemu_cmd+=" -netdev user,id=net0"
    qemu_cmd+=" -device virtio-net-device,netdev=net0"
    
    timeout "$TIMEOUT"s $qemu_cmd 2>&1 | tee "$log" || true
}

# Test ARM64 stress
test_arm64_stress() {
    local kernel="$1"
    local log="$2"
    
    print_info "Running ARM64 stress test..."
    
    local qemu_cmd="qemu-system-aarch64"
    qemu_cmd+=" -machine virt"
    qemu_cmd+=" -cpu cortex-a72"
    qemu_cmd+=" -m 2G"
    qemu_cmd+=" -nographic"
    qemu_cmd+=" -kernel $kernel"
    qemu_cmd+=" -smp 4"
    
    timeout "$TIMEOUT"s $qemu_cmd 2>&1 | tee "$log" || true
}

# Test x86-64 version
test_x86_64() {
    print_info "Testing x86-64 version..."
    
    local x86_64_kernel="$BUILD_DIR/x86-64/kernel.bin"
    local test_log="$BUILD_DIR/x86-64/test.log"
    
    print_info "Running x86-64 test: $TEST_TYPE"
    
    case $TEST_TYPE in
        basic)
            test_x86_64_basic "$x86_64_kernel" "$test_log"
            ;;
        boot)
            test_x86_64_boot "$x86_64_kernel" "$test_log"
            ;;
        network)
            test_x86_64_network "$x86_64_kernel" "$test_log"
            ;;
        stress)
            test_x86_64_stress "$x86_64_kernel" "$test_log"
            ;;
        *)
            print_error "Unknown test type: $TEST_TYPE"
            exit 1
            ;;
    esac
    
    analyze_test_results "$test_log" "x86-64"
}

# Test x86-64 basic functionality
test_x86_64_basic() {
    local kernel="$1"
    local log="$2"
    
    print_info "Running x86-64 basic test..."
    
    local qemu_cmd="qemu-system-x86_64"
    qemu_cmd+=" -machine q35"
    qemu_cmd+=" -cpu qemu64"
    qemu_cmd+=" -m 512M"
    qemu_cmd+=" -nographic"
    qemu_cmd+=" -kernel $kernel"
    
    if [ "$VERBOSE" = true ]; then
        qemu_cmd+=" -d cpu,int,exec"
    fi
    
    if [ "$GDB" = true ]; then
        qemu_cmd+=" -s -S"
        start_gdb_server "x86-64" &
        local gdb_pid=$!
        trap "kill $gdb_pid 2>/dev/null || true" EXIT
    fi
    
    timeout "$TIMEOUT"s $qemu_cmd 2>&1 | tee "$log" || true
    
    if [ "$GDB" = true ]; then
        wait_for_gdb
    fi
}

# Test x86-64 boot
test_x86_64_boot() {
    local kernel="$1"
    local log="$2"
    
    print_info "Running x86-64 boot test..."
    
    local qemu_cmd="qemu-system-x86_64"
    qemu_cmd+=" -machine q35"
    qemu_cmd+=" -cpu qemu64"
    qemu_cmd+=" -m 1G"
    qemu_cmd+=" -nographic"
    qemu_cmd+=" -kernel $kernel"
    qemu_cmd+=" -append \"console=tty0 debug\""
    
    timeout "$TIMEOUT"s $qemu_cmd 2>&1 | tee "$log" || true
}

# Test x86-64 network
test_x86_64_network() {
    local kernel="$1"
    local log="$2"
    
    print_info "Running x86-64 network test..."
    
    local qemu_cmd="qemu-system-x86_64"
    qemu_cmd+=" -machine q35"
    qemu_cmd+=" -cpu qemu64"
    qemu_cmd+=" -m 512M"
    qemu_cmd+=" -nographic"
    qemu_cmd+=" -kernel $kernel"
    qemu_cmd+=" -netdev user,id=net0"
    qemu_cmd+=" -device e1000,netdev=net0"
    
    timeout "$TIMEOUT"s $qemu_cmd 2>&1 | tee "$log" || true
}

# Test x86-64 stress
test_x86_64_stress() {
    local kernel="$1"
    local log="$2"
    
    print_info "Running x86-64 stress test..."
    
    local qemu_cmd="qemu-system-x86_64"
    qemu_cmd+=" -machine q35"
    qemu_cmd+=" -cpu qemu64"
    qemu_cmd+=" -m 2G"
    qemu_cmd+=" -nographic"
    qemu_cmd+=" -kernel $kernel"
    qemu_cmd+=" -smp 4"
    
    timeout "$TIMEOUT"s $qemu_cmd 2>&1 | tee "$log" || true
}

# Start GDB server
start_gdb_server() {
    local arch="$1"
    
    print_info "Starting GDB server for $arch..."
    
    case $arch in
        arm64)
            if command -v aarch64-linux-gnu-gdb &> /dev/null; then
                aarch64-linux-gnu-gdb -ex "target remote localhost:1234" -ex "continue" &
            elif command -v gdb-multiarch &> /dev/null; then
                gdb-multiarch -ex "target remote localhost:1234" -ex "continue" &
            else
                print_warning "No suitable GDB found for ARM64"
            fi
            ;;
        x86-64)
            if command -v gdb &> /dev/null; then
                gdb -ex "target remote localhost:1234" -ex "continue" &
            elif command -v gdb-multiarch &> /dev/null; then
                gdb-multiarch -ex "target remote localhost:1234" -ex "continue" &
            else
                print_warning "No suitable GDB found for x86-64"
            fi
            ;;
    esac
}

# Wait for GDB
wait_for_gdb() {
    print_info "Waiting for GDB connection..."
    print_info "Connect to GDB with: target remote localhost:1234"
    print_info "Press Enter to continue..."
    read -r
}

# Analyze test results
analyze_test_results() {
    local log="$1"
    local arch="$2"
    
    print_info "Analyzing $arch test results..."
    
    if [ ! -f "$log" ]; then
        print_error "Test log not found: $log"
        return 1
    fi
    
    # Check for success indicators
    local success_patterns=(
        "XPARQ OS Booting"
        "kernel_main"
        "Initialization complete"
        "xparq-os"
    )
    
    local found_success=false
    for pattern in "${success_patterns[@]}"; do
        if grep -q "$pattern" "$log"; then
            print_success "Found success pattern: $pattern"
            found_success=true
        fi
    done
    
    # Check for error indicators
    local error_patterns=(
        "panic"
        "error"
        "failed"
        "exception"
        "fault"
    )
    
    local found_error=false
    for pattern in "${error_patterns[@]}"; do
        if grep -i -q "$pattern" "$log"; then
            print_warning "Found error pattern: $pattern"
            found_error=true
        fi
    done
    
    # Summary
    if [ "$found_success" = true ] && [ "$found_error" = false ]; then
        print_success "$arch test PASSED"
    elif [ "$found_success" = true ] && [ "$found_error" = true ]; then
        print_warning "$arch test PARTIAL (success with warnings)"
    else
        print_error "$arch test FAILED"
        return 1
    fi
    
    # Show log summary
    if [ "$VERBOSE" = true ]; then
        print_info "Test log summary:"
        tail -20 "$log"
    fi
}

# Generate test report
generate_test_report() {
    local report="$BUILD_DIR/test_report.txt"
    
    print_info "Generating test report..."
    
    cat > "$report" << EOF
XPARQ OS Test Report
===================

Test Configuration:
- Architecture: $ARCH
- Test Type: $TEST_TYPE
- Timeout: ${TIMEOUT}s
- Timestamp: $(date)

Test Results:
EOF
    
    if [ "$ARCH" = "arm64" -o "$ARCH" = "all" ]; then
        echo "- ARM64: $(cat "$BUILD_DIR/arm64/test.log" 2>/dev/null | grep -c "XPARQ OS" || echo "0") boot messages" >> "$report"
    fi
    
    if [ "$ARCH" = "x86-64" -o "$ARCH" = "all" ]; then
        echo "- x86-64: $(cat "$BUILD_DIR/x86-64/test.log" 2>/dev/null | grep -c "XPARQ OS" || echo "0") boot messages" >> "$report"
    fi
    
    echo "" >> "$report"
    echo "Test logs available in: $BUILD_DIR/*/test.log" >> "$report"
    
    print_success "Test report generated: $report"
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
                TEST_TYPE="$2"
                shift 2
                ;;
            -T|--timeout)
                TIMEOUT="$2"
                shift 2
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -g|--gdb)
                GDB=true
                shift
                ;;
            -n|--network)
                NETWORK=true
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
    
    if [ "$TEST_TYPE" != "basic" ] && [ "$TEST_TYPE" != "boot" ] && [ "$TEST_TYPE" != "network" ] && [ "$TEST_TYPE" != "stress" ]; then
        print_error "Invalid test type: $TEST_TYPE"
        show_help
        exit 1
    fi
}

# Main function
main() {
    print_info "XPARQ OS Test Script v0.1.0"
    print_info "Project root: $PROJECT_ROOT"
    
    parse_args "$@"
    
    print_info "Configuration:"
    print_info "  Architecture: $ARCH"
    print_info "  Test type: $TEST_TYPE"
    print_info "  Timeout: ${TIMEOUT}s"
    print_info "  Verbose: $VERBOSE"
    print_info "  GDB: $GDB"
    
    check_dependencies
    check_build_artifacts
    
    case $ARCH in
        arm64)
            test_arm64
            ;;
        x86-64)
            test_x86_64
            ;;
        all)
            test_arm64
            test_x86_64
            ;;
    esac
    
    generate_test_report
    
    print_success "Test completed successfully!"
    print_info "Test logs available in: $BUILD_DIR/*/test.log"
}

# Run main function
main "$@"
