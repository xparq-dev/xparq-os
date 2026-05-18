/* XPARQ OS x86_64 MBR Bootloader Linker Script */
ENTRY(_mbr_start)

SECTIONS {
    . = 0x7C00;
    
    .mbr : {
        *(.mbr)
        *(.mbr.*)
    }
    
    .text : {
        *(.text .text.*)
    }
    
    .rodata : {
        *(.rodata .rodata.*)
    }
    
    /* Fill to 510 bytes */
    . = 0x7DFE;
    
    .boot_signature : {
        *(.boot_signature)
    }
    
    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
        *(.note*)
        *(.data)
        *(.bss)
    }
}
