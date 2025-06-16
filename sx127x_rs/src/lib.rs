#![no_std]

mod register;

use embedded_hal::delay;
use embedded_hal::digital;
use embedded_hal::spi;
use register::Register;

/// Provides the necessary SPI mode configuration for the radio
pub const MODE: spi::Mode = spi::MODE_0;

// fn print_value(value: &[u8]) {
//     log::info!("reg value: {value:#x?}");
// }

// fn print_register(reg: impl Register) {
//     log::info!("reg {reg:#x?}");
//     let address: u8 = reg.address();
//     log::info!("reg address: {address}");
//     print_value(reg.as_ref());
// }

struct Sx127x<DELAY, SPI, RESET> {
    delay: DELAY,
    spi: SPI,
    reset: RESET,
}

#[derive(Debug)]
enum Sx127xError<SpiError, ResetError> {
    Spi(SpiError),
    Reset(ResetError),
}

impl<DELAY, SPI, RESET> Sx127x<DELAY, SPI, RESET>
where
    DELAY: delay::DelayNs,
    SPI: spi::SpiDevice,
    RESET: digital::OutputPin,
{
    fn new(delay: DELAY, spi: SPI, reset: RESET) -> Self {
        Self { delay, spi, reset }
    }

    fn reset(&mut self) -> Result<(), Sx127xError<SPI::Error, RESET::Error>> {
        self.reset.set_low().map_err(Sx127xError::Reset)?;
        self.delay.delay_ms(10);
        self.reset.set_high().map_err(Sx127xError::Reset)?;
        self.delay.delay_ms(10);
        Ok(())
    }

    // fn read<REG: Register>(&mut self) -> Result<REG, Sx127xError<SPI::Error, RESET::Error>> {
    //     let mut result = REG::default();
    //     self.spi
    //         .transaction(&mut [
    //             spi::Operation::Write(&[result.address()]),
    //             spi::Operation::Read(result.as_mut()),
    //         ])
    //         .map_err(Sx127xError::Spi)?;
    //     Ok(result)
    // }

    // fn read_manual(&mut self, address: u8) -> Result<u8, Sx127xError<SPI::Error, RESET::Error>> {
    //     let mut result: [u8; 1] = [0];
    //     self.spi
    //         .transaction(&mut [
    //             spi::Operation::Write(&[address]),
    //             spi::Operation::Read(&mut result),
    //         ])
    //         .map_err(Sx127xError::Spi)?;

    //     Ok(result[0])
    // }

    fn read_manual(&mut self, address: u8) -> Result<u8, Sx127xError<SPI::Error, RESET::Error>> {
        let mut buffer = [address & 0x7f, 0];
        log::info!("read_manual: {buffer:#x?}");
        self.spi
            .transaction(&mut [spi::Operation::TransferInPlace(&mut buffer)])
            .map_err(Sx127xError::Spi)?;
        log::info!("read_manual: {buffer:#x?}");
        Ok(buffer[1])
    }

    // fn read_register(&mut self, reg: u8) -> Result<u8, Error<E, CS::Error, RESET::Error>> {
    //     self.cs.set_low().map_err(CS)?;

    //     let mut buffer = [reg & 0x7f, 0];
    //     let transfer = self.spi.transfer(&mut buffer).map_err(SPI)?;
    //     self.cs.set_high().map_err(CS)?;
    //     Ok(transfer[1])
    // }
}

pub fn test(delay: impl delay::DelayNs, spi: impl spi::SpiDevice, reset: impl digital::OutputPin) {
    log::info!("Test start!");

    // let range = register::fsk_ook::BitRanges::default();
    // log::info!("range: {range:#?}");
    // let exclusive_range = range.exclusive_range();
    // log::info!("exclusive_range: {exclusive_range}");
    // let value: u8 = range.into();
    // log::info!("value: {value}");

    // print_register(range);

    // let mut reg_fr = register::lora::RegFr::default().with_frf(0x112233);
    // print_register(reg_fr);
    // let frequency_mhz = reg_fr.frequency_mhz();
    // log::info!("reg_fr frequency: {frequency_mhz}");
    // reg_fr.set_frequency_mhz(434_000_000);
    // print_register(reg_fr);
    // let frequency_mhz = reg_fr.frequency_mhz();
    // log::info!("reg_fr frequency: {frequency_mhz}");
    // reg_fr.set_frequency_mhz(1_000_000);
    // print_register(reg_fr);
    // let frequency_mhz = reg_fr.frequency_mhz();
    // log::info!("reg_fr frequency: {frequency_mhz}");
    // reg_fr.set_frequency_mhz(500_000);
    // print_register(reg_fr);
    // let frequency_mhz = reg_fr.frequency_mhz();
    // log::info!("reg_fr frequency: {frequency_mhz}");
    // reg_fr.set_frequency_mhz(62);
    // print_register(reg_fr);
    // let frequency_mhz = reg_fr.frequency_mhz();
    // log::info!("reg_fr frequency: {frequency_mhz}");

    let mut driver = Sx127x::new(delay, spi, reset);
    driver.reset().expect("reset went wrong");

    // let reg = driver.read_manual(0x01);
    // log::info!("read reg: {reg:#x?}");

    // let reg = driver.read_manual(0x07);
    // log::info!("read reg: {reg:#x?}");

    let reg = driver.read_manual(0x42);
    log::info!("read reg: {reg:#x?}");

    // let op_mode = driver
    //     .read::<register::lora::RegOpMode>()
    //     .expect("spi read failure");
    // log::info!("read op_mode");
    // print_register(op_mode);

    // let fr = driver
    //     .read::<register::lora::RegFr>()
    //     .expect("spi read failure");
    // log::info!("read fr");
    // print_register(fr);
}
