OUTPUT_ARCH(riscv)
ENTRY(_start)

/* used after os4 */
/* 将起始地址 BASE_ADDRESS 设置为 0x10000 （我们这里并不设置为 0x0 ，因为它一般代表空指针），显然它只能是一个地址空间中的虚拟地址而非物理地址。 */
BASE_ADDRESS = 0x10000;

SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        /* 将 _start 所在的 .text.entry 放在整个程序的开头， */
        /* 也就是说批处理系统只要在加载之后跳转到 0x80400000 就已经进入了 用户库的入口点，并会在初始化之后跳转到应用程序主逻辑 */ 
        *(.text.entry)
        *(.text .text.*)
    }
    /* 在 .text 和 .rodata 中间以及 .rodata 和 .data 中间我们进行了页面对齐，因为前后两个逻辑段的访问方式限制是不同的，*/
    /* 由于我们只能以页为单位对这个限制进行设置，因此就只能将下一个逻辑段对齐到下一个页面开始放置。 */
    . = ALIGN(4K);
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(4K);
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    /* .data 和 .bss 两个逻辑段由于访问限制相同（可读写），它们中间则无需进行页面对齐。 */
    .bss : {
        start_bss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        end_bss = .;
    }
    /* 
        DISCARD: 用于丢弃一些不需要的段
    */
    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}