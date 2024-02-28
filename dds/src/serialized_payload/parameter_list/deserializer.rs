use crate::serialized_payload::cdr::deserialize::CdrDeserialize;

/// A trait representing an object with the capability of deserializing a value from a CDR parameter list format.
/// The parameters of a CDR Parameter List must be themselves [`CdrDeserialize`].
pub trait ParameterListDeserializer<'de> {
    /// Method to deserialize a parameter without default.
    fn read<T>(&self, id: i16) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>;

    /// Method to deserialize a parameter with default.
    fn read_with_default<T>(&self, id: i16, default: T) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>;

    /// Method to deserialize a collection of parameter of a given type.
    fn read_collection<T>(&self, id: i16) -> Result<Vec<T>, std::io::Error>
    where
        T: CdrDeserialize<'de>;
}
