#!/usr/bin/env sh
# Copyright 2026 Julian Yap (@jyap808)
# SPDX-License-Identifier: GPL-2.0-or-later
#
# Convert the linked ELF to a raw binary and flash it over STM32 DFU.
#
# `dfu-util -D` treats anything that is not a DfuSe/HEX file as a raw
# binary, so it must never be given the ELF directly.
set -eu

elf=${1:?usage: flash.sh <path-to-elf>}
bin=${elf}.bin

arm-none-eabi-objcopy -O binary "$elf" "$bin"

exec dfu-util -a 0 -s 0x08000000:leave -D "$bin"
