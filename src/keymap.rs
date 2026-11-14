// Copyright 2026 Julian Yap (@jyap808)
// SPDX-License-Identifier: GPL-2.0-or-later

//! Pando default keymap.
//!
//! The matrix is 5 rows by 12 columns. Columns 0-5 are the left half
//! (direct GPIO), columns 6-11 are the right half (PCA9555).
//!
//! ```text
//! row 0:  Esc   1    2    3    4    5   |  6    7    8    9    0    -
//! row 1:  Tab   Q    W    E    R    T   |  Y    U    I    O    P    \
//! row 2:  LGui  A    S    D    F    G   |  H    J    K    L    ;    '
//! row 3:  LSft  Z    X    C    V    B   |  N    M    ,    .    /    RSft
//! row 4:   -    Left Rght BSpc LCtl LAlt| Ent  Spc  LOWR Up   Dn    -
//! ```
//!
//! Row 4 columns 0 and 11 have no physical keys. `TL_LOWR` is QMK's
//! `QK_TRI_LAYER_LOWER`, which maps to RMK's `Action::TriLayerLower`.

use rmk::config::Hand;
use rmk::types::action::{Action, KeyAction};
use rmk::{k, layer};

pub const ROW: usize = 5;
pub const COL: usize = 12;
pub const NUM_LAYER: usize = 8;

// QMK DYNAMIC_KEYMAP_LAYER_COUNT is 8. Layers 2-7 are transparent.
const TRNS: KeyAction = KeyAction::Transparent;
const NONE: KeyAction = KeyAction::No;

/// Per-cell hand assignment. Left half is columns 0-5, right half 6-11.
/// Unknown on the thumbs keeps same-hand chordal taps from firing across
/// the centre of the board.
#[rustfmt::skip]
pub const HAND_MAP: [[Hand; COL]; ROW] = [
    [Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left,
     Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right],
    [Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left,
     Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right],
    [Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left,
     Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right],
    [Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left,
     Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right],
    [Hand::Unknown,
     Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left,
     Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right,
     Hand::Unknown],
];

/// Default Pando keymap (layers 0 and 1 from the QMK/Vial firmware).
#[rustfmt::skip]
pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    [
        // Layer 0 - base
        layer!([
            [ k!(Escape), k!(Kc1), k!(Kc2),   k!(Kc3),    k!(Kc4),    k!(Kc5),    k!(Kc6), k!(Kc7),   k!(Kc8),    k!(Kc9),     k!(Kc0),    k!(Minus)     ],
            [ k!(Tab),    k!(Q),   k!(W),     k!(E),      k!(R),      k!(T),      k!(Y),   k!(U),     k!(I),      k!(O),       k!(P),      k!(Backslash) ],
            [ k!(LGui),   k!(A),   k!(S),     k!(D),      k!(F),      k!(G),      k!(H),   k!(J),     k!(K),      k!(L),       k!(Semicolon), k!(Quote)  ],
            [ k!(LShift), k!(Z),   k!(X),     k!(C),      k!(V),      k!(B),      k!(N),   k!(M),     k!(Comma),  k!(Dot),     k!(Slash),  k!(RShift)    ],
            [ NONE,       k!(Left), k!(Right), k!(Backspace), k!(LCtrl), k!(LAlt), k!(Enter), k!(Space), KeyAction::Single(Action::TriLayerLower), k!(Up), k!(Down), NONE ]
        ]),
        // Layer 1 - function / navigation
        layer!([
            [ k!(Grave),  k!(F1),  k!(F2),    k!(F3),     k!(F4),     k!(F5),     k!(F6),  k!(F7),    k!(F8),     k!(F9),      k!(F10),    k!(Equal)     ],
            [ k!(CapsLock), TRNS,  TRNS,      TRNS,       TRNS,       TRNS,       k!(KbVolumeDown), k!(KbVolumeUp), k!(KbMute), TRNS, k!(PrintScreen), TRNS ],
            [ TRNS,       TRNS,    TRNS,      TRNS,       TRNS,       TRNS,       TRNS,    TRNS,      TRNS,       TRNS,        TRNS,       TRNS          ],
            [ TRNS,       TRNS,    TRNS,      TRNS,       TRNS,       TRNS,       TRNS,    TRNS,      k!(LeftBracket), k!(RightBracket), TRNS, TRNS          ],
            [ NONE,       k!(Home), k!(End),  k!(Delete), TRNS,       TRNS,       TRNS,    TRNS,      TRNS,       k!(PageUp),  k!(PageDown), NONE          ]
        ]),
        // Layers 2-7 - transparent, available to Vial.
        [[TRNS; COL]; ROW],
        [[TRNS; COL]; ROW],
        [[TRNS; COL]; ROW],
        [[TRNS; COL]; ROW],
        [[TRNS; COL]; ROW],
        [[TRNS; COL]; ROW],
    ]
}
