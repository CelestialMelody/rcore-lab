// see [`vedio`](https://www.bilibili.com/video/BV1kZ4y167gf/?spm_id_from=333.788&vd_source=fff8a96619bd3da6d1cb5d5c1ede2cf1)
// see [`link`](https://course.rs/cargo/reference/build-script/intro.html)
use std::io::{Result, Write};
use std::fs::{File, read_dir};

static TARGET_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";

fn main() {
    println!("cargo:rerun-if-changed=../user/src/"); // 重新编译
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    insert_app_data().unwrap();
    
}

#[allow(unused)]
fn insert_app_data() -> Result<()> {
    let mut f = File::create("src/link_app.S").unwrap();
    // Vec<_> `_`是类型占位符, Rust 编译器推断什么类型进入Vec
    let mut apps: Vec<_> = read_dir("../user/src/bin")
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            // name_with_ext: file name with extension
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap(); // 获取文件名
            // drain: 从字符串中移除指定范围的字符并返回它们
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len()); // 删除后缀
            name_with_ext // 返回文件名
        })
        .collect();
    apps.sort(); // 排序

    // .align 3: 8字节对齐
    // .section .data: 数据段
    // .global _num_apps: 全局变量, app数量
    // .quad: 8字节, 32bit的值
    // .incbin: 将二进制文件插入到目标文件中

    writeln!{f, r#"
    .align 3
    .section .data
    .global _num_apps

_num_apps:
    .quad {}"#, apps.len()}?; // ?: 如果有错误, 返回错误; -> see src/batch.rs: lazy_static!{...} app_start_raw

    for i in 0..apps.len() {
        writeln!(f, r#"    .quad _app_{}_start"#, apps[i])?;
    }
    writeln!(f, r#"    .quad _app_{}_end"#, apps[apps.len() - 1])?;

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(f, r#"
    .section .data
    .global _app_{0}_start
    .global _app_{0}_end
_app_{0}_start:
    .incbin "{1}{0}.bin"
_app_{0}_end:"#, app, TARGET_PATH)?;
    }
    Ok(())
}