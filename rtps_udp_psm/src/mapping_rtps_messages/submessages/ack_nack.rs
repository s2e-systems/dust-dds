use rust_rtps_psm::messages::submessages::{AckNackSubmessageRead, AckNackSubmessageWrite};

use crate::{mapping_traits::{MappingRead, MappingWrite}};

use std::io::{Error, Write};

impl MappingWrite for AckNackSubmessageWrite {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'de> MappingRead<'de> for AckNackSubmessageRead {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
