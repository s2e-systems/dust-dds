use rust_rtps_pim::messages::types::{SubmessageFlag, SubmessageKindPIM};

use crate::{
    psm::RtpsUdpPsm,
    submessage_elements::{CountUdp, EntityIdUdp, SequenceNumberUdp},
    submessage_header::SubmessageHeader,
};

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct HeartbeatSubmessageUdp {
    pub header: SubmessageHeader,
    reader_id: EntityIdUdp,
    writer_id: EntityIdUdp,
    first_sn: SequenceNumberUdp,
    last_sn: SequenceNumberUdp,
    count: CountUdp,
}

impl<'a> rust_rtps_pim::messages::submessages::HeartbeatSubmessage for HeartbeatSubmessageUdp {
    type EntityIdSubmessageElementType = EntityIdUdp;
    type SequenceNumberSubmessageElementType = SequenceNumberUdp;
    type CountSubmessageElementType = CountUdp;

    fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        liveliness_flag: SubmessageFlag,
        reader_id: EntityIdUdp,
        writer_id: EntityIdUdp,
        first_sn: SequenceNumberUdp,
        last_sn: SequenceNumberUdp,
        count: CountUdp,
    ) -> Self {
        let flags = [endianness_flag, final_flag, liveliness_flag].into();
        let submessage_length = 28;
        let header = SubmessageHeader {
            submessage_id: RtpsUdpPsm::HEARTBEAT.into(),
            flags,
            submessage_length,
        };
        Self {
            header,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        }
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        self.header.flags.is_bit_set(0)
    }

    fn final_flag(&self) -> SubmessageFlag {
        self.header.flags.is_bit_set(1)
    }

    fn liveliness_flag(&self) -> SubmessageFlag {
        self.header.flags.is_bit_set(2)
    }

    fn reader_id(&self) -> &EntityIdUdp {
        &self.reader_id
    }

    fn writer_id(&self) -> &EntityIdUdp {
        &self.writer_id
    }

    fn first_sn(&self) -> &SequenceNumberUdp {
        &self.first_sn
    }

    fn last_sn(&self) -> &SequenceNumberUdp {
        &self.last_sn
    }

    fn count(&self) -> &CountUdp {
        &self.count
    }
}

impl rust_rtps_pim::messages::Submessage for HeartbeatSubmessageUdp {
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_rtps_pim::messages::submessage_elements::SequenceNumberSubmessageElementType;
    use rust_serde_cdr::serializer::RtpsMessageSerializer;
    use serde::Serialize;

    fn create_serializer() -> RtpsMessageSerializer<Vec<u8>> {
        RtpsMessageSerializer {
            writer: Vec::<u8>::new(),
        }
    }

    #[test]
    fn serialize() {
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = false;
        let reader_id = EntityIdUdp([1, 2, 3, 4]);
        let writer_id = EntityIdUdp([6, 7, 8, 9]);
        let first_sn = SequenceNumberUdp::new(1);
        let last_sn = SequenceNumberUdp::new(3);
        let count = CountUdp(5);
        let submessage: HeartbeatSubmessageUdp =
            rust_rtps_pim::messages::submessages::HeartbeatSubmessage::new(
                endianness_flag,
                final_flag,
                liveliness_flag,
                reader_id,
                writer_id,
                first_sn,
                last_sn,
                count,
            );

        let mut serializer = create_serializer();
        submessage.serialize(&mut serializer).unwrap();
        #[rustfmt::skip]
        assert_eq!(
            serializer.writer, vec![
                0x07_u8, 0b_0000_0001, 28, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // firstSN: SequenceNumber: high
                1, 0, 0, 0, // firstSN: SequenceNumber: low
                0, 0, 0, 0, // lastSN: SequenceNumber: high
                3, 0, 0, 0, // lastSN: SequenceNumber: low
                5, 0, 0, 0, // count: Count: value (long)
            ]
        );
        assert_eq!(
            serializer.writer.len() as u16 - 4,
            submessage.header.submessage_length
        )
    }
}
