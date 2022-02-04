use crate::{
    mapping_traits::{MappingRead, MappingWrite},
    messages::submessages::{InfoDestinationSubmessageRead, InfoDestinationSubmessageWrite},
};

use std::io::{Error, Write};

impl MappingWrite for InfoDestinationSubmessageWrite {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'de> MappingRead<'de> for InfoDestinationSubmessageRead {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
