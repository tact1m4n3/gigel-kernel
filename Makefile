target ?= x86_64-unknown-kernel
profile ?= debug

cargo_target = targets/$(target).json
ifeq ($(profile), debug)
cargo_profile = dev
else
cargo_profile = $(profile)
endif
kernel = target/$(target)/$(profile)/kernel

.PHONY: all kernel run clean

all: image.iso

kernel:
	cargo build --target $(cargo_target) --profile $(cargo_profile)

image.iso: kernel
	mkdir -p sysroot/boot
	cp -rf util/grub sysroot/boot
	cp -rf $(kernel) sysroot/boot/gigel-kernel
	grub-mkrescue -o $@ sysroot

run: image.iso
	qemu-system-x86_64 -smp 4 -m 512M -enable-kvm -cdrom $^ -no-reboot -serial stdio

clean:
	cargo clean
	rm -rf image.iso sysroot
