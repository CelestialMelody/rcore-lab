//! much different from os3, we use a new way to load app
//! loader 模块中原有的内核和用户栈则分别作为逻辑段放在内核和用户地址空间中，我们无需再去专门为其定义一种类型(os3)。

/// get_num_app 获取链接到内核内的应用的数目
pub fn get_num_apps() -> usize {
    extern "C" {
        fn _num_apps();
    }
    unsafe {
        // volatile: 直接存取原始内存地址，可以防止编译器对代码优化;
        (_num_apps as usize as *const usize).read_volatile()
    }
}

/// get_app_data 则根据传入的应用编号取出对应应用的 ELF 格式可执行文件数据。
// 它们和之前一样仍是基于 build.rs 生成的 link_app.S 给出的符号来确定其位置，并实际放在内核的数据段中。
// 在创建应用地址空间的时候，我们需要对 get_app_data 得到的 ELF 格式数据进行解析，
// 找到各个逻辑段所在位置和访问限制并插入进来，最终得到一个完整的应用地址空间 -> see os4/src/mm/memory_set.rs: MemorySet::from_elf
pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" {
        fn _num_apps();
    }

    let num_apps_ptr = _num_apps as usize as *const usize;
    let num_apps = get_num_apps();
    let app_start = unsafe { core::slice::from_raw_parts(num_apps_ptr.add(1), num_apps + 1) };

    assert!(app_id < num_apps);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    }
}
