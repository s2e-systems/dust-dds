use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::types::{Locator, LocatorAddress, LocatorKind, LocatorPort},
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

impl MappingWriteByteOrdered for Locator {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        <i32>::from(self.kind()).mapping_write_byte_ordered::<_, B>(&mut writer)?;
        <u32>::from(self.port()).mapping_write_byte_ordered::<_, B>(&mut writer)?;
        <[u8; 16]>::from(self.address()).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for Locator {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let kind = LocatorKind::new(MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?);
        let port = LocatorPort::new(MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?);
        let address =
            LocatorAddress::new(MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?);
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
        let locator = Locator::new(
            LocatorKind::new(1),
            LocatorPort::new(2),
            LocatorAddress::new([3; 16]),
        );
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
        let expected = Locator::new(
            LocatorKind::new(1),
            LocatorPort::new(2),
            LocatorAddress::new([3; 16]),
        );
        let result = from_bytes_le(&[
            1, 0, 0, 0, // kind (long)
            2, 0, 0, 0, // port (unsigned long)
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
        ])
        .unwrap();
        assert_eq!(expected, result);
    }
}
