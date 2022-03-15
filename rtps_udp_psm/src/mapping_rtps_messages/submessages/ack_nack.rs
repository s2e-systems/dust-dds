use rtps_pim::messages::submessages::AckNackSubmessage;

use crate::mapping_traits::{MappingRead, MappingWrite};

use std::io::{Error, Write};

impl<S> MappingWrite for AckNackSubmessage<S> {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'de, S> MappingRead<'de> for AckNackSubmessage<S> {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
