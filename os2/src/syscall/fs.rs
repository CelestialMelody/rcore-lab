const FD_STDOUT: usize = 1;

/// 将传入的位于应用程序内的缓冲区的开始地址和长度转化为一个字符串 &str ，
/// 然后使用批处理操作系统已经实现的 print! 宏打印出来。
/// 注意这里我们并没有检查传入参数的安全性，即使会在出错严重的时候 panic，还是会存在安全隐患。
/// 这里我们出于实现方便暂且不做修补。
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let s = core::str::from_utf8(slice).unwrap();
            print!("{}", s);
            len as isize
        }
        _ => panic!("Unknown fd: {} in sys_write!", fd),
    }
}
