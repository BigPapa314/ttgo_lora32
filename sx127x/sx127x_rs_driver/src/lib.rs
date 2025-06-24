#![no_std]

mod constants;
mod driver;
mod error;
pub mod register;

pub use constants::MODE;
pub use driver::Sx127xDriver;
pub use error::Sx127xError;
use register::Register;

pub trait Sx127x {
    fn reset(&mut self) -> Result<(), Sx127xError>;

    fn read_buffer(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Sx127xError>;

    fn write_buffer(&mut self, address: u8, buffer: &[u8]) -> Result<(), Sx127xError>;

    fn read<Reg: Register>(&mut self, register: &mut Reg) -> Result<(), Sx127xError> {
        self.read_buffer(register.address(), register.as_mut())
    }

    fn write<Reg: Register>(&mut self, register: &Reg) -> Result<(), Sx127xError> {
        self.write_buffer(register.address(), register.as_ref())
    }

    fn read_register<Reg, MakeRegister>(
        &mut self,
        make_register: MakeRegister,
    ) -> Result<Reg, Sx127xError>
    where
        Reg: Register,
        MakeRegister: FnOnce(u8) -> Reg,
    {
        let mut result = make_register(0);
        self.read_buffer(result.address(), result.as_mut())?;
        Ok(result)
    }
}
