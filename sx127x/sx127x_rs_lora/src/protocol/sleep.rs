use crate::driver::Sx127xLoraDriver;
use crate::error::Sx127xLoraError;
use sx127x_rs_driver::Sx127x;

pub struct Sx127xLoraProtocolSleep<Sx127xDriver: Sx127x> {
    driver: Sx127xLoraDriver<Sx127xDriver>,
}

pub trait Sx127xLoraExt<Sx127xDriver: Sx127x> {
    fn into_lora(self) -> Result<Sx127xLoraProtocolSleep<Sx127xDriver>, Sx127xLoraError>;
}

impl<Sx127xDriver: Sx127x> Sx127xLoraExt<Sx127xDriver> for Sx127xDriver {
    fn into_lora(self) -> Result<Sx127xLoraProtocolSleep<Sx127xDriver>, Sx127xLoraError> {
        let driver = Sx127xLoraDriver { sx127x: self };
        let mut protocol_sleep = Sx127xLoraProtocolSleep { driver };
        protocol_sleep.initialize()?;
        Ok(protocol_sleep)
    }
}

impl<Sx127xDriver: Sx127x> Sx127xLoraProtocolSleep<Sx127xDriver> {
    fn initialize(&mut self) -> Result<(), Sx127xLoraError> {
        self.driver.reset()?;
        self.check_version()
    }
    pub fn check_version(&mut self) -> Result<(), Sx127xLoraError> {
        self.driver.check_version()
    }
}
