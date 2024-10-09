#!/bin/sh
set -xe
riscv64-unknown-elf-objdump -b binary -m riscv:rv32 -M numeric -M no-aliases -D main.bin
