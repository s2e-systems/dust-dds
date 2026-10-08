use super::super::{
    error::{RtpsMessageError, RtpsMessageResult},
    overall_structure::{
        Submessage, SubmessageHeaderRead, SubmessageHeaderWrite, Write, WriteIntoBytes,
    },
    submessage_elements::{CryptoFooter, Data},
    types::SubmessageKind,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SecurePostfixSubmessageRead {
    crypto_footer: CryptoFooter,
}

impl SecurePostfixSubmessageRead {
    pub fn new(crypto_footer: CryptoFooter) -> Self {
        Self { crypto_footer }
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
        let crypto_footer = CryptoFooter::new(Data::new(data[..end_position].into()));
        Ok(Self { crypto_footer })
    }

    pub fn crypto_footer(&self) -> &CryptoFooter {
        &self.crypto_footer
    }
}

impl Submessage for SecurePostfixSubmessageRead {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(SubmessageKind::SEC_POSTFIX, &[], octets_to_next_header)
            .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        self.crypto_footer.write_into_bytes(buf);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SecurePostfixSubmessageWrite<'a> {
    crypto_footer: &'a CryptoFooter,
}

impl<'a> SecurePostfixSubmessageWrite<'a> {
    pub fn new(crypto_footer: &'a CryptoFooter) -> Self {
        Self { crypto_footer }
    }

    pub fn crypto_footer(&self) -> &CryptoFooter {
        self.crypto_footer
    }
}

impl Submessage for SecurePostfixSubmessageWrite<'_> {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(SubmessageKind::SEC_POSTFIX, &[], octets_to_next_header)
            .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        self.crypto_footer.write_into_bytes(buf);
    }

    fn submessage_len(&self) -> usize {
        4 + self.crypto_footer.as_ref().len()
    }

    fn write_submessage_into_bytes(&self, buf: &mut [u8]) -> usize {
        let footer_slice = self.crypto_footer.as_ref();
        let total_len = 4 + footer_slice.len();
        let octets_to_next_header = footer_slice.len() as u16;
        SubmessageHeaderWrite::new(SubmessageKind::SEC_POSTFIX, &[], octets_to_next_header)
            .write_into_slice(&mut buf[0..4]);
        buf[4..total_len].copy_from_slice(footer_slice);
        total_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps_messages::overall_structure::write_submessage_into_bytes_vec;
    use alloc::vec;

    #[test]
    fn serialize_secure_postfix() {
        let footer = CryptoFooter::new(Data::new(vec![0xaa, 0xbb, 0xcc, 0xdd].into()));
        let submessage = SecurePostfixSubmessageRead::new(footer);

        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x32, 0b_0000_0001, 4, 0, // Header: ID=0x32, LittleEndian, length=4
                0xaa, 0xbb, 0xcc, 0xdd, // crypto_footer
            ]
        );
    }

    #[test]
    fn serialize_secure_postfix_write() {
        let footer = CryptoFooter::new(Data::new(vec![0xaa, 0xbb, 0xcc, 0xdd].into()));
        let submessage = SecurePostfixSubmessageWrite::new(&footer);

        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x32, 0b_0000_0001, 4, 0, // Header: ID=0x32, LittleEndian, length=4
                0xaa, 0xbb, 0xcc, 0xdd, // crypto_footer
            ]
        );
    }

    #[test]
    fn deserialize_secure_postfix() {
        #[rustfmt::skip]
        let mut data = &[
            0x32, 0b_0000_0001, 4, 0, // Header
            0xaa, 0xbb, 0xcc, 0xdd,
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            SecurePostfixSubmessageRead::try_from_bytes(&submessage_header, data).unwrap();

        let expected_footer = CryptoFooter::new(Data::new(vec![0xaa, 0xbb, 0xcc, 0xdd].into()));
        assert_eq!(submessage.crypto_footer(), &expected_footer);
    }
}
