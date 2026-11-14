// Copyright 2026 Julian Yap (@jyap808)
// SPDX-License-Identifier: GPL-2.0-or-later

//! Vial identity.
//!
//! `VIAL_KEYBOARD_ID` is the stable UID from the original Pando Vial
//! firmware, so Vial recognises this firmware as the same keyboard.
//! `VIAL_KEYBOARD_DEF` is `vial.json` compressed with xz (the format Vial
//! expects when it requests the keyboard definition).

/// Stable Vial UID for the Pando.
pub const VIAL_KEYBOARD_ID: [u8; 8] = [0x25, 0x16, 0x6B, 0xA8, 0x6C, 0x35, 0x8E, 0xD9];

/// XZ-compressed `vial.json`.
pub const VIAL_KEYBOARD_DEF: &[u8] = include_bytes!("../vial.json.xz");

/// Unlock combo `(row, col)`: the top-left two keys, Esc and `1`.
/// Matches QMK's `VIAL_UNLOCK_COMBO_ROWS { 0, 0 }` / `COLS { 0, 1 }`.
pub const UNLOCK_KEYS: [(u8, u8); 2] = [(0, 0), (0, 1)];
