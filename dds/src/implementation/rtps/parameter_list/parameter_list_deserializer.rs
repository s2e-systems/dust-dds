use crate::cdr::deserialize::CdrDeserialize;

pub trait ParameterListDeserializer<'de> {
    fn get<T>(&self, id: i16) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>;

    fn get_with_default<T>(&self, id: i16, default: T) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>;

    fn get_all<T>(&self, id: i16) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>;
}
