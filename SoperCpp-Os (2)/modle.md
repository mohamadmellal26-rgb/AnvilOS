rm Cargo.lock && \
cargo clean && \
nasm -f elf64 src/boot.asm -o build/boot.o && \
cargo build --target x86_64-unknown-none && \
ld -n -T linker.ld -o build/kernel.bin build/boot.o target/x86_64-unknown-none/debug/libkernel.a && \
mkdir -p build/iso/boot/grub && \
cp build/kernel.bin build/iso/boot/kernel.bin && \
grub-mkrescue -o build/os.iso build/iso && \
qemu-system-x86_64 -cdrom build/os.iso -boot d