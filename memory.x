/* Copyright 2026 Julian Yap (@jyap808) */
/* SPDX-License-Identifier: GPL-2.0-or-later */

/* STM32G0B1KB: 128 KiB flash, 144 KiB SRAM.
 *
 * The last 8 KiB of flash are reserved for RMK's keymap/config storage.
 * RMK stores in the last `num_sectors` sectors (2 x 2 KiB by default), so
 * keeping the firmware below 120 KiB leaves them untouched.
 */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 128K - 8K
    RAM   : ORIGIN = 0x20000000, LENGTH = 144K
}
