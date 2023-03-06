# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
# MODE := debug
NAME := os4

CHAPTER ?= 4
TEST ?= $(CHAPTER)
# base is 0 for normal tests, 1 for basic tests, and 2 for both
BASE ?= 1

# NOTE: Makefile 变量后面注意不要有空格
KERNEL_ELF := target/$(TARGET)/$(MODE)/$(NAME)
KERNEL_BIN := $(KERNEL_ELF).bin
KERNEL_ASM := $(KERNEL_ELF).asm

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80200000

# BOARD
BOARD ?= qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64
RV64_OBJDUMP := riscv64-unknown-elf-objdump

build: $(KERNEL_BIN)

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

# disasm:
# 	@$(OBJDUMP) $(KERNEL_ELF) -S > $(KERNEL_ELF).asm

# debug: cargo build
# release: cargo build --release
kernel:
	@make -C ../user build TEST=$(TEST) CHAPTER=$(CHAPTER) BASE=$(BASE)
	@cargo build --release

LINK_APP_S := src/link_app.S

clean:
	@cargo clean
	@cat /dev/null > $(LINK_APP_S)

run: build
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

debug: build
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

dump: run
	@$(RV64_OBJDUMP) $(KERNEL_ELF) -d > os.dump.s

.PHONY: build env kernel clean run-inner
