use crate::{
    mapping_traits::{MappingRead, MappingWrite},
    messages::submessages::{InfoTimestampSubmessageRead, InfoTimestampSubmessageWrite},
};

use std::io::{Error, Write};

impl MappingWrite for InfoTimestampSubmessageWrite {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'de> MappingRead<'de> for InfoTimestampSubmessageRead {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
