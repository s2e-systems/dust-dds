use super::serializer::ParameterListSerializer;

pub trait ParameterListSerialize {
    fn serialize(&self, serializer: &mut ParameterListSerializer) -> Result<(), std::io::Error>;
}
