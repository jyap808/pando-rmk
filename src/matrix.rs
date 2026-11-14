// Copyright 2026 Julian Yap (@jyap808)
// SPDX-License-Identifier: GPL-2.0-or-later

//! Pando matrix scanning.
//!
//! The left half is wired directly to the MCU (`COL2ROW`, active low):
//!
//! ```text
//! rows (outputs, idle HIGH, driven LOW to scan): PB0 PB1 PB2 PA8 PA9
//! cols (inputs, pull-up):                       PA7 PA6 PA2 PA3 PA4 A5
//! ```
//!
//! The right half is a PCA9555 I2C expander at 7-bit address `0x20`:
//! PORT0 senses the six columns, PORT1 drives the five rows. Its columns
//! are reverse-wired, so PORT0 bit `c` maps to matrix column `11 - c`.
//!
//! A single scanner owns both halves. When the expander is missing (left
//! half used standalone) the I2C bus is only touched by a lightweight
//! retry every two seconds, and any keys still held on the right half are
//! released so they cannot stick on the host.

use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::i2c::{I2c, Master};
use embassy_stm32::mode::Blocking;
use embassy_time::{Duration, Instant, Timer};
use rmk::core_traits::Runnable;
use rmk::debounce::{DebounceState, DebouncerTrait, default_debouncer::DefaultDebouncer};
use rmk::event::{KeyboardEvent, publish_event_async};
use rmk::matrix::KeyState;

use crate::keymap::{COL, ROW};

/// Left-half direct GPIO.
const LEFT_COLS: usize = 6;
const LEFT_ROWS: usize = ROW;

/// First matrix column provided by the right half.
const RIGHT_COL_START: usize = LEFT_COLS;

/// PCA9555 7-bit address (A0=A1=A2=GND).
const PCA9555_ADDR: u8 = 0x20;

const REG_INPUT0: u8 = 0x00;
const REG_OUTPUT1: u8 = 0x03;
const REG_CONFIG0: u8 = 0x06;
const REG_CONFIG1: u8 = 0x07;

const PORT_ALL_HIGH: u8 = 0xFF;

/// Re-probe a missing right half at most this often.
const RIGHT_RETRY_MS: u64 = 2000;

/// Time to let a driven row settle before sampling the columns.
///
/// The time driver ticks at 32.768 kHz, so one tick is ~31 us. A
/// `after_micros` value below one tick would round down to an immediate
/// wakeup, so wait in ticks explicitly.
const ROW_SETTLE_TICKS: u64 = 1;

/// Idle time between full scans.
const SCAN_INTERVAL_US: u64 = 500;

pub struct PandoMatrix {
    left_rows: [Output<'static>; LEFT_ROWS],
    left_cols: [Input<'static>; LEFT_COLS],
    i2c: I2c<'static, Blocking, Master>,
    debouncer: DefaultDebouncer<ROW, COL>,
    /// Indexed `[col][row]`, matching `DefaultDebouncer`'s layout.
    key_states: [[KeyState; ROW]; COL],
    right_connected: bool,
    right_retry_at: Instant,
}

impl PandoMatrix {
    pub fn new(
        left_rows: [Output<'static>; LEFT_ROWS],
        left_cols: [Input<'static>; LEFT_COLS],
        mut i2c: I2c<'static, Blocking, Master>,
    ) -> Self {
        let right_connected = configure_right_half(&mut i2c);

        Self {
            left_rows,
            left_cols,
            i2c,
            debouncer: DefaultDebouncer::new(),
            key_states: [[KeyState::new(); ROW]; COL],
            right_connected,
            right_retry_at: Instant::now(),
        }
    }

    /// Configure the expander and return whether it acknowledged.
    fn init_right_half(&mut self) -> bool {
        let ok = configure_right_half(&mut self.i2c);
        self.right_connected = ok;
        self.right_retry_at = Instant::now();
        ok
    }

    /// Release every key attributed to the right half.
    async fn release_right_half(&mut self) {
        for row in 0..ROW {
            for col in RIGHT_COL_START..COL {
                if self.key_states[col][row].pressed {
                    self.key_states[col][row].pressed = false;
                    publish_event_async(KeyboardEvent::key(row as u8, col as u8, false)).await;
                }
            }
        }
    }

    async fn scan_left_row(&mut self, row: usize) -> u16 {
        self.left_rows[row].set_low();
        Timer::after_ticks(ROW_SETTLE_TICKS).await;

        let mut bits = 0u16;
        for (col, input) in self.left_cols.iter().enumerate() {
            if input.is_low() {
                bits |= 1u16 << col;
            }
        }

        self.left_rows[row].set_high();
        bits
    }

    /// Drive one expander row low and sample all six right-half columns.
    /// Bit `c` of PORT0 maps to matrix column `11 - c`.
    fn scan_right_row(&mut self, row: usize) -> Result<u16, ()> {
        let driven = PORT_ALL_HIGH & !(1u8 << row);
        self.i2c
            .blocking_write(PCA9555_ADDR, &[REG_OUTPUT1, driven])
            .map_err(|_| ())?;

        let mut sample = [0u8; 1];
        let read = self
            .i2c
            .blocking_write_read(PCA9555_ADDR, &[REG_INPUT0], &mut sample);

        // Always park the rows high again, even if the read failed, so a
        // half that is still present is not left with a row selected.
        let _ = self
            .i2c
            .blocking_write(PCA9555_ADDR, &[REG_OUTPUT1, PORT_ALL_HIGH]);

        read.map_err(|_| ())?;

        let mut bits = 0u16;
        for bit in 0..LEFT_COLS {
            if sample[0] & (1u8 << bit) == 0 {
                let col = COL - 1 - bit;
                bits |= 1u16 << col;
            }
        }
        Ok(bits)
    }

    async fn scan_once(&mut self) {
        for row in 0..ROW {
            let left = self.scan_left_row(row).await;

            let right = if self.right_connected {
                match self.scan_right_row(row) {
                    Ok(bits) => bits,
                    Err(()) => {
                        self.right_connected = false;
                        self.release_right_half().await;
                        0
                    }
                }
            } else {
                0
            };

            let active = left | right;
            for col in 0..COL {
                let pressed = active & (1u16 << col) != 0;
                let prev = self.key_states[col][row];
                if let DebounceState::Debounced = self
                    .debouncer
                    .detect_change_with_debounce(row, col, pressed, &prev)
                {
                    self.key_states[col][row].toggle_pressed();
                    let pressed = self.key_states[col][row].pressed;
                    publish_event_async(KeyboardEvent::key(row as u8, col as u8, pressed)).await;
                }
            }
        }
    }
}

/// Program PORT0 as inputs, PORT1 as outputs, and idle the rows high.
fn configure_right_half(i2c: &mut I2c<'static, Blocking, Master>) -> bool {
    let config: [[u8; 2]; 3] = [
        [REG_CONFIG0, 0xFF],
        [REG_CONFIG1, 0x00],
        [REG_OUTPUT1, PORT_ALL_HIGH],
    ];

    for cmd in config {
        if i2c.blocking_write(PCA9555_ADDR, &cmd).is_err() {
            return false;
        }
    }

    // A read is the probe QMK uses: an absent expander NACKs here.
    let mut sample = [0u8; 1];
    i2c.blocking_write_read(PCA9555_ADDR, &[REG_INPUT0], &mut sample)
        .is_ok()
}

impl Runnable for PandoMatrix {
    async fn run(&mut self) -> ! {
        loop {
            if !self.right_connected
                && self.right_retry_at.elapsed() >= Duration::from_millis(RIGHT_RETRY_MS)
                && self.init_right_half()
            {
                self.right_retry_at = Instant::now();
            }

            self.scan_once().await;
            Timer::after_micros(SCAN_INTERVAL_US).await;
        }
    }
}
