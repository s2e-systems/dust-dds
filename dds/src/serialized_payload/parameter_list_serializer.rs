use crate::serialized_payload::serialize::CdrSerialize;

pub use dust_dds_derive::ParameterListDeserialize;

pub trait ParameterListSerializer {
    fn write<T>(&mut self, id: i16, value: &T) -> Result<(), std::io::Error>
    where
        T: CdrSerialize;

    fn write_with_default<T>(
        &mut self,
        id: i16,
        value: &T,
        default: &T,
    ) -> Result<(), std::io::Error>
    where
        T: CdrSerialize + PartialEq;

    fn write_collection<T>(&mut self, id: i16, value_list: &[T]) -> Result<(), std::io::Error>
    where
        T: CdrSerialize;
}
