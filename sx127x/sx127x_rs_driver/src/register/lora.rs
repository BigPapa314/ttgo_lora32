use core::u32;

use super::Register;
use int_enum::IntEnum;
use proc_bitfield::bitfield;
use sx127x_rs_macro::register;

bitfield! {
    /// LoRaTM base-band FIFO data input/output. FIFO is cleared and not accessible when device is in SLEEP mode
    #[derive(Clone, Copy, PartialEq, Eq)]
    #[register(address = 0x00, default_u8 = 0x00)]
    pub struct RegFifo(u8): Debug, FromStorage, IntoStorage, DerefStorage {
        pub data: u8 @ ..,
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    #[register(address = 0x01, default_u8 = 0b1001)]
    pub struct RegOpMode(u8): Debug, FromStorage, IntoStorage, DerefStorage {
        /// Device modes
        pub mode: u8 [try_both Mode] @ 0..=2,
        pub low_frequency_mode_on: bool @ 3,
        pub access_shared_reg: bool @ 6,
        pub lora_mode: bool @ 7
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
    RxContinuous = 0b101,
    RxSingle = 0b110,
    Cad = 0b111,
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

bitfield! {
    /// FIFO data input/output
    #[derive(Clone, Copy, PartialEq, Eq)]
    #[register(address = 0x06, default_u24 = 0)]
    pub struct RegFr(u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub frf: u32 @ 0..=24,
        pub frequency_mhz: u32 [get_fn frf_to_frequency_hz -> u64, set_fn frf_fom_frequency_hz(u64)] @ 0..=24,
    }
}

// Frequency formular original: frequency_mhz = (F_XOSC_mhz * frf) / 2 ^ 19
// Frequency formular in hz: frequency_hz = (F_XOSC_mhz * 1_000_000 * frf) / 2 ^ 19

const F_XOSC: u64 = 32_000_000;
const FRF_DEVIDER: u64 = (1 << 19) as u64;

fn frf_to_frequency_hz(frf: u32) -> u64 {
    (F_XOSC * (frf as u64)) / FRF_DEVIDER
}
fn frf_fom_frequency_hz(frequency_hz: u64) -> u32 {
    ((frequency_hz * FRF_DEVIDER) / F_XOSC) as u32
}
