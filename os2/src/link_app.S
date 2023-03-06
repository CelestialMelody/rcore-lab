
    .align 3
    .section .data
    .global _num_apps

_num_apps:
    .quad 5
    .quad _app_ch4b_sleep_start
    .quad _app_ch4b_sleep1_start
    .quad _app_ch4b_yield0_start
    .quad _app_ch4b_yield1_start
    .quad _app_ch4b_yield2_start
    .quad _app_ch4b_yield2_end

    .section .data
    .global _app_ch4b_sleep_start
    .global _app_ch4b_sleep_end
_app_ch4b_sleep_start:
    .incbin "../user/build/bin/ch4b_sleep.bin"
_app_ch4b_sleep_end:

    .section .data
    .global _app_ch4b_sleep1_start
    .global _app_ch4b_sleep1_end
_app_ch4b_sleep1_start:
    .incbin "../user/build/bin/ch4b_sleep1.bin"
_app_ch4b_sleep1_end:

    .section .data
    .global _app_ch4b_yield0_start
    .global _app_ch4b_yield0_end
_app_ch4b_yield0_start:
    .incbin "../user/build/bin/ch4b_yield0.bin"
_app_ch4b_yield0_end:

    .section .data
    .global _app_ch4b_yield1_start
    .global _app_ch4b_yield1_end
_app_ch4b_yield1_start:
    .incbin "../user/build/bin/ch4b_yield1.bin"
_app_ch4b_yield1_end:

    .section .data
    .global _app_ch4b_yield2_start
    .global _app_ch4b_yield2_end
_app_ch4b_yield2_start:
    .incbin "../user/build/bin/ch4b_yield2.bin"
_app_ch4b_yield2_end:
