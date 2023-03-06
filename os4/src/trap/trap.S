# 在 trap.S 开头加上 .altmacro 才能正常使用 .rept 命令
.altmacro
.macro SAVE_GP n # 变量 n; 宏
    sd x\n, \n*8(sp) # \n (解引用)n的值
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    # 用 .section 指令指定当前汇编代码的段名为 .text.trampoline
    .section .text.trampoline
    # 用外部符号 __alltraps 和 __restore 标记为函数
    .globl __alltraps
    .globl __restore
    .align 2 # 4字节对齐

__alltraps:
    # 当应用 Trap 进入内核的时候，硬件会设置一些 CSR 并在 S 特权级下跳转到 __alltraps 保存 Trap 上下文
    # 此时 sp 寄存器仍指向用户栈，但 sscratch 则被设置为指向应用地址空间中存放 Trap 上下文的位置（实际在次高页面）
    # 随后，csrrw 交换 sp 和 sscratch ，并基于指向 Trap 上下文位置的 sp 开始保存通用寄存器和一些 CSR

    # 保存 Trap 上下文

    # csrrw 原型是 csrrw rd, csr, rs 可以将 CSR 当前的值读到通用寄存器 rd 中，然后将通用寄存器 rs 的值写入该 CSR ，起到的是交换 sscratch 和 sp 的效果
    csrrw sp, sscratch, sp

    # now: sp->*TrapContext in user space, sscratch->user stack

    # save general-purpose registers

    # skip x0, x0 被硬编码为 0
    sd x1, 1*8(sp) # put into the top of stack

    # 由于需要使用 x2 来找到每个寄存器应该被保存到的正确的位置，先跳过 x2

    sd x3, 3*8(sp)

    # skip tp(x4), application does not use it

    # save x5~x31
    # 按照 TrapContext 结构体的内存布局，基于内核栈的位置（sp所指地址）来从低地址到高地址分别按顺序放置 x0~x31这些通用寄存器，最后是 sstatus 和 sepc 
    .set n, 5
    # x5~x31 这 27 个通用寄存器通过循环 .rept 每次使用 SAVE_GP 宏来保存
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    # now we can use t0/t1/t2 freely, because they were saved on kernel stack

    # 我们将 CSR sstatus 和 sepc 的值分别读到寄存器 t0 和 t1 中然后保存到内核栈对应的位置上。
    # 指令 csrr rd, csr 的功能就是将 CSR 的值读到寄存器 rd 中。
    # 这里我们不用担心 t0(x5) 和 t1(x6) 被覆盖，因为它们刚刚已经被保存了。
    csrr t0, sstatus
    csrr t1, sepc

    sd t0, 32*8(sp) # sstatus
    sd t1, 33*8(sp) # sepc

    # read user stack from sscratch and save it on the kernel stack
    csrr t2, sscratch # 将 sscratch 的值读到寄存器 t2
    sd t2, 2*8(sp) # 保存到内核栈上

    # 到这里，我们就全程在应用地址空间中完成了保存 Trap 上下文的工作

    # 将内核地址空间的 token 载入到 t0 寄存器中
    ld t0, 34*8(sp) # kernel_stap

    # 将 trap handler 入口点的虚拟地址载入到 t1 寄存器中
    ld t1, 36*8(sp) # trap_handler

    # move to kernel_sp
    # 直接将 sp 修改为应用内核栈顶的地址
    ld sp, 35*8(sp) # kernel_sp

    # switch to kernel space
    # 将 satp 修改为内核地址空间的 token 并使用 sfence.vma 刷新快表，这就切换到了内核地址空间
    csrw satp, t0

    # 清空快表
    # 一旦我们修改 satp 就会切换地址空间，快表中的键值对就会失效（因为快表保存着老地址空间的映射关系，切换到新地址空间后，老的映射关系就没用了）
    # 为了确保 MMU 的地址转换能够及时与 satp 的修改同步，我们需要立即使用 sfence.vma 指令将快表清空，这样 MMU 就不会看到快表中已经过期的键值对了
    sfence.vma

    # jump to trap_handler
    # 通过 jr 指令跳转到 t1 寄存器所保存的trap handler 入口点的地址
    jr t1
    # 不能像之前的章节那样直接 call trap_handler

    # 现在 sscratch 的值是进入 Trap 之前的 sp 的值，指向用户栈，sp 则指向内核栈。

__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap

    # 当内核将 Trap 处理完毕准备返回用户态的时候会 调用 __restore （符合RISC-V函数调用规范），它有两个参数：
    # 第一个是 Trap 上下文在应用地址空间中的位置，这个对于所有的应用来说都是相同的，在 a0 寄存器中传递；
    # 第二个则是即将回到的应用的地址空间的 token(root_ppn or pa) ，在 a1 寄存器中传递

    # a0: *TrapContext in user space(Constant); a1: user space token（内核页表的起始物理地址）
    # switch to user space
    # 先切换回应用地址空间（注：Trap 上下文是保存在应用地址空间中）
    csrw satp, a1
    sfence.vma
    # 将传入的 Trap 上下文位置保存在 sscratch 寄存器中，这样 __alltraps 中才能基于它将 Trap 上下文保存到正确的位置
    csrw sscratch, a0
    # 将 sp 修改为 Trap 上下文的位置，后面基于它恢复各通用寄存器和 CSR
    mv sp, a0

    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc

    # 从内核栈顶的 Trap 上下文恢复通用寄存器和 CSR 。
    # 注意我们要先恢复 CSR 再恢复通用寄存器，这样我们使用的三个临时寄存器才能被正确恢复
    # restore sstatus/sepc
    ld t0, 32*8(sp) # sstatus
    ld t1, 33*8(sp) # sepc
    csrw sstatus, t0
    csrw sepc, t1

    # restore general-purpuse registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    
    # 目前 sp 指向保存了 Trap 上下文之后的内核栈栈顶， sscratch 指向用户栈栈顶。

    # back to user stack
    ld sp, 2*8(sp)

    # 在应用程序控制流状态被还原之后，使用 sret 指令回到 U 特权级继续运行应用程序控制流
    sret
