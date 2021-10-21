use rust_rtps_pim::messages::submessages::DataFragSubmessage;

use crate::{deserialize::{self, MappingRead}, serialize::{self, MappingWrite}};

use std::io::Write;


impl<S> MappingWrite for DataFragSubmessage<S, &[u8]> {
    fn write<W: Write>(&self, mut _writer: W) -> serialize::Result {
        todo!()
    }
}
impl<'a, 'de: 'a, S> MappingRead<'de> for DataFragSubmessage<S, &'a [u8]> {
    fn read(_buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        todo!()
    }
}