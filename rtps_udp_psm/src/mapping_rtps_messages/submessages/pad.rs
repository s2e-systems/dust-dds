use rust_rtps_psm::messages::submessages::{PadSubmessageRead, PadSubmessageWrite};

use crate::{deserialize::MappingRead, serialize::MappingWrite};

use std::io::{Error, Write};

impl MappingWrite for PadSubmessageWrite {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'de> MappingRead<'de> for PadSubmessageRead {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
