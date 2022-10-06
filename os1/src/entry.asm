    # 一般情况下，所有的代码都被放到一个名为 .text 的代码段中，
    # 这里我们将其命名为 .text.entry 从而区别于其他 .text 的目的在于我们想要确保该段被放置在相比任何其他代码段更低的地址上。
    # 这样，作为内核的入口点，这段指令才能被最先执行。
    .section .text.entry
    # 告知编译器 _start 是一个全局符号，因此可以被其他目标文件使用
    .globl _start
_start:
    li x1, 100
#     la sp, boot_stack_top
#     call rust_main

#     .section .bss.stack
#     .globl boot_stack
# boot_stack:
#     .space 4096 * 16
#     .globl boot_stack_top
# boot_stack_top: