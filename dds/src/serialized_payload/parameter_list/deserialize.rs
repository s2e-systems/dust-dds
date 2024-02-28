use super::deserializer::ParameterListDeserializer;

pub use dust_dds_derive::ParameterListDeserialize;

/// A trait representing a structure that can be deserialized from a CDR parameter list format.
pub trait ParameterListDeserialize<'de>: Sized {
    /// Method to deserialize this value using the given deserializer.
    fn deserialize(
        pl_deserializer: &mut impl ParameterListDeserializer<'de>,
    ) -> Result<Self, std::io::Error>;
}
