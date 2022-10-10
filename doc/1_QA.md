**bios和sbi是什么样的关系**

> [解释者](https://github.com/denglj)

- SBI 是 RISC-V Supervisor Binary Interface 规范的缩写，OpenSBI 是RISC-V官方用C语言开发的SBI参考实现；RustSBI 是用Rust语言实现的SBI。

- BIOS 是 Basic Input/Output System，作用是引导计算机系统的启动以及硬件测试，并向OS提供硬件抽象层。

- 机器上电之后，会从ROM中读取引导代码，引导整个计算机软硬件系统的启动。而整个启动过程是分为多个阶段的，现行通用的多阶段引导模型为：

  - ROM -> LOADER -> RUNTIME -> BOOTLOADER -> OS

  - Loader 要干的事情，就是内存初始化，以及加载 Runtime 和 BootLoader 程序。而Loader自己也是一段程序，常见的Loader就包括 BIOS 和 UEFI，后者是前者的继任者。
  - Runtime 固件程序是为了提供运行时服务（runtime services），它是对硬件最基础的抽象，对OS提供服务，当我们要在同一套硬件系统中运行不同的操作系统，或者做硬件级别的虚拟化时，就离不开Runtime服务的支持。SBI就是RISC-V架构的Runtime规范。
  - BootLoader 要干的事情包括文件系统引导、网卡引导、操作系统启动配置项bios和sbi是什么样的关系

- 而 BIOS/UEFI 的大多数实现，都是 Loader、Runtime、BootLoader 三合一的，所以不能粗暴的认为 SBI 跟 BIOS/UEFI 有直接的可比性。如果把BIOS当做一个泛化的术语使用，而不是指某个具体实现的话，那么可以认为 SBI 是 BIOS 的组成部分之一。



**ABI 与 SBI**

> [参考链接](https://blog.csdn.net/u011011827/article/details/119185091)
>
> [SBI](https://zh.m.wikipedia.org/zh-hans/%E7%89%B9%E6%9D%83%E5%B1%82%E4%BA%8C%E8%BF%9B%E5%88%B6%E6%8E%A5%E5%8F%A3)
>
> [ABI](https://zh.wikipedia.org/wiki/%E5%BA%94%E7%94%A8%E4%BA%8C%E8%BF%9B%E5%88%B6%E6%8E%A5%E5%8F%A3)

- risc-v 的 sbi 是个标准 , 且 SBI 是 risc-v 独有的东西,其他架构没有这个概念；实现包括 [opensbi](https://github.com/riscv-software-src/opensbi) 和 [rustsbi](https://github.com/rustsbi/rustsbi)，这些实现都跑在M-mode，这些实现都为运行在 S-mode 上的软件(例如linux) 提供服务
- 它提供了特权层的运行环境，使得特权层软件能使用环境调用指令，来执行平台相关的操作。典型的的特权层接口应用有：
  - 类似于[Unix](https://zh.m.wikipedia.org/wiki/Unix)的操作系统中，机器级和特权级的访问接口；
  - 监视特权级和虚拟特权级中，作为虚拟化环境的调用接口。

- 常见的应用程序其实是运行在由硬件、操作系统内核、运行时库、图形界面支持库等所包起来的一个 [执行环境 (Execution Environment)](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/1what-is-os.html?highlight=abi#exec-env) 中，应用程序只需根据与系统软件约定好的应用程序二进制接口 (ABI, Application Binary Interface) 来请求执行环境提供的各种服务或功能，从而完成应用程序自己的功能
- ABI涵盖了各种细节，如：
  - 数据类型的大小、布局和对齐;
  - [调用约定](https://zh.wikipedia.org/wiki/调用约定)（控制着函数的参数如何传送以及如何接受返回值），例如，是所有的参数都通过栈传递，还是部分参数通过寄存器传递；哪个寄存器用于哪个函数参数；通过栈传递的第一个函数参数是最先push到栈上还是最后； ([risc-v调用规范](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#term-calling-convention))
  - [系统调用](https://zh.wikipedia.org/wiki/系统调用)的编码和一个应用如何向操作系统进行系统调用；
  - 以及在一个完整的操作系统ABI中，[目标文件](https://zh.wikipedia.org/wiki/目标文件)的二进制格式、程序库等等。



**为什么 risc-v 有 sbi，其他架构没有sbi**

- 因为 risc-v 的开放性，risc-v 有 s-mode 与 m-mode 的交互, 并且开放了标准，让更多的人可以参与实现这个标准
- arm 有 EL1 与 EL2 的交互，EL2 与 EL3 的交互，但是这些交互都没有开放标准
- EL0 与 EL1 / s-mode 与 u-mode 交互没有标准, linux 有一套标准(系统调用) , win有一套标准(系统调用)



**Qemu 启动流程**

-  Qemu 的第一阶段固定跳转到 `0x80000000`
-  第二阶段的 bootloader `rustsbi-qemu.bin` 放在以物理地址 `0x80000000` 开头的物理内存中，这样就能保证 `0x80000000` 处正好保存 bootloader 的第一条指令yun x
-  对于不同的 bootloader 而言，下一阶段软件的入口不一定相同，而且获取这一信息的方式和时间点也不同：入口地址可能是一个预先约定好的固定的值，也有可能是在 bootloader 运行期间才动态获取到的值
   - RustSBI 是将下一阶段的入口地址预先约定为固定的 `0x80200000` ，在 RustSBI 的初始化工作完成之后，它会跳转到该地址并将计算机控制权移交给下一阶段的软件——内核镜像
-  第三阶段为了正确地和上一阶段的 RustSBI 对接，需要保证内核的第一条指令位于物理地址 `0x80200000` 处。为此，需要将内核镜像预先加载到 Qemu 物理内存以地址 `0x80200000` 开头的区域上。一旦 CPU 开始执行内核的第一条指令，证明计算机的控制权已经被移交给我们的内核



**0x80200000 可否改为其他地址？**

内核并不是位置无关的，所以我们必须将内存布局的起始地址设置为 `0x80200000` ，与之匹配我们也必须将内核加载到这一地址



**如何得到一个能够在 Qemu 上成功运行的内核镜像呢？**

由于链接器默认的内存布局并不能符合我们的要求，实现与 Qemu 的正确对接，我们可以通过 **链接脚本** (Linker Script) 调整链接器的行为，使得最终生成的可执行文件的内存布局符合我们的预期。

首先我们需要通过链接脚本调整内核可执行文件的内存布局，使得内核被执行的第一条指令位于地址 `0x80200000` 处，同时代码段所在的地址应低于其他段。这是因为 Qemu 物理内存中低于 `0x80200000` 的区域并未分配给内核，而是主要由 RustSBI 使用。

此时得到的内核可执行文件完全符合我们对于内存布局的要求，但是我们不能将其直接提交给 Qemu ，因为它除了实际会被用到的代码和数据段之外还有一些多余的元数据，这些元数据无法yun x被 Qemu 在加载文件时利用，且会使代码和数据段被加载到错误的位置

<img src="http://rcore-os.cn/rCore-Tutorial-Book-v3/_images/link-sections.png" alt="link-sections.png" style="zoom: 67%;" />

- 红色的区域表示内核可执行文件中的元数据

- 深蓝色的区域表示各个段（包括代码段和数据段）(linker.ld session)
- 浅蓝色区域则表示内核被执行的第一条指令，它位于深蓝色区域的开头

图示的上半部分中，我们直接将内核可执行文件 `os` 加载到 Qemu 内存的 `0x80200000` 处，由于内核可执行文件的开头是一段元数据，这会导致 Qemu 内存 `0x80200000` 处无法找到内核第一条指令，也就意味着 RustSBI 无法正常将计算机控制权转交给内核。相反，图示的下半部分中，将元数据丢弃得到的内核镜像 `os.bin` 被加载到 Qemu 之后，则可以在 `0x80200000` 处正确找到内核第一条指令。

其次，我们需要将内核可执行文件中的元数据丢掉得到内核镜像，此内核镜像仅包含实际会用到的代码和数据。这则是因为 Qemu 的加载功能过于简单直接，它直接将输入的文件逐字节拷贝到物理内存中，因此也可以说这一步是我们在帮助 Qemu 手动将可执行文件加载到物理内存中。

> 元数据：为描述数据的数据（data about data），主要是描述数据属性（property）的信息，内存地址并不需要



<img src="http://rcore-os.cn/rCore-Tutorial-Book-v3/_images/StackFrame.png" alt="StackFrame.png" style="zoom: 25%;" />

如何使得函数返回时能够跳转到调用该函数的下一条指令，即使该函数在代码中的多个位置被调用？ [链接](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#:~:text=%E5%AF%B9%E6%AD%A4%EF%BC%8C%E6%8C%87%E4%BB%A4%E9%9B%86%E5%BF%85%E9%A1%BB%E7%BB%99%E7%94%A8%E4%BA%8E%E5%87%BD%E6%95%B0%E8%B0%83%E7%94%A8%E7%9A%84%E8%B7%B3%E8%BD%AC%E6%8C%87%E4%BB%A4%E4%B8%80%E4%BA%9B%E9%A2%9D%E5%A4%96%E7%9A%84%E8%83%BD%E5%8A%9B%EF%BC%8C%E8%80%8C%E4%B8%8D%E5%8F%AA%E6%98%AF%E5%8D%95%E7%BA%AF%E7%9A%84%E8%B7%B3%E8%BD%AC)

> 利用jarl ret 指令实现函数调用
>
> 使用栈保存上下文信息，sp, fp（均为[被调用者保存寄存器](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#:~:text=%E8%A2%AB%E8%B0%83%E7%94%A8%E8%80%85%E4%BF%9D%E5%AD%98(Callee%2DSaved)%20%E5%AF%84%E5%AD%98%E5%99%A8)）记录栈的地址范围
>
> 栈上多个 `fp` 信息实际上保存了一条完整的函数调用链，通过适当的方式我们可以实现对函数调用关系的跟踪
>



对于一个函数而言，保证它调用某个子函数之前，以及该子函数返回到它之后（某些）通用寄存器的值保持不变有何意义？ [链接](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#:~:text=%E5%A6%82%E6%9E%9C%E6%88%91%E4%BB%AC%E8%AF%95%E5%9B%BE,%E6%B0%B8%E4%B9%85%E4%B8%A2%E5%A4%B1%20%E3%80%82)

> 保证被调用函数返回时调用者函数能正确执行



调用者函数和被调用者函数如何合作保证调用子函数前后寄存器内容保持不变？调用者保存和被调用者保存寄存器的保存与恢复各自由谁负责？它们暂时被保存在什么位置？它们于何时被保存和恢复（如函数的开场白/退场白）？

> 使用栈保存[函数调用上下文](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#:~:text=%E5%9C%A8%E6%8E%A7%E5%88%B6%E6%B5%81%E8%BD%AC%E7%A7%BB%E5%89%8D%E5%90%8E%E9%9C%80%E8%A6%81%E4%BF%9D%E6%8C%81%E4%B8%8D%E5%8F%98%E7%9A%84%E5%AF%84%E5%AD%98%E5%99%A8%E9%9B%86%E5%90%88%E7%A7%B0%E4%B9%8B%E4%B8%BA%20%E5%87%BD%E6%95%B0%E8%B0%83%E7%94%A8%E4%B8%8A%E4%B8%8B%E6%96%87)(在控制流转移前后需要保持不变的寄存器集)信息，sp, fp（均为被调用者保存寄存器）记录栈的地址范围
>
> [保存与恢复](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#:~:text=%E5%8F%91%E7%8E%B0%E6%97%A0%E8%AE%BA%E6%98%AF%E8%B0%83%E7%94%A8%E5%87%BD%E6%95%B0%E8%BF%98%E6%98%AF%E8%A2%AB%E8%B0%83%E7%94%A8%E5%87%BD%E6%95%B0%EF%BC%8C%E9%83%BD%E4%BC%9A%E5%9B%A0%E8%B0%83%E7%94%A8%E8%A1%8C%E4%B8%BA%E8%80%8C%E9%9C%80%E8%A6%81%E4%B8%A4%E6%AE%B5%E5%8C%B9%E9%85%8D%E7%9A%84%E4%BF%9D%E5%AD%98%E5%92%8C%E6%81%A2%E5%A4%8D%E5%AF%84%E5%AD%98%E5%99%A8%E7%9A%84%E6%B1%87%E7%BC%96%E4%BB%A3%E7%A0%81%EF%BC%8C%E5%8F%AF%E4%BB%A5%E5%88%86%E5%88%AB%E5%B0%86%E5%85%B6%E7%A7%B0%E4%B8%BA%20%E5%BC%80%E5%9C%BA%20(Prologue)%20%E5%92%8C%20%E7%BB%93%E5%B0%BE%20(Epilogue)%EF%BC%8C%E5%AE%83%E4%BB%AC%E4%BC%9A%E7%94%B1%E7%BC%96%E8%AF%91%E5%99%A8%E5%B8%AE%E6%88%91%E4%BB%AC%E8%87%AA%E5%8A%A8%E6%8F%92%E5%85%A5%EF%BC%8C%E6%9D%A5%E5%AE%8C%E6%88%90%E7%9B%B8%E5%85%B3%E5%AF%84%E5%AD%98%E5%99%A8%E7%9A%84%E4%BF%9D%E5%AD%98%E4%B8%8E%E6%81%A2%E5%A4%8D) 



在 RISC-V 架构上，调用者保存和被调用者保存寄存器如何划分（特别地，思考 sp 和 ra 是调用者还是被调用者保存寄存器？为什么？）？如何使用寄存器传递函数调用的参数和返回值？

> sp 被调用者保持寄存器，指向内存中栈顶地址；函数中的结尾代码负责将开场代码分配的栈帧回收，仅仅需要将 `sp` 的值增加相同的字节数回到分配之前的状态
>
> ra 调用者保存寄存器，在函数的开头和结尾保存/恢复
>
> 寄存器调用规范