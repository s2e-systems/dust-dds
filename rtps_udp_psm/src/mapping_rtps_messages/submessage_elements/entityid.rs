use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{
    messages::submessage_elements::EntityIdSubmessageElement,
};

use crate::{
    deserialize::{self, Deserialize},
    serialize::{self, Serialize},
};


impl Serialize for EntityIdSubmessageElement {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.value.serialize::<_, B>(&mut writer)
    }
}
impl<'de> Deserialize<'de> for EntityIdSubmessageElement {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self { value: Deserialize::deserialize::<B>(buf)? })
    }
}

