use crate::{
    mapping_traits::{MappingRead, MappingWrite},
    messages::submessages::{DataFragSubmessageRead, DataFragSubmessageWrite},
};

use std::io::{Error, Write};

impl MappingWrite for DataFragSubmessageWrite {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'a, 'de: 'a> MappingRead<'de> for DataFragSubmessageRead {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
