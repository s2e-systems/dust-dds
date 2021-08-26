use rust_rtps_pim::messages::submessages::AckNackSubmessage;

use crate::{deserialize::{self, MappingRead}, serialize::{self, MappingWrite}};

use std::io::Write;


impl<S> MappingWrite for AckNackSubmessage<S> {
    fn write<W: Write>(&self, mut _writer: W) -> serialize::Result {
        todo!()
    }
}
impl<'de, S> MappingRead<'de> for AckNackSubmessage<S> {
    fn read(_buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        todo!()
    }
}