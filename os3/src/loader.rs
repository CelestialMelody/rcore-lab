use crate::config::*;
use crate::trap::TrapContext;

#[repr(align(4096))] // 页对齐
#[derive(Copy, Clone)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))] // 页对齐
#[derive(Copy, Clone)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static mut KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

static mut USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];

impl KernelStack {
    fn get_top(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, trap_cx: TrapContext) -> usize {
        // return the top of stack
        let trap_cx_ptr =
            (self.get_top() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *trap_cx_ptr = trap_cx;
        }
        trap_cx_ptr as usize
    }
}

impl UserStack {
    fn get_top(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub fn init_app_cx(app_id: usize) -> usize {
    // return the top of the stack
    unsafe {
        KERNEL_STACK[app_id].push_context(TrapContext::app_init_context(
            get_base(app_id),
            USER_STACK[app_id].get_top(),
        ))
    }
}

/// get app's base address by app id
fn get_base(app_id: usize) -> usize {
    APP_BASE_ADDRESS + APP_SIZE_LIMIT * app_id
}

pub fn get_num_apps() -> usize {
    extern "C" {
        fn _num_apps();
    }
    unsafe {
        // volatile: 直接存取原始内存地址，可以防止编译器对代码优化;
        (_num_apps as usize as *const usize).read_volatile()
    }
}

/// Load nth user app at
/// [APP_BASE_ADDRESS + n * APP_SIZE_LIMIT, APP_BASE_ADDRESS + (n+1) * APP_SIZE_LIMIT).
pub fn load_apps() {
    extern "C" {
        // form link_app.S
        fn _num_apps();
    }

    let num_apps_ptr = _num_apps as usize as *const usize;
    let num_apps = get_num_apps();
    let app_start = unsafe {
        core::slice::from_raw_parts(
            // see section _num_apps in link_app.S
            num_apps_ptr.add(1), // before add 1, this is num_apps's pointer
            num_apps + 1,
        )
    };

    // OS将修改会被 CPU 取指的内存区域，这会使得 i-cache 中含有与内存中不一致的内容。
    // 因此OS在使用 fence.i 指令手动清空 i-cache ，让里面所有的内容全部失效，才能够保证CPU访问内存数据和代码的正确性。
    unsafe {
        core::arch::asm!("fence.i");
    }

    // load apps
    for i in 0..num_apps {
        let base_i = get_base(i);
        // clear region
        // use write_volatile to avoid compiler optimization
        (base_i..base_i + APP_SIZE_LIMIT).for_each(|addr| unsafe {
            (addr as *mut u8).write_volatile(0);
        });
        // or use memset
        // unsafe {
        //     core::slice::from_raw_parts_mut(base_i as usize as *mut usize, APP_SIZE_LIMIT + 1)
        //         .fill(0)
        // }

        // load app from data section to memory
        let src = unsafe {
            core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i])
        };
        let dst = unsafe { core::slice::from_raw_parts_mut(base_i as usize as *mut u8, src.len()) };
        dst.copy_from_slice(src);
    }
}
