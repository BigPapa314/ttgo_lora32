use crate::Sx127xLora;
use crate::Sx127xLoraError;
use sx127x_rs_driver::*;

/// the expected sx127x version
pub const EXPECTED_VERSION: register::common::RegVersion =
    register::common::RegVersion::const_default();

pub struct Sx127xLoraProtocol<Driver> {
    driver: Driver,
}

pub trait Sx127xLoraExt {
    fn into_lora(self) -> impl Sx127xLora;
}

impl<Driver: Sx127x> Sx127xLoraExt for Driver {
    fn into_lora(self) -> impl Sx127xLora {
        Sx127xLoraProtocol { driver: self }
    }
}

impl<Driver: Sx127x> Sx127xLora for Sx127xLoraProtocol<Driver> {
    fn check_version(&mut self) -> Result<(), Sx127xLoraError> {
        let version = self
            .driver
            .read_register(register::common::RegVersion)
            .map_err(Sx127xLoraError::Sx127xError)?;
        if version != EXPECTED_VERSION {
            return Err(Sx127xLoraError::VersionNotSupported {
                expected_version: EXPECTED_VERSION,
                actual_version: version,
            });
        }
        Ok(())
    }

    fn read_fifo(&mut self, data: &mut [u8]) -> Result<(), Sx127xLoraError> {
        self.driver
            .read_buffer(0x00, data)
            .map_err(Sx127xLoraError::Sx127xError)
    }

    fn write_fifo(&mut self, data: &[u8]) -> Result<(), Sx127xLoraError> {
        self.driver
            .write_buffer(0x00, data)
            .map_err(Sx127xLoraError::Sx127xError)
    }
}
