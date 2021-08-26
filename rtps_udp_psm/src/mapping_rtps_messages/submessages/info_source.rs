use rust_rtps_pim::messages::submessages::InfoSourceSubmessage;

use crate::{deserialize::{self, MappingRead}, serialize::{self, MappingWrite}};

use std::io::Write;


impl MappingWrite for InfoSourceSubmessage {
    fn write<W: Write>(&self, mut _writer: W) -> serialize::Result {
        todo!()
    }
}
impl<'de> MappingRead<'de> for InfoSourceSubmessage {
    fn read(_buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        todo!()
    }
}