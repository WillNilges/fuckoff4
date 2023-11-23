#!/bin/bash

podman run --rm -it -v ./target/:/target/ esp-qemu:latest

