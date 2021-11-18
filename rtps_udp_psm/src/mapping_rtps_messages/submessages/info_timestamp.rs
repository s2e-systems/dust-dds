use rust_rtps_psm::messages::submessages::{
    InfoTimestampSubmessageRead, InfoTimestampSubmessageWrite,
};

use crate::{deserialize::MappingRead, serialize::MappingWrite};

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
