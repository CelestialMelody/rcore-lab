
    .align 3
    .section .data
    .global _num_apps
_num_apps:
    .quad 5

    .quad _app_0_start

    .quad _app_1_start

    .quad _app_2_start

    .quad _app_3_start

    .quad _app_4_start

    .quad _app_4_end

    .section .data
    .global _app_0_start
    .global _app_0_end
    .align 3
_app_0_start:
    .incbin "../user/build/elf/ch4b_sleep.elf"
_app_0_end:

    .section .data
    .global _app_1_start
    .global _app_1_end
    .align 3
_app_1_start:
    .incbin "../user/build/elf/ch4b_sleep1.elf"
_app_1_end:

    .section .data
    .global _app_2_start
    .global _app_2_end
    .align 3
_app_2_start:
    .incbin "../user/build/elf/ch4b_yield0.elf"
_app_2_end:

    .section .data
    .global _app_3_start
    .global _app_3_end
    .align 3
_app_3_start:
    .incbin "../user/build/elf/ch4b_yield1.elf"
_app_3_end:

    .section .data
    .global _app_4_start
    .global _app_4_end
    .align 3
_app_4_start:
    .incbin "../user/build/elf/ch4b_yield2.elf"
_app_4_end:
