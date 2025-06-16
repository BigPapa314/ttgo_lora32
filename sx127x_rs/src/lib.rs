#![no_std]

mod register;

use embedded_hal::spi::Operation;
use embedded_hal::spi::SpiDevice;
use register::Register;

fn print_value(value: &[u8]) {
    log::info!("reg value: {value:#x?}");
}

fn print_register(reg: impl Register) {
    log::info!("reg {reg:#x?}");
    let address: u8 = reg.address();
    log::info!("reg address: {address}");
    print_value(reg.as_ref());
}

struct Sx127x<SPI> {
    spi: SPI,
}

impl<SPI> Sx127x<SPI>
where
    SPI: SpiDevice,
{
    fn new(spi: SPI) -> Self {
        Self { spi }
    }

    fn read<REG: Register>(&mut self) -> Result<REG, SPI::Error> {
        let mut result = REG::default();
        self.spi.transaction(&mut [
            Operation::Write(&[result.address()]),
            Operation::Read(result.as_mut()),
        ])?;
        Ok(result)
    }
}

pub fn test(spi: impl SpiDevice) {
    log::info!("Test start!");

    let range = register::fsk_ook::BitRanges::default();
    log::info!("range: {range:#?}");
    let exclusive_range = range.exclusive_range();
    log::info!("exclusive_range: {exclusive_range}");
    let value: u8 = range.into();
    log::info!("value: {value}");

    print_register(range);

    let mut reg_fr = register::lora::RegFr::default().with_frf(0x112233);
    print_register(reg_fr);
    let frequency_mhz = reg_fr.frequency_mhz();
    log::info!("reg_fr frequency: {frequency_mhz}");
    reg_fr.set_frequency_mhz(434_000_000);
    print_register(reg_fr);
    let frequency_mhz = reg_fr.frequency_mhz();
    log::info!("reg_fr frequency: {frequency_mhz}");
    reg_fr.set_frequency_mhz(1_000_000);
    print_register(reg_fr);
    let frequency_mhz = reg_fr.frequency_mhz();
    log::info!("reg_fr frequency: {frequency_mhz}");
    reg_fr.set_frequency_mhz(500_000);
    print_register(reg_fr);
    let frequency_mhz = reg_fr.frequency_mhz();
    log::info!("reg_fr frequency: {frequency_mhz}");
    reg_fr.set_frequency_mhz(62);
    print_register(reg_fr);
    let frequency_mhz = reg_fr.frequency_mhz();
    log::info!("reg_fr frequency: {frequency_mhz}");

    let mut driver = Sx127x::new(spi);
    let op_mode = driver
        .read::<register::lora::RegOpMode>()
        .expect("spi read failure");
    log::info!("read op_mode");
    print_register(op_mode);

    let fr = driver
        .read::<register::lora::RegFr>()
        .expect("spi read failure");
    log::info!("read fr");
    print_register(fr);
}
