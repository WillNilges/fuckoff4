FROM ubuntu:focal

RUN apt-get update; DEBIAN_FRONTEND=noninteractive apt-get -y install libgcrypt20-dev build-essential git python3 python3-venv ninja-build libglib2.0-dev libfdt-dev libpixman-1-dev zlib1g-dev

RUN git clone https://github.com/espressif/qemu.git /usr/src/qemu
WORKDIR /usr/src/qemu

RUN alias python=python3; \
    ./configure --target-list=xtensa-softmmu \
    --enable-gcrypt \
    --enable-debug --enable-sanitizers \
    --disable-strip --disable-user \
    --disable-capstone --disable-vnc \
    --disable-sdl --disable-gtk; \
    ninja -C build;
    
# Pass -v <path_to_image_dir>:/target/ to the image
# Example:
# $ podman run --rm -it -v ~/Code/fuckoff4/esp/target/:/target/ esp-qemu:latest
CMD /usr/src/qemu/build/qemu-system-xtensa -nographic -machine esp32 -drive file=/target/qemu-image,if=mtd,format=raw
