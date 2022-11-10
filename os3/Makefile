# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release

CHAPTER ?= 3
TEST ?= $(CHAPTER)
# base is 0 for normal tests, 1 for basic tests, and 2 for both
BASE ?= 1

# fix: should not add annotation after variable
KERNEL_ELF := target/$(TARGET)/$(MODE)/os$(CHAPTER)
KERNEL_BIN := $(KERNEL_ELF).bin
KERNEL_ASM := $(KERNEL_ELF).asm 

# BOARD
BOARD ?= qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64
RV64_OBJDUMP := riscv64-unknown-elf-objdump


build: env $(KERNEL_BIN)

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

disasm:
	@$(OBJDUMP) $(KERNEL_ELF) -S > $(KERNEL_ELF).asm

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils --vers ~0.3
	rustup component add rust-src
	rustup component add llvm-tools-preview

kernel:
	@make -C ../user build TEST=$(TEST) CHAPTER=$(CHAPTER) BASE=$(BASE)
	@cargo build --release

clean:
	@cargo clean

run: build
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

debug: build
	@cp ../.gdbinit .
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

dbg: build
	qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -s -S

dump: run
	@$(RV64_OBJDUMP) $(KERNEL_ELF) -d > os.dump.s
.PHONY: build env kernel clean run-inner
