use super::parameter_list_serializer::ParameterListCdrSerializer;

pub use dust_dds_derive::ParameterListSerialize;

pub trait ParameterListSerialize {
    fn serialize(&self, serializer: &mut ParameterListCdrSerializer) -> Result<(), std::io::Error>;
}
