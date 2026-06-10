ENTRY(_start)

SECTIONS
{
    /* Load address of the kernel */
    . = 0x10000;

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
        *(.bss .bss.*)
    }
}
