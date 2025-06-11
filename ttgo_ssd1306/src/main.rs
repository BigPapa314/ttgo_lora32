use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::i2c::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::units::Hertz;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, PrimitiveStyleBuilder},
    text::{Baseline, Text},
};
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use ssd1306::mode::TerminalDisplaySize;
use ssd1306::mode::TerminalMode;
use std::fmt::Write;

struct Ttgo32LoraDisplay<DI, SIZE, TDelay>
where
    TDelay: embedded_hal::delay::DelayNs,
{
    driver: Ssd1306<DI, SIZE, TerminalMode>,
    delay: TDelay,
}

impl<DI, SIZE, TDelay> Ttgo32LoraDisplay<DI, SIZE, TDelay>
where
    TDelay: embedded_hal::delay::DelayNs,
{
    fn new(driver: Ssd1306<DI, SIZE, TerminalMode>, delay: TDelay) -> Self {
        Self { driver, delay }
    }

    fn driver(&mut self) -> &mut Ssd1306<DI, SIZE, TerminalMode> {
        &mut self.driver
    }
}

impl<DI, SIZE, TDelay> std::fmt::Write for Ttgo32LoraDisplay<DI, SIZE, TDelay>
where
    DI: WriteOnlyDataCommand,
    SIZE: TerminalDisplaySize,
    TDelay: embedded_hal::delay::DelayNs,
{
    fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
        s.chars().for_each(|c| {
            self.driver().print_char(c);
        });

        // for c in s.chars() {
        //     self.driver().print_char(c);
        //     // self.delay.delay_ms(1);
        // }

        Ok(())
    }
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, ssd1306!");

    let peripherals = Peripherals::take().unwrap();

    let i2c0 = peripherals.i2c0;

    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;

    let display_i2c_config = I2cConfig::new().baudrate(400.kHz().into());
    let display_i2c = I2cDriver::new(i2c0, sda, scl, &display_i2c_config).unwrap();

    let interface = I2CDisplayInterface::new(display_i2c);

    let mut display_driver =
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_terminal_mode();

    let mut display = Ttgo32LoraDisplay::new(display_driver, FreeRtos);

    display.driver().init().unwrap();
    display.driver().clear().unwrap();

    write!(display, "Hello World!");
    FreeRtos::delay_ms(1);
}
