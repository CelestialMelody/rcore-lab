1. 当内核仅运行单个应用的时候，无论该任务主动/被动交出 CPU 资源最终都会交还给自己，这将导致传给 `__switch` 的两个参数相同，也就是某个 Trap 控制流自己切换到自己的情形，所以怎样处理这种情况？



2. 区分与联系

   1. TrapContext 与 TaskContext
   2. switch 与 alltraps，restore
   
   > 原问题：TaskContext 保存了寄存器，而这些寄存器也都保存在 TrapContext 里。switch 的时候恢复了TaskContext，之后又跳到 restore 恢复了 TrapContext，所以 TaskContext 保存的 s 寄存器实际没有起到作用吧？
   
   > TrapContext 中保存的寄存器记录了应用陷入 S 特权级之前的 CPU 状态，而 TaskContext 则可以看成一个应用在 S 特权级进行 Trap 处理的过程中调用 switch 之前的 CPU 状态。
   >
   > 当恢复 TaskContext 之后会继续进行 Trap 处理，而 restore 恢复 TrapContext 之后则是会回到用户态执行应用。
   >
   > 另外，保存 TrapContext 之后进行 Trap 处理的时候，s0-s11 寄存器可能会被覆盖，后面进行任务切换时这些寄存器会被保存到 TaskContext 中，也就是说这两个 上下文 中的 s0-s11 也很可能是不同的。



3. 原问题：ra 与 ret ：ret指令执行后，程序直接跳转到ra寄存器所存的地址？

   > ret 指令执行后，程序就是直接跳转到ra寄存器所存的地址的吧。rcore 第一个 task 就是通过构造 trap context 模拟 alltraps 完成的状态，然后将 ra 指向 restore，然后在 switch.S 中 ret 直接执行 restore，再通过 restore 回到 user space 执行代码。

   > ra 在 switch 中被修改了。过程 trap -> switch taskA -> switch taskB -> restore



4. 原问题：每次处理 trap 的时候从 call trap_handler 一直到 switch 函数执行，这中间的调用链是有一些压栈操作的，但是当一个 task exit 之后，就再也不会 switch 回来了，那么这个 task 对应的调用链里的栈空间就是无法再使用了是么？

   > 第二章为了简单起见，每个 task 的栈空间仅会被使用一次，对应的 task 退出之后就会永久闲置。等到第五章引入了进程之后，可以看到在进程退出之后，它的栈空间所在的物理内存会被回收并可以供其他进程使用。



5. 为什么对比第二章的 trap.S 文件少了 `mv sp, a0`

   > `__restore`在这里被两种情况复用了：
   >
   > 1. 正常从`__alltraps`走下来的`trap_handler`流程。如果是这种情况，`trap_handler`会在`a0`里返回之前通过`mv a0, sp`传进去的`&mut TrapContext`，所以这里`sp`和`a0`相同没有必要再`mv sp, a0`重新设置一遍。
   > 2. app 第一次被`__switch`的时候通过`__restore`开始运行。这时候`a0`是个无关的数据（指向上一个`TaskContext`的指针），这里再 `mv sp a0` 就不对了，而`__restore`要的`TrapContext`已经在`__switch`的恢复过程中被放在`sp`上了。（这个`sp`就是初始化时写完`TrapContext`后的内核栈顶）



6. 为什么本章会有多个内核栈？

   > 这章改成了每个任务都有一个内核栈了。
   >
   > 一开始我也是想着怎么能优雅地在不同的任务间切换并安全地共享同一个内核栈，发现很困难。
   >
   > 每个任务一个内核栈是最简单直接的做法。

   > 所有应用程序共享同一个内核栈的设计[参考](https://github.com/YdrMaster/rCore-Tutorial-in-single-workspace/blob/f6a9a65ea4f393c8d8226af5f802d8b298a863bb/ch4/src/main.rs#L99)



7. 如何确定 在QEMU环境下 CLOCK_FREQ的值？我简单搜索了一圈也没有找到源

   > 在 qemu 提供的设备树里有，可以用：
   >
   > ```
   > qemu-system-riscv64 -machine virt,dumpdtb=dump.dtb
   > ```
   >
   > 得到 dump.dtb 文件，然后：
   >
   > ```
   > dtc -o dump.dts dump.dtb
   > ```
   >
   > 得到你的环境里的 dts 文件，进去查一下，不同版本不一定一样。
   
   
   
   但是，看上去似乎数值上有些偏差 `timebase-frequency = <0x989680>` => `10000000` != `12500000`
   另外，这个似乎是cpu主屏，而我们找的`CLOCK_FREQ` 更像是对应[这个](https://github.com/YdrMaster/dtb-walker/blob/e9f86dcee033f57399191806aedf41a20c3d03b5/examples/qemu-virt.dts#L145)？
   
   > 不是，其实是教程有问题，老版本 qemu 是 12.5 MHz，新的改了。`CLOCK_FREQ` 就是 CPU 主频。不过说是 CPU 主频，但实际就是 time 寄存器自增的频率而已，这个必须是一个稳定的值，真正 CPU 运行的频率不一定。



8. 我们在使用 switch 加载了 s0-s11 寄存器之后调用 restore 有重新加载了 x5-x27 这个过程， switch 加载的寄存器数据不久被 restore 覆盖了么？

   > 在这里我们讨论的是如何从内核态第一次进入用户态执行用户态代码。
   >
   > 此时`__switch`加载的TaskContext是由`TaskContext::goto_restore`生成的，可以看到里面的 s0-s11 均为 0，也就是并不带有任何信息，只是起到一个占位作用。真正有意义的是 TaskContext中 ra 和 sp 两个寄存器的值，它们能帮助我们从内核栈的位置开始执行`__restore`回到用户态。这个过程中 s0-s11 会被覆盖，但正如之前所说这些寄存器的值目前本来就是无意义的，可以随意覆盖。



9. 关于计时器：

   ```rust
   /// 10ms: 1s / 100;
   /// TICKS_PRE_SECOND = 100;
   /// CLOCK_FREQ: the number of ticks in 1s;
   /// CLOCK_FREQ / TICKS_PRE_SECOND: number of ticks in 10ms.
   pub fn set_next_trigger() {
       // get_time: get mtime value
       // set_timer: set mtimecmp value
       // time interrupt will be triggered when mtime == mtimecmp
       set_timer(get_time() + (CLOCK_FREQ / TICKS_PRE_SECOND));
   }
   ```

   > `CLOCK_FREQ`是时钟频率，而计数器也是对该时钟的时钟周期进行计数，因此`CLOCK_FREQ`也是一秒之内计数器的增量。
   >
   > 而我们将一秒钟分为`TICKS_PER_SEC`，也即100个时间片，每个时间片10ms，那么每个时间片内计数器的增量就应该是一秒内的总体增量除以这个整体被分为的份数，所以是`CLOCK_FREQ/100`



10. “RISC-V 架构要求处理器要有一个内置时钟，其频率一般低于 CPU 主频”

    > 实际上在 CPU 的微架构内通常有多个不同频率的时钟源（比如各种晶振等），然后他们进行一些组合电路的处理又会得到更多不同频率的信号，不同的电路模块可能使用不同频率的时钟信号以满足同步需求。
    >
    > CPU 的主体流水线所采用的时钟信号的频率是 CPU 的主频，但同时还有另一个用来计时的时钟模块（也就是上面提到的时钟）运行在另一个不同的频率。
    >
    > 他们两个的另一个区别是，CPU 的时钟周期在`mcycle`寄存器中计数，而时钟的时钟周期在`mtime`寄存器中计数，因此这是两个独立且不同的频率。



11. 处理陷入时，是不是应该改成只要切换了任务，就重设定时器？现在因为定时器中断得到执行权的任务会因为定时器中断提前换出。

    > 这样做能使得分时更加精细，但我们目前着重于保底机制：即任务运行了超过一个时间片一定会被切换，而不保证任务单次运行时间的下限。