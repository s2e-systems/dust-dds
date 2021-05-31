use rust_rtps_pim::messages::{Submessage, types::SubmessageKindType, submessages::Data};

use crate::{EntityId, ParameterList, RtpsUdpPsm, SequenceNumber, SerializedData, SubmessageFlag};

use super::SubmessageHeader;

pub struct DataSubmesage<'a> {
    header: SubmessageHeader,
    extra_flags: u16,
    octets_to_inline_qos: u16,
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    inline_qos: Option<ParameterList>,
    serialized_payload: SerializedData<'a>,
}

impl<'a> DataSubmesage<'a> {
    fn new(
        endianness_flag: SubmessageFlag,
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        inline_qos: Option<ParameterList>,
        serialized_payload: SerializedData<'a>,
    ) -> Self {
        let inline_qos_flag = inline_qos.is_some();
        let flags = [endianness_flag, inline_qos_flag, data_flag, key_flag, non_standard_payload_flag].into();

        let mut submessage_length = 20;
        if let Some(ref inline_qos) = inline_qos {
            let parameter_list_length: u16 = inline_qos.parameter.iter().map(|parameter|4 + parameter.length as u16).sum();
            submessage_length += parameter_list_length;
        }
        if data_flag || key_flag {
            submessage_length += serialized_payload.len() as u16;
        }
        let header = SubmessageHeader{
            submessage_id: <RtpsUdpPsm as SubmessageKindType>::DATA.into(),
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
            serialized_payload, //TODO!!!!!!!!!!!!!!!!
        }
    }
}

impl<'a> rust_rtps_pim::messages::submessages::Data<RtpsUdpPsm> for DataSubmesage<'a> {
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type SerializedData = SerializedData<'a>;

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

    fn serialized_payload(&self) -> &Self::SerializedData {
        &self.serialized_payload
    }
}

impl<'a> Submessage<RtpsUdpPsm> for DataSubmesage<'a> {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
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

        if let Some(inline_qos) = &self.inline_qos {
            state.serialize_field("inline_qos", inline_qos)?;
        }
        if self.data_flag() || self.key_flag() {
            state.serialize_field("serialized_payload", &self.serialized_payload)?;
        };
        state.end()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use rust_serde_cdr::serializer::RtpsMessageSerializer;

    fn get_serializer() -> RtpsMessageSerializer<Vec<u8>> {
        RtpsMessageSerializer {
            writer: Vec::<u8>::new(),
        }
    }

    #[test]
    fn serialize_no_inline_qos_no_serialized_payload() {
        let data = &vec![];
        let endianness_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = [1, 2, 3, 4].into();
        let writer_id = [6, 7, 8, 9].into();
        let writer_sn = 5.into();
        let inline_qos = None;
        let serialized_payload = SerializedData(data);
        let submessage = DataSubmesage::new(endianness_flag, data_flag, key_flag, non_standard_payload_flag, reader_id, writer_id, writer_sn, inline_qos, serialized_payload);

        let mut serializer = get_serializer();
        submessage.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.writer, vec![
            0x15_u8, 0b_0000_0001, 20, 0, // Submessage header
            0, 0, 12, 0,               // extraFlags, octetsToInlineQos
            1, 2, 3, 4,               // readerId: value[4]
            6, 7, 8, 9,               // writerId: value[4]
            0, 0, 0, 0,               // writerSN: high
            5, 0, 0, 0,               // writerSN: low
        ]);
        assert_eq!(serializer.writer.len() as u16 - 4, submessage.header.submessage_length)
    }

    #[test]
    fn serialize_with_inline_qos_no_serialized_payload() {
        let data = &vec![];
        let endianness_flag = true.into();
        let data_flag = false.into();
        let key_flag = false.into();
        let non_standard_payload_flag = false.into();
        let reader_id = [1, 2, 3, 4].into();
        let writer_id = [6, 7, 8, 9].into();
        let writer_sn = 5.into();
        let param1 = crate::Parameter{parameter_id: 6, length: 4, value: vec![10, 11, 12, 13].into()};
        let param2 = crate::Parameter{parameter_id: 7, length: 4, value: vec![20, 21, 22, 23].into()};
        let inline_qos = Some(crate::ParameterList{parameter: vec![param1, param2]});
        let serialized_payload = SerializedData(data);
        let submessage = DataSubmesage::new(endianness_flag, data_flag, key_flag, non_standard_payload_flag, reader_id, writer_id, writer_sn, inline_qos, serialized_payload);

        let mut serializer = get_serializer();
        submessage.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.writer, vec![
            0x15, 0b_0000_0011, 36, 0, // Submessage header
            0, 0, 12, 0,               // extraFlags, octetsToInlineQos
            1, 2, 3, 4,                // readerId: value[4]
            6, 7, 8, 9,                // writerId: value[4]
            0, 0, 0, 0,                // writerSN: high
            5, 0, 0, 0,                // writerSN: low
            6, 0, 4, 0,                // inlineQos: parameterId_1, length_1
            10, 11, 12, 13,            // inlineQos: value_1[length_1]
            7, 0, 4, 0,                // inlineQos: parameterId_2, length_2
            20, 21, 22, 23,            // inlineQos: value_2[length_2]
        ]);
        assert_eq!(serializer.writer.len() as u16 - 4, submessage.header.submessage_length)
    }
}