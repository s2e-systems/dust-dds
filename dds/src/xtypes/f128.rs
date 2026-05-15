use dust_dds_derive::TypeSupport;

#[derive(Debug, Clone, PartialEq, TypeSupport)]
pub struct F128(f32);

impl From<f32> for F128 {
    fn from(value: f32) -> Self {
        Self(value)
    }
}
