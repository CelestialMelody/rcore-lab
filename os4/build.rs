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

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(
            file,
            r#"
    .section .data
    .global _app_{0}_start
    .global _app_{0}_end
_app_{0}_start:
    .incbin "{2}{1}.bin"
_app_{0}_end:"#,
            idx, app, TARGET_PATH
        )?;
    }
    Ok(())
}
