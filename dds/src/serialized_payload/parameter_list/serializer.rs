use crate::serialized_payload::cdr::serialize::CdrSerialize;

pub use dust_dds_derive::ParameterListDeserialize;

/// A trait representing an object with the capability of serializing a value into a CDR parameter list format.
/// All the parameters of a CDR Parameter List must be themselves [`CdrSerialize`].
pub trait ParameterListSerializer {
    /// Method to serialize a parameter without default.
    fn write<T>(&mut self, id: i16, value: &T) -> Result<(), std::io::Error>
    where
        T: CdrSerialize;

    /// Method to serialize a parameter with default.
    fn write_with_default<T>(
        &mut self,
        id: i16,
        value: &T,
        default: &T,
    ) -> Result<(), std::io::Error>
    where
        T: CdrSerialize + PartialEq;

    /// Method to serialize a collection of parameters of a given type.
    fn write_collection<T>(&mut self, id: i16, value_list: &[T]) -> Result<(), std::io::Error>
    where
        T: CdrSerialize;
}
