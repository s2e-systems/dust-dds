use super::parameter_list_serializer::ParameterListSerializer;

pub use dust_dds_derive::ParameterListSerialize;

pub trait ParameterListSerialize {
    fn serialize(&self, serializer: &mut ParameterListSerializer) -> Result<(), std::io::Error>;
}
