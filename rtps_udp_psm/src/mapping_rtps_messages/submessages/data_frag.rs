use rtps_pim::messages::{submessage_elements::Parameter, submessages::DataFragSubmessage};

use crate::mapping_traits::{MappingRead, MappingWrite};

use std::io::{Error, Write};

impl MappingWrite for DataFragSubmessage<Vec<Parameter<'_>>, &'_ [u8]> {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'a, 'de: 'a> MappingRead<'de> for DataFragSubmessage<Vec<Parameter<'_>>, &'_ [u8]> {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
