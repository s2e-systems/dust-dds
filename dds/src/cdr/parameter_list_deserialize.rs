use super::parameter_list_deserializer::ParameterListDeserializer;

pub trait ParameterListDeserialize<'de>: Sized {
    fn deserialize(
        pl_deserializer: &mut ParameterListDeserializer<'de>,
    ) -> Result<Self, std::io::Error>;
}
