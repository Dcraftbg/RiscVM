#!/bin/sh
set -xe
# clang -target riscv32 -c sum.S -o sum.o
# riscv64-unknown-elf-as -march=rv32if sum.S -o sum.o
riscv64-unknown-elf-gcc -march=rv32im -mabi=ilp32 main.c -o main.o -ffreestanding -c
riscv64-unknown-elf-objcopy -O binary main.o main.bin
