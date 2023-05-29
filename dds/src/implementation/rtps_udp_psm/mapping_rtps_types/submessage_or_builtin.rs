use crate::implementation::{
    rtps::types::Locator,
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

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

impl NumberOfBytes for Locator {
    fn number_of_bytes(&self) -> usize {
        24
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::types::{LocatorAddress, LocatorKind, LocatorPort},
        rtps_udp_psm::mapping_traits::to_bytes_le,
    };

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
}
