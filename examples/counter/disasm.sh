#!/bin/sh
set -xe
riscv64-unknown-elf-objdump -b binary -m riscv:rv32 -M numeric -M no-aliases -M no-reg-names -D main.bin
