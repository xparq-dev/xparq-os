ENTRY(_start)

SECTIONS
{
    /* Load address of the kernel */
    . = 0x100000;

    .text :
    {
        /* Keep entry point function at the very beginning of the binary */
        KEEP(*(.text.init))
        *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }

    .data :
    {
        *(.data .data.*)
    }

    .bss :
    {
        __bss_start = .;
        *(.bss .bss.*)
        __bss_end = .;
    }
}
