/// 对于系统调用而言， syscall 函数并不会实际处理系统调用，而只是根据 syscall ID 分发到具体的处理函数
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;

mod fs;
mod process;

use fs::*;
use process::*;

pub fn syscall(syscall_id: usize, args:[usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        _ => panic!("Unknown syscall: {}", syscall_id),
    }
}