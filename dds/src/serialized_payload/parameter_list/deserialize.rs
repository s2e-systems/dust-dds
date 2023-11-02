use super::deserializer::ParameterListDeserializer;

pub use dust_dds_derive::ParameterListDeserialize;

pub trait ParameterListDeserialize<'de>: Sized {
    fn deserialize(
        pl_deserializer: &mut impl ParameterListDeserializer<'de>,
    ) -> Result<Self, std::io::Error>;
}
