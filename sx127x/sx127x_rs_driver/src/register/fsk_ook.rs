use super::Register;
use int_enum::IntEnum;
use proc_bitfield::bitfield;
use sx127x_rs_macro::register;

bitfield! {
    /// A bitfield showcasing how to specify bit ranges.
    #[derive(Clone, Copy, PartialEq, Eq)]
    #[register(address = 0x12, default_u8 = 0x10)]
    pub struct BitRanges(u8): Debug, FromStorage, IntoStorage, DerefStorage {
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

bitfield! {
    /// FIFO data input/output
    #[derive(Clone, Copy, PartialEq, Eq)]
    #[register(address = 0x00, default_u8 = 0x00)]
    pub struct RegFifo(u8): Debug, FromStorage, IntoStorage, DerefStorage {
        pub data: u8 @ ..,
    }
}

bitfield! {
    /// FIFO data input/output
    #[derive(Clone, Copy, PartialEq, Eq)]
    #[register(address = 0x01, default_u8 = 0x00)]
    pub struct RegOpMode(u8): Debug, FromStorage, IntoStorage, DerefStorage {
        pub mode: u8 [try_both Mode] @ 0..=2,
        pub low_frequency_mode_on: bool @ 3,
        pub modulation_type: u8 [try_both ModulationType] @ 5..=6,
        pub long_range_mode: u8 [try_both LongRangeMode] @ 7..8,
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, IntEnum)]
pub enum Mode {
    Sleep = 0b000,
    Stdby = 0b001,
    FsTx = 0b010,
    Tx = 0b011,
    FsRx = 0b100,
    Rx = 0b101,
}

#[repr(u8)]
#[derive(Debug, PartialEq, IntEnum)]
pub enum ModulationType {
    Fsk = 0b00,
    Ook = 0b01,
}

#[repr(u8)]
#[derive(Debug, PartialEq, IntEnum)]
pub enum LongRangeMode {
    FskOok = 0b0,
    LoRa = 0b1,
}
