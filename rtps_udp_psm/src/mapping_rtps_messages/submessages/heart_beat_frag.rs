
use crate::{mapping_traits::{MappingRead, MappingWrite}, messages::submessages::{HeartbeatFragSubmessageRead, HeartbeatFragSubmessageWrite}};

use std::io::{Error, Write};

impl MappingWrite for HeartbeatFragSubmessageWrite {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'de> MappingRead<'de> for HeartbeatFragSubmessageRead {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
