# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
# fix: should not add annotation after variable
KERNEL_ELF := target/$(TARGET)/$(MODE)/os2
KERNEL_BIN := $(KERNEL_ELF).bin
# os2 add
KERNEL_ASM := $(KERNEL_ELF).asm 

# BOARD
BOARD ?= qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=r
OBJCOPY := rust-objcopy --binary-architecture=riscv64
RV64_OBJDUMP := riscv64-unknown-elf-objdump

# os2: add
CHAPTER ?= $(shell git rev-parse --abbrev-ref HEAD | grep -oP 'ch\K[0-9]')
TEST ?= $(CHAPTER)
BASE ?= 1

build: env $(KERNEL_BIN)

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

# os2: add
disasm:
	@$(OBJDUMP) $(KERNEL_ELF) -S > $(KERNEL_ELF).asm

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils --vers ~0.3
	rustup component add rust-src
	rustup component add llvm-tools-preview

# os2: add make -C
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
	@$(RV64_OBJDUMP) $(KERNEL_ELF) -d > os2.dump.s
.PHONY: build env kernel clean run-inner
