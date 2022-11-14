> [常见工具的使用方法](http://rcore-os.cn/rCore-Tutorial-Book-v3/appendix-b/index.html)

`cargo new os --bin`

> `--bin` 选项来告诉 Cargo 我们创建一个可执行程序项目而不是函数库项目



`rustc --print target-list | grep riscv`

> 查看目前 Rust 编译器支持哪些基于 RISC-V 的平台



`cargo run --target`

> 切换目标平台



[Rust语言标准库](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/1app-ee-platform.html#:~:text=Rust-,Tips%EF%BC%9ARust%E8%AF%AD%E8%A8%80%E6%A0%87%E5%87%86%E5%BA%93,-Rust%20%E8%AF%AD%E8%A8%80%E6%A0%87%E5%87%86)



`rustup target add` 

> 添加目标平台



[Rust std 库和 core 库](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/2remove-std.html#id2:~:text=Rust%20Tips%3A-,Rust%20std%20%E5%BA%93%E5%92%8C%20core%20%E5%BA%93,-Rust%20%E7%9A%84%E6%A0%87%E5%87%86)



[#[panic_handler]](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/2remove-std.html#id2:~:text=%E6%B3%A8%E8%A7%A3-,%23%5Bpanic_handler%5D,-%23%5Bpanic_handler%5D%20%E6%98%AF)

在使用 Rust 编写应用程序的时候，我们常常在遇到了一些无法恢复的致命错误（panic），导致程序无法继续向下运行。这时手动或自动调用 `panic!` 宏来打印出错的位置，让我们能够意识到它的存在，并进行一些后续处理。 `panic!` 宏最典型的应用场景包括断言宏 `assert!` 失败或者对 `Option::None/Result::Err` 进行 `unwrap` 操作。所以Rust编译器在编译程序时，从安全性考虑，需要有 `panic!` 宏的具体实现。

在标准库 std 中提供了关于 `panic!` 宏的具体实现，其大致功能是打印出错位置和原因并杀死当前应用。但我们要实现的操作系统是不能使用还需依赖操作系统的标准库std，而更底层的核心库 core 中只有一个 `panic!` 宏的空壳，并没有提供 `panic!` 宏的精简实现。



[Rust 模块化编程](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/2remove-std.html#id2:~:text=Rust-,Tips%EF%BC%9ARust%20%E6%A8%A1%E5%9D%97%E5%8C%96%E7%BC%96%E7%A8%8B,-%E5%B0%86%E4%B8%80%E4%B8%AA%E8%BD%AF%E4%BB%B6)



`cargo install cargo-binutils`

`rustup component add llvm-tools-preview`

```
文件格式
file target/riscv64gc-unknown-none-elf/debug/os
target/riscv64gc-unknown-none-elf/debug/os: ELF 64-bit LSB executable, UCB RISC-V, ......

文件头信息
rust-readobj -h target/riscv64gc-unknown-none-elf/debug/os
   File: target/riscv64gc-unknown-none-elf/debug/os
   Format: elf64-littleriscv
   Arch: riscv64
   AddressSize: 64bit
   ......
   Type: Executable (0x2)
   Machine: EM_RISCV (0xF3)
   Version: 1
   Entry: 0x0
   ......
   }

反汇编导出汇编程序
rust-objdump -S target/riscv64gc-unknown-none-elf/debug/os
   target/riscv64gc-unknown-none-elf/debug/os:       file format elf64-littleriscv
```

通过 `file` 工具对二进制程序 `os` 的分析可以看到它好像是一个合法的 RISC-V 64 可执行程序

但通过 `rust-readobj` 工具进一步分析，发现它的入口地址 Entry 是 `0` ，从 C/C++ 等语言中得来的经验告诉我们， `0` 一般表示 NULL 或空指针，因此等于 `0` 的入口地址看上去无法对应到任何指https://doc.rust-lang.org/reference/comments.html令

再通过 `rust-objdump` 工具把它反汇编，可以看到没有生成汇编代码。所以，我们可以断定，这个二进制程序虽然合法，但它是一个空程序

产生该现象的原因是：目前我们的程序（参考上面的源代码）没有进行任何有意义的工作，由于我们移除了 `main` 函数并将项目设置为 `#![no_main]` ，它甚至没有一个传统意义上的入口点（即程序首条被执行的指令所在的位置），因此 Rust 编译器会生成一个空程序。



`cargo build --release`

> 以 `release` 模式生成了内核可执行文件，它的位置在 `os/target/riscv64gc.../release/os`



丢弃内核可执行文件中的元数据得到内核镜像

```
rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os -O binary target/riscv64gc-unknown-none-elf/release/os.bin
```



使用 `stat` 工具来比较内核可执行文件和内核镜像的大小

```
stat target/riscv64gc-unknown-none-elf/release/os
File: target/riscv64gc-unknown-none-elf/release/os
Size: 1016              Blocks: 8          IO Block: 4096   regular file
...
$ stat target/riscv64gc-unknown-none-elf/release/os.bin
File: target/riscv64gc-unknown-none-elf/release/os.bin
Size: 4                 Blocks: 8          IO Block: 4096   regular file
...
```



[外部符号引用](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#:~:text=Rust%20Tips%EF%BC%9A-,%E5%A4%96%E9%83%A8%E7%AC%A6%E5%8F%B7%E5%BC%95%E7%94%A8,-extern%20%E2%80%9CC%E2%80%9D%20%E5%8F%AF%E4%BB%A5)



[Unsafe](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/5support-func-call.html#:~:text=Rust-,Tips%EF%BC%9AUnsafe,-%E4%BB%A3%E7%A0%81%E7%AC%AC%2014)



[RustSBI 职责](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/6print-and-shutdown-based-on-sbi.html#:~:text=%E5%AF%B9%20RustSBI%20%E7%9A%84%E4%BA%86%E8%A7%A3%E4%BB%85%E9%99%90%E4%BA%8E%E5%AE%83%E4%BC%9A%E5%9C%A8%E8%AE%A1%E7%AE%97%E6%9C%BA%E5%90%AF%E5%8A%A8%E6%97%B6%E8%BF%9B%E8%A1%8C%E5%AE%83%E6%89%80%E8%B4%9F%E8%B4%A3%E7%9A%84%E7%8E%AF%E5%A2%83%E5%88%9D%E5%A7%8B%E5%8C%96%E5%B7%A5%E4%BD%9C%EF%BC%8C%E5%B9%B6%E5%B0%86%E8%AE%A1%E7%AE%97%E6%9C%BA%E6%8E%A7%E5%88%B6%E6%9D%83%E7%A7%BB%E4%BA%A4%E7%BB%99%E5%86%85%E6%A0%B8%E3%80%82%E4%BD%86%E5%AE%9E%E9%99%85%E4%B8%8A%E4%BD%9C%E4%B8%BA%E5%86%85%E6%A0%B8%E7%9A%84%E6%89%A7%E8%A1%8C%E7%8E%AF%E5%A2%83%EF%BC%8C%E5%AE%83%E8%BF%98%E6%9C%89%E5%8F%A6%E4%B8%80%E9%A1%B9%E8%81%8C%E8%B4%A3%EF%BC%9A%E5%8D%B3%E5%9C%A8%E5%86%85%E6%A0%B8%E8%BF%90%E8%A1%8C%E6%97%B6%E5%93%8D%E5%BA%94%E5%86%85%E6%A0%B8%E7%9A%84%E8%AF%B7%E6%B1%82%E4%B8%BA%E5%86%85%E6%A0%B8%E6%8F%90%E4%BE%9B%E6%9C%8D%E5%8A%A1%E3%80%82)

- 在计算机启动时进行它所负责的环境初始化工作，并将计算机控制权移交给内核

- 作为内核的执行环境，在内核运行时响应内核的请求为内核提供服务

  > 当内核发出请求时，计算机控制权交给SBI，由 RustSBI 控制来响应内核的请求，待请求处理完毕后，计算机控制权会被交还给内核。
  >
  > 从内存布局的角度来思考，每一层执行环境（或称软件栈）都对应到内存中的一段代码和数据，这里的控制权转移指的是 CPU 从执行一层软件的代码到执行另一层软件的代码的过程。
  >
  > 这个过程和函数调用比较像，但是内核无法通过函数调用来请求 RustSBI 提供的服务，这是因为内核并没有和 RustSBI 链接到一起，我们仅仅使用 RustSBI 构建后的可执行文件，因此内核对于 RustSBI 的符号一无所知。事实上，内核需要通过另一种复杂的方式来“调用” RustSBI 的服务



[使用 extern 函数调用外部代码](https://rustwiki.org/zh-CN/book/ch19-01-unsafe-rust.html#%E4%BD%BF%E7%94%A8-extern-%E5%87%BD%E6%95%B0%E8%B0%83%E7%94%A8%E5%A4%96%E9%83%A8%E4%BB%A3%E7%A0%81)

[从其它语言调用 Rust 函数](https://rustwiki.org/zh-CN/book/ch19-01-unsafe-rust.html#%E4%BB%8E%E5%85%B6%E5%AE%83%E8%AF%AD%E8%A8%80%E8%B0%83%E7%94%A8-rust-%E5%87%BD%E6%95%B0)



----



文档注释

[//!, /// /\*\*, /\*!](https://doc.rust-lang.org/reference/comments.html)



[What's the difference between use and extern?](https://stackoverflow.com/questions/29403920/whats-the-difference-between-use-and-extern)

- extern crate is equivalent to use, but has [one or two exceptions](https://doc.rust-lang.org/edition-guide/rust-2018/path-changes.html#an-exception))
- 导入所有包时？



`mangle` -> 解决命名冲突mangle



-----



**全局 mut 变量**

在 Rust 中，任何对于 `static mut` 变量的访问控制都是 unsafe 的，而我们要在编程中尽量避免使用 unsafe ，这样才能让编译器负责更多的安全性检查。因此，我们需要考虑如何在尽量避免触及 unsafe 的情况下仍能声明并使用可变的全局变量。

如果单独使用 `static` 而去掉 `mut` 的话，我们可以声明一个初始化之后就不可变的全局变量，但是我们需要 `AppManager` 里面的内容在运行时发生变化。这涉及到 Rust 中的 **内部可变性** （Interior Mutability），也即在变量自身不可变或仅在不可变借用的情况下仍能修改绑定到变量上的值。我们可以通过用上面提到的 `RefCell` 来包裹 `AppManager` ，这样 `RefCell` 无需被声明为 `mut` ，同时被包裹的 `AppManager` 也能被修改。

但是，我们能否将 `RefCell` 声明为一个全局变量呢？

Rust 编译器提示我们 `RefCell<i32>` 未被标记为 `Sync` ，因此 Rust 编译器认为它不能被安全的在线程间共享，也就不能作为全局变量使用。这可能会令人迷惑，这只是一个单线程程序，因此它不会有任何线程间共享数据的行为，为什么不能通过编译呢？事实上，Rust 对于并发安全的检查较为粗糙，当声明一个全局变量的时候，编译器会默认程序员会在多线程上使用它，而并不会检查程序员是否真的这样做。如果一个变量实际上仅会在单线程上使用，那 Rust 会期待我们将变量分配在栈上作为局部变量而不是全局变量。目前我们的内核仅支持单核，也就意味着只有单线程，那么我们可不可以使用局部变量来绕过这个错误呢？

很可惜，在这里和后面章节的很多场景中，有些变量无法作为局部变量使用。这是因为后面内核会并发执行多条控制流，这些控制流都会用到这些变量。如果我们最初将变量分配在某条控制流的栈上，那么我们就需要考虑如何将变量传递到其他控制流上，由于控制流的切换等操作并非常规的函数调用，我们很难将变量传递出去。因此最方便的做法是使用全局变量，这意味着在程序的任何地方均可随意访问它们，自然也包括这些控制流。

除了 `Sync` 的问题之外，看起来 `RefCell` 已经非常接近我们的需求了，因此我们在 `RefCell` 的基础上再封装一个 `UPSafeCell` ，它名字的含义是：允许我们在 *单核* 上安全使用可变全局变量。



`lazy_static!` 宏提供了全局变量的运行时初始化功能。一般情况下，全局变量必须在编译期设置一个初始值，但是有些全局变量依赖于运行期间才能得到的数据作为初始值。这导致这些全局变量需要在运行时发生变化，即需要重新设置初始值之后才能使用。如果我们手动实现的话有诸多不便之处，比如需要把这种全局变量声明为 `static mut` 并衍生出很多 unsafe 代码 。这种情况下我们可以使用 `lazy_static!` 宏来帮助我们解决这个问题。

因此，借助我们设计的 `UPSafeCell<T>` 和外部库 `lazy_static!`，我们就能使用尽量少的 unsafe 代码完成可变全局变量的声明和初始化，且一旦初始化完成，在后续的使用过程中便不再触及 unsafe 代码。



**Rust 所有权模型和借用检查**

> 我们这里简单介绍一下 Rust 的所有权模型。它可以用一句话来概括： **值** （Value）在同一时间只能被绑定到一个 **变量** （Variable）上。这里，“值”指的是储存在内存中固定位置，且格式属于某种特定类型的数据；而变量就是我们在 Rust 代码中通过 `let` 声明的局部变量或者函数的参数等，变量的类型与值的类型相匹配。在这种情况下，我们称值的 **所有权** （Ownership）属于它被绑定到的变量，且变量可以作为访问/控制绑定到它上面的值的一个媒介。变量可以将它拥有的值的所有权转移给其他变量，或者当变量退出其作用域之后，它拥有的值也会被销毁，这意味着值占用的内存或其他资源会被回收。
>
> 有些场景下，特别是在函数调用的时候，我们并不希望将当前上下文中的值的所有权转移到其他上下文中，因此类似于 C/C++ 中的按引用传参， Rust 可以使用 `&` 或 `&mut` 后面加上值被绑定到的变量的名字来分别生成值的不可变引用和可变引用，我们称这些引用分别不可变/可变 **借用** (Borrow) 它们引用的值。顾名思义，我们可以通过可变引用来修改它借用的值，但通过不可变引用则只能读取而不能修改。这些引用同样是需要被绑定到变量上的值，只是它们的类型是引用类型。在 Rust 中，引用类型的使用需要被编译器检查，但在数据表达上，和 C 的指针一样它只记录它借用的值所在的地址，因此在内存中它随平台不同仅会占据 4 字节或 8 字节空间。
>
> 无论值的类型是否是引用类型，我们都定义值的 **生存期** （Lifetime）为代码执行期间该值必须持续合法的代码区域集合（见 [1](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/3batch-system.html#rust-nomicon-lifetime) ），大概可以理解为该值在代码中的哪些地方被用到了：简单情况下，它可能等同于拥有它的变量的作用域，也有可能是从它被绑定开始直到它的拥有者变量最后一次出现或是它被解绑。
>
> 当我们使用 `&` 和 `&mut` 来借用值的时候，则我们编写的代码必须满足某些约束条件，不然无法通过编译：
>
> - 不可变/可变引用的生存期不能 **超出** （Outlive）它们借用的值的生存期，也即：前者必须是后者的子集；
> - 同一时间，借用同一个值的不可变和可变引用不能共存；
> - 同一时间，借用同一个值的不可变引用可以存在多个，但可变引用只能存在一个。
>
> 这是为了 Rust 内存安全而设计的重要约束条件。第一条很好理解，如果值的生存期未能完全覆盖借用它的引用的生存期，就会在某一时刻发生值已被销毁而我们仍然尝试通过引用来访问该值的情形。反过来说，显然当值合法时引用才有意义。最典型的例子是 **悬垂指针** （Dangling Pointer）问题：即我们尝试在一个函数中返回函数中声明的局部变量的引用，并在调用者函数中试图通过该引用访问已被销毁的局部变量，这会产生未定义行为并导致错误。第二、三条的主要目的则是为了避免通过多个引用对同一个值进行的读写操作产生冲突。例如，当对同一个值的读操作和写操作在时间上相互交错时（即不可变/可变引用的生存期部分重叠），读操作便有可能读到被修改到一半的值，通常这会是一个不合法的值从而导致程序无法正确运行。这可能是由于我们在编程上的疏忽，使得我们在读取一个值的时候忘记它目前正处在被修改到一半的状态，一个可能的例子是在 C++ 中正对容器进行迭代访问的时候修改了容器本身。也有可能被归结为 **别名** （Aliasing）问题，例如在 C 函数中有两个指针参数，如果它们指向相同的地址且编译器没有注意到这一点就进行过激的优化，将会使得编译结果偏离我们期望的语义。
>
> 上述约束条件要求借用同一个值的不可变引用和不可变/可变引用的生存期相互隔离，从而能够解决这些问题。Rust 编译器会在编译时使用 **借用检查器** （Borrow Checker）检查这些约束条件是否被满足：其具体做法是尽可能精确的估计引用和值的生存期并将它们进行比较。随着 Rust 语言的愈发完善，其估计的精确度也会越来越高，使得程序员能够更容易通过借用检查。引用相关的借用检查发生在编译期，因此我们可以称其为编译期借用检查。
>
> 相对的，对值的借用方式运行时可变的情况下，我们可以使用 Rust 内置的数据结构将借用检查推迟到运行时，这可以称为运行时借用检查，它的约束条件和编译期借用检查一致。当我们想要发起借用或终止借用时，只需调用对应数据结构提供的接口即可。值的借用状态会占用一部分额外内存，运行时还会有额外的代码对借用合法性进行检查，这是为满足借用方式的灵活性产生的必要开销。当无法通过借用检查时，将会产生一个不可恢复错误，导致程序打印错误信息并立即退出。具体来说，我们通常使用 `RefCell` 包裹可被借用的值，随后调用 `borrow` 和 `borrow_mut` 便可发起借用并获得一个对值的不可变/可变借用的标志，它们可以像引用一样使用。为了终止借用，我们只需手动销毁这些标志或者等待它们被自动销毁。 `RefCell` 的详细用法请参考 [2](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/3batch-system.html#rust-refcell) 。