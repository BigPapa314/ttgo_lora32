use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::i2c::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;

use ssd1306::{mode::TerminalMode, prelude::*, I2CDisplayInterface, Ssd1306};

use ssd1306::mode::TerminalDisplaySize;
use std::fmt::Write;

struct Ttgo32LoraDisplay<DI, SIZE> {
    driver: Ssd1306<DI, SIZE, TerminalMode>,
}

impl<DI, SIZE> Ttgo32LoraDisplay<DI, SIZE> {
    fn new(driver: Ssd1306<DI, SIZE, TerminalMode>) -> Self {
        Self { driver }
    }

    fn driver(&mut self) -> &mut Ssd1306<DI, SIZE, TerminalMode> {
        &mut self.driver
    }
}

impl<DI, SIZE> std::fmt::Write for Ttgo32LoraDisplay<DI, SIZE>
where
    DI: WriteOnlyDataCommand,
    SIZE: TerminalDisplaySize,
{
    fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
        s.chars()
            .try_for_each(|c| self.driver().print_char(c))
            .map_err(|_| std::fmt::Error)
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

    let display_driver =
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_terminal_mode();

    let mut display = Ttgo32LoraDisplay::new(display_driver);

    display.driver().init().unwrap();
    display.driver().clear().unwrap();

    let mut counter = 0;
    loop {
        writeln!(display, "Hello World: {counter}").unwrap();
        counter += 1;
        FreeRtos::delay_ms(500);
    }
}
