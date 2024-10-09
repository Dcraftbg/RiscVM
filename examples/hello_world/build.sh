#!/bin/sh
set -xe
riscv64-unknown-elf-gcc -march=rv32i -mabi=ilp32 main.c -o main.o -ffreestanding -fno-builtin -static -c
riscv64-unknown-elf-ld -march=rv32i -melf32lriscv main.o -T linker.ld -o main 
riscv64-unknown-elf-objcopy -O binary main main.bin
