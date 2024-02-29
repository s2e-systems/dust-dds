use super::serializer::ParameterListSerializer;

pub use dust_dds_derive::ParameterListSerialize;

/// A trait representing a structure that can be serialized into a CDR parameter list format.
pub trait ParameterListSerialize {
    /// Method to serialize this value using the given serializer.
    fn serialize(
        &self,
        serializer: &mut impl ParameterListSerializer,
    ) -> Result<(), std::io::Error>;
}
