use sx127x_rs_driver::Sx127xError;
use sx127x_rs_driver::register::common::RegVersion;

#[derive(Debug)]
pub enum Sx127xLoraError {
    Sx127xError(Sx127xError),
    VersionNotSupported {
        expected_version: RegVersion,
        actual_version: RegVersion,
    },
}
