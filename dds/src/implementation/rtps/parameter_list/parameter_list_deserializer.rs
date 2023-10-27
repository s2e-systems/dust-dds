use crate::cdr::deserialize::CdrDeserialize;

pub trait ParameterListDeserializer<'de> {
    fn read<T>(&self, id: i16) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>;

    fn read_with_default<T>(&self, id: i16, default: T) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>;

    fn read_all<T>(&self, id: i16) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>;
}
