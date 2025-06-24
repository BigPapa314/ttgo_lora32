use embedded_hal::delay;
use embedded_hal::digital;
use embedded_hal::spi;

use crate::Sx127x;
use crate::error::Sx127xError;

pub struct Sx127xDriver<DELAY, SPI, RESET> {
    delay: DELAY,
    spi: SPI,
    reset: RESET,
}

impl<DELAY, SPI, RESET> Sx127xDriver<DELAY, SPI, RESET>
where
    DELAY: delay::DelayNs,
    SPI: spi::SpiDevice,
    RESET: digital::OutputPin,
{
    pub fn new(delay: DELAY, spi: SPI, reset: RESET) -> Result<Self, Sx127xError> {
        let mut driver = Self { delay, spi, reset };
        driver.reset()?;
        Ok(driver)
    }
}

impl<DELAY, SPI, RESET> Sx127x for Sx127xDriver<DELAY, SPI, RESET>
where
    DELAY: delay::DelayNs,
    SPI: spi::SpiDevice,
    RESET: digital::OutputPin,
{
    fn reset(&mut self) -> Result<(), Sx127xError> {
        self.reset
            .set_low()
            .map_err(|_| Sx127xError::ResetPinFailed)?;
        self.delay.delay_ms(10);
        self.reset
            .set_high()
            .map_err(|_| Sx127xError::ResetPinFailed)?;
        self.delay.delay_ms(10);
        Ok(())
    }

    fn read_buffer(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Sx127xError> {
        self.spi
            .transaction(&mut [
                spi::Operation::Write(&[address]),
                spi::Operation::Read(buffer),
            ])
            .map_err(|_| Sx127xError::SpiCommunicationFailed)?;
        Ok(())
    }

    fn write_buffer(&mut self, address: u8, buffer: &[u8]) -> Result<(), Sx127xError> {
        self.spi
            .transaction(&mut [
                spi::Operation::Write(&[address | 0x80]),
                spi::Operation::Write(buffer),
            ])
            .map_err(|_| Sx127xError::SpiCommunicationFailed)?;
        Ok(())
    }
}
