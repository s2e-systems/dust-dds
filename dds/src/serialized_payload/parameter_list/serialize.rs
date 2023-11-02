use super::serializer::ParameterListSerializer;

pub use dust_dds_derive::ParameterListSerialize;

pub trait ParameterListSerialize {
    fn serialize(
        &self,
        serializer: &mut impl ParameterListSerializer,
    ) -> Result<(), std::io::Error>;
}
