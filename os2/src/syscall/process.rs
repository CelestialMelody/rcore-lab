use crate::batch::run_next_app;

/// 打印退出的应用程序的返回值并同样调用 run_next_app 切换到下一个应用程序。
pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    run_next_app();
}