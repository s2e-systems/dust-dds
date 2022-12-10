use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::types::{Locator, LocatorKind, LocatorPort, LocatorAddress},
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

impl MappingWriteByteOrdered for LocatorKind {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(&self, writer: W) -> Result<(), Error> {
        todo!()
    }
}

impl MappingWriteByteOrdered for LocatorPort {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(&self, writer: W) -> Result<(), Error> {
        todo!()
    }
}

impl MappingWriteByteOrdered for LocatorAddress {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(&self, writer: W) -> Result<(), Error> {
        todo!()
    }
}

impl MappingWriteByteOrdered for Locator {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.kind()
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.port()
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.address()
            .mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for LocatorKind {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}

impl<'de> MappingReadByteOrdered<'de> for LocatorPort {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}

impl<'de> MappingReadByteOrdered<'de> for LocatorAddress {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}

impl<'de> MappingReadByteOrdered<'de> for Locator {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let kind = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let port = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let address = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        Ok(Self::new(kind, port, address))
    }
}

impl NumberOfBytes for Locator {
    fn number_of_bytes(&self) -> usize {
        24
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps_udp_psm::mapping_traits::{from_bytes_le, to_bytes_le};

    use super::*;

    #[test]
    fn serialize_locator() {
        let locator = Locator::new(1, 2, [3; 16]);
        assert_eq!(
            to_bytes_le(&locator).unwrap(),
            vec![
                1, 0, 0, 0, // kind (long)
                2, 0, 0, 0, // port (unsigned long)
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
            ]
        );
    }

    #[test]
    fn deserialize_locator() {
        let expected = Locator::new(1, 2, [3; 16]);
        #[rustfmt::skip]
        let result = from_bytes_le(&[
            1, 0, 0, 0, // kind (long)
            2, 0, 0, 0, // port (unsigned long)
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])

        ]).unwrap();
        assert_eq!(expected, result);
    }
}
