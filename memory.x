/* Copyright 2026 Julian Yap (@jyap808) */
/* SPDX-License-Identifier: GPL-2.0-or-later */

/* STM32G0B1KB: 128 KiB flash, 144 KiB SRAM.
 *
 * The last 12 KiB of flash are reserved for RMK's keymap/config storage.
 * RMK stores in the last `num_sectors` sectors (`num_sectors = 6` in
 * `src/main.rs`, 6 x 2 KiB), so keeping the firmware below 116 KiB leaves
 * them untouched. A 5x12x8 keymap is 480 entries; with the two default
 * sectors the storage is far too small for it.
 */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 128K - 12K
    RAM   : ORIGIN = 0x20000000, LENGTH = 144K
}
