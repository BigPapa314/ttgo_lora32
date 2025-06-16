use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::spi::*;

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
    let spi_driver = SpiDriver::new(spi_hw, sclk, miso, Some(mosi), &spi_driver_config).unwrap();
    let spi_config = SpiConfig::new()
        .baudrate(20.kHz().into())
        .data_mode(sx127x_rs::MODE)
        .allow_pre_post_delays(true)
        .cs_pre_delay_us(10)
        .cs_post_delay_us(10)
        .polling(true);
    let spi = SpiDeviceDriver::new(spi_driver, Some(cs), &spi_config).unwrap();

    log::info!("spistuff");

    // let mut lora = sx127x_lora::LoRa::new(
    //     spi,
    //     PinDriver::input_output(cs).unwrap(),
    //     PinDriver::input_output(rst).unwrap(),
    //     915_000_000,
    //     FreeRtos,
    // )
    // .unwrap();

    sx127x_rs::test(FreeRtos, spi, rst);
}
