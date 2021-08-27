use rust_rtps_pim::messages::submessages::NackFragSubmessage;

use crate::{deserialize::{self, MappingRead}, serialize::{self, MappingWrite}};

use std::io::Write;


impl<F> MappingWrite for NackFragSubmessage<F> {
    fn write<W: Write>(&self, mut _writer: W) -> serialize::Result {
        todo!()
    }
}
impl<'de, F> MappingRead<'de> for NackFragSubmessage<F> {
    fn read(_buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        todo!()
    }
}