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
    // inline_qos: ParameterList,
    serialized_payload: SerializedData<'a>,
}

impl<'a> rust_rtps_pim::messages::submessages::Data<RtpsUdpPsm> for DataSubmesage<'a> {
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
        // inline_qos: ParameterList,
        serialized_payload: &crate::Data,
    ) -> Self {
        let flags = [endianness_flag, inline_qos_flag, data_flag, key_flag, non_standard_payload_flag].into();

        let mut submessage_length = 20;
        // if inline_qos_flag {
        //     let parameter_list_length: u16 = inline_qos.iter().map(|parameter|4 + parameter.length as u16).sum();
        //     submessage_length += parameter_list_length;
        // }
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
            // inline_qos,
            serialized_payload: SerializedData(&[]), //TODO!!!!!!!!!!!!!!!!
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

        if self.inline_qos_flag() {
            // state.serialize_field("inline_qos", self.inline_qos)?;
        }
        if self.data_flag() || self.key_flag() {
            state.serialize_field("serialized_payload", &self.serialized_payload)?;
        };
        state.end()
    }
}
