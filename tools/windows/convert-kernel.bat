@echo off
echo Converting kernel ELF to binary...

:: Find llvm-objcopy from Rust toolchain
for /f "tokens=*" %%a in ('rustup which rustc') do set RUSTC=%%a
for %%i in (%RUSTC%) do set RUSTC_DIR=%%~dpi
set LLVM_OBJCOPY=%RUSTC_DIR%llvm-objcopy.exe

echo Using: %LLVM_OBJCOPY%

set KERNEL_ELF=target\x86_64-unknown-none\release\xparq_kernel
if not exist "%KERNEL_ELF%" (
    echo Kernel ELF not found. Building xparq-kernel...
    cargo build --target x86_64-unknown-none --release --package xparq-kernel
)
if not exist "%KERNEL_ELF%" (
    echo Kernel ELF still missing: %KERNEL_ELF%
    exit /b 1
)

:: Convert kernel
if not exist build\x86-64 mkdir build\x86-64
"%LLVM_OBJCOPY%" -O binary "%KERNEL_ELF%" build\x86-64\kernel.bin

echo Done!
dir build\x86-64\kernel.bin
