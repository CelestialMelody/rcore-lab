
    .align 3
    .section .data
    .global _num_apps

_num_apps:
    .quad 5
    .quad _app_00_hello_world_start
    .quad _app_01_bad_address_start
    .quad _app_02_bad_register_start
    .quad _app_03_bad_instructions_start
    .quad _app_04_power_3_start
    .quad _app_04_power_3_end

    .section .data
    .global _app_00_hello_world_start
    .global _app_00_hello_world_end
_app_00_hello_world_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/00_hello_world.bin"
_app_00_hello_world_end:

    .section .data
    .global _app_01_bad_address_start
    .global _app_01_bad_address_end
_app_01_bad_address_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/01_bad_address.bin"
_app_01_bad_address_end:

    .section .data
    .global _app_02_bad_register_start
    .global _app_02_bad_register_end
_app_02_bad_register_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/02_bad_register.bin"
_app_02_bad_register_end:

    .section .data
    .global _app_03_bad_instructions_start
    .global _app_03_bad_instructions_end
_app_03_bad_instructions_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/03_bad_instructions.bin"
_app_03_bad_instructions_end:

    .section .data
    .global _app_04_power_3_start
    .global _app_04_power_3_end
_app_04_power_3_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/04_power_3.bin"
_app_04_power_3_end:
