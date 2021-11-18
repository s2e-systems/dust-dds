use rust_rtps_psm::messages::submessages::{NackFragSubmessageRead, NackFragSubmessageWrite};

use crate::{deserialize::MappingRead, serialize::MappingWrite};

use std::io::{Error, Write};

impl MappingWrite for NackFragSubmessageWrite {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'de> MappingRead<'de> for NackFragSubmessageRead {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
