/* XPARQ OS - Phase 02: ARM64 Linker Script for QEMU virt machine */
/* Memory layout for QEMU virt machine (ARM64) */

ENTRY(_start)

MEMORY {
    RAM (rwx) : ORIGIN = 0x40000000, LENGTH = 512M
}

SECTIONS {
    . = 0x40000000;

    .text : {
        *(.text.entry)
        *(.text .text.*)
    } > RAM

    .rodata : {
        *(.rodata .rodata.*)
    } > RAM

    .data : {
        *(.data .data.*)
    } > RAM

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } > RAM

    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
        *(.note*)
    }
}
