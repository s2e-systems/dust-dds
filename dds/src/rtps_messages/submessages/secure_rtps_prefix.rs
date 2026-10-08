use super::super::{
    error::{RtpsMessageError, RtpsMessageResult},
    overall_structure::{
        Submessage, SubmessageHeaderRead, SubmessageHeaderWrite, TryReadFromBytes, Write,
        WriteIntoBytes,
    },
    submessage_elements::CryptoHeader,
    types::{SubmessageFlag, SubmessageKind},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SecureRTPSPrefixSubmessageRead {
    additional_authenticated_data_flag: SubmessageFlag,
    pre_shared_key_flag: SubmessageFlag,
    crypto_header: CryptoHeader,
}

impl SecureRTPSPrefixSubmessageRead {
    pub fn new(
        additional_authenticated_data_flag: SubmessageFlag,
        pre_shared_key_flag: SubmessageFlag,
        crypto_header: CryptoHeader,
    ) -> Self {
        Self {
            additional_authenticated_data_flag,
            pre_shared_key_flag,
            crypto_header,
        }
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
        let flags = submessage_header.flags();
        let additional_authenticated_data_flag = flags[1];
        let pre_shared_key_flag = flags[2];

        let mut slice = &data[..end_position];
        let crypto_header =
            CryptoHeader::try_read_from_bytes(&mut slice, submessage_header.endianness())?;

        Ok(Self {
            additional_authenticated_data_flag,
            pre_shared_key_flag,
            crypto_header,
        })
    }

    pub fn additional_authenticated_data_flag(&self) -> bool {
        self.additional_authenticated_data_flag
    }

    pub fn pre_shared_key_flag(&self) -> bool {
        self.pre_shared_key_flag
    }

    pub fn crypto_header(&self) -> &CryptoHeader {
        &self.crypto_header
    }
}

impl Submessage for SecureRTPSPrefixSubmessageRead {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(
            SubmessageKind::SRTPS_PREFIX,
            &[
                self.additional_authenticated_data_flag,
                self.pre_shared_key_flag,
            ],
            octets_to_next_header,
        )
        .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        self.crypto_header.write_into_bytes(buf);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SecureRTPSPrefixSubmessageWrite<'a> {
    additional_authenticated_data_flag: SubmessageFlag,
    pre_shared_key_flag: SubmessageFlag,
    crypto_header: &'a CryptoHeader,
}

impl<'a> SecureRTPSPrefixSubmessageWrite<'a> {
    pub fn new(
        additional_authenticated_data_flag: SubmessageFlag,
        pre_shared_key_flag: SubmessageFlag,
        crypto_header: &'a CryptoHeader,
    ) -> Self {
        Self {
            additional_authenticated_data_flag,
            pre_shared_key_flag,
            crypto_header,
        }
    }

    pub fn additional_authenticated_data_flag(&self) -> bool {
        self.additional_authenticated_data_flag
    }

    pub fn pre_shared_key_flag(&self) -> bool {
        self.pre_shared_key_flag
    }

    pub fn crypto_header(&self) -> &CryptoHeader {
        self.crypto_header
    }
}

impl Submessage for SecureRTPSPrefixSubmessageWrite<'_> {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(
            SubmessageKind::SRTPS_PREFIX,
            &[
                self.additional_authenticated_data_flag,
                self.pre_shared_key_flag,
            ],
            octets_to_next_header,
        )
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
        SubmessageHeaderWrite::new(
            SubmessageKind::SRTPS_PREFIX,
            &[
                self.additional_authenticated_data_flag,
                self.pre_shared_key_flag,
            ],
            octets_to_next_header,
        )
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
    fn serialize_secure_rtps_prefix_no_flags() {
        let transform_id = CryptoTransformIdentifier::new([1, 2, 3, 4], [5, 6, 7, 8]);
        let extra = Data::new(vec![9, 10, 11, 12].into());
        let header = CryptoHeader::new(transform_id, extra);
        let submessage = SecureRTPSPrefixSubmessageRead::new(false, false, header);

        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x33, 0b_0000_0001, 12, 0, // Header: ID=0x33, LittleEndian, length=12
                1, 2, 3, 4,
                5, 6, 7, 8,
                9, 10, 11, 12,
            ]
        );
    }

    #[test]
    fn serialize_secure_rtps_prefix_write() {
        let transform_id = CryptoTransformIdentifier::new([1, 2, 3, 4], [5, 6, 7, 8]);
        let extra = Data::new(vec![9, 10, 11, 12].into());
        let header = CryptoHeader::new(transform_id, extra);
        let submessage = SecureRTPSPrefixSubmessageWrite::new(false, false, &header);

        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x33, 0b_0000_0001, 12, 0, // Header: ID=0x33, LittleEndian, length=12
                1, 2, 3, 4,
                5, 6, 7, 8,
                9, 10, 11, 12,
            ]
        );
    }

    #[test]
    fn serialize_secure_rtps_prefix_with_flags() {
        let transform_id = CryptoTransformIdentifier::new([1, 2, 3, 4], [5, 6, 7, 8]);
        let extra = Data::new(vec![9, 10, 11, 12].into());
        let header = CryptoHeader::new(transform_id, extra);
        let submessage = SecureRTPSPrefixSubmessageRead::new(true, true, header);

        // flags: bit 0 (E=1), bit 1 (A=1), bit 2 (P=1) -> 0b_0000_0111 = 0x07
        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x33, 0b_0000_0111, 12, 0, // Header: ID=0x33, flags=0x07, length=12
                1, 2, 3, 4,
                5, 6, 7, 8,
                9, 10, 11, 12,
            ]
        );
    }

    #[test]
    fn deserialize_secure_rtps_prefix() {
        #[rustfmt::skip]
        let mut data = &[
            0x33, 0b_0000_0111, 12, 0, // Header
            1, 2, 3, 4,
            5, 6, 7, 8,
            9, 10, 11, 12,
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            SecureRTPSPrefixSubmessageRead::try_from_bytes(&submessage_header, data).unwrap();

        assert!(submessage.additional_authenticated_data_flag());
        assert!(submessage.pre_shared_key_flag());
        let transform_id = CryptoTransformIdentifier::new([1, 2, 3, 4], [5, 6, 7, 8]);
        let extra = Data::new(vec![9, 10, 11, 12].into());
        let expected_header = CryptoHeader::new(transform_id, extra);
        assert_eq!(submessage.crypto_header(), &expected_header);
    }
}
