use rust_rtps_pim::{
    messages::{submessages::DataSubmessage, types::SubmessageKindPIM, Submessage},
    structure::types::ParameterListPIM,
};

use crate::{EntityId, ParameterList, RtpsUdpPsm, SequenceNumber, SerializedData, SubmessageFlag};

use super::SubmessageHeader;

pub struct DataSubmesage<'a> {
    header: SubmessageHeader,
    extra_flags: u16,
    octets_to_inline_qos: u16,
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    inline_qos: &'a ParameterList,
    serialized_payload: SerializedData<'a>,
}

impl<'a> rust_rtps_pim::messages::submessages::DataSubmessage<'a, RtpsUdpPsm>
    for DataSubmesage<'a>
{
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type SerializedData = SerializedData<'a>;

    fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        inline_qos: &'a <RtpsUdpPsm as ParameterListPIM<RtpsUdpPsm>>::ParameterListType,
        serialized_payload: SerializedData<'a>,
    ) -> Self {
        let flags = [
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
        ]
        .into();
        let submessage_length = 20 + inline_qos.len() + serialized_payload.len();
        let header = SubmessageHeader {
            submessage_id: RtpsUdpPsm::DATA.into(),
            flags,
            submessage_length,
        };
        Self {
            header,
            extra_flags: 0b_0000_0000_0000_0000,
            octets_to_inline_qos: 12,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        }
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        self.header.flags.is_bit_set(0)
    }

    fn inline_qos_flag(&self) -> SubmessageFlag {
        self.header.flags.is_bit_set(1)
    }

    fn data_flag(&self) -> SubmessageFlag {
        self.header.flags.is_bit_set(2)
    }

    fn key_flag(&self) -> SubmessageFlag {
        self.header.flags.is_bit_set(3)
    }

    fn non_standard_payload_flag(&self) -> SubmessageFlag {
        self.header.flags.is_bit_set(4)
    }

    fn reader_id(&self) -> &Self::EntityId {
        &self.reader_id
    }

    fn writer_id(&self) -> &Self::EntityId {
        &self.writer_id
    }

    fn writer_sn(&self) -> &Self::SequenceNumber {
        &self.writer_sn
    }

    fn inline_qos(&self) -> &<RtpsUdpPsm as ParameterListPIM<RtpsUdpPsm>>::ParameterListType {
        todo!()
    }

    fn serialized_payload(&self) -> &Self::SerializedData {
        &self.serialized_payload
    }
}

impl<'a> Submessage<RtpsUdpPsm> for DataSubmesage<'a> {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}

impl<'a> serde::Serialize for DataSubmesage<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let mut len = 6;
        if self.inline_qos_flag() {
            len += 1
        };
        if self.data_flag() || self.key_flag() {
            len += 1;
        };
        let mut state = serializer.serialize_struct("Data", len)?;
        state.serialize_field("header", &self.header)?;
        state.serialize_field("extra_flags", &self.extra_flags)?;
        state.serialize_field("octets_to_inline_qos", &self.octets_to_inline_qos)?;
        state.serialize_field("reader_id", &self.reader_id)?;
        state.serialize_field("writer_id", &self.writer_id)?;
        state.serialize_field("writer_sn", &self.writer_sn)?;
        if self.inline_qos_flag() {
            state.serialize_field("inline_qos", &self.inline_qos)?;
        }
        if self.data_flag() || self.key_flag() {
            state.serialize_field("serialized_payload", &self.serialized_payload)?;
        }
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_serde_cdr::serializer::RtpsMessageSerializer;
    use serde::Serialize;

    fn get_serializer() -> RtpsMessageSerializer<Vec<u8>> {
        RtpsMessageSerializer {
            writer: Vec::<u8>::new(),
        }
    }

    #[test]
    fn serialize_no_inline_qos_no_serialized_payload() {
        let endianness_flag = true;
        let inline_qos_flag = false;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = [1, 2, 3, 4].into();
        let writer_id = [6, 7, 8, 9].into();
        let writer_sn = 5.into();
        let inline_qos = ParameterList { parameter: vec![] };
        let data = [];
        let serialized_payload = SerializedData(&data);
        let submessage = DataSubmesage::new(
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            &inline_qos,
            serialized_payload,
        );

        let mut serializer = get_serializer();
        submessage.serialize(&mut serializer).unwrap();
        #[rustfmt::skip]
        assert_eq!(serializer.writer, vec![
                0x15_u8, 0b_0000_0001, 20, 0, // Submessage header
                0, 0, 12, 0, // extraFlags, octetsToInlineQos
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: high
                5, 0, 0, 0, // writerSN: low
            ]
        );
        assert_eq!(
            serializer.writer.len() as u16 - 4,
            submessage.header.submessage_length
        )
    }

    #[test]
    fn serialize_with_inline_qos_no_serialized_payload() {
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = [1, 2, 3, 4].into();
        let writer_id = [6, 7, 8, 9].into();
        let writer_sn = 5.into();
        let param1 = crate::Parameter {
            parameter_id: 6,
            length: 4,
            value: vec![10, 11, 12, 13].into(),
        };
        let param2 = crate::Parameter {
            parameter_id: 7,
            length: 4,
            value: vec![20, 21, 22, 23].into(),
        };
        let inline_qos = ParameterList {
            parameter: vec![param1, param2],
        };
        let data = [];
        let serialized_payload = SerializedData(&data);
        let submessage = DataSubmesage::new(
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            &inline_qos,
            serialized_payload,
        );

        let mut serializer = get_serializer();
        submessage.serialize(&mut serializer).unwrap();
        #[rustfmt::skip]
        assert_eq!(serializer.writer, vec![
                0x15, 0b_0000_0011, 36, 0, // Submessage header
                0, 0, 12, 0, // extraFlags, octetsToInlineQos
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: high
                5, 0, 0, 0, // writerSN: low
                6, 0, 4, 0, // inlineQos: parameterId_1, length_1
                10, 11, 12, 13, // inlineQos: value_1[length_1]
                7, 0, 4, 0, // inlineQos: parameterId_2, length_2
                20, 21, 22, 23, // inlineQos: value_2[length_2]
            ]
        );
        assert_eq!(
            serializer.writer.len() as u16 - 4,
            submessage.header.submessage_length
        )
    }
}
