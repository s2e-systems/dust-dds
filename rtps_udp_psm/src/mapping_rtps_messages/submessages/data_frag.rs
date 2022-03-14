use rtps_pim::messages::{submessage_elements::Parameter, submessages::DataFragSubmessage};

use crate::mapping_traits::{MappingRead, MappingWrite};

use std::io::{Error, Write};

impl MappingWrite for DataFragSubmessage<'_, Vec<Parameter<'_>>> {
    fn mapping_write<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'a, 'de: 'a> MappingRead<'de> for DataFragSubmessage<'a, Vec<Parameter<'a>>> {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
