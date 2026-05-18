/* XPARQ OS - Phase 02: x86_64 Linker Script for QEMU */
ENTRY(_start)

MEMORY {
    RAM (rwx) : ORIGIN = 0x10000, LENGTH = 512M
}

SECTIONS {
    . = 0x10000;

    .note.xparq.pvh : {
        *(.note.xparq.pvh)
    } > RAM

    .multiboot2 : {
        *(.multiboot2)
    } > RAM

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
        *(.note.gnu.*)
        *(.note.ABI-tag)
        *(.note.gnu.property)
        ; intentionally keep .note.xparq.pvh and .note.xen for boot
    }
}
