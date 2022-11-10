use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use riscv::register::time;

const TICKS_PRE_SECOND: usize = 100;
const MICRO_PRE_SECOND: usize = 1_000_000;

/// read the mtime register
pub fn get_time() -> usize {
    time::read()
}

/// get current time in microsecond
pub fn get_time_micro() -> usize {
    time::read() / (CLOCK_FREQ / MICRO_PRE_SECOND) // 12.5MHz
}

/// set the next timer interrupt
pub fn set_next_trigger() {
    set_timer(get_time() + (CLOCK_FREQ / TICKS_PRE_SECOND));
}
