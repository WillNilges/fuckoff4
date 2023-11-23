#!/bin/bash

cargo build
espflash save-image --merge --chip esp32 target/xtensa-esp32-espidf/debug/fuckoff4 target/qemu-image

