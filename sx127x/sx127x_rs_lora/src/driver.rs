use crate::error::Sx127xLoraError;
use sx127x_rs_driver::*;

/// the expected sx127x version
const EXPECTED_VERSION: register::common::RegVersion =
    register::common::RegVersion::const_default();

pub struct Sx127xLoraDriver<Sx127xDriver: Sx127x> {
    pub sx127x: Sx127xDriver,
}

impl<Sx127xDriver: Sx127x> Sx127xLoraDriver<Sx127xDriver> {
    pub fn reset(&mut self) -> Result<(), Sx127xLoraError> {
        self.sx127x.reset().map_err(Sx127xLoraError::Sx127xError)
    }

    pub fn check_version(&mut self) -> Result<(), Sx127xLoraError> {
        let version = self
            .sx127x
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

    pub fn read_fifo(&mut self, data: &mut [u8]) -> Result<(), Sx127xLoraError> {
        self.sx127x
            .read_buffer(0x00, data)
            .map_err(Sx127xLoraError::Sx127xError)
    }

    pub fn write_fifo(&mut self, data: &[u8]) -> Result<(), Sx127xLoraError> {
        self.sx127x
            .write_buffer(0x00, data)
            .map_err(Sx127xLoraError::Sx127xError)
    }
}
