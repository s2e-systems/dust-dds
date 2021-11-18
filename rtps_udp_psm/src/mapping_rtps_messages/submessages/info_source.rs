use rust_rtps_psm::messages::submessages::{InfoSourceSubmessageRead, InfoSourceSubmessageWrite};

use crate::{
    deserialize::{self, MappingRead},
    serialize::{self, MappingWrite},
};

use std::io::Write;

impl MappingWrite for InfoSourceSubmessageWrite {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> serialize::Result {
        todo!()
    }
}
impl<'de> MappingRead<'de> for InfoSourceSubmessageRead {
    fn mapping_read(_buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        todo!()
    }
}
