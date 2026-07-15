ENTRY(_start)
SECTIONS {
    . = 0x40000000; /* 1GB */
    
    .text : {
        *(.text .text.*)
    }
    
    .rodata : {
        *(.rodata .rodata.*)
    }
    
    .data : {
        *(.data .data.*)
    }
    
    .bss : {
        *(.bss .bss.*)
    }
    
    /DISCARD/ : {
        *(.eh_frame)
    }
}
