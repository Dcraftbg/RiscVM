#!/bin/sh
set -xe
# clang -target riscv32 -c sum.S -o sum.o
riscv64-unknown-elf-as -march=rv32if sum.S -o sum.o
riscv64-unknown-elf-objcopy -O binary sum.o sum.bin
