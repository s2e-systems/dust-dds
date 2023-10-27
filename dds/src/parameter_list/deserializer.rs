use crate::cdr::deserialize::CdrDeserialize;

pub struct ParameterListDeserializer<'de> {
    bytes: &'de [u8],
}

impl<'de> ParameterListDeserializer<'de> {
    pub fn read<T>(&self, id: i16) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>,
    {
        todo!()
    }

    pub fn read_with_default<T>(&self, id: i16, default: T) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>,
    {
        todo!()
    }

    pub fn read_all<T>(&self, id: i16) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>,
    {
        todo!()
    }
}
