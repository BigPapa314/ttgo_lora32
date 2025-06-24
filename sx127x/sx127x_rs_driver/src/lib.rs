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

    fn update<Reg, UpdateRegister>(
        &mut self,
        update_register: UpdateRegister,
    ) -> Result<(), Sx127xError>
    where
        Reg: Register + From<u8>,
        UpdateRegister: FnOnce(&mut Reg),
    {
        let mut register = self.read_register::<Reg>()?;
        update_register(&mut register);
        self.write(&register)
    }

    fn read_register<Reg>(&mut self) -> Result<Reg, Sx127xError>
    where
        Reg: Register + From<u8>,
    {
        let mut result: Reg = 0.into();
        self.read_buffer(result.address(), result.as_mut())?;
        Ok(result)
    }
}
