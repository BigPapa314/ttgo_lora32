#![no_std]

use proc_bitfield::bitfield;

use sx127x_rs_macro::register;

trait Register: Into<u8> + core::fmt::Debug {
    const ADDRESS: u8;
    fn address(&self) -> u8 {
        Self::ADDRESS
    }
}

bitfield! {
    /// A bitfield showcasing how to specify bit ranges.
    #[derive(Clone, Copy, PartialEq, Eq)]
    #[register(address = 0x12)]
    pub struct BitRanges(pub u8): Debug, FromStorage, IntoStorage, DerefStorage {
        // A single field spanning the entire bitfield, using an unbounded range:
        pub whole_bitfield: u32 @ ..,                 // Bits 0 to 31

        // Multi-bit field, specified using an inclusive range:
        pub inclusive_range: u8 @ 0..=3,              // Bits 0 to 3

        // Multi-bit field, specified using an exclusive range:
        pub exclusive_range: u8 @ 4..7,               // Bits 4 to 6

        // Multi-bit field specified using its start bit and length:
        pub start_and_length: u8 @ 7; 1,              // Bits 7 to 8
    }
}

fn print_register(reg: impl Register) {
    log::info!("reg {reg:#?}");
    let address: u8 = reg.address();
    log::info!("reg address: {address}");
    let value: u8 = reg.into();
    log::info!("reg value: {value}");
}

pub fn test() {
    log::info!("Test start!");

    let range = BitRanges(10);
    log::info!("range: {range:#?}");
    let exclusive_range = range.exclusive_range();
    log::info!("exclusive_range: {exclusive_range}");
    let value: u8 = range.into();
    log::info!("value: {value}");

    print_register(range);
}
