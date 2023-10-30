use super::parameter_list_deserializer::ParameterListDeserializer;

pub use dust_dds_derive::ParameterListDeserialize;

pub trait ParameterListDeserialize<'de>: Sized {
    fn deserialize(
        pl_deserializer: &mut ParameterListDeserializer<'de>,
    ) -> Result<Self, std::io::Error>;
}
