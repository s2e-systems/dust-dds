use crate::{
    implementation::rtps::{
        messages::{
            overall_structure::{SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite},
            types::SubmessageKind,
        },
        types::WriteIntoBytes,
    },
    infrastructure::error::DdsResult,
};

#[derive(Debug, PartialEq, Eq)]
pub struct PadSubmessageRead {}

impl PadSubmessageRead {
    pub fn try_from_bytes(
        _submessage_header: &SubmessageHeaderRead,
        _data: &[u8],
    ) -> DdsResult<Self> {
        Ok(Self {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PadSubmessageWrite {}

impl PadSubmessageWrite {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for PadSubmessageWrite {
    fn default() -> Self {
        Self::new()
    }
}

impl SubmessageHeader for &PadSubmessageWrite {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(SubmessageKind::PAD, &[], octets_to_next_header)
    }
}

impl WriteIntoBytes for &PadSubmessageWrite {
    fn write_into_bytes(&self, _buf: &mut &mut [u8]) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::messages::overall_structure::{
        into_bytes_vec, RtpsSubmessageWriteKind, SubmessageHeaderRead,
    };

    #[test]
    fn serialize_pad() {
        let submessage = RtpsSubmessageWriteKind::Pad(PadSubmessageWrite::new());
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
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
