
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
_app_0_start:
    .incbin "../user/build/bin/ch4b_sleep.bin"
_app_0_end:

    .section .data
    .global _app_1_start
    .global _app_1_end
_app_1_start:
    .incbin "../user/build/bin/ch4b_sleep1.bin"
_app_1_end:

    .section .data
    .global _app_2_start
    .global _app_2_end
_app_2_start:
    .incbin "../user/build/bin/ch4b_yield0.bin"
_app_2_end:

    .section .data
    .global _app_3_start
    .global _app_3_end
_app_3_start:
    .incbin "../user/build/bin/ch4b_yield1.bin"
_app_3_end:

    .section .data
    .global _app_4_start
    .global _app_4_end
_app_4_start:
    .incbin "../user/build/bin/ch4b_yield2.bin"
_app_4_end:
