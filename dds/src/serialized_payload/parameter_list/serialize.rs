use crate::implementation::payload_serializer_deserializer::parameter_list_serializer::ParameterListCdrSerializer;
pub use dust_dds_derive::ParameterListSerialize;

/// A trait representing a structure that can be serialized into a CDR parameter list format.
pub trait ParameterListSerialize {
    /// Method to serialize this value using the given serializer.
    fn serialize(
        &self,
        serializer: &mut ParameterListCdrSerializer,
    ) -> Result<(), std::io::Error>;
}
