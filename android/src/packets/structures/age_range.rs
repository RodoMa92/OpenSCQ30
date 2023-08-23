use openscq30_lib::packets::structures::AgeRange as LibAgeRange;
use rifgen::rifgen_attr::{generate_interface, generate_interface_doc};

#[generate_interface_doc]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct AgeRange {
    inner: LibAgeRange,
}

impl AgeRange {
    #[generate_interface(constructor)]
    pub fn new(age_range: u8) -> AgeRange {
        Self {
            inner: LibAgeRange(age_range),
        }
    }
}

impl From<LibAgeRange> for AgeRange {
    fn from(inner: LibAgeRange) -> Self {
        Self { inner }
    }
}
impl From<AgeRange> for LibAgeRange {
    fn from(value: AgeRange) -> Self {
        value.inner
    }
}
