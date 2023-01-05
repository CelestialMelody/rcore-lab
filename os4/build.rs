// see [`vedio`](https://www.bilibili.com/video/BV1kZ4y167gf/?spm_id_from=333.788&vd_source=fff8a96619bd3da6d1cb5d5c1ede2cf1)
// see [`link`](https://course.rs/cargo/reference/build-script/intro.html)
use std::fs::{read_dir, File};
use std::io::{Result, Write};

// static TARGET_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";
static TARGET_PATH: &str = "../user/build/bin/";

fn main() {
    println!("cargo:rerun-if-changed=../user/src/");
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    insert_app_data().unwrap();
}

fn insert_app_data() -> Result<()> {
    let mut file = File::create("src/link_app.S")?;

    let mut apps: Vec<_> = read_dir(TARGET_PATH)?
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();

    apps.sort();

    writeln!(
        file,
        r#"
    .align 3
    .section .data
    .global _num_apps
_num_apps:
    .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(
            file,
            r#"
    .quad _app_{}_start"#,
            i
        )?;
    }
    writeln!(
        file,
        r#"
    .quad _app_{}_end"#,
        apps.len() - 1
    )?;

    // 应用构建器 os/build.rs 的改动：
    // - 首先，我们在 .incbin 中不再插入清除全部符号的应用二进制镜像 *.bin ，而是将应用的 ELF 执行文件直接链接进来；
    // - 其次，在链接每个 ELF 执行文件之前我们都加入一行 .align 3 来确保它们对齐到 8 字节，这是由于如果不这样做，
    //  xmas-elf crate 可能会在解析 ELF 的时候进行不对齐的内存读写，
    //  例如使用 ld 指令从内存的一个没有对齐到 8 字节的地址加载一个 64 位的值到一个通用寄存器。
    //  而在 k210 平台上，由于其硬件限制，这种情况会触发一个内存读写不对齐的异常，导致解析无法正常完成。
    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(
            file,
            r#"
    .section .data
    .global _app_{0}_start
    .global _app_{0}_end
    .align 3
_app_{0}_start:
    .incbin "{2}{1}.elf"
_app_{0}_end:"#,
            idx, app, TARGET_PATH
        )?;
    }
    Ok(())
}
