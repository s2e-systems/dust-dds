use super::super::{
    error::{RtpsMessageError, RtpsMessageResult},
    overall_structure::{
        Submessage, SubmessageHeaderRead, SubmessageHeaderWrite, Write, WriteIntoBytes,
    },
    submessage_elements::{CryptoContent, Data},
    types::SubmessageKind,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SecureBodySubmessageRead {
    crypto_content: CryptoContent,
}

impl SecureBodySubmessageRead {
    pub fn new(crypto_content: CryptoContent) -> Self {
        Self { crypto_content }
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
        let crypto_content = CryptoContent::new(Data::new(data[..end_position].into()));
        Ok(Self { crypto_content })
    }

    pub fn crypto_content(&self) -> &CryptoContent {
        &self.crypto_content
    }
}

impl Submessage for SecureBodySubmessageRead {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(SubmessageKind::SEC_BODY, &[], octets_to_next_header)
            .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        self.crypto_content.write_into_bytes(buf);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SecureBodySubmessageWrite<'a> {
    crypto_content: &'a CryptoContent,
}

impl<'a> SecureBodySubmessageWrite<'a> {
    pub fn new(crypto_content: &'a CryptoContent) -> Self {
        Self { crypto_content }
    }

    pub fn crypto_content(&self) -> &CryptoContent {
        self.crypto_content
    }
}

impl Submessage for SecureBodySubmessageWrite<'_> {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(SubmessageKind::SEC_BODY, &[], octets_to_next_header)
            .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        self.crypto_content.write_into_bytes(buf);
    }

    fn submessage_len(&self) -> usize {
        4 + self.crypto_content.as_ref().len()
    }

    fn write_submessage_into_bytes(&self, buf: &mut [u8]) -> usize {
        let content_slice = self.crypto_content.as_ref();
        let total_len = 4 + content_slice.len();
        let octets_to_next_header = content_slice.len() as u16;
        SubmessageHeaderWrite::new(SubmessageKind::SEC_BODY, &[], octets_to_next_header)
            .write_into_slice(&mut buf[0..4]);
        buf[4..total_len].copy_from_slice(content_slice);
        total_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps_messages::overall_structure::write_submessage_into_bytes_vec;
    use alloc::vec;

    #[test]
    fn serialize_secure_body() {
        let content = CryptoContent::new(Data::new(vec![0x10, 0x20, 0x30, 0x40].into()));
        let submessage = SecureBodySubmessageRead::new(content);

        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x30, 0b_0000_0001, 4, 0, // Header: ID=0x30, LittleEndian, length=4
                0x10, 0x20, 0x30, 0x40, // crypto_content
            ]
        );
    }

    #[test]
    fn serialize_secure_body_write() {
        let content = CryptoContent::new(Data::new(vec![0x10, 0x20, 0x30, 0x40].into()));
        let submessage = SecureBodySubmessageWrite::new(&content);

        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x30, 0b_0000_0001, 4, 0, // Header: ID=0x30, LittleEndian, length=4
                0x10, 0x20, 0x30, 0x40, // crypto_content
            ]
        );
    }

    #[test]
    fn deserialize_secure_body() {
        #[rustfmt::skip]
        let mut data = &[
            0x30, 0b_0000_0001, 4, 0, // Header
            0x10, 0x20, 0x30, 0x40,
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            SecureBodySubmessageRead::try_from_bytes(&submessage_header, data).unwrap();

        let expected_content = CryptoContent::new(Data::new(vec![0x10, 0x20, 0x30, 0x40].into()));
        assert_eq!(submessage.crypto_content(), &expected_content);
    }
}
