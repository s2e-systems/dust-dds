use rust_rtps_psm::messages::submessages::{DataFragSubmessageRead, DataFragSubmessageWrite};

use crate::{deserialize::MappingRead, serialize::MappingWrite};

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
