# == KERNEL STUFF == 

linux-6.3.1.tar.xz:
	wget https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.3.1.tar.xz

linux-6.3.1: linux-6.3.1.tar.xz
	tar xf linux-6.3.1.tar.xz

linux-6.3.1/.config: linux-6.3.1
	cp linux-config linux-6.3.1/.config

# Utils for quality of life
linux-config: linux-6.3.1/.config
	cp linux-6.3.1/.config linux-config

$menuconfig: 
	make -C linux-6.3.1 menuconfig
	cp linux-6.3.1/.config linux-config

$clean-kernel:
	rm -rf linux-6.3.1 linux-6.3.1.tar.xz

linux-6.3.1/arch/x86/boot/bzImage: linux-6.3.1/.config linux-6.3.1
	cd linux-6.3.1 && make -j8

# == LLAMIX STUFF ==

llamix/target/x86_64-unknown-linux-musl/release/llamix: ^llamix/src llamix/Cargo.toml
	cd llamix && cargo build --release --target x86_64-unknown-linux-musl

$clean-llamix:
	rm -rf llamix/target

# == ISO STUFF ==

bootable_iso: llamix/target/x86_64-unknown-linux-musl/release/llamix linux-6.3.1/arch/x86/boot/bzImage ^workspace
	rm -rf bootable_iso
	mkdir -p bootable_iso/bin
	cp -a workspace/. bootable_iso
	cp linux-6.3.1/arch/x86/boot/bzImage bootable_iso/boot/bzImage
	cp llamix/target/x86_64-unknown-linux-musl/release/llamix bootable_iso/bin/llamix

bootable_iso.iso: bootable_iso
	grub-mkrescue -o bootable_iso.iso bootable_iso

$all: bootable_iso.iso

$run: 
	./run.sh

$clean: $clean-kernel $clean-llamix