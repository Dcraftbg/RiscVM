ENTRY(_start)
SECTIONS
{
    . = 0x0;
    .entry : {
        *(.entry)
    } 

    . += 100;
    .text : {
        *(.text .text.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }

    .dynamic : {
        *(.dynamic)
    } 

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    }
    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
    }
}
