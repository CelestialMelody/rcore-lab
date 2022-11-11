# 在 trap.S 开头加上 .altmacro 才能正常使用 .rept 命令
.altmacro
.macro SAVE_GP n # 变量 n; 宏
    sd x\n, \n*8(sp) # \n 大概解释为n的值
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    # 用外部符号 __alltraps 和 __restore 标记为函数
    .globl __alltraps
    .globl __restore
    .align 2 # 4字节对齐

# 进入 S 特权级的 Trap 处理之前，必须保存原控制流的寄存器状态，这一般通过 内核栈来保存
# 首先通过 __alltraps 将 Trap 上下文保存在内核栈上，
# 然后跳转到使用 Rust 编写的 trap_handler 函数完成 Trap 分发及处理。
# 当 trap_handler 返回之后，使用 __restore 从保存在内核栈上的 Trap 上下文恢复寄存器。
# 最后通过一条 sret 指令回到应用程序执行

# 用外部符号 __alltraps 和 __restore 标记为函数
__alltraps:
    # 保存 Trap 上下文
    csrrw sp, sscratch, sp

    # csrrw 原型是 csrrw rd, csr, rs 可以将 CSR 当前的值读到通用寄存器 rd 中，然后将通用寄存器 rs 的值写入该 CSR 。
    # 因此这里起到的是交换 sscratch 和 sp 的效果。
    # 在这一行之前 sp 指向用户栈， sscratch 指向内核栈（原因稍后说明），现在 sp 指向内核栈， sscratch 指向用户栈。

    # now sp->kernel stack, sscratch->user stack
    # allocate a TrapContext on kernel stack
    # RISC-V 架构中，栈是从高地址向低地址增长的，因此这里先减去 TrapContext 的大小，然后将 sp 指向这个位置。   
    addi sp, sp, -34*8
    # save general-purpose registers
    # skip x0, x0 被硬编码为 0
    sd x1, 1*8(sp) # put into the top of stack
    # skip sp(x2), we will save it later
    # need to use x2 来找到每个寄存器应该被保存到的正确的位置
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    # 按照 TrapContext 结构体的内存布局，基于内核栈的位置（sp所指地址）来从低地址到高地址分别按顺序放置 x0~x31这些通用寄存器，最后是 sstatus 和 sepc 
    .set n, 5
    # x5~x31 这 27 个通用寄存器通过类似循环的 .rept 每次使用 SAVE_GP 宏来保存
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    # we can use t0/t1/t2 freely, because they were saved on kernel stack
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
    # 注意： sscratch 的值是进入 Trap 之前的 sp 的值，指向用户栈。而现在的 sp 则指向内核栈。
    # set input argument of trap_handler(cx: &mut TrapContext)
    mv a0, sp # 寄存器 a0 指向内核栈的栈顶 sp
    # 让寄存器 a0 指向内核栈的栈指针也就是我们刚刚保存的 Trap 上下文的地址，
    # 这是由于我们接下来要调用 trap_handler 进行 Trap 处理，它的第一个参数 cx 由调用规范要从 a0 中获取。
    # 而 Trap 处理函数 trap_handler 需要 Trap 上下文的原因在于：
    # 它需要知道其中某些寄存器的值，比如在系统调用的时候应用程序传过来的 syscall ID 和对应参数。
    # 我们不能直接使用这些寄存器现在的值，因为它们可能已经被修改了，因此要去内核栈上找已经被保存下来的值

    # trap_handler(cx: &mut TrapContext) -> &mut TrapContext
    # 参数 cx 使用 a0 传参，使用 a0 作为返回 故没有改变
    call trap_handler

__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap
    #  __restore 的实现需要做出变化：它不再需要 在开头 mv sp, a0 了
    # 因为在 __switch 之后, sp 就已经正确指向了我们需要的 Trap 上下文地址
    # mv sp, a0 # 传入参数a0: 内核栈中保存的用户上下文; os3 delete this

    # now sp->kernel stack(after allocated), sscratch->user stack

    # 从内核栈顶的 Trap 上下文恢复通用寄存器和 CSR 。
    # 注意我们要先恢复 CSR 再恢复通用寄存器，这样我们使用的三个临时寄存器才能被正确恢复
    # restore sstatus/sepc
    ld t0, 32*8(sp) # sstatus
    ld t1, 33*8(sp) # sepc
    ld t2, 2*8(sp) # sscratch; x2(sp) 指向 USER_STACK 栈
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    # restore general-purpuse registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    
    # now sp 指向保存了 Trap 上下文之后的内核栈栈顶， sscratch 指向用户栈栈顶。

    # 在内核栈上回收 Trap 上下文所占用的内存，回归进入 Trap 之前的内核栈栈顶 
    # release TrapContext on kernel stack
    addi sp, sp, 34*8

    # 再次交换 sscratch 和 sp，现在 sp 重新指向用户栈，sscratch 也依然保存进入 Trap 之前的状态并指向内核栈栈顶
    # now sp->kernel stack, sscratch->user stack
    csrrw sp, sscratch, sp

    # 在应用程序控制流状态被还原之后，使用 sret 指令回到 U 特权级继续运行应用程序控制流
    sret
