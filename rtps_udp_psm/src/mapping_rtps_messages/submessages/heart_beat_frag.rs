use rust_rtps_psm::messages::submessages::{
    HeartbeatFragSubmessageRead, HeartbeatFragSubmessageWrite,
};

use crate::{
    deserialize::{self, MappingRead},
    serialize::{self, MappingWrite},
};

use std::io::Write;

impl MappingWrite for HeartbeatFragSubmessageWrite {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> serialize::Result {
        todo!()
    }
}
impl<'de> MappingRead<'de> for HeartbeatFragSubmessageRead {
    fn mapping_read(_buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        todo!()
    }
}
