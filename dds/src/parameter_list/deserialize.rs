use super::deserializer::ParameterListDeserializer;

pub trait ParameterListDeserialize<'de>: Sized {
    fn deserialize(
        pl_deserializer: &mut impl ParameterListDeserializer<'de>,
    ) -> Result<Self, std::io::Error>;
}
