use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use crate::syscall::timer::TimeVal;
use riscv::register::time;

const TICKS_PRE_SECOND: usize = 100; // can change this value to change the time slice
const MICRO_PRE_SECOND: usize = 1_000_000;

/// read the mtime register:
/// get_time 函数可以取得当前 mtime 计数器的值
/// the value is the tikcs since boot
pub fn get_time() -> usize {
    time::read()
}

// 一个大胆的想法
// pub struct TimeVal {
//     pub sec: usize,  // second
//     pub usec: usize, // microsecond
// }

/// get current time in microsecond:
/// 统计一个应用的运行时长;
/// 1us: 1s / 1_000_000;
/// CLOCK_FREQ: the number of ticks per second;
/// CLOCK_FREQ / MICRO_PRE_SECOND: the number of ticks per microsecond;
pub fn get_time_micro() -> usize {
    time::read() / (CLOCK_FREQ / MICRO_PRE_SECOND)
}

pub fn get_time_val() -> TimeVal {
    let us = get_time_micro();
    let sec = us / MICRO_PRE_SECOND;
    let usec = us % MICRO_PRE_SECOND;
    TimeVal { sec, usec }
}

/// set the next timer interrupt:
/// 对 set_timer 进行了封装，它首先读取当前 mtime 的值，
/// 然后计算出 10ms 之内计数器的增量，再将 mtimecmp 设置为二者的和,
/// 这样，10ms 之后一个 S 特权级时钟中断就会被触发。
/// 10ms: 1s / 100;
/// CLOCK_FREQ: the number of ticks in 1s;
/// CLOCK_FREQ / TICKS_PRE_SECOND: number of ticks in 10ms.
pub fn set_next_trigger() {
    // get_time: get mtime value
    // set_timer: set mtimecmp value
    // time interrupt will be triggered when mtime == mtimecmp
    set_timer(get_time() + (CLOCK_FREQ / TICKS_PRE_SECOND));
}
