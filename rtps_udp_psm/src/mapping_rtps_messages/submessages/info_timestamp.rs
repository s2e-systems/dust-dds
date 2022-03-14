use rtps_pim::messages::submessages::InfoTimestampSubmessage;

use crate::mapping_traits::{MappingRead, MappingWrite};

use std::io::{Error, Write};

impl MappingWrite for InfoTimestampSubmessage {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'de> MappingRead<'de> for InfoTimestampSubmessage {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
