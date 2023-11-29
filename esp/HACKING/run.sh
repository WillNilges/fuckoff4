#!/bin/bash

podman run --rm -it -v ./target/:/target/ --name esp-qemu esp-qemu:latest

