use super::parameter_list_deserializer::ParameterListDeserilizer;

pub trait ParameterListDeserialize<'de>: Sized {
    fn deserialize(
        pl_deserializer: &mut impl ParameterListDeserilizer<'de>,
    ) -> Result<Self, std::io::Error>;
}
