# XPARQ OS Build System
# Phase 1 & 2: Multi-Architecture Build Pipeline

.PHONY: all clean arm64 x86_64 docs

# Default target
all: arm64 x86_64

# ARM64 target
arm64:
	@echo "Building XPARQ OS for ARM64..."
	./tools/build-arm64.sh

# x86-64 target  
x86_64:
	@echo "Building XPARQ OS for x86-64..."
	./tools/build-x86_64.sh

# Documentation
docs:
	@echo "Building documentation..."
	@mkdir -p docs/build
	@echo "Documentation built successfully"

# Clean all targets
clean:
	@echo "Cleaning all build artifacts..."
	@rm -rf target/
	@rm -f *.bin *.elf *.img *.iso
	@rm -rf docs/build/
	@echo "Clean complete"

# Install dependencies
deps:
	@echo "Installing dependencies..."
	@rustup target add aarch64-unknown-none
	@rustup target add x86_64-unknown-none
	@cargo install cargo-binutils
	@rustup component add llvm-tools-preview
	@echo "Dependencies installed"

# Run on QEMU ARM64
run-arm64: arm64
	@echo "Running XPARQ OS on QEMU ARM64..."
	@if [ ! -f build/arm64/kernel.bin ]; then echo "Missing artifact: build/arm64/kernel.bin"; exit 1; fi
	qemu-system-aarch64 -machine virt -cpu cortex-a72 -m 512M -nographic -kernel build/arm64/kernel.bin

# Run on QEMU x86-64
run-x86_64: x86_64
	@echo "Running XPARQ OS on QEMU x86-64..."
	@if [ ! -f build/x86-64/kernel.bin ]; then echo "Missing artifact: build/x86-64/kernel.bin"; exit 1; fi
	qemu-system-x86_64 -nographic -kernel build/x86-64/kernel.bin

# Flash to ARM device
flash-arm64: arm64
	@echo "Flashing XPARQ OS to ARM device..."
	./tools/flash-device.sh

# Help
help:
	@echo "XPARQ OS Build System"
	@echo "====================="
	@echo ""
	@echo "Targets:"
	@echo "  all        - Build all targets"
	@echo "  arm64      - Build ARM64 target"
	@echo "  x86_64     - Build x86-64 target"
	@echo "  docs       - Build documentation"
	@echo "  clean      - Clean all build artifacts"
	@echo "  deps       - Install dependencies"
	@echo "  run-arm64  - Build and run on QEMU ARM64"
	@echo "  run-x86_64 - Build and run on QEMU x86-64"
	@echo "  flash-arm64- Flash to ARM device"
	@echo "  help       - Show this help"
