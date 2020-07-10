use super::serdes::{SubmessageElement, Endianness, RtpsSerdesResult, };
use super::submessage_elements;
use super::types::{SubmessageKind, SubmessageFlag, };
use super::{SubmessageHeader, Submessage, UdpPsmMapping};

#[derive(PartialEq, Debug)]
pub struct Heartbeat {
    endianness_flag: SubmessageFlag,
    final_flag: SubmessageFlag,
    liveliness_flag: SubmessageFlag,
    // group_info_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId,
    writer_id: submessage_elements::EntityId,
    first_sn: submessage_elements::SequenceNumber,
    last_sn: submessage_elements::SequenceNumber,
    count: submessage_elements::Count,
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}

impl Heartbeat {
    const FINAL_FLAG_MASK: u8 = 0x02;
    const LIVELINESS_FLAG_MASK: u8 = 0x04;

    pub fn new(
        reader_id: submessage_elements::EntityId,
        writer_id: submessage_elements::EntityId,
        first_sn: submessage_elements::SequenceNumber,
        last_sn: submessage_elements::SequenceNumber,
        count: submessage_elements::Count,
        final_flag: bool,
        manual_liveliness: bool,
        endianness_flag: Endianness) -> Self {
            Heartbeat {
                reader_id,
                writer_id,
                first_sn,
                last_sn,
                count,
                final_flag,
                liveliness_flag: manual_liveliness,
                endianness_flag: endianness_flag.into(),
            }
        }

    pub fn is_valid(&self) -> bool{
        if self.first_sn.0 < 1 {
            return false;
        };

        if self.last_sn.0 < 0 {
            return false;
        }

        if self.last_sn.0 < self.first_sn.0 - 1 {
            return false;
        }

        true
    }

    pub fn reader_id(&self) -> &submessage_elements::EntityId {
        &self.reader_id
    }

    pub fn writer_id(&self) -> &submessage_elements::EntityId {
        &self.writer_id
    }

    pub fn first_sn(&self) -> &submessage_elements::SequenceNumber {
        &self.first_sn
    }

    pub fn last_sn(&self) -> &submessage_elements::SequenceNumber {
        &self.last_sn
    }

    pub fn count(&self) -> &submessage_elements::Count {
        &self.count
    }

    pub fn is_final(&self) -> bool {
        self.final_flag
    }
}

impl Submessage for Heartbeat {
    fn submessage_header(&self) -> SubmessageHeader {
        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
        let f = self.final_flag; //Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
        let l = self.liveliness_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
        // X|X|X|X|X|L|F|E
        let flags = [e, f, l, x, x, x, x, x];

        let octets_to_next_header = 
            self.reader_id.octets() + 
            self.writer_id.octets() +
            self.first_sn.octets() +
            self.last_sn.octets() +
            self.count.octets();

        SubmessageHeader { 
            submessage_id: SubmessageKind::Heartbeat,
            flags,
            submessage_length: octets_to_next_header as u16,
        }
    }

    fn is_valid(&self) -> bool {
        if self.first_sn.0 <= 0 ||
           self.last_sn.0 < 0 ||
           self.last_sn.0 < self.first_sn.0 - 1 {
            false
        } else {
            true
        }
    }
}

impl UdpPsmMapping for Heartbeat {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = self.endianness_flag.into();
        self.submessage_header().compose(writer)?;
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.first_sn.serialize(writer, endianness)?;
        self.last_sn.serialize(writer, endianness)?;
        self.count.serialize(writer, endianness)?;
        Ok(())
    }

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> { 
        let header = SubmessageHeader::parse(bytes)?;
        let flags = header.flags();
        // X|X|X|X|X|L|F|E
        /*E*/ let endianness_flag = flags[0];
        /*F*/ let final_flag = flags[1];
        /*L*/ let liveliness_flag = flags[2];

        let endianness = Endianness::from(endianness_flag);

        const HEADER_SIZE : usize = 8;
        let reader_id = submessage_elements::EntityId::deserialize(&bytes[4..8], endianness)?;
        let writer_id = submessage_elements::EntityId::deserialize(&bytes[8..12], endianness)?;
        let first_sn = submessage_elements::SequenceNumber::deserialize(&bytes[12..20], endianness)?;
        let last_sn = submessage_elements::SequenceNumber::deserialize(&bytes[20..28], endianness)?;
        let count = submessage_elements::Count::deserialize(&bytes[28..32], endianness)?;
        

        Ok(Heartbeat {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::ENTITYID_UNKNOWN;
    use crate::types::EntityKind;

    #[test]
    fn test_heartbeat_validity_function() {
        let valid_heartbeat = Heartbeat {
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            first_sn: submessage_elements::SequenceNumber(2),
            last_sn: submessage_elements::SequenceNumber(5), 
            count: submessage_elements::Count(0),
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(valid_heartbeat.is_valid(), true);

        let valid_heartbeat_first_message = Heartbeat {
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            first_sn: submessage_elements::SequenceNumber(1),
            last_sn: submessage_elements::SequenceNumber(0), 
            count: submessage_elements::Count(2),
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(valid_heartbeat_first_message.is_valid(), true);

        let invalid_heartbeat_zero_first_value = Heartbeat {
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            first_sn: submessage_elements::SequenceNumber(0),
            last_sn: submessage_elements::SequenceNumber(1), 
            count: submessage_elements::Count(2),
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_zero_first_value.is_valid(), false);

        let invalid_heartbeat_negative_last_value = Heartbeat {
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            first_sn: submessage_elements::SequenceNumber(5),
            last_sn: submessage_elements::SequenceNumber(-6), 
            count: submessage_elements::Count(2),
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_negative_last_value.is_valid(), false);

        let invalid_heartbeat_wrong_first_last_value = Heartbeat {
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            first_sn: submessage_elements::SequenceNumber(6),
            last_sn: submessage_elements::SequenceNumber(4), 
            count: submessage_elements::Count(2),
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_wrong_first_last_value.is_valid(), false);
    }

    #[test]
    fn test_serialize_deserialize_heartbeat() {
        let mut writer = Vec::new();

        let reader_id = submessage_elements::EntityId(crate::types::EntityId::new([0x10, 0x12, 0x14], EntityKind::UserDefinedReaderWithKey));
        let writer_id = submessage_elements::EntityId(crate::types::EntityId::new([0x26, 0x24, 0x22], EntityKind::UserDefinedWriterWithKey));
        let first_sn = submessage_elements::SequenceNumber(1233);
        let last_sn = submessage_elements::SequenceNumber(1237);
        let count = submessage_elements::Count(8);
        let is_final = true;
        let manual_liveliness = false;

        let heartbeat_big_endian = Heartbeat::new(reader_id, writer_id, first_sn, last_sn, count, is_final, manual_liveliness, Endianness::BigEndian);
        heartbeat_big_endian.compose(&mut writer).unwrap();
        let submessage_big_endian = [
            0x07, 0x02, 0x00, 0x1C, // Submessage Header
            0x10, 0x12, 0x14, 0x04, // Reader ID
            0x26, 0x24, 0x22, 0x02, // Writer ID
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD1, // First Sequence Number
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD5, // Last Sequence Number
            0x00, 0x00, 0x00, 0x08, // Count
        ];

        assert_eq!(writer, submessage_big_endian);
        assert_eq!(Heartbeat::parse(&writer).unwrap(), heartbeat_big_endian);

        writer.clear();

        let heartbeat_little_endian = Heartbeat::new(reader_id, writer_id, first_sn, last_sn, count, is_final, manual_liveliness, Endianness::LittleEndian);
        heartbeat_little_endian.compose(&mut writer).unwrap();
        let submessage_little_endian = [
            0x07, 0x03, 0x1C, 0x00, // Submessage Header
            0x10, 0x12, 0x14, 0x04, // Reader ID
            0x26, 0x24, 0x22, 0x02, // Writer ID
            0x00, 0x00, 0x00, 0x00, 0xD1, 0x04, 0x00, 0x00, // First Sequence Number
            0x00, 0x00, 0x00, 0x00, 0xD5, 0x04, 0x00, 0x00, // Last Sequence Number
            0x08, 0x00, 0x00, 0x00, // Count
        ];
        assert_eq!(writer, submessage_little_endian);
        assert_eq!(Heartbeat::parse(&writer).unwrap(), heartbeat_little_endian);
    }
}
