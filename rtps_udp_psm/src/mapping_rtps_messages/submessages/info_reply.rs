use rust_rtps_pim::{messages::submessages::InfoReplySubmessage, structure::types::Locator};

use crate::mapping_traits::{MappingRead, MappingWrite};

use std::io::{Error, Write};

impl MappingWrite for InfoReplySubmessage<Vec<Locator>> {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'de> MappingRead<'de> for InfoReplySubmessage<Vec<Locator>> {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
