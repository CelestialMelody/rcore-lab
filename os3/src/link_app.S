
    .align 3
    .section .data
    .global _num_apps
_num_apps:
    .quad 10

    .quad _app_0_start

    .quad _app_1_start

    .quad _app_2_start

    .quad _app_3_start

    .quad _app_4_start

    .quad _app_5_start

    .quad _app_6_start

    .quad _app_7_start

    .quad _app_8_start

    .quad _app_9_start

    .quad _app_9_end

    .section .data
    .global _app_0_start
    .global _app_0_end
_app_0_start:
    .incbin "../user/build/bin/ch2b_bad_address.bin"
_app_0_end:

    .section .data
    .global _app_1_start
    .global _app_1_end
_app_1_start:
    .incbin "../user/build/bin/ch2b_bad_instructions.bin"
_app_1_end:

    .section .data
    .global _app_2_start
    .global _app_2_end
_app_2_start:
    .incbin "../user/build/bin/ch2b_bad_register.bin"
_app_2_end:

    .section .data
    .global _app_3_start
    .global _app_3_end
_app_3_start:
    .incbin "../user/build/bin/ch2b_hello_world.bin"
_app_3_end:

    .section .data
    .global _app_4_start
    .global _app_4_end
_app_4_start:
    .incbin "../user/build/bin/ch2b_power_3.bin"
_app_4_end:

    .section .data
    .global _app_5_start
    .global _app_5_end
_app_5_start:
    .incbin "../user/build/bin/ch3b_sleep.bin"
_app_5_end:

    .section .data
    .global _app_6_start
    .global _app_6_end
_app_6_start:
    .incbin "../user/build/bin/ch3b_sleep1.bin"
_app_6_end:

    .section .data
    .global _app_7_start
    .global _app_7_end
_app_7_start:
    .incbin "../user/build/bin/ch3b_yield0.bin"
_app_7_end:

    .section .data
    .global _app_8_start
    .global _app_8_end
_app_8_start:
    .incbin "../user/build/bin/ch3b_yield1.bin"
_app_8_end:

    .section .data
    .global _app_9_start
    .global _app_9_end
_app_9_start:
    .incbin "../user/build/bin/ch3b_yield2.bin"
_app_9_end:
