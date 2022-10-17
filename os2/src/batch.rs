use crate::sync::UnSafeCell;
use crate::trap::TrapContext;
use lazy_static::*;

const MAX_APP_NUM: usize = 16;
const USER_STACK_SIZE: usize = 4096;
const KERNEL_STACK_SIZE: usize = 4096 * 20;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

#[repr(align(4096))] // represent the alignment of the struct
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};

static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

/// 两个栈以全局变量的形式实例化在批处理操作系统的 .bss 段中
/// 为两个类型实现了 get_sp 方法来获取栈顶地址。
/// 由于在 RISC-V 中栈是向下增长的， 我们只需返回包裹的数组的结尾地址
/// 换栈是非常简单的，只需将 sp 寄存器的值修改为 get_sp 的返回值即可
impl KernelStack {
    // get the top address of kernel stack; sp register
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    // push_context 返回值是内核栈压入 Trap 上下文之后的栈顶，它会被作为 __restore 的参数
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext; // get the address of TrapContext
        unsafe {
            *cx_ptr = cx;
        }
        unsafe {
            cx_ptr.as_mut().unwrap()
        }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

/// 应用管理器
/// - 保存应用数量和各自的位置信息，以及当前执行到第几个应用了。
/// - 根据应用程序位置信息，初始化好应用所需内存空间，并加载应用执行。
/// 将 AppManager 实例化为一个全局变量，使得任何函数都可以直接访问
/// current_app 字段表示当前执行的是第几个应用，它是一个可修改的变量，会在系统运行期间发生变化
struct AppManager {
    num_apps: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    pub fn print_app_info(&self) {
        info!("[kernel] num_apps: {}", self.num_apps);
        for i in 0..self.num_apps {
            info!("[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        // self.current_app = (self.current_app + 1) % self.num_apps; // BUG: if you want to in infinite loop :(
        self.current_app += 1; // BUG: if you want to in infinite loop :(
    }

    /// 方法负责将参数 app_id 对应的应用程序的二进制镜像加载到物理内存以 0x80400000 起始的位置，
    /// 这个位置是批处理操作系统和应用程序之间约定的常数地址
    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_apps {
            panic!("All apps have been completed!");
        }
        info!("[kernel] load app_{}", app_id);
        // clear icache(instruction cache)
        // 汇编指令 fence.i ，它是用来清理 i-cache 的
        // 通常情况下， CPU 会认为程序的代码段不会发生变化，因此 i-cache 是一种只读缓存。
        // 但在这里，OS将修改会被 CPU 取指的内存区域，这会使得 i-cache 中含有与内存中不一致的内容。
        // 因此OS在这里必须使用 fence.i 指令手动清空 i-cache ，让里面所有的内容全部失效，才能够保证CPU访问内存数据和代码的正确性。
        // [fence.i](https://five-embeddev.com/riscv-isa-manual/latest/zifencei.html)
        core::arch::asm!("fence.i");
        // clear app area
        // 首先将一块内存清空，然后找到待加载应用二进制镜像的位置，并将它复制到正确的位置。
        // 它本质上是把数据从一块内存复制到另一块内存，从批处理操作系统的角度来看，
        // 是将操作系统数据段的一部分数据（实际上是应用程序）复制到了一个可以执行代码的内存区域。
        // 在这一点上也体现了冯诺依曼计算机的 代码即数据 的特征。
        core::slice::from_raw_parts_mut(
            APP_BASE_ADDRESS as *mut u8,
            APP_SIZE_LIMIT
        ).fill(0);
        // copy app to app area
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id]
        );
        // 从 app_src 拷贝到 APP_BASE_ADDRESS
        let app_dst = core::slice::from_raw_parts_mut(
            APP_BASE_ADDRESS as *mut u8,
            app_src.len()
        );
        app_dst.copy_from_slice(app_src);
    }
}

lazy_static! {
    static ref APP_MANAGER: UnSafeCell<AppManager> = unsafe {
        UnSafeCell::new({
            extern "C" {
                // 找到 link_app.S 中提供的符号 _num_apps，并从这里开始解析出应用数量以及各个应用的起始地址
                fn _num_apps(); 
            }
            let num_app_ptr = _num_apps as usize as *const usize;
            let num_apps = num_app_ptr.read_volatile(); // read_volatile; 对ptr的值进行易失性读取，而无需移动它 
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            // app_start_raw
            // slice::from_raw_parts 根据指针和长度形成切片
            let app_start_raw: &[usize] = core::slice::from_raw_parts(
                // why add? -> see build.rs
                num_app_ptr.add(1), num_apps + 1
            );
            //  slice::copy_from_slice: 使用 memcpy 将所有元素从 src 复制到 self
            app_start[..=num_apps].copy_from_slice(app_start_raw); 
            AppManager {
                num_apps,
                current_app: 0,
                app_start,
            }
        })
    };
}


/// 调用 print_app_info 的时候第一次用到了全局变量 APP_MANAGER ，它也是在这个时候完成初始化
pub fn init() {
    print_app_info();
}

pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

pub fn run_next_app() -> !{
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();

    unsafe {
        app_manager.load_app(current_app);
    }

    app_manager.move_to_next_app();
    // DROP app_manager
    drop(app_manager);
    // before this we have to drop local variables related to resources manually
    // and release the resources
    extern "C" {
        fn __restore(cx_addr:usize);// restore context; cx_addr: context address
    }
    unsafe {
        // 在内核栈上压入一个 Trap 上下文，其 sepc 是应用程序入口地址 0x80400000 ，
        // 其 sp 寄存器指向用户栈，其 sstatus 的 SPP 字段被设置为 User 。
        // push_context 的返回值是内核栈压入 Trap 上下文之后的栈顶，它会被作为 __restore 的参数
        // -> trap.S, 这时我们可以理解为何 __restore 函数的起始部分会完成
        // 这使得在 __restore 函数中 sp 仍然可以指向内核栈的栈顶。这之后，就和执行一次普通的 __restore 函数调用一样了
        __restore(KERNEL_STACK.push_context(
            TrapContext::app_init_context(
                APP_BASE_ADDRESS, 
                USER_STACK.get_sp(),
            )
        ) as *const _ as usize);
    }
    panic!("Unreachable in batch::run_next_app!");        
}