#![no_std]

mod error;
mod protocol;

pub use error::Sx127xLoraError;
pub use protocol::Sx127xLoraExt;

pub trait Sx127xLora {
    fn check_version(&mut self) -> Result<(), Sx127xLoraError>;

    fn read_fifo(&mut self, data: &mut [u8]) -> Result<(), Sx127xLoraError>;

    fn write_fifo(&mut self, data: &[u8]) -> Result<(), Sx127xLoraError>;
}
