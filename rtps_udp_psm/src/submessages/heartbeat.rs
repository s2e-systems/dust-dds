use rust_rtps_pim::messages::types::SubmessageKindType;
use crate::{Count, EntityId, RtpsUdpPsm, SequenceNumber, SubmessageFlag};
use super::SubmessageHeader;

#[derive(serde::Serialize)]
pub struct HeartbeatSubmessage {
    header: SubmessageHeader,
    reader_id: EntityId,
    writer_id: EntityId,
    first_sn: SequenceNumber,
    last_sn: SequenceNumber,
    count: Count,
}

impl HeartbeatSubmessage {
    pub fn new(endianness_flag: SubmessageFlag, reader_id: EntityId, writer_id: EntityId, first_sn: SequenceNumber, last_sn: SequenceNumber, count: Count) -> Self {
        let flags = [endianness_flag].into();
        let submessage_length = 28;
        let header = SubmessageHeader {
            submessage_id: <RtpsUdpPsm as SubmessageKindType>::HEARTBEAT.into(),
            flags,
            submessage_length,
        };
        Self { header, reader_id, writer_id, first_sn, last_sn, count }
    }
}

impl rust_rtps_pim::messages::submessages::Heartbeat<RtpsUdpPsm> for HeartbeatSubmessage {
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type Count = Count;

    fn endianness_flag(&self) -> SubmessageFlag {
        self.header.flags.is_bit_set(0)
    }

    fn final_flag(&self) -> SubmessageFlag {
        self.header.flags.is_bit_set(1)
    }

    fn liveliness_flag(&self) -> SubmessageFlag {
        self.header.flags.is_bit_set(2)
    }

    fn reader_id(&self) -> &Self::EntityId {
        &self.reader_id
    }

    fn writer_id(&self) -> &Self::EntityId {
        &self.writer_id
    }

    fn first_sn(&self) -> &Self::SequenceNumber {
        &self.first_sn
    }

    fn last_sn(&self) -> &Self::SequenceNumber {
        &self.last_sn
    }

    fn count(&self) -> &Self::Count {
        &self.count
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for HeartbeatSubmessage {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use rust_serde_cdr::serializer::RtpsMessageSerializer;

    fn create_serializer() -> RtpsMessageSerializer<Vec<u8>> {
        RtpsMessageSerializer {
            writer: Vec::<u8>::new(),
        }
    }

    #[test]
    fn serialize() {
        let endianness_flag = true;
        let reader_id = [1, 2, 3, 4].into();
        let writer_id = [6, 7, 8, 9].into();
        let first_sn = 1.into();
        let last_sn = 3.into();
        let count = Count(5);
        let submessage = HeartbeatSubmessage::new(endianness_flag, reader_id, writer_id, first_sn, last_sn, count);

        let mut serializer = create_serializer();
        submessage.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.writer, vec![
            0x07_u8, 0b_0000_0001, 28, 0, // Submessage header
             1, 2, 3, 4,               // readerId: value[4]
             6, 7, 8, 9,               // writerId: value[4]
             0, 0, 0, 0,               // firstSN: SequenceNumber: high
             1, 0, 0, 0,               // firstSN: SequenceNumber: low
             0, 0, 0, 0,               // lastSN: SequenceNumber: high
             3, 0, 0, 0,               // lastSN: SequenceNumber: low
             5, 0, 0, 0,               // count: Count: value (long)

        ]);
        assert_eq!(serializer.writer.len() as u16 - 4, submessage.header.submessage_length)
    }
}
