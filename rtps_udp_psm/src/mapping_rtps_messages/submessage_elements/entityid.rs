use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{
    messages::submessage_elements::EntityIdSubmessageElement,
};

use crate::{
    deserialize::{self, MappingReadByteOrdered},
    serialize::{self, MappingWriteByteOrdered},
};


impl MappingWriteByteOrdered for EntityIdSubmessageElement {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.value.write_byte_ordered::<_, B>(&mut writer)
    }
}
impl<'de> MappingReadByteOrdered<'de> for EntityIdSubmessageElement {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self { value: MappingReadByteOrdered::read_byte_ordered::<B>(buf)? })
    }
}

