use super::super::{
    error::{RtpsMessageError, RtpsMessageResult},
    overall_structure::{
        Submessage, SubmessageHeaderRead, SubmessageHeaderWrite, TryReadFromBytes, Write,
        WriteIntoBytes,
    },
    submessage_elements::CryptoHeader,
    types::SubmessageKind,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SecurePrefixSubmessageRead {
    crypto_header: CryptoHeader,
}

impl SecurePrefixSubmessageRead {
    pub fn new(crypto_header: CryptoHeader) -> Self {
        Self { crypto_header }
    }

    pub fn try_from_bytes(
        submessage_header: &SubmessageHeaderRead,
        data: &[u8],
    ) -> RtpsMessageResult<Self> {
        if submessage_header.submessage_length() as usize > data.len() {
            return Err(RtpsMessageError::InvalidData);
        }
        let end_position = if submessage_header.submessage_length() == 0 {
            data.len()
        } else {
            submessage_header.submessage_length() as usize
        };
        let mut slice = &data[..end_position];
        let crypto_header =
            CryptoHeader::try_read_from_bytes(&mut slice, submessage_header.endianness())?;
        Ok(Self { crypto_header })
    }

    pub fn crypto_header(&self) -> &CryptoHeader {
        &self.crypto_header
    }
}

impl Submessage for SecurePrefixSubmessageRead {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(SubmessageKind::SEC_PREFIX, &[], octets_to_next_header)
            .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        self.crypto_header.write_into_bytes(buf);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SecurePrefixSubmessageWrite<'a> {
    crypto_header: &'a CryptoHeader,
}

impl<'a> SecurePrefixSubmessageWrite<'a> {
    pub fn new(crypto_header: &'a CryptoHeader) -> Self {
        Self { crypto_header }
    }

    pub fn crypto_header(&self) -> &CryptoHeader {
        self.crypto_header
    }
}

impl Submessage for SecurePrefixSubmessageWrite<'_> {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(SubmessageKind::SEC_PREFIX, &[], octets_to_next_header)
            .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        self.crypto_header.write_into_bytes(buf);
    }

    fn submessage_len(&self) -> usize {
        4 + self.crypto_header.size()
    }

    fn write_submessage_into_bytes(&self, buf: &mut [u8]) -> usize {
        let total_len = self.submessage_len();
        let octets_to_next_header = (total_len - 4) as u16;
        SubmessageHeaderWrite::new(SubmessageKind::SEC_PREFIX, &[], octets_to_next_header)
            .write_into_slice(&mut buf[0..4]);
        self.crypto_header.write_into_slice(&mut buf[4..total_len]);
        total_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps_messages::{
        overall_structure::write_submessage_into_bytes_vec,
        submessage_elements::{CryptoTransformIdentifier, Data},
    };
    use alloc::vec;

    #[test]
    fn serialize_secure_prefix() {
        let transform_id = CryptoTransformIdentifier::new([1, 2, 3, 4], [5, 6, 7, 8]);
        let extra = Data::new(vec![9, 10, 11, 12].into());
        let header = CryptoHeader::new(transform_id, extra);
        let submessage = SecurePrefixSubmessageRead::new(header);

        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x31, 0b_0000_0001, 12, 0, // Header: ID=0x31, LittleEndian, length=12
                1, 2, 3, 4, // transformation_kind
                5, 6, 7, 8, // transformation_key_id
                9, 10, 11, 12, // plugin_crypto_header_extra
            ]
        );
    }

    #[test]
    fn serialize_secure_prefix_write() {
        let transform_id = CryptoTransformIdentifier::new([1, 2, 3, 4], [5, 6, 7, 8]);
        let extra = Data::new(vec![9, 10, 11, 12].into());
        let header = CryptoHeader::new(transform_id, extra);
        let submessage = SecurePrefixSubmessageWrite::new(&header);

        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x31, 0b_0000_0001, 12, 0, // Header: ID=0x31, LittleEndian, length=12
                1, 2, 3, 4, // transformation_kind
                5, 6, 7, 8, // transformation_key_id
                9, 10, 11, 12, // plugin_crypto_header_extra
            ]
        );
    }

    #[test]
    fn deserialize_secure_prefix() {
        #[rustfmt::skip]
        let mut data = &[
            0x31, 0b_0000_0001, 12, 0, // Header
            1, 2, 3, 4,
            5, 6, 7, 8,
            9, 10, 11, 12,
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage = SecurePrefixSubmessageRead::try_from_bytes(&submessage_header, data).unwrap();

        let transform_id = CryptoTransformIdentifier::new([1, 2, 3, 4], [5, 6, 7, 8]);
        let extra = Data::new(vec![9, 10, 11, 12].into());
        let expected_header = CryptoHeader::new(transform_id, extra);
        assert_eq!(submessage.crypto_header(), &expected_header);
    }
}
