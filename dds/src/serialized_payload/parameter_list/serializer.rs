use crate::xtypes::serialize::XTypesSerialize;

/// A trait representing an object with the capability of serializing a value into a CDR parameter list format.
/// All the parameters of a CDR Parameter List must be themselves [`XTypesSerialize`].
pub trait ParameterListSerializer {
    /// Method to serialize a parameter without default.
    fn write<T>(&mut self, id: i16, value: &T) -> Result<(), std::io::Error>
    where
        T: XTypesSerialize;

    /// Method to serialize a parameter with default.
    fn write_with_default<T>(
        &mut self,
        id: i16,
        value: &T,
        default: &T,
    ) -> Result<(), std::io::Error>
    where
        T: XTypesSerialize + PartialEq;

    /// Method to serialize a collection of parameters of a given type.
    fn write_collection<T>(&mut self, id: i16, value_list: &[T]) -> Result<(), std::io::Error>
    where
        T: XTypesSerialize;
}
