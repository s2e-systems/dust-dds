use super::super::{
    error::RtpsMessageResult,
    overall_structure::{
        Read, Submessage, SubmessageHeaderRead, SubmessageHeaderWrite, TryReadFromBytes, Write,
        WriteIntoBytes,
    },
    submessage_elements::{Checksum, ParameterList},
    types::{
        MESSAGE_LENGTH_INVALID, MessageLength, SubmessageFlag, SubmessageKind, TIME_INVALID, Time,
        UExtension4, WExtension8,
    },
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HeaderExtensionSubmessageRead {
    length_flag: SubmessageFlag,
    timestamp_flag: SubmessageFlag,
    u_extension_flag: SubmessageFlag,
    w_extension_flag: SubmessageFlag,
    c1_flag: SubmessageFlag,
    c2_flag: SubmessageFlag,
    parameters_flag: SubmessageFlag,
    message_length: MessageLength,
    rtps_send_timestamp: Time,
    u_extension4: UExtension4,
    w_extension8: WExtension8,
    message_checksum: Checksum,
    parameters: ParameterList,
}

impl HeaderExtensionSubmessageRead {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        length_flag: SubmessageFlag,
        timestamp_flag: SubmessageFlag,
        u_extension_flag: SubmessageFlag,
        w_extension_flag: SubmessageFlag,
        c1_flag: SubmessageFlag,
        c2_flag: SubmessageFlag,
        parameters_flag: SubmessageFlag,
        message_length: MessageLength,
        rtps_send_timestamp: Time,
        u_extension4: UExtension4,
        w_extension8: WExtension8,
        message_checksum: Checksum,
        parameters: ParameterList,
    ) -> Self {
        Self {
            length_flag,
            timestamp_flag,
            u_extension_flag,
            w_extension_flag,
            c1_flag,
            c2_flag,
            parameters_flag,
            message_length,
            rtps_send_timestamp,
            u_extension4,
            w_extension8,
            message_checksum,
            parameters,
        }
    }

    pub fn try_from_bytes(
        submessage_header: &SubmessageHeaderRead,
        mut data: &[u8],
    ) -> RtpsMessageResult<Self> {
        let endianness = submessage_header.endianness();
        let flags = submessage_header.flags();
        let length_flag = flags[1];
        let timestamp_flag = flags[2];
        let u_extension_flag = flags[3];
        let w_extension_flag = flags[4];
        let c1_flag = flags[5];
        let c2_flag = flags[6];
        let parameters_flag = flags[7];

        let message_length = if length_flag {
            MessageLength::try_read_from_bytes(&mut data, endianness)?
        } else {
            MESSAGE_LENGTH_INVALID
        };

        let rtps_send_timestamp = if timestamp_flag {
            Time::try_read_from_bytes(&mut data, endianness)?
        } else {
            TIME_INVALID
        };

        let u_extension4 = if u_extension_flag {
            let mut bytes = [0; 4];
            data.read_exact(&mut bytes)?;
            bytes
        } else {
            [0; 4]
        };

        let w_extension8 = if w_extension_flag {
            let mut bytes = [0; 8];
            data.read_exact(&mut bytes)?;
            bytes
        } else {
            [0; 8]
        };

        let message_checksum = Checksum::try_read_from_bytes(&mut data, c1_flag, c2_flag)?;

        let parameters = if parameters_flag {
            ParameterList::try_read_from_bytes(&mut data, endianness)?
        } else {
            ParameterList::empty()
        };

        Ok(Self {
            length_flag,
            timestamp_flag,
            u_extension_flag,
            w_extension_flag,
            c1_flag,
            c2_flag,
            parameters_flag,
            message_length,
            rtps_send_timestamp,
            u_extension4,
            w_extension8,
            message_checksum,
            parameters,
        })
    }

    pub fn length_flag(&self) -> bool {
        self.length_flag
    }

    pub fn timestamp_flag(&self) -> bool {
        self.timestamp_flag
    }

    pub fn u_extension_flag(&self) -> bool {
        self.u_extension_flag
    }

    pub fn w_extension_flag(&self) -> bool {
        self.w_extension_flag
    }

    pub fn c1_flag(&self) -> bool {
        self.c1_flag
    }

    pub fn c2_flag(&self) -> bool {
        self.c2_flag
    }

    pub fn checksum_flags(&self) -> (bool, bool) {
        (self.c1_flag, self.c2_flag)
    }

    pub fn parameters_flag(&self) -> bool {
        self.parameters_flag
    }

    pub fn message_length(&self) -> MessageLength {
        self.message_length
    }

    pub fn rtps_send_timestamp(&self) -> Time {
        self.rtps_send_timestamp
    }

    pub fn u_extension4(&self) -> UExtension4 {
        self.u_extension4
    }

    pub fn w_extension8(&self) -> WExtension8 {
        self.w_extension8
    }

    pub fn message_checksum(&self) -> Checksum {
        self.message_checksum
    }

    pub fn parameters(&self) -> &ParameterList {
        &self.parameters
    }
}

impl Submessage for HeaderExtensionSubmessageRead {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(
            SubmessageKind::RTPS_HE,
            &[
                self.length_flag,
                self.timestamp_flag,
                self.u_extension_flag,
                self.w_extension_flag,
                self.c1_flag,
                self.c2_flag,
                self.parameters_flag,
            ],
            octets_to_next_header,
        )
        .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        if self.length_flag {
            self.message_length.write_into_bytes(buf);
        }
        if self.timestamp_flag {
            self.rtps_send_timestamp.write_into_bytes(buf);
        }
        if self.u_extension_flag {
            self.u_extension4.write_into_bytes(buf);
        }
        if self.w_extension_flag {
            self.w_extension8.write_into_bytes(buf);
        }
        if self.c1_flag || self.c2_flag {
            self.message_checksum.write_into_bytes(buf);
        }
        if self.parameters_flag {
            self.parameters.write_into_bytes(buf);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HeaderExtensionSubmessageWrite<'a> {
    length_flag: SubmessageFlag,
    timestamp_flag: SubmessageFlag,
    u_extension_flag: SubmessageFlag,
    w_extension_flag: SubmessageFlag,
    c1_flag: SubmessageFlag,
    c2_flag: SubmessageFlag,
    parameters_flag: SubmessageFlag,
    message_length: MessageLength,
    rtps_send_timestamp: Time,
    u_extension4: UExtension4,
    w_extension8: WExtension8,
    message_checksum: Checksum,
    parameters: &'a ParameterList,
}

impl<'a> HeaderExtensionSubmessageWrite<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        length_flag: SubmessageFlag,
        timestamp_flag: SubmessageFlag,
        u_extension_flag: SubmessageFlag,
        w_extension_flag: SubmessageFlag,
        c1_flag: SubmessageFlag,
        c2_flag: SubmessageFlag,
        parameters_flag: SubmessageFlag,
        message_length: MessageLength,
        rtps_send_timestamp: Time,
        u_extension4: UExtension4,
        w_extension8: WExtension8,
        message_checksum: Checksum,
        parameters: &'a ParameterList,
    ) -> Self {
        Self {
            length_flag,
            timestamp_flag,
            u_extension_flag,
            w_extension_flag,
            c1_flag,
            c2_flag,
            parameters_flag,
            message_length,
            rtps_send_timestamp,
            u_extension4,
            w_extension8,
            message_checksum,
            parameters,
        }
    }
}

impl Submessage for HeaderExtensionSubmessageWrite<'_> {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(
            SubmessageKind::RTPS_HE,
            &[
                self.length_flag,
                self.timestamp_flag,
                self.u_extension_flag,
                self.w_extension_flag,
                self.c1_flag,
                self.c2_flag,
                self.parameters_flag,
            ],
            octets_to_next_header,
        )
        .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        if self.length_flag {
            self.message_length.write_into_bytes(buf);
        }
        if self.timestamp_flag {
            self.rtps_send_timestamp.write_into_bytes(buf);
        }
        if self.u_extension_flag {
            self.u_extension4.write_into_bytes(buf);
        }
        if self.w_extension_flag {
            self.w_extension8.write_into_bytes(buf);
        }
        if self.c1_flag || self.c2_flag {
            self.message_checksum.write_into_bytes(buf);
        }
        if self.parameters_flag {
            self.parameters.write_into_bytes(buf);
        }
    }

    fn submessage_len(&self) -> usize {
        let mut len = 4;
        if self.length_flag {
            len += 4;
        }
        if self.timestamp_flag {
            len += 8;
        }
        if self.u_extension_flag {
            len += 4;
        }
        if self.w_extension_flag {
            len += 8;
        }
        if self.c1_flag || self.c2_flag {
            len += self.message_checksum.size();
        }
        if self.parameters_flag {
            len += self.parameters.size();
        }
        len
    }

    fn write_submessage_into_bytes(&self, buf: &mut [u8]) -> usize {
        let total_len = self.submessage_len();
        let octets_to_next_header = (total_len - 4) as u16;
        SubmessageHeaderWrite::new(
            SubmessageKind::RTPS_HE,
            &[
                self.length_flag,
                self.timestamp_flag,
                self.u_extension_flag,
                self.w_extension_flag,
                self.c1_flag,
                self.c2_flag,
                self.parameters_flag,
            ],
            octets_to_next_header,
        )
        .write_into_slice(&mut buf[0..4]);
        let mut offset = 4;
        if self.length_flag {
            buf[offset..offset + 4].copy_from_slice(&self.message_length.to_le_bytes());
            offset += 4;
        }
        if self.timestamp_flag {
            self.rtps_send_timestamp
                .write_into_slice(&mut buf[offset..offset + 8]);
            offset += 8;
        }
        if self.u_extension_flag {
            buf[offset..offset + 4].copy_from_slice(&self.u_extension4);
            offset += 4;
        }
        if self.w_extension_flag {
            buf[offset..offset + 8].copy_from_slice(&self.w_extension8);
            offset += 8;
        }
        if self.c1_flag || self.c2_flag {
            offset += self.message_checksum.write_into_slice(&mut buf[offset..]);
        }
        if self.parameters_flag {
            offset += self.parameters.write_into_slice(&mut buf[offset..]);
        }
        offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps_messages::{
        overall_structure::write_submessage_into_bytes_vec, submessage_elements::Parameter,
    };
    use alloc::vec;

    #[test]
    fn serialize_header_extension_no_flags() {
        let submessage = HeaderExtensionSubmessageRead::new(
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            MESSAGE_LENGTH_INVALID,
            TIME_INVALID,
            [0; 4],
            [0; 8],
            Checksum::None,
            ParameterList::empty(),
        );

        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x00, 0b_0000_0001, 0, 0, // Submessage header: ID=0x00, LittleEndian
            ]
        );
    }

    #[test]
    fn serialize_header_extension_all_flags_checksum32() {
        let parameter = Parameter::new(0x0005, vec![1, 2, 3, 4].into());
        let parameters = ParameterList::new(vec![parameter]);
        let submessage = HeaderExtensionSubmessageRead::new(
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            128,
            Time::new(10, 20),
            [1, 2, 3, 4],
            [5, 6, 7, 8, 9, 10, 11, 12],
            Checksum::Checksum32([0xaa, 0xbb, 0xcc, 0xdd]),
            parameters,
        );

        // flags:
        // bit 0: Endianness (1 = LittleEndian)
        // bit 1: LengthFlag (1)
        // bit 2: TimestampFlag (1)
        // bit 3: UExtensionFlag (1)
        // bit 4: WExtensionFlag (1)
        // bit 5: C1 (0)
        // bit 6: C2 (1)
        // bit 7: ParametersFlag (1)
        // binary: 1101_1111 = 0xdf
        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x00, 0b_1101_1111, 40, 0, // Submessage header: ID=0, flags=0xdf, length=40
                128, 0, 0, 0, // messageLength
                10, 0, 0, 0, 20, 0, 0, 0, // rtpsSendTimestamp (seconds, fraction)
                1, 2, 3, 4, // uExtension4
                5, 6, 7, 8, 9, 10, 11, 12, // wExtension8
                0xaa, 0xbb, 0xcc, 0xdd, // messageChecksum (Checksum32)
                0x05, 0x00, 4, 0, // parameter: ID=0x0005, length=4
                1, 2, 3, 4, // parameter value
                0x01, 0x00, 0, 0, // PID_SENTINEL, length=0
            ]
        );
    }

    #[test]
    fn serialize_header_extension_write_all_flags_checksum32() {
        let parameter = Parameter::new(0x0005, vec![1, 2, 3, 4].into());
        let parameters = ParameterList::new(vec![parameter]);
        let submessage = HeaderExtensionSubmessageWrite::new(
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            128,
            Time::new(10, 20),
            [1, 2, 3, 4],
            [5, 6, 7, 8, 9, 10, 11, 12],
            Checksum::Checksum32([0xaa, 0xbb, 0xcc, 0xdd]),
            &parameters,
        );

        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x00, 0b_1101_1111, 40, 0, // Submessage header: ID=0, flags=0xdf, length=40
                128, 0, 0, 0, // messageLength
                10, 0, 0, 0, 20, 0, 0, 0, // rtpsSendTimestamp (seconds, fraction)
                1, 2, 3, 4, // uExtension4
                5, 6, 7, 8, 9, 10, 11, 12, // wExtension8
                0xaa, 0xbb, 0xcc, 0xdd, // messageChecksum (Checksum32)
                0x05, 0x00, 4, 0, // parameter: ID=0x0005, length=4
                1, 2, 3, 4, // parameter value
                0x01, 0x00, 0, 0, // PID_SENTINEL, length=0
            ]
        );
    }

    #[test]
    fn serialize_header_extension_checksum64() {
        let submessage = HeaderExtensionSubmessageRead::new(
            false,
            false,
            false,
            false,
            true,
            false,
            false,
            MESSAGE_LENGTH_INVALID,
            TIME_INVALID,
            [0; 4],
            [0; 8],
            Checksum::Checksum64([1, 2, 3, 4, 5, 6, 7, 8]),
            ParameterList::empty(),
        );

        // flags:
        // bit 0: Endianness (1)
        // bit 5: C1 (1)
        // binary: 0010_0001 = 0x21
        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x00, 0b_0010_0001, 8, 0, // Submessage header
                1, 2, 3, 4, 5, 6, 7, 8, // Checksum64
            ]
        );
    }

    #[test]
    fn serialize_header_extension_checksum128() {
        let submessage = HeaderExtensionSubmessageRead::new(
            false,
            false,
            false,
            false,
            true,
            true,
            false,
            MESSAGE_LENGTH_INVALID,
            TIME_INVALID,
            [0; 4],
            [0; 8],
            Checksum::Checksum128([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
            ParameterList::empty(),
        );

        // flags:
        // bit 0: Endianness (1)
        // bit 5: C1 (1)
        // bit 6: C2 (1)
        // binary: 0110_0001 = 0x61
        #[rustfmt::skip]
        assert_eq!(
            write_submessage_into_bytes_vec(&submessage),
            vec![
                0x00, 0b_0110_0001, 16, 0, // Submessage header
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // Checksum128
            ]
        );
    }

    #[test]
    fn deserialize_header_extension_no_flags() {
        #[rustfmt::skip]
        let mut data = &[
            0x00_u8, 0b_0000_0001, 0, 0, // Submessage header
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            HeaderExtensionSubmessageRead::try_from_bytes(&submessage_header, data).unwrap();

        assert!(!submessage.length_flag());
        assert!(!submessage.timestamp_flag());
        assert!(!submessage.u_extension_flag());
        assert!(!submessage.w_extension_flag());
        assert_eq!(submessage.checksum_flags(), (false, false));
        assert!(!submessage.parameters_flag());
        assert_eq!(submessage.message_checksum(), Checksum::None);
        assert_eq!(submessage.parameters(), &ParameterList::empty());
    }

    #[test]
    fn deserialize_header_extension_all_flags_checksum32() {
        #[rustfmt::skip]
        let mut data = &[
            0x00_u8, 0b_1101_1111, 40, 0, // Submessage header
            128, 0, 0, 0, // messageLength
            10, 0, 0, 0, 20, 0, 0, 0, // rtpsSendTimestamp
            1, 2, 3, 4, // uExtension4
            5, 6, 7, 8, 9, 10, 11, 12, // wExtension8
            0xaa, 0xbb, 0xcc, 0xdd, // messageChecksum (Checksum32)
            0x05, 0x00, 4, 0, // parameter ID
            1, 2, 3, 4, // parameter value
            0x01, 0x00, 0, 0, // PID_SENTINEL
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            HeaderExtensionSubmessageRead::try_from_bytes(&submessage_header, data).unwrap();

        assert!(submessage.length_flag());
        assert!(submessage.timestamp_flag());
        assert!(submessage.u_extension_flag());
        assert!(submessage.w_extension_flag());
        assert_eq!(submessage.checksum_flags(), (false, true));
        assert!(submessage.parameters_flag());
        assert_eq!(submessage.message_length(), 128);
        assert_eq!(submessage.rtps_send_timestamp(), Time::new(10, 20));
        assert_eq!(submessage.u_extension4(), [1, 2, 3, 4]);
        assert_eq!(submessage.w_extension8(), [5, 6, 7, 8, 9, 10, 11, 12]);
        assert_eq!(
            submessage.message_checksum(),
            Checksum::Checksum32([0xaa, 0xbb, 0xcc, 0xdd])
        );
        let expected_param = Parameter::new(0x0005, vec![1, 2, 3, 4].into());
        assert_eq!(
            submessage.parameters(),
            &ParameterList::new(vec![expected_param])
        );
    }

    #[test]
    fn deserialize_header_extension_checksum64() {
        #[rustfmt::skip]
        let mut data = &[
            0x00_u8, 0b_0010_0001, 8, 0, // Submessage header
            1, 2, 3, 4, 5, 6, 7, 8, // Checksum64
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            HeaderExtensionSubmessageRead::try_from_bytes(&submessage_header, data).unwrap();

        assert_eq!(submessage.checksum_flags(), (true, false));
        assert_eq!(
            submessage.message_checksum(),
            Checksum::Checksum64([1, 2, 3, 4, 5, 6, 7, 8])
        );
    }

    #[test]
    fn deserialize_header_extension_checksum128() {
        #[rustfmt::skip]
        let mut data = &[
            0x00_u8, 0b_0110_0001, 16, 0, // Submessage header
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // Checksum128
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            HeaderExtensionSubmessageRead::try_from_bytes(&submessage_header, data).unwrap();

        assert_eq!(submessage.checksum_flags(), (true, true));
        assert_eq!(
            submessage.message_checksum(),
            Checksum::Checksum128([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])
        );
    }
}
