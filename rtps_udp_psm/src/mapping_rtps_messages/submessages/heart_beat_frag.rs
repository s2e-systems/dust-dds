use rust_rtps_pim::messages::submessages::HeartbeatFragSubmessage;

use crate::{deserialize::{self, MappingRead}, serialize::{self, MappingWrite}};

use std::io::Write;


impl MappingWrite for HeartbeatFragSubmessage {
    fn write<W: Write>(&self, mut _writer: W) -> serialize::Result {
        todo!()
    }
}
impl<'de> MappingRead<'de> for HeartbeatFragSubmessage {
    fn read(_buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        todo!()
    }
}