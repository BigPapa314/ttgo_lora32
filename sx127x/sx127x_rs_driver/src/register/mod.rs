pub mod common;
pub mod fsk_ook;
pub mod lora;

pub trait Register: Default + AsRef<[u8]> + AsMut<[u8]> + core::fmt::Debug {
    const ADDRESS: u8;
    fn address(&self) -> u8 {
        Self::ADDRESS
    }
}
