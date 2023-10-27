use crate::cdr::serialize::CdrSerialize;

pub struct ParameterListSerializer {}

impl ParameterListSerializer {
    pub fn write<T>(&self, id: i16, value: &T) -> Result<(), std::io::Error>
    where
        T: CdrSerialize,
    {
        todo!()
    }

    pub fn write_with_default<T>(
        &self,
        id: i16,
        value: &T,
        default: &T,
    ) -> Result<(), std::io::Error>
    where
        T: CdrSerialize,
    {
        todo!()
    }
}
