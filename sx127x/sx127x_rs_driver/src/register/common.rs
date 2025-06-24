use super::Register;
use proc_bitfield::bitfield;
use sx127x_rs_macro::register;

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    #[register(address = 0x42, default_u8 = 0x12)]
    pub struct RegVersion(pub u8): Debug, FromStorage, IntoStorage, DerefStorage {
        pub metal_mask_revision: u8 [ro] @ 0..=3,
        pub full_revision: u8 [ro] @ 4..=7,
    }
}
