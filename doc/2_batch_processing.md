# 批处理系统

### 引言

```

├── bootloader
│   └── rustsbi-qemu.bin
├── os
│   ├── build.rs(新增：生成 link_app.S 将应用作为一个数据段链接到内核)
│   ├── Cargo.toml
│   ├── Makefile(修改：构建内核之前先构建应用)
│   └── src
│       ├── batch.rs(新增：实现了一个简单的批处理系统)
│       ├── console.rs
│       ├── entry.asm
│       ├── lang_items.rs
│       ├── link_app.S(构建产物，由 os/build.rs 输出)
│       ├── linker-qemu.ld
│       ├── main.rs(修改：主函数中需要初始化 Trap 处理并加载和执行应用)
│       ├── sbi.rs
│       ├── sync(新增：同步子模块 sync ，目前唯一功能是提供 UPSafeCell)
│       │   ├── mod.rs
│       │   └── up.rs(包含 UPSafeCell，它可以帮助我们以更 Rust 的方式使用全局变量)
│       ├── syscall(新增：系统调用子模块 syscall)
│       │   ├── fs.rs(包含文件 I/O 相关的 syscall)
│       │   ├── mod.rs(提供 syscall 方法根据 syscall ID 进行分发处理)
│       │   └── process.rs(包含任务处理相关的 syscall)
│       └── trap(新增：Trap 相关子模块 trap)
│           ├── context.rs(包含 Trap 上下文 TrapContext)
│           ├── mod.rs(包含 Trap 处理入口 trap_handler)
│           └── trap.S(包含 Trap 上下文保存与恢复的汇编代码)
└── user(新增：应用测例保存在 user 目录下)
   ├── Cargo.toml
   ├── Makefile
   └── src
      ├── bin(基于用户库 user_lib 开发的应用，每个应用放在一个源文件中)
      │   ├── 00hello_world.rs
      │   ├── 01store_fault.rs
      │   ├── 02power.rs
      │   ├── 03priv_inst.rs
      │   └── 04priv_csr.rs
      ├── console.rs
      ├── lang_items.rs
      ├── lib.rs(用户库 user_lib)
      ├── linker.ld(应用的链接脚本)
      └── syscall.rs(包含 syscall 方法生成实际用于系统调用的汇编指令，
                     各个具体的 syscall 都是通过 syscall 来实现的)
```



[添加新功能](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/0intro.html#id2:~:text=%E6%84%9F%E7%9F%A5%E5%A4%9A%E4%B8%AA%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E7%9A%84%E5%AD%98%E5%9C%A8%EF%BC%8C%E5%B9%B6%E4%B8%80%E4%B8%AA%E6%8E%A5%E4%B8%80%E4%B8%AA%E5%9C%B0%E8%BF%90%E8%A1%8C%E8%BF%99%E4%BA%9B%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%EF%BC%8C%E5%BD%93%E4%B8%80%E4%B8%AA%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E6%89%A7%E8%A1%8C%E5%AE%8C%E6%AF%95%E5%90%8E%EF%BC%8C%E4%BC%9A%E5%90%AF%E5%8A%A8%E4%B8%8B%E4%B8%80%E4%B8%AA%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%EF%BC%8C%E7%9B%B4%E5%88%B0%E6%89%80%E6%9C%89%E7%9A%84%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E9%83%BD%E6%89%A7%E8%A1%8C%E5%AE%8C%E6%AF%95%E3%80%82)

- 构造包含操作系统内核和多个应用程序的单一执行程序
- 通过批处理支持多个程序的自动加载和运行
- 操作系统利用硬件特权级机制，实现对操作系统自身的保护
- 实现特权级的穿越
- 支持跨特权级的系统调用功能

[特权级](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/0intro.html#id2:~:text=%E4%BA%BA%E4%BB%AC%E5%B8%8C%E6%9C%9B%E4%B8%80%E4%B8%AA%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E7%9A%84%E9%94%99%E8%AF%AF%E4%B8%8D%E8%A6%81%E5%BD%B1%E5%93%8D%E5%88%B0%E5%85%B6%E5%AE%83%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E3%80%81%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E5%92%8C%E6%95%B4%E4%B8%AA%E8%AE%A1%E7%AE%97%E6%9C%BA%E7%B3%BB%E7%BB%9F%E3%80%82%E8%BF%99%E5%B0%B1%E9%9C%80%E8%A6%81%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E8%83%BD%E5%A4%9F%E7%BB%88%E6%AD%A2%E5%87%BA%E9%94%99%E7%9A%84%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%EF%BC%8C%E8%BD%AC%E8%80%8C%E8%BF%90%E8%A1%8C%E4%B8%8B%E4%B8%80%E4%B8%AA%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E3%80%82%E8%BF%99%E7%A7%8D%20%E4%BF%9D%E6%8A%A4%20%E8%AE%A1%E7%AE%97%E6%9C%BA%E7%B3%BB%E7%BB%9F%E4%B8%8D%E5%8F%97%E6%9C%89%E6%84%8F%E6%88%96%E6%97%A0%E6%84%8F%E5%87%BA%E9%94%99%E7%9A%84%E7%A8%8B%E5%BA%8F%E7%A0%B4%E5%9D%8F%E7%9A%84%E6%9C%BA%E5%88%B6%E8%A2%AB%E7%A7%B0%E4%B8%BA%20%E7%89%B9%E6%9D%83%E7%BA%A7%20(Privilege)%20%E6%9C%BA%E5%88%B6%EF%BC%8C%E5%AE%83%E8%AE%A9%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E8%BF%90%E8%A1%8C%E5%9C%A8%E7%94%A8%E6%88%B7%E6%80%81%EF%BC%8C%E8%80%8C%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E8%BF%90%E8%A1%8C%E5%9C%A8%E5%86%85%E6%A0%B8%E6%80%81%EF%BC%8C%E4%B8%94%E5%AE%9E%E7%8E%B0%E7%94%A8%E6%88%B7%E6%80%81%E5%92%8C%E5%86%85%E6%A0%B8%E6%80%81%E7%9A%84%E9%9A%94%E7%A6%BB%EF%BC%8C%E8%BF%99%E9%9C%80%E8%A6%81%E8%AE%A1%E7%AE%97%E6%9C%BA%E8%BD%AF%E4%BB%B6%E5%92%8C%E7%A1%AC%E4%BB%B6%E7%9A%84%E5%85%B1%E5%90%8C%E5%8A%AA%E5%8A%9B%E3%80%82)

保护计算机系统不受有意或无意出错的程序破坏的机制

**应用程序**

- 改进应用程序，让它能够在用户态执行，并能发出系统调用。

- 编写多个应用小程序，修改编译应用所需的 `linker.ld` 文件来 [调整程序的内存布局](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/2application.html#term-app-mem-layout) ，让操作系统能够把应用加载到指定内存地址，然后顺利启动并运行应用程序

**系统调用**

- 在应用程序的运行过程中，操作系统要支持应用程序的输出功能，并还能支持应用程序退出。这需要实现跨特权级的系统调用接口，以及 `sys_write` 和 `sys_exit` 等具体的系统调用功能。
- 在具体设计实现上，涉及到内联汇编的编写，以及应用与操作系统内核之间系统调用的参数传递的约定。

**批处理**

- 实现支持多个应用程序轮流启动运行的操作系统。
- 首先能把本来相对松散的应用程序执行代码和操作系统执行代码连接在一起，便于 `qemu-system-riscv64` 模拟器一次性地加载二者到内存中，并让操作系统能够找到应用程序的位置。
- 为把二者连在一起，需要对生成的应用程序进行改造
  - 首先是把应用程序执行文件从ELF执行文件格式变成Binary格式（通过 `rust-objcopy` 可以轻松完成）；
  - 然后这些Binary格式的文件通过编译器辅助脚本 `os/build.rs` 转变变成 `os/src/link_app.S` 这个汇编文件的一部分，并生成各个Binary应用的辅助信息，便于操作系统能够找到应用的位置。
- 编译器会把操作系统的源码和 `os/src/link_app.S` 合在一起，编译出操作系统+Binary应用的ELF执行文件，并进一步转变成Binary格式。

- 为了定位 Binary 应用在被加载后的内存位置，操作系统本身需要完成对 Binary 应用的位置查找，找到后（通过 `os/src/link_app.S` 中的变量和标号信息完成），会把 Binary 应用从加载位置拷贝到 `user/src/linker.ld` 指定的物理内存位置（OS的加载应用功能）。
- 在一个应用执行完毕后，操作系统还能加载另外一个应用，这主要是通过 `AppManagerInner` 数据结构和对应的函数 `load_app` 和 `run_next_app` 等来完成对应用的一系列管理功能。

- 为了让 Binary 应用能够启动和运行，操作系统还需给 Binary 应用分配好对应执行环境所需一系列的资源。
  - 这主要包括设置好用户栈和内核栈（在用户态的应用程序与在内核态的操作系统内核需要有各自的栈，避免应用程序破坏内核的执行），实现 Trap 上下文的保存与恢复（让应用能够在发出系统调用到内核态后，还能回到用户态继续执行），完成Trap 分发与处理等工作。
- 最后，实现 **执行应用程序** 的操作系统功能，其主要实现在 `run_next_app` 内核函数中 。



---

### 特权级机制

[特权级隔离机制](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E4%B8%BA%E4%BA%86%E4%BF%9D%E6%8A%A4%E6%88%91%E4%BB%AC%E7%9A%84%E6%89%B9%E5%A4%84%E7%90%86%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E4%B8%8D%E5%8F%97%E5%88%B0%E5%87%BA%E9%94%99%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E7%9A%84%E5%BD%B1%E5%93%8D%E5%B9%B6%E5%85%A8%E7%A8%8B%E7%A8%B3%E5%AE%9A%E5%B7%A5%E4%BD%9C%EF%BC%8C%E5%8D%95%E5%87%AD%E8%BD%AF%E4%BB%B6%E5%AE%9E%E7%8E%B0%E6%98%AF%E5%BE%88%E9%9A%BE%E5%81%9A%E5%88%B0%E7%9A%84%EF%BC%8C%E8%80%8C%E6%98%AF%E9%9C%80%E8%A6%81%20CPU%20%E6%8F%90%E4%BE%9B%E4%B8%80%E7%A7%8D%E7%89%B9%E6%9D%83%E7%BA%A7%E9%9A%94%E7%A6%BB%E6%9C%BA%E5%88%B6%EF%BC%8C%E4%BD%BF%20CPU%20%E5%9C%A8%E6%89%A7%E8%A1%8C%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E5%92%8C%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E5%86%85%E6%A0%B8%E7%9A%84%E6%8C%87%E4%BB%A4%E6%97%B6%E5%A4%84%E4%BA%8E%E4%B8%8D%E5%90%8C%E7%9A%84%E7%89%B9%E6%9D%83%E7%BA%A7%E3%80%82)

#### 特权级的软硬件协同设计

实现特权级机制的根本[原因](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E4%BB%A5%E5%BA%93%E7%9A%84%E5%BD%A2%E5%BC%8F%E5%92%8C%E5%BA%94%E7%94%A8%E7%B4%A7%E5%AF%86%E8%BF%9E%E6%8E%A5%E5%9C%A8%E4%B8%80%E8%B5%B7%EF%BC%8C%E6%9E%84%E6%88%90%E4%B8%80%E4%B8%AA%E6%95%B4%E4%BD%93%E6%9D%A5%E6%89%A7%E8%A1%8C%E3%80%82%E9%9A%8F%E7%9D%80%E5%BA%94%E7%94%A8%E9%9C%80%E6%B1%82%E7%9A%84%E5%A2%9E%E5%8A%A0%EF%BC%8C%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E7%9A%84%E4%BD%93%E7%A7%AF%E4%B9%9F%E8%B6%8A%E6%9D%A5%E8%B6%8A%E5%A4%A7%EF%BC%9B%E5%90%8C%E6%97%B6%E5%BA%94%E7%94%A8%E8%87%AA%E8%BA%AB%E4%B9%9F%E4%BC%9A%E8%B6%8A%E6%9D%A5%E8%B6%8A%E5%A4%8D%E6%9D%82%E3%80%82%E7%94%B1%E4%BA%8E%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E4%BC%9A%E8%A2%AB%E9%A2%91%E7%B9%81%E8%AE%BF%E9%97%AE%EF%BC%8C%E6%9D%A5%E7%BB%99%E5%A4%9A%E4%B8%AA%E5%BA%94%E7%94%A8%E6%8F%90%E4%BE%9B%E6%9C%8D%E5%8A%A1%EF%BC%8C%E6%89%80%E4%BB%A5%E5%AE%83%E5%8F%AF%E8%83%BD%E7%9A%84%E9%94%99%E8%AF%AF%E4%BC%9A%E6%AF%94%E8%BE%83%E5%BF%AB%E5%9C%B0%E8%A2%AB%E5%8F%91%E7%8E%B0%E3%80%82%E4%BD%86%E5%BA%94%E7%94%A8%E8%87%AA%E8%BA%AB%E7%9A%84%E9%94%99%E8%AF%AF%E5%8F%AF%E8%83%BD%E5%B0%B1%E4%B8%8D%E4%BC%9A%E5%BE%88%E5%BF%AB%E5%8F%91%E7%8E%B0%E3%80%82%E7%94%B1%E4%BA%8E%E4%BA%8C%E8%80%85%E9%80%9A%E8%BF%87%E7%BC%96%E8%AF%91%E5%99%A8%E5%BD%A2%E6%88%90%E4%B8%80%E4%B8%AA%E5%8D%95%E4%B8%80%E6%89%A7%E8%A1%8C%E7%A8%8B%E5%BA%8F%E6%9D%A5%E6%89%A7%E8%A1%8C%EF%BC%8C%E5%AF%BC%E8%87%B4%E5%8D%B3%E4%BD%BF%E6%98%AF%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E6%9C%AC%E8%BA%AB%E7%9A%84%E9%97%AE%E9%A2%98%EF%BC%8C%E4%B9%9F%E4%BC%9A%E8%AE%A9%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E5%8F%97%E5%88%B0%E8%BF%9E%E7%B4%AF%EF%BC%8C%E4%BB%8E%E8%80%8C%E5%8F%AF%E8%83%BD%E5%AF%BC%E8%87%B4%E6%95%B4%E4%B8%AA%E8%AE%A1%E7%AE%97%E6%9C%BA%E7%B3%BB%E7%BB%9F%E9%83%BD%E4%B8%8D%E5%8F%AF%E7%94%A8%E4%BA%86%E3%80%82)是应用程序运行的安全性不可充分信任

[方法](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E6%96%B9%E6%B3%95%EF%BC%8C%E8%AE%A9%E7%9B%B8%E5%AF%B9%E5%AE%89%E5%85%A8%E5%8F%AF%E9%9D%A0%E7%9A%84%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E8%BF%90%E8%A1%8C%E5%9C%A8%E4%B8%80%E4%B8%AA%E7%A1%AC%E4%BB%B6%E4%BF%9D%E6%8A%A4%E7%9A%84%E5%AE%89%E5%85%A8%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E4%B8%AD%EF%BC%8C%E4%B8%8D%E5%8F%97%E5%88%B0%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E7%9A%84%E7%A0%B4%E5%9D%8F%EF%BC%9B%E8%80%8C%E8%AE%A9%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E8%BF%90%E8%A1%8C%E5%9C%A8%E5%8F%A6%E5%A4%96%E4%B8%80%E4%B8%AA%E6%97%A0%E6%B3%95%E7%A0%B4%E5%9D%8F%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E7%9A%84%E5%8F%97%E9%99%90%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E4%B8%AD%E3%80%82) 设置两个不同安全等级的执行环境

[限制](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E9%99%90%E5%88%B6%E7%9A%84%E4%B8%BB%E8%A6%81,%E6%9C%AC%E7%AB%A0%E7%9A%84%E9%87%8D%E7%82%B9%EF%BC%89) 应用程序 两个方面：指令级别 与 地址空间

[操作系统与应用程序交互](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E5%81%87%E8%AE%BE%E6%9C%89%E4%BA%86%E8%BF%99%E6%A0%B7%E7%9A%84%E9%99%90%E5%88%B6%EF%BC%8C%E6%88%91%E4%BB%AC%E8%BF%98%E9%9C%80%E8%A6%81%E7%A1%AE%E4%BF%9D%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E8%83%BD%E5%A4%9F%E5%BE%97%E5%88%B0%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E7%9A%84%E6%9C%8D%E5%8A%A1%EF%BC%8C%E5%8D%B3%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E5%92%8C%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E8%BF%98%E9%9C%80%E8%A6%81%E6%9C%89%E4%BA%A4%E4%BA%92%E7%9A%84%E6%89%8B%E6%AE%B5%E3%80%82%E4%BD%BF%E5%BE%97%E4%BD%8E%E7%89%B9%E6%9D%83%E7%BA%A7%E8%BD%AF%E4%BB%B6%E5%8F%AA%E8%83%BD%E5%81%9A%E9%AB%98%E7%89%B9%E6%9D%83%E7%BA%A7%E8%BD%AF%E4%BB%B6%E5%85%81%E8%AE%B8%E5%AE%83%E5%81%9A%E7%9A%84%EF%BC%8C%E4%B8%94%E8%B6%85%E5%87%BA%E4%BD%8E%E7%89%B9%E6%9D%83%E7%BA%A7%E8%BD%AF%E4%BB%B6%E8%83%BD%E5%8A%9B%E7%9A%84%E5%8A%9F%E8%83%BD%E5%BF%85%E9%A1%BB%E5%AF%BB%E6%B1%82%E9%AB%98%E7%89%B9%E6%9D%83%E7%BA%A7%E8%BD%AF%E4%BB%B6%E7%9A%84%E5%B8%AE%E5%8A%A9%E3%80%82%E8%BF%99%E6%A0%B7%EF%BC%8C%E9%AB%98%E7%89%B9%E6%9D%83%E7%BA%A7%E8%BD%AF%E4%BB%B6%EF%BC%88%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%EF%BC%89%E5%B0%B1%E6%88%90%E4%B8%BA%E4%BD%8E%E7%89%B9%E6%9D%83%E7%BA%A7%E8%BD%AF%E4%BB%B6%EF%BC%88%E4%B8%80%E8%88%AC%E5%BA%94%E7%94%A8%EF%BC%89%E7%9A%84%E8%BD%AF%E4%BB%B6%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E7%9A%84%E9%87%8D%E8%A6%81%E7%BB%84%E6%88%90%E9%83%A8%E5%88%86%E3%80%82) —— 高特权级软件（操作系统）就成为低特权级软件（一般应用）的软件执行环境的重要组成部分

[软硬件协同设计方法](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E4%B8%BA%E4%BA%86%E5%AE%9E%E7%8E%B0%E8%BF%99%E6%A0%B7%E7%9A%84%E7%89%B9%E6%9D%83%E7%BA%A7%E6%9C%BA%E5%88%B6%EF%BC%8C%E9%9C%80%E8%A6%81%E8%BF%9B%E8%A1%8C%E8%BD%AF%E7%A1%AC%E4%BB%B6%E5%8D%8F%E5%90%8C%E8%AE%BE%E8%AE%A1%E3%80%82%E4%B8%80%E4%B8%AA%E6%AF%94%E8%BE%83%E7%AE%80%E6%B4%81%E7%9A%84%E6%96%B9%E6%B3%95%E5%B0%B1%E6%98%AF%EF%BC%8C%E5%A4%84%E7%90%86%E5%99%A8%E8%AE%BE%E7%BD%AE%E4%B8%A4%E4%B8%AA%E4%B8%8D%E5%90%8C%E5%AE%89%E5%85%A8%E7%AD%89%E7%BA%A7%E7%9A%84%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%EF%BC%9A%E7%94%A8%E6%88%B7%E6%80%81%E7%89%B9%E6%9D%83%E7%BA%A7%E7%9A%84%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E5%92%8C%E5%86%85%E6%A0%B8%E6%80%81%E7%89%B9%E6%9D%83%E7%BA%A7%E7%9A%84%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E3%80%82%E4%B8%94%E6%98%8E%E7%A1%AE%E6%8C%87%E5%87%BA%E5%8F%AF%E8%83%BD%E7%A0%B4%E5%9D%8F%E8%AE%A1%E7%AE%97%E6%9C%BA%E7%B3%BB%E7%BB%9F%E7%9A%84%E5%86%85%E6%A0%B8%E6%80%81%E7%89%B9%E6%9D%83%E7%BA%A7%E6%8C%87%E4%BB%A4%E5%AD%90%E9%9B%86%EF%BC%8C%E8%A7%84%E5%AE%9A%E5%86%85%E6%A0%B8%E6%80%81%E7%89%B9%E6%9D%83%E7%BA%A7%E6%8C%87%E4%BB%A4%E5%AD%90%E9%9B%86%E4%B8%AD%E7%9A%84%E6%8C%87%E4%BB%A4%E5%8F%AA%E8%83%BD%E5%9C%A8%E5%86%85%E6%A0%B8%E6%80%81%E7%89%B9%E6%9D%83%E7%BA%A7%E7%9A%84%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E4%B8%AD%E6%89%A7%E8%A1%8C%E3%80%82%E5%A4%84%E7%90%86%E5%99%A8%E5%9C%A8%E6%89%A7%E8%A1%8C%E6%8C%87%E4%BB%A4%E5%89%8D%E4%BC%9A%E8%BF%9B%E8%A1%8C%E7%89%B9%E6%9D%83%E7%BA%A7%E5%AE%89%E5%85%A8%E6%A3%80%E6%9F%A5%EF%BC%8C%E5%A6%82%E6%9E%9C%E5%9C%A8%E7%94%A8%E6%88%B7%E6%80%81%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E4%B8%AD%E6%89%A7%E8%A1%8C%E8%BF%99%E4%BA%9B%E5%86%85%E6%A0%B8%E6%80%81%E7%89%B9%E6%9D%83%E7%BA%A7%E6%8C%87%E4%BB%A4%EF%BC%8C%E4%BC%9A%E4%BA%A7%E7%94%9F%E5%BC%82%E5%B8%B8%E3%80%82) —— 处理器设置两个不同安全等级的执行环境

为了让应用程序获得操作系统的函数服务，采用传统的函数调用方式（即通常的 `call` 和 `ret` 指令或指令组合）将会直接绕过硬件的特权级保护检查。所以可以设计新的机器指令：执行环境调用（Execution Environment Call，简称 `ecall` ）和执行环境返回(Execution Environment Return，简称 `eret` )）：

- `ecall` ：具有用户态到内核态的执行环境切换能力的函数调用指令
- `eret` ：具有内核态到用户态的执行环境切换能力的函数返回指令

> 一般来说， `ecall` 和 `eret` 两条指令分别可以用来让 CPU 从当前特权级切换到比当前高一级的特权级和切换到不高于当前的特权级，因此上面提到的两条指令的功能仅是其中一种用法。

#### RISC-V 特权级架构

[4 种特权级](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=RISC%2DV%20%E6%9E%B6%E6%9E%84%E4%B8%AD%E4%B8%80%E5%85%B1%E5%AE%9A%E4%B9%89%E4%BA%86%204%20%E7%A7%8D%E7%89%B9%E6%9D%83%E7%BA%A7%EF%BC%9A)

[执行环境栈](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E5%BC%A0%E5%9B%BE%E7%89%87%E7%BB%99%E5%87%BA%E4%BA%86%E8%83%BD%E5%A4%9F%E6%94%AF%E6%8C%81%E8%BF%90%E8%A1%8C%20Unix%20%E8%BF%99%E7%B1%BB%E5%A4%8D%E6%9D%82%E7%B3%BB%E7%BB%9F%E7%9A%84%E8%BD%AF%E4%BB%B6%E6%A0%88%E3%80%82%E5%85%B6%E4%B8%AD%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E5%86%85%E6%A0%B8%E4%BB%A3%E7%A0%81%E8%BF%90%E8%A1%8C%E5%9C%A8%20S%20%E6%A8%A1%E5%BC%8F%E4%B8%8A%EF%BC%9B%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E8%BF%90%E8%A1%8C%E5%9C%A8%20U%20%E6%A8%A1%E5%BC%8F%E4%B8%8A%E3%80%82%E8%BF%90%E8%A1%8C%E5%9C%A8%20M%20%E6%A8%A1%E5%BC%8F%E4%B8%8A%E7%9A%84%E8%BD%AF%E4%BB%B6%E8%A2%AB%E7%A7%B0%E4%B8%BA%20%E7%9B%91%E7%9D%A3%E6%A8%A1%E5%BC%8F%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%20(SEE%2C%20Supervisor%20Execution%20Environment)%EF%BC%8C%E5%A6%82%E5%9C%A8%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E8%BF%90%E8%A1%8C%E5%89%8D%E8%B4%9F%E8%B4%A3%E5%8A%A0%E8%BD%BD%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E7%9A%84%20Bootloader%20%E2%80%93%20RustSBI%E3%80%82%E7%AB%99%E5%9C%A8%E8%BF%90%E8%A1%8C%E5%9C%A8%20S%20%E6%A8%A1%E5%BC%8F%E4%B8%8A%E7%9A%84%E8%BD%AF%E4%BB%B6%E8%A7%86%E8%A7%92%E6%9D%A5%E7%9C%8B%EF%BC%8C%E5%AE%83%E7%9A%84%E4%B8%8B%E9%9D%A2%E4%B9%9F%E9%9C%80%E8%A6%81%E4%B8%80%E5%B1%82%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E6%94%AF%E6%92%91%EF%BC%8C%E5%9B%A0%E6%AD%A4%E8%A2%AB%E5%91%BD%E5%90%8D%E4%B8%BA%20SEE%EF%BC%8C%E5%AE%83%E9%9C%80%E8%A6%81%E5%9C%A8%E7%9B%B8%E6%AF%94%20S%20%E6%A8%A1%E5%BC%8F%E6%9B%B4%E9%AB%98%E7%9A%84%E7%89%B9%E6%9D%83%E7%BA%A7%E4%B8%8B%E8%BF%90%E8%A1%8C%EF%BC%8C%E4%B8%80%E8%88%AC%E6%83%85%E5%86%B5%E4%B8%8B%20SEE%20%E5%9C%A8%20M%20%E6%A8%A1%E5%BC%8F%E4%B8%8A%E8%BF%90%E8%A1%8C%E3%80%82) 

<img src="pic/PrivilegeStack.png" alt="PrivilegeStack.png" style="zoom: 25%;" />

- 白色块表示一层执行环境

- 黑色块表示相邻两层执行环境之间的接口

  - M 模式软件 SEE 和 S 模式的内核之间的接口被称为 **监督模式二进制接口** (Supervisor Binary Interface, SBI)

  - 内核和 U 模式的应用程序之间的接口被称为 **应用程序二进制接口** (Application Binary Interface, ABI) 或者 **系统调用** (syscall, System Call) 

[监督模式执行环境](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E8%BF%90%E8%A1%8C%E5%9C%A8%20M%20%E6%A8%A1%E5%BC%8F%E4%B8%8A%E7%9A%84%E8%BD%AF%E4%BB%B6%E8%A2%AB%E7%A7%B0%E4%B8%BA%20%E7%9B%91%E7%9D%A3%E6%A8%A1%E5%BC%8F%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%20(SEE%2C%20Supervisor%20Execution%20Environment)%EF%BC%8C%E5%A6%82%E5%9C%A8%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E8%BF%90%E8%A1%8C%E5%89%8D%E8%B4%9F%E8%B4%A3%E5%8A%A0%E8%BD%BD%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E7%9A%84%20Bootloader%20%E2%80%93%20RustSBI%E3%80%82%E7%AB%99%E5%9C%A8%E8%BF%90%E8%A1%8C%E5%9C%A8%20S%20%E6%A8%A1%E5%BC%8F%E4%B8%8A%E7%9A%84%E8%BD%AF%E4%BB%B6%E8%A7%86%E8%A7%92%E6%9D%A5%E7%9C%8B%EF%BC%8C%E5%AE%83%E7%9A%84%E4%B8%8B%E9%9D%A2%E4%B9%9F%E9%9C%80%E8%A6%81%E4%B8%80%E5%B1%82%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E6%94%AF%E6%92%91%EF%BC%8C%E5%9B%A0%E6%AD%A4%E8%A2%AB%E5%91%BD%E5%90%8D%E4%B8%BA%20SEE%EF%BC%8C%E5%AE%83%E9%9C%80%E8%A6%81%E5%9C%A8%E7%9B%B8%E6%AF%94%20S%20%E6%A8%A1%E5%BC%8F%E6%9B%B4%E9%AB%98%E7%9A%84%E7%89%B9%E6%9D%83%E7%BA%A7%E4%B8%8B%E8%BF%90%E8%A1%8C%EF%BC%8C%E4%B8%80%E8%88%AC%E6%83%85%E5%86%B5%E4%B8%8B%20SEE%20%E5%9C%A8%20M%20%E6%A8%A1%E5%BC%8F%E4%B8%8A%E8%BF%90%E8%A1%8C%E3%80%82) —— RustSBI

- 功能如引导加载程序会在加电后对整个系统进行初始化

执行环境的功能

- 执行它支持的上层软件之前进行一些初始化工作

- [另一种功能](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E7%9A%84%E5%8F%A6%E4%B8%80%E7%A7%8D%E5%8A%9F%E8%83%BD%E6%98%AF%E5%AF%B9%E4%B8%8A%E5%B1%82%E8%BD%AF%E4%BB%B6%E7%9A%84%E6%89%A7%E8%A1%8C%E8%BF%9B%E8%A1%8C%E7%9B%91%E6%8E%A7%E7%AE%A1%E7%90%86%E3%80%82%E7%9B%91%E6%8E%A7%E7%AE%A1%E7%90%86%E5%8F%AF%E4%BB%A5%E7%90%86%E8%A7%A3%E4%B8%BA%EF%BC%8C%E5%BD%93%E4%B8%8A%E5%B1%82%E8%BD%AF%E4%BB%B6%E6%89%A7%E8%A1%8C%E7%9A%84%E6%97%B6%E5%80%99%E5%87%BA%E7%8E%B0%E4%BA%86%E4%B8%80%E4%BA%9B%E5%BC%82%E5%B8%B8%E6%88%96%E7%89%B9%E6%AE%8A%E6%83%85%E5%86%B5%EF%BC%8C%E5%AF%BC%E8%87%B4%E9%9C%80%E8%A6%81%E7%94%A8%E5%88%B0%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E4%B8%AD%E6%8F%90%E4%BE%9B%E7%9A%84%E5%8A%9F%E8%83%BD%EF%BC%8C%E5%9B%A0%E6%AD%A4%E9%9C%80%E8%A6%81%E6%9A%82%E5%81%9C%E4%B8%8A%E5%B1%82%E8%BD%AF%E4%BB%B6%E7%9A%84%E6%89%A7%E8%A1%8C%EF%BC%8C%E8%BD%AC%E8%80%8C%E8%BF%90%E8%A1%8C%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E7%9A%84%E4%BB%A3%E7%A0%81) —— 对上层软件的执行进行监控管理

[risc-v: 异常](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E7%94%B1%E4%BA%8E%E4%B8%8A%E5%B1%82%E8%BD%AF%E4%BB%B6,%E5%90%84%E7%A7%8D%20%E5%BC%82%E5%B8%B8%EF%BC%9A)

[用户态应用直接触发从用户态到内核态的异常的原因](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E7%94%A8%E6%88%B7%E6%80%81%E5%BA%94%E7%94%A8%E7%9B%B4%E6%8E%A5%E8%A7%A6%E5%8F%91%E4%BB%8E%E7%94%A8%E6%88%B7%E6%80%81%E5%88%B0%E5%86%85%E6%A0%B8%E6%80%81%E7%9A%84%E5%BC%82%E5%B8%B8%E7%9A%84%E5%8E%9F%E5%9B%A0%E6%80%BB%E4%BD%93%E4%B8%8A%E5%8F%AF%E4%BB%A5%E5%88%86%E4%B8%BA%E4%B8%A4%E7%A7%8D%EF%BC%9A%E5%85%B6%E4%B8%80%E6%98%AF%E7%94%A8%E6%88%B7%E6%80%81%E8%BD%AF%E4%BB%B6%E4%B8%BA%E8%8E%B7%E5%BE%97%E5%86%85%E6%A0%B8%E6%80%81%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E7%9A%84%E6%9C%8D%E5%8A%A1%E5%8A%9F%E8%83%BD%E8%80%8C%E6%89%A7%E8%A1%8C%E7%89%B9%E6%AE%8A%E6%8C%87%E4%BB%A4%EF%BC%9B%E5%85%B6%E4%BA%8C%E6%98%AF%E5%9C%A8%E6%89%A7%E8%A1%8C%E6%9F%90%E6%9D%A1%E6%8C%87%E4%BB%A4%E6%9C%9F%E9%97%B4%E4%BA%A7%E7%94%9F%E4%BA%86%E9%94%99%E8%AF%AF%EF%BC%88%E5%A6%82%E6%89%A7%E8%A1%8C%E4%BA%86%E7%94%A8%E6%88%B7%E6%80%81%E4%B8%8D%E5%85%81%E8%AE%B8%E6%89%A7%E8%A1%8C%E7%9A%84%E6%8C%87%E4%BB%A4%E6%88%96%E8%80%85%E5%85%B6%E4%BB%96%E9%94%99%E8%AF%AF%EF%BC%89%E5%B9%B6%E8%A2%AB%20CPU%20%E6%A3%80%E6%B5%8B%E5%88%B0%E3%80%82%E4%B8%8B%E8%A1%A8%E4%B8%AD%E6%88%91%E4%BB%AC%E7%BB%99%E5%87%BA%E4%BA%86%20RISC%2DV%20%E7%89%B9%E6%9D%83%E7%BA%A7%E8%A7%84%E8%8C%83%E5%AE%9A%E4%B9%89%E7%9A%84%E4%BC%9A%E5%8F%AF%E8%83%BD%E5%AF%BC%E8%87%B4%E4%BB%8E%E4%BD%8E%E7%89%B9%E6%9D%83%E7%BA%A7%E5%88%B0%E9%AB%98%E7%89%B9%E6%9D%83%E7%BA%A7%E7%9A%84%E5%90%84%E7%A7%8D%20%E5%BC%82%E5%B8%B8%EF%BC%9A)

[RISC-V 特权级规范定义的会可能导致从低特权级到高特权级的各种异常](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E4%B8%8B%E8%A1%A8%E4%B8%AD%E6%88%91%E4%BB%AC%E7%BB%99%E5%87%BA%E4%BA%86%20RISC%2DV%20%E7%89%B9%E6%9D%83%E7%BA%A7%E8%A7%84%E8%8C%83%E5%AE%9A%E4%B9%89%E7%9A%84%E4%BC%9A%E5%8F%AF%E8%83%BD%E5%AF%BC%E8%87%B4%E4%BB%8E%E4%BD%8E%E7%89%B9%E6%9D%83%E7%BA%A7%E5%88%B0%E9%AB%98%E7%89%B9%E6%9D%83%E7%BA%A7%E7%9A%84%E5%90%84%E7%A7%8D%20%E5%BC%82%E5%B8%B8%EF%BC%9A)

[陷入 或 trap 类指令](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E5%85%B6%E4%B8%AD%20%E6%96%AD%E7%82%B9%20(Breakpoint)%20%E5%92%8C%20%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E8%B0%83%E7%94%A8%20(Environment%20call)%20%E4%B8%A4%E7%A7%8D%E5%BC%82%E5%B8%B8%EF%BC%88%E4%B8%BA%E4%BA%86%E4%B8%8E%E5%85%B6%E4%BB%96%E9%9D%9E%E6%9C%89%E6%84%8F%E4%B8%BA%E4%B9%8B%E7%9A%84%E5%BC%82%E5%B8%B8%E5%8C%BA%E5%88%86%EF%BC%8C%E4%BC%9A%E6%8A%8A%E8%BF%99%E7%A7%8D%E6%9C%89%E6%84%8F%E4%B8%BA%E4%B9%8B%E7%9A%84%E6%8C%87%E4%BB%A4%E7%A7%B0%E4%B8%BA%20%E9%99%B7%E5%85%A5%20%E6%88%96%20trap%20%E7%B1%BB%E6%8C%87%E4%BB%A4%EF%BC%8C%E6%AD%A4%E5%A4%84%E7%9A%84%E9%99%B7%E5%85%A5%E4%B8%BA%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E4%B8%AD%E4%BC%A0%E7%BB%9F%E6%A6%82%E5%BF%B5%EF%BC%89%E6%98%AF%E9%80%9A%E8%BF%87%E5%9C%A8%E4%B8%8A%E5%B1%82%E8%BD%AF%E4%BB%B6%E4%B8%AD%E6%89%A7%E8%A1%8C%E4%B8%80%E6%9D%A1%E7%89%B9%E5%AE%9A%E7%9A%84%E6%8C%87%E4%BB%A4%E8%A7%A6%E5%8F%91%E7%9A%84%EF%BC%9A%E6%89%A7%E8%A1%8C%20ebreak%20%E8%BF%99%E6%9D%A1%E6%8C%87%E4%BB%A4%E4%B9%8B%E5%90%8E%E5%B0%B1%E4%BC%9A%E8%A7%A6%E5%8F%91%E6%96%AD%E7%82%B9%E9%99%B7%E5%85%A5%E5%BC%82%E5%B8%B8%EF%BC%9B%E8%80%8C%E6%89%A7%E8%A1%8C%20ecall%20%E8%BF%99%E6%9D%A1%E6%8C%87%E4%BB%A4%E6%97%B6%E5%80%99%E5%88%99%E4%BC%9A%E9%9A%8F%E7%9D%80%20CPU%20%E5%BD%93%E5%89%8D%E6%89%80%E5%A4%84%E7%89%B9%E6%9D%83%E7%BA%A7%E8%80%8C%E8%A7%A6%E5%8F%91%E4%B8%8D%E5%90%8C%E7%9A%84%E5%BC%82%E5%B8%B8%E3%80%82)

[ecall 指令](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E5%9C%A8%E8%BF%99%E9%87%8C%E6%88%91%E4%BB%AC,%E5%92%8C%E7%81%B5%E6%B4%BB%E6%80%A7) —— 陷入机制

- 随着 CPU 当前所处特权级而触发不同的异常
- 实现相邻两特权级软件之间的接口（特权级切换）

[异常行为](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E6%AF%8F%E5%B1%82%E7%89%B9%E6%9D%83%E7%BA%A7%E7%9A%84%E8%BD%AF%E4%BB%B6%E9%83%BD%E5%8F%AA%E8%83%BD%E5%81%9A%E9%AB%98%E7%89%B9%E6%9D%83%E7%BA%A7%E8%BD%AF%E4%BB%B6%E5%85%81%E8%AE%B8%E5%AE%83%E5%81%9A%E7%9A%84%E3%80%81%E4%B8%94%E4%B8%8D%E4%BC%9A%E4%BA%A7%E7%94%9F%E4%BB%80%E4%B9%88%E6%92%BC%E5%8A%A8%E9%AB%98%E7%89%B9%E6%9D%83%E7%BA%A7%E8%BD%AF%E4%BB%B6%E7%9A%84%E4%BA%8B%E6%83%85%EF%BC%8C%E4%B8%80%E6%97%A6%E4%BD%8E%E7%89%B9%E6%9D%83%E7%BA%A7%E8%BD%AF%E4%BB%B6%E7%9A%84%E8%A6%81%E6%B1%82%E8%B6%85%E5%87%BA%E4%BA%86%E5%85%B6%E8%83%BD%E5%8A%9B%E8%8C%83%E5%9B%B4%EF%BC%8C%E5%B0%B1%E5%BF%85%E9%A1%BB%E5%AF%BB%E6%B1%82%E9%AB%98%E7%89%B9%E6%9D%83%E7%BA%A7%E8%BD%AF%E4%BB%B6%E7%9A%84%E5%B8%AE%E5%8A%A9%EF%BC%8C%E5%90%A6%E5%88%99%E5%B0%B1%E6%98%AF%E4%B8%80%E7%A7%8D%E5%BC%82%E5%B8%B8%E8%A1%8C%E4%B8%BA%E4%BA%86%E3%80%82)

<img src="pic/EnvironmentCallFlow.png" alt="EnvironmentCallFlow.png" style="zoom: 25%;" />

#### RISC-V的特权指令

[特权级保护机制](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/1rv-privilege.html#:~:text=%E5%A6%82%E6%9E%9C%E5%A4%84%E4%BA%8E%E4%BD%8E%E7%89%B9%E6%9D%83%E7%BA%A7%E7%8A%B6%E6%80%81%E7%9A%84%E5%A4%84%E7%90%86%E5%99%A8%E6%89%A7%E8%A1%8C%E4%BA%86%E9%AB%98%E7%89%B9%E6%9D%83%E7%BA%A7%E7%9A%84%E6%8C%87%E4%BB%A4%EF%BC%8C%E4%BC%9A%E4%BA%A7%E7%94%9F%E9%9D%9E%E6%B3%95%E6%8C%87%E4%BB%A4%E9%94%99%E8%AF%AF%E7%9A%84%E5%BC%82%E5%B8%B8%E3%80%82%E8%BF%99%E6%A0%B7%EF%BC%8C%E4%BD%8D%E4%BA%8E%E9%AB%98%E7%89%B9%E6%9D%83%E7%BA%A7%E7%9A%84%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E8%83%BD%E5%A4%9F%E5%BE%97%E7%9F%A5%E4%BD%8E%E7%89%B9%E6%9D%83%E7%BA%A7%E7%9A%84%E8%BD%AF%E4%BB%B6%E5%87%BA%E7%8E%B0%E4%BA%86%E9%94%99%E8%AF%AF%EF%BC%8C%E8%BF%99%E4%B8%AA%E9%94%99%E8%AF%AF%E4%B8%80%E8%88%AC%E6%98%AF%E4%B8%8D%E5%8F%AF%E6%81%A2%E5%A4%8D%E7%9A%84%EF%BC%8C%E6%AD%A4%E6%97%B6%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%E4%BC%9A%E5%B0%86%E4%BD%8E%E7%89%B9%E6%9D%83%E7%BA%A7%E7%9A%84%E8%BD%AF%E4%BB%B6%E7%BB%88%E6%AD%A2%E3%80%82%E8%BF%99%E5%9C%A8%E6%9F%90%E7%A7%8D%E7%A8%8B%E5%BA%A6%E4%B8%8A%E4%BD%93%E7%8E%B0%E4%BA%86%E7%89%B9%E6%9D%83%E7%BA%A7%E4%BF%9D%E6%8A%A4%E6%9C%BA%E5%88%B6%E7%9A%84%E4%BD%9C%E7%94%A8%E3%80%82)

- 控制状态寄存器 (CSR, Control and Status Register) —— 控制该特权级的某些行为并描述其状态
- sret —— 从 S 模式返回 U 模式：在 U 模式下执行会产生非法指令异常


- sfence.vma —— 刷新 TLB 缓存：在 U 模式下执行会产生非法指令异常




---

### 实现应用程序

#### 内存布局

#### 系统调用

在子模块 `syscall` 中，应用程序通过 `ecall` 调用批处理系统提供的接口，由于应用程序运行在用户态（即 U 模式）， `ecall` 指令会触发 名为 *Environment call from U-mode* 的异常，并 Trap 进入 S 模式执行批处理系统针对这个异常特别提供的服务代码。由于这个接口处于 S 模式的批处理系统和 U 模式的应用程序之间，这个接口可以被称为 ABI 或者系统调用

在实际调用的时候，需要按照 RISC-V 调用规范（即ABI格式）在合适的寄存器中放置系统调用的参数，然后执行 `ecall` 指令触发 Trap。在 Trap 回到 U 模式的应用程序代码之后，会从 `ecall` 的下一条指令继续执行，同时能够按照调用规范在合适的寄存器中读取返回值

[静态绑定与动态加载](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/3batch-system.html#:~:text=%EF%BC%8C%E5%BA%94%E7%94%A8%E6%94%BE%E7%BD%AE%E9%87%87%E7%94%A8,%E5%86%85%E5%AD%98%E4%B8%AD%E8%BF%90%E8%A1%8C%E3%80%82)



---

### 实现批处理操作系统

#### 将应用程序链接到内核

build.rs -> link_app.S

#### 找到并加载应用程序二进制码

[应用管理器](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/3batch-system.html#:~:text=%E5%BA%94%E7%94%A8%E7%AE%A1%E7%90%86%E5%99%A8%EF%BC%8C%E5%AE%83%E7%9A%84,%E5%8A%A0%E8%BD%BD%E5%BA%94%E7%94%A8%E6%89%A7%E8%A1%8C%E3%80%82) -> 全局实例 `APP_MANAGER` 

[rust全局可变变量](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/3batch-system.html#:~:text=Rust%20%E7%BC%96%E8%AF%91%E5%99%A8%E6%8F%90%E7%A4%BA,%E4%BD%BF%E7%94%A8%E5%8F%AF%E5%8F%98%E5%85%A8%E5%B1%80%E5%8F%98%E9%87%8F%E3%80%82) lazy_static + RefCell + Trait Sync -> [UnSafeCell](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/3batch-system.html#:~:text=UPSafeCell%20%E5%AF%B9%E4%BA%8E%20RefCell%20%E7%AE%80%E5%8D%95%E8%BF%9B%E8%A1%8C%E5%B0%81%E8%A3%85%EF%BC%8C%E5%AE%83%E5%92%8C%20RefCell%20%E4%B8%80%E6%A0%B7%E6%8F%90%E4%BE%9B%E5%86%85%E9%83%A8%E5%8F%AF%E5%8F%98%E6%80%A7%E5%92%8C%E8%BF%90%E8%A1%8C%E6%97%B6%E5%80%9F%E7%94%A8%E6%A3%80%E6%9F%A5%EF%BC%8C%E5%8F%AA%E6%98%AF%E6%9B%B4%E5%8A%A0%E4%B8%A5%E6%A0%BC%EF%BC%9A%E8%B0%83%E7%94%A8%20exclusive_access%20%E5%8F%AF%E4%BB%A5%E5%BE%97%E5%88%B0%E5%AE%83%E5%8C%85%E8%A3%B9%E7%9A%84%E6%95%B0%E6%8D%AE%E7%9A%84%E7%8B%AC%E5%8D%A0%E8%AE%BF%E9%97%AE%E6%9D%83%E3%80%82%E5%9B%A0%E6%AD%A4%E5%BD%93%E6%88%91%E4%BB%AC%E8%A6%81%E8%AE%BF%E9%97%AE%E6%95%B0%E6%8D%AE%E6%97%B6%EF%BC%88%E6%97%A0%E8%AE%BA%E8%AF%BB%E8%BF%98%E6%98%AF%E5%86%99%EF%BC%89%EF%BC%8C%E9%9C%80%E8%A6%81%E9%A6%96%E5%85%88%E8%B0%83%E7%94%A8%20exclusive_access%20%E8%8E%B7%E5%BE%97%E6%95%B0%E6%8D%AE%E7%9A%84%E5%8F%AF%E5%8F%98%E5%80%9F%E7%94%A8%E6%A0%87%E8%AE%B0%EF%BC%8C%E9%80%9A%E8%BF%87%E5%AE%83%E5%8F%AF%E4%BB%A5%E5%AE%8C%E6%88%90%E6%95%B0%E6%8D%AE%E7%9A%84%E8%AF%BB%E5%86%99%EF%BC%8C%E5%9C%A8%E6%93%8D%E4%BD%9C%E5%AE%8C%E6%88%90%E4%B9%8B%E5%90%8E%E6%88%91%E4%BB%AC%E9%9C%80%E8%A6%81%E9%94%80%E6%AF%81%E8%BF%99%E4%B8%AA%E6%A0%87%E8%AE%B0%EF%BC%8C%E6%AD%A4%E5%90%8E%E6%89%8D%E8%83%BD%E5%BC%80%E5%A7%8B%E5%AF%B9%E8%AF%A5%E6%95%B0%E6%8D%AE%E7%9A%84%E4%B8%8B%E4%B8%80%E6%AC%A1%E8%AE%BF%E9%97%AE%E3%80%82%E7%9B%B8%E6%AF%94%20RefCell%20%E5%AE%83%E4%B8%8D%E5%86%8D%E5%85%81%E8%AE%B8%E5%A4%9A%E4%B8%AA%E8%AF%BB%E6%93%8D%E4%BD%9C%E5%90%8C%E6%97%B6%E5%AD%98%E5%9C%A8%E3%80%82)

[手动加载app](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/3batch-system.html#:~:text=%E8%BF%99%E4%B8%AA%E6%96%B9%E6%B3%95%E8%B4%9F%E8%B4%A3%E5%B0%86%E5%8F%82%E6%95%B0%20app_id%20%E5%AF%B9%E5%BA%94%E7%9A%84%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E7%9A%84%E4%BA%8C%E8%BF%9B%E5%88%B6%E9%95%9C%E5%83%8F%E5%8A%A0%E8%BD%BD%E5%88%B0%E7%89%A9%E7%90%86%E5%86%85%E5%AD%98%E4%BB%A5%200x80400000%20%E8%B5%B7%E5%A7%8B%E7%9A%84%E4%BD%8D%E7%BD%AE%EF%BC%8C%E8%BF%99%E4%B8%AA%E4%BD%8D%E7%BD%AE%E6%98%AF%E6%89%B9%E5%A4%84%E7%90%86%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E5%92%8C%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E4%B9%8B%E9%97%B4%E7%BA%A6%E5%AE%9A%E7%9A%84%E5%B8%B8%E6%95%B0%E5%9C%B0%E5%9D%80%EF%BC%8C%E5%9B%9E%E5%BF%86%E4%B8%8A%E4%B8%80%E5%B0%8F%E8%8A%82%E4%B8%AD%EF%BC%8C%E6%88%91%E4%BB%AC%E4%B9%9F%E8%B0%83%E6%95%B4%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%E7%9A%84%E5%86%85%E5%AD%98%E5%B8%83%E5%B1%80%E4%BB%A5%E5%90%8C%E4%B8%80%E4%B8%AA%E5%9C%B0%E5%9D%80%E5%BC%80%E5%A4%B4%E3%80%82%E7%AC%AC%208%20%E8%A1%8C%E5%BC%80%E5%A7%8B%EF%BC%8C%E6%88%91%E4%BB%AC%E9%A6%96%E5%85%88%E5%B0%86%E4%B8%80%E5%9D%97%E5%86%85%E5%AD%98%E6%B8%85%E7%A9%BA%EF%BC%8C%E7%84%B6%E5%90%8E%E6%89%BE%E5%88%B0%E5%BE%85%E5%8A%A0%E8%BD%BD%E5%BA%94%E7%94%A8%E4%BA%8C%E8%BF%9B%E5%88%B6%E9%95%9C%E5%83%8F%E7%9A%84%E4%BD%8D%E7%BD%AE%EF%BC%8C%E5%B9%B6%E5%B0%86%E5%AE%83%E5%A4%8D%E5%88%B6%E5%88%B0%E6%AD%A3%E7%A1%AE%E7%9A%84%E4%BD%8D%E7%BD%AE%E3%80%82%E5%AE%83%E6%9C%AC%E8%B4%A8%E4%B8%8A%E6%98%AF%E6%8A%8A%E6%95%B0%E6%8D%AE%E4%BB%8E%E4%B8%80%E5%9D%97%E5%86%85%E5%AD%98%E5%A4%8D%E5%88%B6%E5%88%B0%E5%8F%A6%E4%B8%80%E5%9D%97%E5%86%85%E5%AD%98%EF%BC%8C%E4%BB%8E%E6%89%B9%E5%A4%84%E7%90%86%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E7%9A%84%E8%A7%92%E5%BA%A6%E6%9D%A5%E7%9C%8B%EF%BC%8C%E6%98%AF%E5%B0%86%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E6%95%B0%E6%8D%AE%E6%AE%B5%E7%9A%84%E4%B8%80%E9%83%A8%E5%88%86%E6%95%B0%E6%8D%AE%EF%BC%88%E5%AE%9E%E9%99%85%E4%B8%8A%E6%98%AF%E5%BA%94%E7%94%A8%E7%A8%8B%E5%BA%8F%EF%BC%89%E5%A4%8D%E5%88%B6%E5%88%B0%E4%BA%86%E4%B8%80%E4%B8%AA%E5%8F%AF%E4%BB%A5%E6%89%A7%E8%A1%8C%E4%BB%A3%E7%A0%81%E7%9A%84%E5%86%85%E5%AD%98%E5%8C%BA%E5%9F%9F%E3%80%82%E5%9C%A8%E8%BF%99%E4%B8%80%E7%82%B9%E4%B8%8A%E4%B9%9F%E4%BD%93%E7%8E%B0%E4%BA%86%E5%86%AF%E8%AF%BA%E4%BE%9D%E6%9B%BC%E8%AE%A1%E7%AE%97%E6%9C%BA%E7%9A%84%20%E4%BB%A3%E7%A0%81%E5%8D%B3%E6%95%B0%E6%8D%AE%20%E7%9A%84%E7%89%B9%E5%BE%81%E3%80%82) [清理icache](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/3batch-system.html#:~:text=%E6%B3%A8%E6%84%8F%E7%AC%AC%207,%E7%9A%84%E6%AD%A3%E7%A1%AE%E6%80%A7%E3%80%82)



----

### RISC-V特权级切换

[特权级切换的起因](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/4trap-handling.html#:~:text=%E7%89%B9%E6%9D%83%E7%BA%A7%E5%88%87%E6%8D%A2-,%E7%89%B9%E6%9D%83%E7%BA%A7%E5%88%87%E6%8D%A2%E7%9A%84%E8%B5%B7%E5%9B%A0,-%23)

批处理操作系统为了建立好应用程序的执行环境，需要在执行应用程序之前进行一些初始化工作，并监控应用程序的执行    [具体体现](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/4trap-handling.html#:~:text=%E5%BD%93%E5%90%AF%E5%8A%A8%E5%BA%94%E7%94%A8,%E6%9D%A5%E5%AE%9E%E7%8E%B0%E7%9A%84%EF%BC%89%E3%80%82)

这些处理都涉及到特权级切换，因此需要应用程序、操作系统和硬件一起协同，完成特权级切换机制

#### 特权级切换相关的控制状态寄存器

[进入 S 特权级 Trap 的相关 CSR](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/4trap-handling.html#:~:text=%E8%BF%9B%E5%85%A5%20S%20%E7%89%B9%E6%9D%83,%E7%9A%84%E5%85%A5%E5%8F%A3%E5%9C%B0%E5%9D%80)

S模式下最重要的 sstatus 寄存器

[特权级切换](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/4trap-handling.html#:~:text=%E5%92%8C%E6%89%A7%E8%A1%8C%E7%8A%B6%E6%80%81%E3%80%82-,%E7%89%B9%E6%9D%83%E7%BA%A7%E5%88%87%E6%8D%A2,%E7%9B%B4%E6%8E%A5%E5%AE%8C%E6%88%90%EF%BC%8C%E5%8F%A6%E4%B8%80%E9%83%A8%E5%88%86%E5%88%99%E9%9C%80%E8%A6%81%E7%94%B1%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E6%9D%A5%E5%AE%9E%E7%8E%B0%E3%80%82,-%E7%89%B9%E6%9D%83%E7%BA%A7%E5%88%87%E6%8D%A2)

[特权级切换的硬件控制机制](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/4trap-handling.html#:~:text=%E5%BD%93%20CPU%20%E6%89%A7%E8%A1%8C,%E5%A4%84%E5%BC%80%E5%A7%8B%E6%89%A7%E8%A1%8C%E3%80%82)

#### 用户栈与内核栈

`KernelStack` 和 `UserStack` 分别表示用户栈和内核栈

- [目的：安全性](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/4trap-handling.html#:~:text=%E5%9C%A8%20Trap%20%E8%A7%A6%E5%8F%91,%E7%9A%84%E5%AF%84%E5%AD%98%E5%99%A8%E7%8A%B6%E6%80%81%E3%80%82)

- 操作：批处理操作系统中添加一段汇编代码，实现从用户栈切换到内核栈，并在内核栈上保存应用程序控制流的寄存器状态

- `TrapContext`

#### Trap 管理

- 应用程序通过 `ecall` 进入到内核状态时，操作系统保存被打断的应用程序的 Trap 上下文；
- 操作系统根据Trap相关的CSR寄存器内容，完成系统调用服务的分发与处理；
- 操作系统完成系统调用服务后，需要恢复被打断的应用程序的Trap 上下文，并通 `sret` 让应用程序继续执行。

**Trap 上下文的保存与恢复**

- 在批处理操作系统初始化的时候，我们需要修改 `stvec` 寄存器来指向正确的 Trap 处理入口点。

#### Trap 分发与处理

Trap 在使用 Rust 实现的 `trap_handler` 函数中完成分发和处理



#### 实现系统调用功能

#### 执行应用程序

当批处理操作系统初始化完成，或者是某个应用程序运行结束或出错的时候，我们要调用 `run_next_app` 函数切换到下一个应用程序。此时 CPU 运行在 S 特权级，而它希望能够切换到 U 特权级。在 RISC-V 架构中，唯一一种能够使得 CPU 特权级下降的方法就是执行 Trap 返回的特权指令，如 `sret` 、`mret` 等。事实上，在从操作系统内核返回到运行应用程序之前，要完成如下这些工作：

- 构造应用程序开始执行所需的 Trap 上下文；
- 通过 `__restore` 函数，从刚构造的 Trap 上下文中，恢复应用程序执行的部分寄存器；
- 设置 `sepc` CSR的内容为应用程序入口点 `0x80400000`；
- 切换 `scratch` 和 `sp` 寄存器，设置 `sp` 指向应用程序用户栈；
- 执行 `sret` 从 S 特权级切换到 U 特权级。

它们可以通过复用 `__restore` 的代码来更容易的实现上述工作。在内核栈上压入一个为启动应用程序而特殊构造的 Trap 上下文，再通过 `__restore` 函数，就能让这些寄存器到达启动应用程序所需要的上下文状态。

> sscratch 是何时被设置为内核栈顶的？ 
>
> sscratch 在完成各子系统初始化后，跳转到用户态之前设置为内核栈顶