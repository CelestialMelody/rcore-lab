#### 实验思考题

深入理解 [trap.S](https://github.com/LearningOS/rust-based-os-comp2022/blob/main/os3-ref/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用

1. 刚进入 `__restore` 时，`a0` 代表了什么值，`__restore` 的两种使用情景

   > 在当前章节中，a0 的值与 os2 并不相同；
   >
   > os2 中，对于第一次运行的 app 或者 run_next_app，a0 寄存器的值为新构造的 trap_context。而对于 系统调用来说是进入 \_\_alltraps 之前，或者说 \_\_alltraps 保存的 trap_context
   >
   > os3 中，a0 为\_\_alltraps 保存的 trap_context（当然，对于run_first_app 或者第一次运行的 app 来说，a0 的值是无效的。
   >
   > 

2. restore: 这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释

   ````
   ld t0, 32*8(sp)
   ld t1, 33*8(sp)
   ld t2, 2*8(sp)
   csrw sstatus, t0
   csrw sepc, t1
   csrw sscratch, t2
   ````

   > [分配并使用启动栈](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#jump-practice)
   >
   > 从用户特权级 陷入（ `Trap` ）到 S 特权级的时候
   >
   > - `sstatus` 的 `SPP` 字段会被修改为 CPU 当前的特权级（U/S）。
   > - `sepc` 会被修改为 Trap 处理完成后默认会执行的下一条指令的地址。
   > - `scause/stval` 分别会被修改成这次 Trap 的原因以及相关的附加信息。
   > - CPU 会跳转到 `stvec` 所设置的 Trap 处理入口地址，并将当前特权级设置为 S ，然后从Trap 处理入口地址处开始执行
   >
   > [Trap 上下文的保存与恢复](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/4trap-handling.html#id8)
   >
   > 要先恢复 CSR 再恢复通用寄存器，这样我们使用的三个临时寄存器才能被正确恢复。

3. `restore`: 为何先跳过了 `x2` 和 `x4`？

   > sp( `x2` ) 是栈顶指针，此时我们还需要它来恢复通用寄存器，之后使用 sscratch 交换恢复 sp；
   >
   > tp( `x4` ) 是线程指针，目前我们是单线程，不会变化。

4. `restore`: `csrrw sp, sscratch, sp` 该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

   > 在这一行之前 sp 指向内核栈， sscratch 指向用户栈，现在 sp 指向用户栈， sscratch 指向内核栈；
   >
   > [Trap 上下文的保存与恢复](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/4trap-handling.html#id8)
   >
   > `csrrw` 原型是 csrrw rd, csr, rs 可以将 CSR 当前的值读到通用寄存器 rd 中，然后将通用寄存器 rs 的值写入该 CSR 。因此这里起到的是交换 sscratch 和 sp 的效果。

5. `__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

   > sret；
   >
   > 当 CPU 完成 Trap 处理准备返回的时候，需要通过一条 S 特权级的特权指令 `sret` 来完成，这一条指令具体完成以下功能：
   >
   > - CPU 会将当前的特权级按照 `sstatus` 的 `SPP` 字段设置为 U 或者 S ；
   > - CPU 会跳转到 `sepc` 寄存器指向的那条指令，然后继续执行。

6. __alltraps: 该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

   ```
   csrrw sp, sscratch, sp
   ```

   > 在这一行之前 sp 指向用户栈， sscratch 指向内核栈，现在 sp 指向内核栈， sscratch 指向用户栈；
   >
   > [Trap 上下文的保存与恢复](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/4trap-handling.html#id8)

7. 从 U 态进入 S 态是哪一条指令发生的？

   > user/src/bin 的用户程序系统调用或异常退出越界访问，使用 ecall 指令时发生的。



---

#### 实验感悟

实验要求中，统计任务的相关系统调用的次数

我一开始是想通过寻找函数`syscall`的调用位置来进行补充，但发现仅在函数`trap_handler`中使用过系统调用，而后者实际是与嵌入的汇编代码进行交互的，不知从何处下手，故陷入僵局

>[系统调用-特权级机制](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html)
>
>- 内核和 U 模式的应用程序之间的接口被称为 **应用程序二进制接口** (Application Binary Interface, ABI) —— **系统调用**
>- 系统调用的本质是一种异常 —— **陷入异常控制流**，在该过程中会切换 CPU 特权级
>- 当调用一个系统调用时会触发 CPU 异常，CPU 进入异常处理流程
>
>目前系统调用是用`trap_handler`控制的

> [不同类型的上下文与切换](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/2task-switching.html#id4)
>
> [任务切换的设计与实现](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/2task-switching.html#term-task-switch-impl)
>
> - 与 Trap 切换不同，它不涉及特权级切换
> - 与 Trap 切换不同，它的一部分是由编译器帮忙完成的
> - 与 Trap 切换相同，它对应用是透明的
>
> 任务切换是来自两个不同应用在内核中的 Trap 控制流之间的切换 ——  **换栈**
>

于是考虑到从系统调用函数`syscall`中寻找答案；

实际上该函数也是由各种系统调用函数组成，在深入部分函数，如 `sys_yield`，明白该函数是通过 `TaskManager` 调用的，而我们维护了一个全局的任务管理器；

结合任务提示:

- `系统调用次数可以考虑在进入内核态系统调用异常处理函数之后，进入具体系统调用函数之前维护`
- 我们可以在进入具体函数调用之前，通过任务管理器，获取当前任务，对应的`syscall_times[syscall_id] += 1`即可（当然需要修改`TaskControlBlock`）
- 补充函数 `pub fn update_syscall_record(syscall_id: usize)`
