.altmacro
.macro SAVE_SN n
    # n+2: save reg[s], so skip sp, ra -> see TaskContext
    sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN n
    # n+2: load reg[s], so skip sp, ra -> see TaskContext
    ld s\n, (\n+2)*8(a1)
.endm
    .section .text
    .globl __switch
__switch:
    # part [1] 
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext,
    #     next_task_cx_ptr: *const TaskContext
    # )

    # 使用 `ra` 寄存器（即 `x1` 寄存器）保存返回地址: 
    # ra 记录了 __switch 函数返回之后应该跳转到哪里继续执行，从而在任务切换完成并 ret 之后能到正确的位置

    # s0~s11 作为临时寄存器使用，被调函数保存后才能在被调函数中使用
    # 对于一般的函数而言，Rust/C 编译器会在函数的起始位置自动生成代码来保存 s0~s11 这些被调用者保存的寄存器。
    # 但 __switch 是一个用汇编代码写的特殊函数，它不会被 Rust/C 编译器处理，所以我们需要在 __switch 中手动编写保存 s0~s11 的汇编代码
    
    # 其它寄存器中，属于调用者保存的寄存器是由编译器在高级语言编写的调用函数中自动生成的代码来完成保存的 (a0 ~ a7)；
    # 我们会通过 `extern "C"` 来使用 `__switch` 函数完成切换功能 而不是直接跳转到符号 __switch 的地址。
    # 因此在调用前后 Rust 编译器会自动帮助我们插入保存/恢复调用者保存寄存器的汇编代码
    # 还有一些寄存器属于临时寄存器 (t0 ~ t6)，不需要保存和恢复

    # part [2] 
    # save kernel stack of current task
    # a0 = current_task_cx_ptr
    # add 8: skip TaskStatus -> see TaskControlBlock
    sd sp, 8(a0)
    # save ra & s0~s11 of current execution
    sd ra, 0(a0)
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr

    # part [3] 
    # a1 = next_task_cx_ptr
    # restore ra & s0~s11 of next execution
    # for first app, 0(a1) is __restore, so ra = __restore; after __switch, ret to __restore
    ld ra, 0(a1)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr

    # part [4] 
    # restore kernel stack of next task
    # add 8: skip TaskStatus -> see TaskControlBlock
    ld sp, 8(a1)
    # 现在 a1 寄存器保存的是 TaskContaxt 或者说是 TaskContaxt.ra 或者说是 trap_return （去执行__restore)
    # 当每个应用第一次获得 CPU 使用权即将进入用户态执行的时候，它的内核栈顶放置着我们在 内核加载应用的时候 构造的一个任务上下文
    # 在 __switch 切换到该应用的任务上下文的时候，内核将会跳转到 trap_return 并返回用户态开始该应用的启动执行

    ret

