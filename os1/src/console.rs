use crate::sbi::console_putchar;
use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

/// Prints to the host console using the `print!` format syntax.
pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export] // 将宏进行了导出
/// Prints to the host console using the same syntax as `print!`.
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
/// Prints to the host console using the same syntax as `println!`.
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
       $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
