### 应用程序执行环境

```
├── bootloader(内核依赖的运行在 M 特权级的 SBI 实现，本项目中我们使用 RustSBI)
│   └── rustsbi-qemu.bin(可运行在 qemu 虚拟机上的预编译二进制版本)
└── os(我们的内核实现放在 os 目录下)
    ├── Cargo.toml(内核实现的一些配置文件)
    ├── Makefile (构建文件)
    └── src(所有内核的源代码放在 os/src 目录下)
        ├── console.rs(将打印字符的 SBI 接口进一步封装实现更加强大的格式化输出)
        ├── entry.asm(设置内核执行环境的的一段汇编代码)
        ├── lang_items.rs(需要我们提供给 Rust 编译器的一些语义项，目前包含内核 panic 时的处理逻辑)
        ├── linker.ld(控制内核内存布局的链接脚本以使内核运行在 qemu 虚拟机上)
        ├── main.rs(内核主函数)
        └── sbi.rs(调用底层 SBI 实现提供的 SBI 接口)
```



```
├── bootloader(内核依赖的运行在 M 特权级的 SBI 实现，本项目中我们使用 RustSBI)
│   └── rustsbi-qemu.bin(可运行在 qemu 虚拟机上的预编译二进制版本)
└── os(我们的内核实现放在 os 目录下)
    ├── Cargo.toml(内核实现的一些配置文件)
    ├── Makefile (构建文件)
    └── src(所有内核的源代码放在 os/src 目录下)
        ├── console.rs(将打印字符的 SBI 接口进一步封装实现更加强大的格式化输出)
        ├── entry.asm(设置内核执行环境的的一段汇编代码)
        ├── lang_items.rs(需要我们提供给 Rust 编译器的一些语义项，目前包含内核 panic 时的处理逻辑)
        ├── linker.ld(控制内核内存布局的链接脚本以使内核运行在 qemu 虚拟机上)
        ├── main.rs(内核主函数)
        └── sbi.rs(调用底层 SBI 实现提供的 SBI 接口)
```



![app-software-stack.png](./pic/app-software-stack.png)



> 1. [执行环境](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/1what-is-os.html?highlight=abi#:~:text=%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E5%8F%AA%E9%9C%80%E6%A0%B9%E6%8D%AE%E4%B8%8E%E7%B3%BB%E7%BB%9F%E8%BD%AF%E4%BB%B6%E7%BA%A6%E5%AE%9A%E5%A5%BD%E7%9A%84%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E4%BA%8C%E8%BF%9B%E5%88%B6%E6%8E%A5%E5%8F%A3%20(ABI%2C%20Application%20Binary%20Interface)%20%E6%9D%A5%E8%AF%B7%E6%B1%82%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E6%8F%90%E4%BE%9B%E7%9A%84%E5%90%84%E7%A7%8D%E6%9C%8D%E5%8A%A1%E6%88%96%E5%8A%9F%E8%83%BD%EF%BC%8C%E4%BB%8E%E8%80%8C%E5%AE%8C%E6%88%90%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E8%87%AA%E5%B7%B1%E7%9A%84%E5%8A%9F%E8%83%BD%E3%80%82%E5%9F%BA%E4%BA%8E%E8%BF%99%E6%A0%B7%E7%9A%84%E8%A7%82%E5%AF%9F%EF%BC%8C%E6%88%91%E4%BB%AC%E5%8F%AF%E4%BB%A5%E6%8A%8A%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E7%9A%84%E5%AE%9A%E4%B9%89%E7%AE%80%E5%8C%96%E4%B8%BA%EF%BC%9A%20%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E7%9A%84%E8%BD%AF%E4%BB%B6%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83)
> 2. [查看程序运行使用的系统调用](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/1app-ee-platform.html#:~:text=Hello%2C%20world!%20%E7%94%A8%E5%88%B0%E4%BA%86%E5%93%AA%E4%BA%9B%E7%B3%BB%E7%BB%9F%E8%B0%83%E7%94%A8%EF%BC%9F)
> 3. [多层执行环境都是必需的吗](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/1app-ee-platform.html#:~:text=%E6%B3%A8%E8%A7%A3-,%E5%A4%9A%E5%B1%82%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E9%83%BD%E6%98%AF%E5%BF%85%E9%9C%80%E7%9A%84%E5%90%97%EF%BC%9F,-%E9%99%A4%E4%BA%86%E6%9C%80%E4%B8%8A%E5%B1%82)



#### 平台与目标三元组

> 1. [现代编译器工具集（以C编译器为例）的主要工作流程](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/1app-ee-platform.html#:~:text=%E7%8E%B0%E4%BB%A3%E7%BC%96%E8%AF%91%E5%99%A8%E5%B7%A5%E5%85%B7%E9%9B%86%EF%BC%88%E4%BB%A5C%E7%BC%96%E8%AF%91%E5%99%A8%E4%B8%BA%E4%BE%8B%EF%BC%89%E7%9A%84%E4%B8%BB%E8%A6%81%E5%B7%A5%E4%BD%9C%E6%B5%81%E7%A8%8B)

- 通过 **目标三元组** (Target Triplet) 来描述一个目标平台，一般包括 CPU 架构、CPU 厂商、操作系统和运行时库，它们确实都会控制可执行文件的生成



#### Rust 标准库与核心库

- [裸机平台 (bare-metal)](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/1app-ee-platform.html#:~:text=%E8%A2%AB%E6%88%91%E4%BB%AC%E7%A7%B0%E4%B8%BA-,%E8%A3%B8%E6%9C%BA%E5%B9%B3%E5%8F%B0%20(bare%2Dmetal),-%E3%80%82%E8%BF%99%E6%84%8F%E5%91%B3%E7%9D%80)

- [Rust 语言核心库 core](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/1app-ee-platform.html#:~:text=%E8%A3%81%E5%89%AA%E8%BF%87%E5%90%8E%E7%9A%84-,Rust%20%E8%AF%AD%E8%A8%80%E6%A0%B8%E5%BF%83%E5%BA%93%20core,-%E3%80%82core%E5%BA%93%E6%98%AF)



---

### 移除标准库依赖

> 1. [RV64GC](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/2remove-std.html#id2:~:text=%E5%AE%83%E8%83%BD%E5%9C%A8-,RV64GC,-%EF%BC%88%E5%8D%B3%E5%AE%9E%E7%8E%B0%E4%BA%86)
> 2. [库操作系统（Library OS，LibOS）](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/2remove-std.html#id2:~:text=%E5%BA%93%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%EF%BC%88Library%20OS%EF%BC%8CLibOS%EF%BC%89)

#### 移除 println! 宏

println! 宏所在的 Rust 标准库 std 需要通过系统调用获得操作系统的服务，而如果要构建运行在裸机上的操作系统，就不能再依赖标准库了

> [本地编译与交叉编译](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/2remove-std.html#id2:~:text=%E6%B3%A8%E8%A7%A3-,%E6%9C%AC%E5%9C%B0%E7%BC%96%E8%AF%91%E4%B8%8E%E4%BA%A4%E5%8F%89%E7%BC%96%E8%AF%91,-%E4%B8%8B%E9%9D%A2%E6%8C%87%E7%9A%84)

- 在 `os` 目录下新建 `.cargo` 目录，并在这个目录下创建 `config` 文件 (在 `cargo build` 的时候不必再加上 `--target` 参数)
- 在 `main.rs` 的开头加上一行 `#![no_std]` 来告诉 Rust 编译器不使用 Rust 标准库 std 转而使用核心库 core，core库不需要操作系统的支持）
- 提供panic_handler功能应对致命错误
- 我们创建一个新的子模块 `lang_items.rs` 实现panic函数，并通过 `#[panic_handler]` 属性通知编译器用panic函数来对接 `panic!` 宏



#### 移除 main 函数

- [`start` 语义项](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/2remove-std.html#id2:~:text=%E7%BC%BA%E5%B0%91%E4%B8%80%E4%B8%AA%E5%90%8D%E4%B8%BA-,start%20%E7%9A%84%E8%AF%AD%E4%B9%89%E9%A1%B9,-%E3%80%82%E6%88%91%E4%BB%AC%E5%9B%9E%E5%BF%86%E4%B8%80%E4%B8%8B)代表了标准库 std 在执行应用程序之前需要进行的一些初始化工作



---

### 内核第一条指令

[端序或尾序](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E6%B3%A8%E8%A7%A3-,%E7%AB%AF%E5%BA%8F%E6%88%96%E5%B0%BE%E5%BA%8F,-%E7%AB%AF%E5%BA%8F%E6%88%96)

[内存地址对齐](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E6%B3%A8%E8%A7%A3-,%E5%86%85%E5%AD%98%E5%9C%B0%E5%9D%80%E5%AF%B9%E9%BD%90,-%E5%86%85%E5%AD%98%E5%9C%B0%E5%9D%80%E5%AF%B9)

[Qemu 模拟器](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E7%9A%84%E4%B8%80%E7%A7%8D%20bug%E3%80%82-,%E4%BA%86%E8%A7%A3%20Qemu%20%E6%A8%A1%E6%8B%9F%E5%99%A8,-%23)

[Qemu 启动流程](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E8%BF%9B%E8%A1%8C%E6%B7%B1%E5%85%A5%E8%AE%A8%E8%AE%BA%E3%80%82-,Qemu%20%E5%90%AF%E5%8A%A8%E6%B5%81%E7%A8%8B,-%23)

[真实计算机的加电启动流程](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E6%B3%A8%E8%A7%A3-,%E7%9C%9F%E5%AE%9E%E8%AE%A1%E7%AE%97%E6%9C%BA%E7%9A%84%E5%8A%A0%E7%94%B5%E5%90%AF%E5%8A%A8%E6%B5%81%E7%A8%8B,-%E7%9C%9F%E5%AE%9E%E8%AE%A1%E7%AE%97%E6%9C%BA)

#### 程序内存布局与编译流程

<img src="./pic/MemoryLayout.png" alt="/MemoryLayout.png" style="zoom: 25%;" />



[程序内存布局](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E4%B8%8E%E7%BC%96%E8%AF%91%E6%B5%81%E7%A8%8B-,%E7%A8%8B%E5%BA%8F%E5%86%85%E5%AD%98%E5%B8%83%E5%B1%80,-%23)

[局部变量与全局变量](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E6%B3%A8%E8%A7%A3-,%E5%B1%80%E9%83%A8%E5%8F%98%E9%87%8F%E4%B8%8E%E5%85%A8%E5%B1%80%E5%8F%98%E9%87%8F,-%E5%9C%A8%E4%B8%80%E4%B8%AA%E5%87%BD%E6%95%B0)

[编译流程](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E5%85%A8%E5%B1%80%E6%95%B0%E6%8D%AE%E6%AE%B5%E4%B8%AD%E3%80%82-,%E7%BC%96%E8%AF%91%E6%B5%81%E7%A8%8B,-%23)

<img src="pic/link-sections.png" alt="link-sections.png" style="zoom: 67%;" />

链接器所做的事情

- 将来自不同目标文件的段在目标内存布局中[重新排布](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E7%AC%AC%E4%B8%80%E4%BB%B6%E4%BA%8B%E6%83%85%E6%98%AF%E5%B0%86%E6%9D%A5%E8%87%AA%E4%B8%8D%E5%90%8C%E7%9B%AE%E6%A0%87%E6%96%87%E4%BB%B6%E7%9A%84%E6%AE%B5%E5%9C%A8%E7%9B%AE%E6%A0%87%E5%86%85%E5%AD%98%E5%B8%83%E5%B1%80%E4%B8%AD%E9%87%8D%E6%96%B0%E6%8E%92%E5%B8%83)
- 将符号[替换](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E9%87%8D%E6%96%B0%E6%8E%92%E5%B8%83-,%E7%AC%AC%E4%BA%8C%E4%BB%B6%E4%BA%8B%E6%83%85%E6%98%AF%E5%B0%86%E7%AC%A6%E5%8F%B7%E6%9B%BF%E6%8D%A2%E4%B8%BA%E5%85%B7%E4%BD%93%E5%9C%B0%E5%9D%80%E3%80%82,-%E8%BF%99%E9%87%8C%E7%9A%84%E7%AC%A6%E5%8F%B7)为具体地址

[如何得到一个能够在 Qemu 上成功运行的内核镜像呢？](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/3first-instruction-in-kernel1.html#:~:text=%E9%82%A3%E4%B9%88%E5%A6%82%E4%BD%95%E5%BE%97%E5%88%B0%E4%B8%80%E4%B8%AA%E8%83%BD%E5%A4%9F%E5%9C%A8%20Qemu%20%E4%B8%8A%E6%88%90%E5%8A%9F%E8%BF%90%E8%A1%8C%E7%9A%84%E5%86%85%E6%A0%B8%E9%95%9C%E5%83%8F%E5%91%A2)



#### 编写内核第一条指令

- 在 `main.rs` 中嵌入这段汇编代码 `global_asm!`



#### 调整内核的内存布局

> 由于链接器默认的内存布局并不能符合我们的要求，实现与 Qemu 的正确对接，我们可以通过 **链接脚本** (Linker Script) 调整链接器的行为，使得最终生成的可执行文件的内存布局符合我们的预期。
>
> os/.cargo/config  os/src/linker.ld

[0x80200000 可否改为其他地址？](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/4first-instruction-in-kernel2.html#:~:text=0x80200000%20%E5%8F%AF%E5%90%A6%E6%94%B9%E4%B8%BA%E5%85%B6%E4%BB%96%E5%9C%B0%E5%9D%80%EF%BC%9F)

> 内核并不是位置无关的，所以我们必须将内存布局的起始地址设置为 `0x80200000` ，与之匹配我们也必须将内核加载到这一地址

[静态链接与动态链接](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/4first-instruction-in-kernel2.html#:~:text=%E6%B3%A8%E8%A7%A3-,%E9%9D%99%E6%80%81%E9%93%BE%E6%8E%A5%E4%B8%8E%E5%8A%A8%E6%80%81%E9%93%BE%E6%8E%A5,-%E9%9D%99%E6%80%81%E9%93%BE%E6%8E%A5)

> Qemu 不支持在加载时动态链接，因此内核采用静态链接进行编译



#### 手动加载内核可执行文件

[丢弃元数据](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/4first-instruction-in-kernel2.html#:~:text=%E4%B8%8A%E9%9D%A2%E5%BE%97%E5%88%B0%E7%9A%84%E5%86%85%E6%A0%B8%E5%8F%AF%E6%89%A7%E8%A1%8C%E6%96%87%E4%BB%B6%E5%AE%8C%E5%85%A8%E7%AC%A6%E5%90%88%E6%88%91%E4%BB%AC%E5%AF%B9%E4%BA%8E%E5%86%85%E5%AD%98%E5%B8%83%E5%B1%80%E7%9A%84%E8%A6%81%E6%B1%82%EF%BC%8C%E4%BD%86%E6%98%AF%E6%88%91%E4%BB%AC%E4%B8%8D%E8%83%BD%E5%B0%86%E5%85%B6%E7%9B%B4%E6%8E%A5%E6%8F%90%E4%BA%A4%E7%BB%99%20Qemu%20%EF%BC%8C%E5%9B%A0%E4%B8%BA%E5%AE%83%E9%99%A4%E4%BA%86%E5%AE%9E%E9%99%85%E4%BC%9A%E8%A2%AB%E7%94%A8%E5%88%B0%E7%9A%84%E4%BB%A3%E7%A0%81%E5%92%8C%E6%95%B0%E6%8D%AE%E6%AE%B5%E4%B9%8B%E5%A4%96%E8%BF%98%E6%9C%89%E4%B8%80%E4%BA%9B%E5%A4%9A%E4%BD%99%E7%9A%84%E5%85%83%E6%95%B0%E6%8D%AE%EF%BC%8C%E8%BF%99%E4%BA%9B%E5%85%83%E6%95%B0%E6%8D%AE%E6%97%A0%E6%B3%95%E8%A2%AB%20Qemu%20%E5%9C%A8%E5%8A%A0%E8%BD%BD%E6%96%87%E4%BB%B6%E6%97%B6%E5%88%A9%E7%94%A8%EF%BC%8C%E4%B8%94%E4%BC%9A%E4%BD%BF%E4%BB%A3%E7%A0%81%E5%92%8C%E6%95%B0%E6%8D%AE%E6%AE%B5%E8%A2%AB%E5%8A%A0%E8%BD%BD%E5%88%B0%E9%94%99%E8%AF%AF%E7%9A%84%E4%BD%8D%E7%BD%AE%E3%80%82)

上面得到的内核可执行文件完全符合我们对于内存布局的要求，但是我们不能将其直接提交给 Qemu ，因为它除了实际会被用到的代码和数据段之外还有一些多余的元数据，这些元数据无法被 Qemu 在加载文件时利用，且会使代码和数据段被加载到错误的位置。



**小结**

> 首先我们编写内核第一条指令并嵌入到我们的内核项目中，接着指定内核的内存布局使得我们的内核可以正确对接到 Qemu 中。
>
> 由于 Qemu 的文件加载功能过于简单，它不支持完整的可执行文件，因此我们从内核可执行文件中剥离多余的元数据得到内核镜像并提供给 Qemu 。
>
> 最后，我们使用 GDB 来跟踪 Qemu 的整个启动流程并验证内核的第一条指令被正确执行。



---

### 为内核支持函数调用

- 如何使得函数返回时能够跳转到调用该函数的下一条指令，即使该函数在代码中的多个位置被调用？
- 对于一个函数而言，保证它调用某个子函数之前，以及该子函数返回到它之后（某些）通用寄存器的值保持不变有何意义？
- 调用者函数和被调用者函数如何合作保证调用子函数前后寄存器内容保持不变？调用者保存和被调用者保存寄存器的保存与恢复各自由谁负责？它们暂时被保存在什么位置？它们于何时被保存和恢复（如函数的开场白/退场白）？
- 在 RISC-V 架构上，调用者保存和被调用者保存寄存器如何划分（特别地，思考 sp 和 ra 是调用者还是被调用者保存寄存器？为什么？）？如何使用寄存器传递函数调用的参数和返回值？



#### 函数调用与栈

[控制流 (Control Flow)](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#:~:text=%E6%8E%A7%E5%88%B6%E6%B5%81%20(Control,(Function%20Call)%E3%80%82) 

- 分支结构（如 if/switch 语句）
- 循环结构（如 for/while 语句）

-  函数调用 (Function Call)
- 其他控制流都只需要跳转到一个 编译期固定下来 的地址，而函数调用的返回跳转是跳转到一个 运行时确定 （确切地说是在函数调用发生的时候）的地址。

![function-call.png](./pic/function-call.png)



[栈帧 stack frame](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#:~:text=%E6%B3%A8%E8%A7%A3-,%E6%A0%88%E5%B8%A7%20stack%20frame,-%E6%88%91%E4%BB%AC%E7%9F%A5%E9%81%93%E7%A8%8B%E5%BA%8F)

<img src="./pic/CallStack.png" alt="CallStack.png" style="zoom: 25%;" />

一般而言，当前执行函数的栈帧的两个边界分别由栈指针 (Stack Pointer)寄存器和栈帧指针（frame pointer）寄存器来限定

<img src="./pic/StackFrame.png" alt="StackFrame.png" style="zoom:25%;" />

它的开头和结尾分别在 sp(x2) 和 fp(s0) 所指向的地址。按照地址从高到低分别有以下内容，它们都是通过 `sp` 加上一个偏移量来访问的：

- `ra` 寄存器保存其返回之后的跳转地址，是一个调用者保存寄存器；
- 父亲栈帧的结束地址 `fp` ，是一个被调用者保存寄存器；
- 其他被调用者保存寄存器 `s1` ~ `s11` ；
- 函数所使用到的局部变量。

因此，栈上多个 `fp` 信息实际上保存了一条完整的函数调用链，通过适当的方式我们可以实现对函数调用关系的跟踪。

#### 分配并使用启动栈

为了将控制权转交给我们使用 Rust 语言编写的内核入口，需要手写若干行汇编代码，这些汇编代码放在 `entry.asm` 中并在控制权被转交给内核后最先被执行，但它们的功能会更加复杂：首先设置栈来在内核内使能函数调用，随后直接调用使用 Rust 编写的内核入口点，从而控制权便被移交给 Rust 代码

- 在 `entry.asm` 中[分配启动栈空间](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#:~:text=entry.asm%20%E4%B8%AD-,%E5%88%86%E9%85%8D%E5%90%AF%E5%8A%A8%E6%A0%88%E7%A9%BA%E9%97%B4,-%EF%BC%8C%E5%B9%B6%E5%9C%A8%E6%8E%A7%E5%88%B6)，并在控制权被转交给 Rust 入口之前将栈指针 `sp` 设置为栈顶的位置

- 通过伪指令 `call` 调用 Rust 编写的![image-20221009150238044](/home/clstilmldy/note/OS/rCore/pic/image-20221009150238044.png)内核入口点 `rust_main` 将[控制权转交]()给 Rust 代码，该入口点在 `main.rs` 中实现
- 完成对 `.bss` 段的[清零](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#:~:text=%E6%88%91%E4%BB%AC%E9%A1%BA%E4%BE%BF-,%E5%AE%8C%E6%88%90%E5%AF%B9%20.bss%20%E6%AE%B5%E7%9A%84%E6%B8%85%E9%9B%B6,-%E3%80%82%E8%BF%99%E6%98%AF%E5%86%85)



---

### 基于 SBI 服务完成输出和关机

> RustSBI 介于底层硬件和内核之间，是内核的底层执行环境

#### 使用 RustSBI 提供的服务

- [RustSBI 职责](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/6print-and-shutdown-based-on-sbi.html#:~:text=%E5%AF%B9%20RustSBI%20%E7%9A%84%E4%BA%86%E8%A7%A3%E4%BB%85%E9%99%90%E4%BA%8E%E5%AE%83%E4%BC%9A%E5%9C%A8%E8%AE%A1%E7%AE%97%E6%9C%BA%E5%90%AF%E5%8A%A8%E6%97%B6%E8%BF%9B%E8%A1%8C%E5%AE%83%E6%89%80%E8%B4%9F%E8%B4%A3%E7%9A%84%E7%8E%AF%E5%A2%83%E5%88%9D%E5%A7%8B%E5%8C%96%E5%B7%A5%E4%BD%9C%EF%BC%8C%E5%B9%B6%E5%B0%86%E8%AE%A1%E7%AE%97%E6%9C%BA%E6%8E%A7%E5%88%B6%E6%9D%83%E7%A7%BB%E4%BA%A4%E7%BB%99%E5%86%85%E6%A0%B8%E3%80%82%E4%BD%86%E5%AE%9E%E9%99%85%E4%B8%8A%E4%BD%9C%E4%B8%BA%E5%86%85%E6%A0%B8%E7%9A%84%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%EF%BC%8C%E5%AE%83%E8%BF%98%E6%9C%89%E5%8F%A6%E4%B8%80%E9%A1%B9%E8%81%8C%E8%B4%A3%EF%BC%9A%E5%8D%B3%E5%9C%A8%E5%86%85%E6%A0%B8%E8%BF%90%E8%A1%8C%E6%97%B6%E5%93%8D%E5%BA%94%E5%86%85%E6%A0%B8%E7%9A%84%E8%AF%B7%E6%B1%82%E4%B8%BA%E5%86%85%E6%A0%B8%E6%8F%90%E4%BE%9B%E6%9C%8D%E5%8A%A1%E3%80%82)



#### 实现格式化输出

- 编写基于 `console_putchar` 的 [`println!` 宏](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/6print-and-shutdown-based-on-sbi.html#id2:~:text=%E5%9B%A0%E6%AD%A4%E6%88%91%E4%BB%AC%E5%B0%9D%E8%AF%95%E8%87%AA%E5%B7%B1%E7%BC%96%E5%86%99%E5%9F%BA%E4%BA%8E%20console_putchar%20%E7%9A%84%20println!%20%E5%AE%8F%E3%80%82)



#### 处理致命错误

- 不可恢复错误
