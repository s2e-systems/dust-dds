use super::parameter_list_deserializer::ParameterListCdrDeserializer;

pub use dust_dds_derive::ParameterListDeserialize;

pub trait ParameterListDeserialize<'de>: Sized {
    fn deserialize(
        pl_deserializer: &mut ParameterListCdrDeserializer<'de>,
    ) -> Result<Self, std::io::Error>;
}
