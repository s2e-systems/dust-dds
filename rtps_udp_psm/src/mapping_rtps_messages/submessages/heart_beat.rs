use rust_rtps_pim::messages::submessages::HeartbeatSubmessage;

use crate::{deserialize::{self, MappingRead}, serialize::{self, MappingWrite}};

use std::io::Write;


impl MappingWrite for HeartbeatSubmessage {
    fn write<W: Write>(&self, mut _writer: W) -> serialize::Result {
        todo!()
    }
}
impl<'de> MappingRead<'de> for HeartbeatSubmessage {
    fn read(_buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        todo!()
    }
}