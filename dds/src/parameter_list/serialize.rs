use super::serializer::ParameterListSerializer;

pub trait ParameterListSerialize {
    fn serialize(
        &self,
        serializer: &mut impl ParameterListSerializer,
    ) -> Result<(), std::io::Error>;
}
