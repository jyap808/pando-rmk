// Copyright 2026 Julian Yap (@jyap808)
// SPDX-License-Identifier: GPL-2.0-or-later

#![no_std]
#![no_main]

mod keymap;
mod matrix;
mod vial;

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::bind_interrupts;
use embassy_stm32::flash::Flash;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::pac::{RCC, SYSCFG};
use embassy_stm32::peripherals::USB;
use embassy_stm32::rcc::{
    AHBPrescaler, APBPrescaler, Hsi48Config, Pll, PllMul, PllPreDiv, PllRDiv, PllSource, Sysclk,
};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::{Driver, InterruptHandler};
use embassy_time::{Duration, Timer};
use rmk::config::{
    BehaviorConfig, DeviceConfig, PositionalConfig, RmkConfig, StorageConfig, VialConfig,
};
use rmk::core_traits::Runnable;
use rmk::futures::future::join3;
use rmk::host::HostService;
use rmk::keyboard::Keyboard;
use rmk::storage::async_flash_wrapper;
use rmk::usb::UsbTransport;
use rmk::{KeymapData, initialize_keymap_and_storage, run_all};

use panic_halt as _;

use matrix::PandoMatrix;

bind_interrupts!(struct Irqs {
    USB_UCPD1_2 => InterruptHandler<USB>;
});

/// USB identity of the Pando.
const USB_VID: u16 = 0x4A59;
const USB_PID: u16 = 0x3538;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // 64 MHz SYSCLK from HSI16: 16 MHz / 2 * 16 / 2. USB runs from HSI48,
    // which is the reset selection for the USB kernel clock.
    let mut config = Config::default();
    config.rcc.sys = Sysclk::PLL1_R;
    config.rcc.pll = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV2,
        mul: PllMul::MUL16,
        divp: None,
        divq: None,
        divr: Some(PllRDiv::DIV2),
    });
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.hsi48 = Some(Hsi48Config {
        sync_from_usb: true,
    });

    let mut p = embassy_stm32::init(config);

    // Equivalent of QMK's keyboard_pre_init_kb(): set the UCPD strobe bits
    // so the UCPD peripheral cannot claim its CC pins as alternate
    // functions. PA8, a matrix row here, is one of those pins. SYSCFG is
    // clock-gated, so enable it before touching CFGR1.
    RCC.apbenr2().modify(|w| w.set_syscfgen(true));
    SYSCFG.cfgr1().modify(|w| {
        w.set_ucpd1_strobe(true);
        w.set_ucpd2_strobe(true);
    });

    // Mac silicon workaround, carried over from the QMK firmware: hold D+
    // low for 100 ms so the host re-enumerates even when VBUS never dropped
    // (for example after a soft reset). The USB driver reconfigures the pin
    // as an alternate function when it starts.
    {
        let _dp_low = Output::new(p.PA12.reborrow(), Level::Low, Speed::Low);
        Timer::after_millis(100).await;
    }

    // Let the expander's power rail settle before the first probe.
    Timer::after_millis(5).await;

    // Left half: rows PB0 PB1 PB2 PA8 PA9, columns PA7 PA6 PA2 PA3 PA4 PA5.
    let left_rows = [
        Output::new(p.PB0, Level::High, Speed::Low),
        Output::new(p.PB1, Level::High, Speed::Low),
        Output::new(p.PB2, Level::High, Speed::Low),
        Output::new(p.PA8, Level::High, Speed::Low),
        Output::new(p.PA9, Level::High, Speed::Low),
    ];
    let left_cols = [
        Input::new(p.PA7, Pull::Up),
        Input::new(p.PA6, Pull::Up),
        Input::new(p.PA2, Pull::Up),
        Input::new(p.PA3, Pull::Up),
        Input::new(p.PA4, Pull::Up),
        Input::new(p.PA5, Pull::Up),
    ];

    // I2C1 on PB6/PB7 (AF6), 400 kHz. A short timeout keeps a missing
    // right half from stalling the scan while it is being probed.
    let mut i2c_config = i2c::Config::default();
    i2c_config.frequency = Hertz::khz(400);
    i2c_config.timeout = Duration::from_millis(5);
    let i2c = I2c::new_blocking(p.I2C1, p.PB6, p.PB7, i2c_config);

    let mut matrix = PandoMatrix::new(left_rows, left_cols, i2c);

    let rmk_config = RmkConfig {
        device_config: DeviceConfig {
            vid: USB_VID,
            pid: USB_PID,
            manufacturer: "jyap808",
            product_name: "Pando",
            ..Default::default()
        },
        vial_config: VialConfig::new(
            &vial::VIAL_KEYBOARD_ID,
            vial::VIAL_KEYBOARD_DEF,
            &vial::UNLOCK_KEYS,
        ),
        ..Default::default()
    };

    // Keymap and settings live in the top sectors of flash.
    let flash = async_flash_wrapper(Flash::new_blocking(p.FLASH));
    let storage_config = StorageConfig::default();

    let mut keymap_data = KeymapData::new(keymap::get_default_keymap());
    let mut behavior_config = BehaviorConfig::default();
    // QMK's QK_TRI_LAYER_LOWER/upper: layers 1 and 2 together enable layer 3.
    behavior_config.tri_layer = Some([1, 2, 3]);
    let positional_config = PositionalConfig::new(keymap::HAND_MAP);

    let (keymap, mut storage) = initialize_keymap_and_storage(
        &mut keymap_data,
        flash,
        &storage_config,
        &mut behavior_config,
        &positional_config,
    )
    .await;

    let mut keyboard = Keyboard::new(&keymap);
    let host_service = HostService::new(&keymap, &rmk_config);

    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);
    let mut usb_transport =
        UsbTransport::new(driver, rmk_config.device_config).with_host_service(&host_service);

    join3(
        run_all!(matrix, storage),
        keyboard.run(),
        usb_transport.run(),
    )
    .await;
}
