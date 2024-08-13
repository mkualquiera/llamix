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

# == BUSYBOX STUFF ==

busybox-1.36.1.tar.bz2:
	wget https://busybox.net/downloads/busybox-1.36.1.tar.bz2

busybox-1.36.1: busybox-1.36.1.tar.bz2
	tar xf busybox-1.36.1.tar.bz2

busybox-1.36.1/.config: busybox-1.36.1
	cp busybox-config busybox-1.36.1/.config

# Utils for quality of life

busybox-config: busybox-1.36.1/.config
	cp busybox-1.36.1/.config busybox-config

$busybox-menuconfig:
	make -C busybox-1.36.1 menuconfig
	cp busybox-1.36.1/.config busybox-config

$clean-busybox:
	rm -rf busybox-1.36.1 busybox-1.36.1.tar.bz2

busybox-1.36.1/busybox: busybox-1.36.1/.config busybox-1.36.1
	cd busybox-1.36.1 && make -j8

# == ISO STUFF ==

bootable_iso: busybox-1.36.1/busybox llamix/target/x86_64-unknown-linux-musl/release/llamix linux-6.3.1/arch/x86/boot/bzImage ^workspace
	rm -rf bootable_iso
	mkdir -p bootable_iso/bin
	cp -a workspace/. bootable_iso
	cp linux-6.3.1/arch/x86/boot/bzImage bootable_iso/boot/bzImage
	cp llamix/target/x86_64-unknown-linux-musl/release/llamix bootable_iso/bin/llamix
	cp busybox-1.36.1/busybox bootable_iso/bin/busybox
	mkdir bootable_iso/proc

bootable_iso.iso: bootable_iso
	grub-mkrescue -o bootable_iso.iso bootable_iso

$all: bootable_iso.iso

$run: 
	./run.sh

$clean: $clean-kernel $clean-llamix