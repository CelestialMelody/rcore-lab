use core::panic::PanicInfo;
use crate::sbi::shutdown;
// use crate::println; // main -> micro_use

#[panic_handler] // 当遇到不可恢复错误的时候，被标记为语义项 #[panic_handler] 的 panic 函数将会被调用
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("Panicked: {}", info.message().unwrap()); // main -> panic_info_message
    }
    shutdown()
}