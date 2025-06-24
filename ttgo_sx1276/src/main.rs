use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::spi::*;

use sx127x_rs::prelude::*;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, sx127x_lora!");

    let peripherals = Peripherals::take().unwrap();

    let spi_hw = peripherals.spi2;

    let sclk = peripherals.pins.gpio5;
    let miso = peripherals.pins.gpio19;
    let mosi = peripherals.pins.gpio27;
    let cs = peripherals.pins.gpio18;
    let rst = PinDriver::output(peripherals.pins.gpio23).unwrap();
    // let dio = peripherals.pins.gpio26;

    log::info!("Starting SPI loopback test");
    // let spi_driver_config = SpiDriverConfig::new().baudrate(18.MHz().into());
    let spi_driver_config = SpiDriverConfig::new();
    let spi_driver = SpiDriver::new(spi_hw, sclk, mosi, Some(miso), &spi_driver_config).unwrap();
    let spi_config = SpiConfig::new()
        .baudrate(20.MHz().into())
        .data_mode(sx127x_rs::driver::MODE);
    let spi = SpiDeviceDriver::new(spi_driver, Some(cs), &spi_config).unwrap();

    log::info!("spistuff");

    let driver = Sx127xDriver::new(FreeRtos, spi, rst).unwrap();
    let lora = driver.into_lora();
    test_lora(lora);
}

pub fn test_lora(mut lora: impl Sx127xLora) {
    log::info!("Test start!");

    lora.check_version().expect("version is not known");
}
