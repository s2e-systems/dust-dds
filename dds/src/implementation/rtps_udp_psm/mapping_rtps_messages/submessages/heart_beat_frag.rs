use std::io::{Error, Write};

use crate::implementation::{
    rtps::messages::submessages::HeartbeatFragSubmessage,
    rtps_udp_psm::mapping_traits::{MappingRead, MappingWrite},
};

impl MappingWrite for HeartbeatFragSubmessage {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'de> MappingRead<'de> for HeartbeatFragSubmessage {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
