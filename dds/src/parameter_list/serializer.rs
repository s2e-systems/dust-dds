use crate::cdr::serialize::CdrSerialize;

pub trait ParameterListSerializer {
    fn write<T>(&self, id: i16, value: &T) -> Result<(), std::io::Error>
    where
        T: CdrSerialize;

    fn write_with_default<T>(&self, id: i16, value: &T, default: &T) -> Result<(), std::io::Error>
    where
        T: CdrSerialize;
}
