use rust_rtps_pim::messages::{submessages::NackFragSubmessage, types::FragmentNumber};

use crate::mapping_traits::{MappingRead, MappingWrite};

use std::io::{Error, Write};

impl MappingWrite for NackFragSubmessage<Vec<FragmentNumber>> {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'de> MappingRead<'de> for NackFragSubmessage<Vec<FragmentNumber>> {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
