qemu-system-x86_64 -drive file=bootable_iso.iso,media=disk,if=virtio,format=raw,snapshot=on \
    -m 512 \
    -netdev user,id=net0 \
    -device virtio-net,netdev=net0