use rust_rtps_pim::messages::submessages::InfoReplySubmessage;

use crate::{deserialize::{self, MappingRead}, serialize::{self, MappingWrite}};

use std::io::Write;


impl<L> MappingWrite for InfoReplySubmessage<L> {
    fn write<W: Write>(&self, mut _writer: W) -> serialize::Result {
        todo!()
    }
}
impl<'de, L> MappingRead<'de> for InfoReplySubmessage<L> {
    fn read(_buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        todo!()
    }
}