use super::super::{
    error::RtpsMessageResult,
    overall_structure::{
        Submessage, SubmessageHeaderRead, SubmessageHeaderWrite, Write, WriteIntoBytes,
    },
    types::SubmessageKind,
};

#[derive(Debug, PartialEq, Eq)]
pub struct PadSubmessageRead {}

impl PadSubmessageRead {
    pub fn try_from_bytes(
        _submessage_header: &SubmessageHeaderRead,
        _data: &[u8],
    ) -> RtpsMessageResult<Self> {
        Ok(Self {})
    }
}

impl PadSubmessageRead {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for PadSubmessageRead {
    fn default() -> Self {
        Self::new()
    }
}

impl Submessage for PadSubmessageRead {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(SubmessageKind::PAD, &[], octets_to_next_header)
            .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, _buf: &mut dyn Write) {}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct PadSubmessageWrite {}

impl PadSubmessageWrite {
    pub fn new() -> Self {
        Self {}
    }
}

impl Submessage for PadSubmessageWrite {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(SubmessageKind::PAD, &[], octets_to_next_header)
            .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, _buf: &mut dyn Write) {}

    fn submessage_len(&self) -> usize {
        4
    }

    fn write_submessage_into_bytes(&self, buf: &mut [u8]) -> usize {
        SubmessageHeaderWrite::new(SubmessageKind::PAD, &[], 0).write_into_slice(&mut buf[0..4]);
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps_messages::overall_structure::{
        SubmessageHeaderRead, write_submessage_into_bytes_vec,
    };

    #[test]
    fn serialize_pad() {
        let submessage = PadSubmessageRead::new();
        #[rustfmt::skip]
        assert_eq!(write_submessage_into_bytes_vec(&submessage), vec![
                0x01, 0b_0000_0001, 0, 0, // Submessage header
            ]
        );
    }

    #[test]
    fn serialize_pad_write() {
        let submessage = PadSubmessageWrite::new();
        #[rustfmt::skip]
        assert_eq!(write_submessage_into_bytes_vec(&submessage), vec![
                0x01, 0b_0000_0001, 0, 0, // Submessage header
            ]
        );
    }

    #[test]
    fn deserialize_pad() {
        #[rustfmt::skip]
        let mut data = &[
            0x01, 0b_0000_0001, 0, 0, // Submessage header
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage = PadSubmessageRead::try_from_bytes(&submessage_header, data);

        assert!(submessage.is_ok())
    }
}
