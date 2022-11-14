
    .align 3
    .section .data
    .global _num_apps

_num_apps:
    .quad 75
    .quad _app_ch2b_bad_address_start
    .quad _app_ch2b_bad_instructions_start
    .quad _app_ch2b_bad_register_start
    .quad _app_ch2b_hello_world_start
    .quad _app_ch2b_power_3_start
    .quad _app_ch2b_power_5_start
    .quad _app_ch2b_power_7_start
    .quad _app_ch3_taskinfo_start
    .quad _app_ch3b_sleep_start
    .quad _app_ch3b_sleep1_start
    .quad _app_ch3b_yield0_start
    .quad _app_ch3b_yield1_start
    .quad _app_ch3b_yield2_start
    .quad _app_ch4_mmap0_start
    .quad _app_ch4_mmap1_start
    .quad _app_ch4_mmap2_start
    .quad _app_ch4_mmap3_start
    .quad _app_ch4_unmap_start
    .quad _app_ch4_unmap2_start
    .quad _app_ch5_exit0_start
    .quad _app_ch5_exit1_start
    .quad _app_ch5_getpid_start
    .quad _app_ch5_setprio_start
    .quad _app_ch5_spawn0_start
    .quad _app_ch5_spawn1_start
    .quad _app_ch5_stride_start
    .quad _app_ch5_stride0_start
    .quad _app_ch5_stride1_start
    .quad _app_ch5_stride2_start
    .quad _app_ch5_stride3_start
    .quad _app_ch5_stride4_start
    .quad _app_ch5_stride5_start
    .quad _app_ch5_usertest_start
    .quad _app_ch5b_exit_start
    .quad _app_ch5b_forktest_start
    .quad _app_ch5b_forktest2_start
    .quad _app_ch5b_forktest_simple_start
    .quad _app_ch5b_forktree_start
    .quad _app_ch5b_initproc_start
    .quad _app_ch5b_user_shell_start
    .quad _app_ch6_file0_start
    .quad _app_ch6_file1_start
    .quad _app_ch6_file2_start
    .quad _app_ch6_file3_start
    .quad _app_ch6_usertest_start
    .quad _app_ch6b_cat_start
    .quad _app_ch6b_filetest_simple_start
    .quad _app_ch6b_initproc_start
    .quad _app_ch6b_user_shell_start
    .quad _app_ch7_usertest_start
    .quad _app_ch7b_cat_start
    .quad _app_ch7b_initproc_start
    .quad _app_ch7b_pipe_large_test_start
    .quad _app_ch7b_pipetest_start
    .quad _app_ch7b_run_pipe_test_start
    .quad _app_ch7b_user_shell_start
    .quad _app_ch7b_yield_start
    .quad _app_ch8_deadlock_mutex1_start
    .quad _app_ch8_deadlock_sem1_start
    .quad _app_ch8_deadlock_sem2_start
    .quad _app_ch8_usertest_start
    .quad _app_ch8b_initproc_start
    .quad _app_ch8b_mpsc_sem_start
    .quad _app_ch8b_phil_din_mutex_start
    .quad _app_ch8b_race_adder_start
    .quad _app_ch8b_race_adder_atomic_start
    .quad _app_ch8b_race_adder_loop_start
    .quad _app_ch8b_race_adder_mutex_spin_start
    .quad _app_ch8b_stackful_coroutine_start
    .quad _app_ch8b_stackless_coroutine_start
    .quad _app_ch8b_sync_sem_start
    .quad _app_ch8b_test_condvar_start
    .quad _app_ch8b_threads_start
    .quad _app_ch8b_threads_arg_start
    .quad _app_ch8b_user_shell_start
    .quad _app_ch8b_user_shell_end

    .section .data
    .global _app_ch2b_bad_address_start
    .global _app_ch2b_bad_address_end
_app_ch2b_bad_address_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch2b_bad_address.bin"
_app_ch2b_bad_address_end:

    .section .data
    .global _app_ch2b_bad_instructions_start
    .global _app_ch2b_bad_instructions_end
_app_ch2b_bad_instructions_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch2b_bad_instructions.bin"
_app_ch2b_bad_instructions_end:

    .section .data
    .global _app_ch2b_bad_register_start
    .global _app_ch2b_bad_register_end
_app_ch2b_bad_register_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch2b_bad_register.bin"
_app_ch2b_bad_register_end:

    .section .data
    .global _app_ch2b_hello_world_start
    .global _app_ch2b_hello_world_end
_app_ch2b_hello_world_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch2b_hello_world.bin"
_app_ch2b_hello_world_end:

    .section .data
    .global _app_ch2b_power_3_start
    .global _app_ch2b_power_3_end
_app_ch2b_power_3_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch2b_power_3.bin"
_app_ch2b_power_3_end:

    .section .data
    .global _app_ch2b_power_5_start
    .global _app_ch2b_power_5_end
_app_ch2b_power_5_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch2b_power_5.bin"
_app_ch2b_power_5_end:

    .section .data
    .global _app_ch2b_power_7_start
    .global _app_ch2b_power_7_end
_app_ch2b_power_7_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch2b_power_7.bin"
_app_ch2b_power_7_end:

    .section .data
    .global _app_ch3_taskinfo_start
    .global _app_ch3_taskinfo_end
_app_ch3_taskinfo_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch3_taskinfo.bin"
_app_ch3_taskinfo_end:

    .section .data
    .global _app_ch3b_sleep_start
    .global _app_ch3b_sleep_end
_app_ch3b_sleep_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch3b_sleep.bin"
_app_ch3b_sleep_end:

    .section .data
    .global _app_ch3b_sleep1_start
    .global _app_ch3b_sleep1_end
_app_ch3b_sleep1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch3b_sleep1.bin"
_app_ch3b_sleep1_end:

    .section .data
    .global _app_ch3b_yield0_start
    .global _app_ch3b_yield0_end
_app_ch3b_yield0_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch3b_yield0.bin"
_app_ch3b_yield0_end:

    .section .data
    .global _app_ch3b_yield1_start
    .global _app_ch3b_yield1_end
_app_ch3b_yield1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch3b_yield1.bin"
_app_ch3b_yield1_end:

    .section .data
    .global _app_ch3b_yield2_start
    .global _app_ch3b_yield2_end
_app_ch3b_yield2_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch3b_yield2.bin"
_app_ch3b_yield2_end:

    .section .data
    .global _app_ch4_mmap0_start
    .global _app_ch4_mmap0_end
_app_ch4_mmap0_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch4_mmap0.bin"
_app_ch4_mmap0_end:

    .section .data
    .global _app_ch4_mmap1_start
    .global _app_ch4_mmap1_end
_app_ch4_mmap1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch4_mmap1.bin"
_app_ch4_mmap1_end:

    .section .data
    .global _app_ch4_mmap2_start
    .global _app_ch4_mmap2_end
_app_ch4_mmap2_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch4_mmap2.bin"
_app_ch4_mmap2_end:

    .section .data
    .global _app_ch4_mmap3_start
    .global _app_ch4_mmap3_end
_app_ch4_mmap3_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch4_mmap3.bin"
_app_ch4_mmap3_end:

    .section .data
    .global _app_ch4_unmap_start
    .global _app_ch4_unmap_end
_app_ch4_unmap_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch4_unmap.bin"
_app_ch4_unmap_end:

    .section .data
    .global _app_ch4_unmap2_start
    .global _app_ch4_unmap2_end
_app_ch4_unmap2_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch4_unmap2.bin"
_app_ch4_unmap2_end:

    .section .data
    .global _app_ch5_exit0_start
    .global _app_ch5_exit0_end
_app_ch5_exit0_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_exit0.bin"
_app_ch5_exit0_end:

    .section .data
    .global _app_ch5_exit1_start
    .global _app_ch5_exit1_end
_app_ch5_exit1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_exit1.bin"
_app_ch5_exit1_end:

    .section .data
    .global _app_ch5_getpid_start
    .global _app_ch5_getpid_end
_app_ch5_getpid_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_getpid.bin"
_app_ch5_getpid_end:

    .section .data
    .global _app_ch5_setprio_start
    .global _app_ch5_setprio_end
_app_ch5_setprio_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_setprio.bin"
_app_ch5_setprio_end:

    .section .data
    .global _app_ch5_spawn0_start
    .global _app_ch5_spawn0_end
_app_ch5_spawn0_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_spawn0.bin"
_app_ch5_spawn0_end:

    .section .data
    .global _app_ch5_spawn1_start
    .global _app_ch5_spawn1_end
_app_ch5_spawn1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_spawn1.bin"
_app_ch5_spawn1_end:

    .section .data
    .global _app_ch5_stride_start
    .global _app_ch5_stride_end
_app_ch5_stride_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_stride.bin"
_app_ch5_stride_end:

    .section .data
    .global _app_ch5_stride0_start
    .global _app_ch5_stride0_end
_app_ch5_stride0_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_stride0.bin"
_app_ch5_stride0_end:

    .section .data
    .global _app_ch5_stride1_start
    .global _app_ch5_stride1_end
_app_ch5_stride1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_stride1.bin"
_app_ch5_stride1_end:

    .section .data
    .global _app_ch5_stride2_start
    .global _app_ch5_stride2_end
_app_ch5_stride2_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_stride2.bin"
_app_ch5_stride2_end:

    .section .data
    .global _app_ch5_stride3_start
    .global _app_ch5_stride3_end
_app_ch5_stride3_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_stride3.bin"
_app_ch5_stride3_end:

    .section .data
    .global _app_ch5_stride4_start
    .global _app_ch5_stride4_end
_app_ch5_stride4_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_stride4.bin"
_app_ch5_stride4_end:

    .section .data
    .global _app_ch5_stride5_start
    .global _app_ch5_stride5_end
_app_ch5_stride5_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_stride5.bin"
_app_ch5_stride5_end:

    .section .data
    .global _app_ch5_usertest_start
    .global _app_ch5_usertest_end
_app_ch5_usertest_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5_usertest.bin"
_app_ch5_usertest_end:

    .section .data
    .global _app_ch5b_exit_start
    .global _app_ch5b_exit_end
_app_ch5b_exit_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5b_exit.bin"
_app_ch5b_exit_end:

    .section .data
    .global _app_ch5b_forktest_start
    .global _app_ch5b_forktest_end
_app_ch5b_forktest_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5b_forktest.bin"
_app_ch5b_forktest_end:

    .section .data
    .global _app_ch5b_forktest2_start
    .global _app_ch5b_forktest2_end
_app_ch5b_forktest2_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5b_forktest2.bin"
_app_ch5b_forktest2_end:

    .section .data
    .global _app_ch5b_forktest_simple_start
    .global _app_ch5b_forktest_simple_end
_app_ch5b_forktest_simple_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5b_forktest_simple.bin"
_app_ch5b_forktest_simple_end:

    .section .data
    .global _app_ch5b_forktree_start
    .global _app_ch5b_forktree_end
_app_ch5b_forktree_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5b_forktree.bin"
_app_ch5b_forktree_end:

    .section .data
    .global _app_ch5b_initproc_start
    .global _app_ch5b_initproc_end
_app_ch5b_initproc_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5b_initproc.bin"
_app_ch5b_initproc_end:

    .section .data
    .global _app_ch5b_user_shell_start
    .global _app_ch5b_user_shell_end
_app_ch5b_user_shell_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch5b_user_shell.bin"
_app_ch5b_user_shell_end:

    .section .data
    .global _app_ch6_file0_start
    .global _app_ch6_file0_end
_app_ch6_file0_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch6_file0.bin"
_app_ch6_file0_end:

    .section .data
    .global _app_ch6_file1_start
    .global _app_ch6_file1_end
_app_ch6_file1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch6_file1.bin"
_app_ch6_file1_end:

    .section .data
    .global _app_ch6_file2_start
    .global _app_ch6_file2_end
_app_ch6_file2_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch6_file2.bin"
_app_ch6_file2_end:

    .section .data
    .global _app_ch6_file3_start
    .global _app_ch6_file3_end
_app_ch6_file3_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch6_file3.bin"
_app_ch6_file3_end:

    .section .data
    .global _app_ch6_usertest_start
    .global _app_ch6_usertest_end
_app_ch6_usertest_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch6_usertest.bin"
_app_ch6_usertest_end:

    .section .data
    .global _app_ch6b_cat_start
    .global _app_ch6b_cat_end
_app_ch6b_cat_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch6b_cat.bin"
_app_ch6b_cat_end:

    .section .data
    .global _app_ch6b_filetest_simple_start
    .global _app_ch6b_filetest_simple_end
_app_ch6b_filetest_simple_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch6b_filetest_simple.bin"
_app_ch6b_filetest_simple_end:

    .section .data
    .global _app_ch6b_initproc_start
    .global _app_ch6b_initproc_end
_app_ch6b_initproc_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch6b_initproc.bin"
_app_ch6b_initproc_end:

    .section .data
    .global _app_ch6b_user_shell_start
    .global _app_ch6b_user_shell_end
_app_ch6b_user_shell_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch6b_user_shell.bin"
_app_ch6b_user_shell_end:

    .section .data
    .global _app_ch7_usertest_start
    .global _app_ch7_usertest_end
_app_ch7_usertest_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch7_usertest.bin"
_app_ch7_usertest_end:

    .section .data
    .global _app_ch7b_cat_start
    .global _app_ch7b_cat_end
_app_ch7b_cat_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch7b_cat.bin"
_app_ch7b_cat_end:

    .section .data
    .global _app_ch7b_initproc_start
    .global _app_ch7b_initproc_end
_app_ch7b_initproc_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch7b_initproc.bin"
_app_ch7b_initproc_end:

    .section .data
    .global _app_ch7b_pipe_large_test_start
    .global _app_ch7b_pipe_large_test_end
_app_ch7b_pipe_large_test_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch7b_pipe_large_test.bin"
_app_ch7b_pipe_large_test_end:

    .section .data
    .global _app_ch7b_pipetest_start
    .global _app_ch7b_pipetest_end
_app_ch7b_pipetest_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch7b_pipetest.bin"
_app_ch7b_pipetest_end:

    .section .data
    .global _app_ch7b_run_pipe_test_start
    .global _app_ch7b_run_pipe_test_end
_app_ch7b_run_pipe_test_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch7b_run_pipe_test.bin"
_app_ch7b_run_pipe_test_end:

    .section .data
    .global _app_ch7b_user_shell_start
    .global _app_ch7b_user_shell_end
_app_ch7b_user_shell_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch7b_user_shell.bin"
_app_ch7b_user_shell_end:

    .section .data
    .global _app_ch7b_yield_start
    .global _app_ch7b_yield_end
_app_ch7b_yield_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch7b_yield.bin"
_app_ch7b_yield_end:

    .section .data
    .global _app_ch8_deadlock_mutex1_start
    .global _app_ch8_deadlock_mutex1_end
_app_ch8_deadlock_mutex1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8_deadlock_mutex1.bin"
_app_ch8_deadlock_mutex1_end:

    .section .data
    .global _app_ch8_deadlock_sem1_start
    .global _app_ch8_deadlock_sem1_end
_app_ch8_deadlock_sem1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8_deadlock_sem1.bin"
_app_ch8_deadlock_sem1_end:

    .section .data
    .global _app_ch8_deadlock_sem2_start
    .global _app_ch8_deadlock_sem2_end
_app_ch8_deadlock_sem2_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8_deadlock_sem2.bin"
_app_ch8_deadlock_sem2_end:

    .section .data
    .global _app_ch8_usertest_start
    .global _app_ch8_usertest_end
_app_ch8_usertest_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8_usertest.bin"
_app_ch8_usertest_end:

    .section .data
    .global _app_ch8b_initproc_start
    .global _app_ch8b_initproc_end
_app_ch8b_initproc_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_initproc.bin"
_app_ch8b_initproc_end:

    .section .data
    .global _app_ch8b_mpsc_sem_start
    .global _app_ch8b_mpsc_sem_end
_app_ch8b_mpsc_sem_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_mpsc_sem.bin"
_app_ch8b_mpsc_sem_end:

    .section .data
    .global _app_ch8b_phil_din_mutex_start
    .global _app_ch8b_phil_din_mutex_end
_app_ch8b_phil_din_mutex_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_phil_din_mutex.bin"
_app_ch8b_phil_din_mutex_end:

    .section .data
    .global _app_ch8b_race_adder_start
    .global _app_ch8b_race_adder_end
_app_ch8b_race_adder_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_race_adder.bin"
_app_ch8b_race_adder_end:

    .section .data
    .global _app_ch8b_race_adder_atomic_start
    .global _app_ch8b_race_adder_atomic_end
_app_ch8b_race_adder_atomic_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_race_adder_atomic.bin"
_app_ch8b_race_adder_atomic_end:

    .section .data
    .global _app_ch8b_race_adder_loop_start
    .global _app_ch8b_race_adder_loop_end
_app_ch8b_race_adder_loop_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_race_adder_loop.bin"
_app_ch8b_race_adder_loop_end:

    .section .data
    .global _app_ch8b_race_adder_mutex_spin_start
    .global _app_ch8b_race_adder_mutex_spin_end
_app_ch8b_race_adder_mutex_spin_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_race_adder_mutex_spin.bin"
_app_ch8b_race_adder_mutex_spin_end:

    .section .data
    .global _app_ch8b_stackful_coroutine_start
    .global _app_ch8b_stackful_coroutine_end
_app_ch8b_stackful_coroutine_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_stackful_coroutine.bin"
_app_ch8b_stackful_coroutine_end:

    .section .data
    .global _app_ch8b_stackless_coroutine_start
    .global _app_ch8b_stackless_coroutine_end
_app_ch8b_stackless_coroutine_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_stackless_coroutine.bin"
_app_ch8b_stackless_coroutine_end:

    .section .data
    .global _app_ch8b_sync_sem_start
    .global _app_ch8b_sync_sem_end
_app_ch8b_sync_sem_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_sync_sem.bin"
_app_ch8b_sync_sem_end:

    .section .data
    .global _app_ch8b_test_condvar_start
    .global _app_ch8b_test_condvar_end
_app_ch8b_test_condvar_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_test_condvar.bin"
_app_ch8b_test_condvar_end:

    .section .data
    .global _app_ch8b_threads_start
    .global _app_ch8b_threads_end
_app_ch8b_threads_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_threads.bin"
_app_ch8b_threads_end:

    .section .data
    .global _app_ch8b_threads_arg_start
    .global _app_ch8b_threads_arg_end
_app_ch8b_threads_arg_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_threads_arg.bin"
_app_ch8b_threads_arg_end:

    .section .data
    .global _app_ch8b_user_shell_start
    .global _app_ch8b_user_shell_end
_app_ch8b_user_shell_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/ch8b_user_shell.bin"
_app_ch8b_user_shell_end:
