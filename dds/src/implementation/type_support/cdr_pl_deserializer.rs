use std::marker::PhantomData;

use byteorder::ByteOrder;

pub struct ParameterListDeserializer<'de, E> {
    bytes: &'de [u8],
    reader: &'de [u8],
    phantom: PhantomData<E>,
}

impl<'de, E> ParameterListDeserializer<'de, E>
where
    E: ByteOrder,
{
    pub fn new(reader: &'de [u8]) -> Self {
        Self {
            bytes: reader,
            reader,
            phantom: PhantomData,
        }
    }
}
