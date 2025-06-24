pub use sx127x_rs_driver as driver;

#[cfg(feature = "lora")]
pub use sx127x_rs_lora as lora;

pub mod prelude {
    pub use crate::driver::Sx127xDriver;
    #[cfg(feature = "lora")]
    pub use crate::prelude_lora::*;
}

#[cfg(feature = "lora")]
mod prelude_lora {
    pub use crate::lora::Sx127xLora;
    pub use crate::lora::Sx127xLoraExt;
}
