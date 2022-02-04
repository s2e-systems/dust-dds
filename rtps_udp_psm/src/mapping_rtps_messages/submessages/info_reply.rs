use crate::{
    mapping_traits::{MappingRead, MappingWrite},
    messages::submessages::{InfoReplySubmessageRead, InfoReplySubmessageWrite},
};

use std::io::{Error, Write};

impl MappingWrite for InfoReplySubmessageWrite {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'de> MappingRead<'de> for InfoReplySubmessageRead {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
