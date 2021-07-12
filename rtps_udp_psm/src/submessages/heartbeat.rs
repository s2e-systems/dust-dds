use rust_rtps_pim::messages::{
    types::{SubmessageFlag, SubmessageKind},
    RtpsSubmessageHeader,
};

use crate::{
    submessage_elements::{CountUdp, EntityIdUdp, SequenceNumberUdp},
    submessage_header::SubmessageHeaderUdp,
};

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct HeartbeatSubmessageUdp {
    pub header: SubmessageHeaderUdp,
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
        let header = SubmessageHeaderUdp {
            submessage_id: SubmessageKind::HEARTBEAT.into(),
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
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::submessage_elements::Octet;

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
        let reader_id = EntityIdUdp {
            entity_key: [Octet(1), Octet(2), Octet(3)],
            entity_kind: Octet(4),
        };
        let writer_id = EntityIdUdp {
            entity_key: [Octet(6), Octet(7), Octet(8)],
            entity_kind: Octet(9),
        };
        let first_sn = SequenceNumberUdp::new(&1);
        let last_sn = SequenceNumberUdp::new(&3);
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
