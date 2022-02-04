use rust_rtps_pim::messages::{
    submessages::{
        DataSubmessageAttributes, DataSubmessageConstructor, GapSubmessageConstructor,
        HeartbeatSubmessageConstructor, InfoTimestampSubmessageAttributes,
    },
    types::SubmessageFlag,
};

use super::submessage_elements::{
    CountSubmessageElementPsm, EntityIdSubmessageElementPsm,
    ParameterListSubmessageElementWrite,
    SequenceNumberSetSubmessageElementPsm, SequenceNumberSubmessageElementPsm,
    SerializedDataSubmessageElementPsm, TimestampSubmessageElementPsm, ParameterListSubmessageElementRead, ParameterOwned,
};

#[derive(Debug, PartialEq)]
pub struct AckNackSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElementPsm,
    pub writer_id: EntityIdSubmessageElementPsm,
    pub reader_sn_state: SequenceNumberSetSubmessageElementPsm,
    pub count: CountSubmessageElementPsm,
}

impl AckNackSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct AckNackSubmessageRead {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElementPsm,
    pub writer_id: EntityIdSubmessageElementPsm,
    pub reader_sn_state: SequenceNumberSetSubmessageElementPsm,
    pub count: CountSubmessageElementPsm,
}

impl AckNackSubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct DataSubmessageWrite<'a> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub data_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElementPsm,
    pub writer_id: EntityIdSubmessageElementPsm,
    pub writer_sn: SequenceNumberSubmessageElementPsm,
    pub inline_qos: ParameterListSubmessageElementWrite<'a>,
    pub serialized_payload: SerializedDataSubmessageElementPsm<'a>,
}

impl<'a> DataSubmessageConstructor for DataSubmessageWrite<'a> {
    type EntityIdSubmessageElementType = EntityIdSubmessageElementPsm;
    type SequenceNumberSubmessageElementType = SequenceNumberSubmessageElementPsm;
    type ParameterListSubmessageElementType = &'a [ParameterOwned];
    type SerializedDataSubmessageElementType = &'a [u8];

    fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: Self::EntityIdSubmessageElementType,
        writer_id: Self::EntityIdSubmessageElementType,
        writer_sn: Self::SequenceNumberSubmessageElementType,
        inline_qos: Self::ParameterListSubmessageElementType,
        serialized_payload: Self::SerializedDataSubmessageElementType,
    ) -> Self {
        Self {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos: ParameterListSubmessageElementWrite {
                parameter: inline_qos,
            },
            serialized_payload: SerializedDataSubmessageElementPsm {
                value: serialized_payload,
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DataSubmessageRead<'a> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub data_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElementPsm,
    pub writer_id: EntityIdSubmessageElementPsm,
    pub writer_sn: SequenceNumberSubmessageElementPsm,
    pub inline_qos: ParameterListSubmessageElementRead<'a>,
    pub serialized_payload: SerializedDataSubmessageElementPsm<'a>,
}

impl<'a> DataSubmessageRead<'a> {
    pub fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElementPsm,
        writer_id: EntityIdSubmessageElementPsm,
        writer_sn: SequenceNumberSubmessageElementPsm,
        inline_qos: ParameterListSubmessageElementRead<'a>,
        serialized_payload: SerializedDataSubmessageElementPsm<'a>,
    ) -> Self {
        Self {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        }
    }
}

impl<'a> DataSubmessageAttributes for DataSubmessageRead<'a> {
    type EntityIdSubmessageElementType = EntityIdSubmessageElementPsm;
    type SequenceNumberSubmessageElementType = SequenceNumberSubmessageElementPsm;
    type ParameterListSubmessageElementType = ParameterListSubmessageElementRead<'a>;
    type SerializedDataSubmessageElementType = SerializedDataSubmessageElementPsm<'a>;

    fn endianness_flag(&self) -> &SubmessageFlag {
        &self.endianness_flag
    }

    fn inline_qos_flag(&self) -> &SubmessageFlag {
        &self.inline_qos_flag
    }

    fn data_flag(&self) -> &SubmessageFlag {
        &self.data_flag
    }

    fn key_flag(&self) -> &SubmessageFlag {
        &self.key_flag
    }

    fn non_standard_payload_flag(&self) -> &SubmessageFlag {
        &self.non_standard_payload_flag
    }

    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType {
        &self.reader_id
    }

    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType {
        &self.writer_id
    }

    fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType {
        &self.writer_sn
    }

    fn inline_qos(&self) -> &Self::ParameterListSubmessageElementType {
        &self.inline_qos
    }

    fn serialized_payload(&self) -> &Self::SerializedDataSubmessageElementType {
        &self.serialized_payload
    }
}

#[derive(Debug, PartialEq)]
pub struct DataFragSubmessageWrite();

impl DataFragSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct DataFragSubmessageRead();

// #[derive(Debug, PartialEq)]
// pub struct GapSubmessage<S> {
//     pub endianness_flag: SubmessageFlag,
//     pub reader_id: EntityIdSubmessageElement,
//     pub writer_id: EntityIdSubmessageElement,
//     pub gap_start: SequenceNumberSubmessageElement,
//     pub gap_list: SequenceNumberSetSubmessageElement<S>,
//     // gap_start_gsn: submessage_elements::SequenceNumber,
//     // gap_end_gsn: submessage_elements::SequenceNumber,
// }

#[derive(Debug, PartialEq)]
pub struct GapSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElementPsm,
    pub writer_id: EntityIdSubmessageElementPsm,
    pub gap_start: SequenceNumberSubmessageElementPsm,
    pub gap_list: SequenceNumberSetSubmessageElementPsm,
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}

impl GapSubmessageConstructor for GapSubmessageWrite {
    type EntityIdSubmessageElementType = EntityIdSubmessageElementPsm;
    type SequenceNumberSubmessageElementType = SequenceNumberSubmessageElementPsm;
    type SequenceNumberSetSubmessageElementType = SequenceNumberSetSubmessageElementPsm;

    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: Self::EntityIdSubmessageElementType,
        writer_id: Self::EntityIdSubmessageElementType,
        gap_start: Self::SequenceNumberSubmessageElementType,
        gap_list: Self::SequenceNumberSetSubmessageElementType,
    ) -> Self {
        Self {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GapSubmessageRead {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElementPsm,
    pub writer_id: EntityIdSubmessageElementPsm,
    pub gap_start: SequenceNumberSubmessageElementPsm,
    pub gap_list: SequenceNumberSetSubmessageElementPsm,
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}

impl GapSubmessageRead {
    pub fn new(
        endianness_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElementPsm,
        writer_id: EntityIdSubmessageElementPsm,
        gap_start: SequenceNumberSubmessageElementPsm,
        gap_list: SequenceNumberSetSubmessageElementPsm,
    ) -> Self {
        Self {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct HeartbeatSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub liveliness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElementPsm,
    pub writer_id: EntityIdSubmessageElementPsm,
    pub first_sn: SequenceNumberSubmessageElementPsm,
    pub last_sn: SequenceNumberSubmessageElementPsm,
    pub count: CountSubmessageElementPsm,
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}

#[derive(Debug, PartialEq)]
pub struct HeartbeatSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub liveliness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElementPsm,
    pub writer_id: EntityIdSubmessageElementPsm,
    pub first_sn: SequenceNumberSubmessageElementPsm,
    pub last_sn: SequenceNumberSubmessageElementPsm,
    pub count: CountSubmessageElementPsm,
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}

impl HeartbeatSubmessageWrite {
    pub fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        liveliness_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElementPsm,
        writer_id: EntityIdSubmessageElementPsm,
        first_sn: SequenceNumberSubmessageElementPsm,
        last_sn: SequenceNumberSubmessageElementPsm,
        count: CountSubmessageElementPsm,
    ) -> Self {
        Self {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        }
    }
}

impl HeartbeatSubmessageConstructor for HeartbeatSubmessageWrite {
    type EntityIdSubmessageElementType = EntityIdSubmessageElementPsm;
    type SequenceNumberSubmessageElementType = SequenceNumberSubmessageElementPsm;
    type CountSubmessageElementType = CountSubmessageElementPsm;

    fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        liveliness_flag: SubmessageFlag,
        reader_id: Self::EntityIdSubmessageElementType,
        writer_id: Self::EntityIdSubmessageElementType,
        first_sn: Self::SequenceNumberSubmessageElementType,
        last_sn: Self::SequenceNumberSubmessageElementType,
        count: Self::CountSubmessageElementType,
    ) -> Self {
        Self {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HeartbeatSubmessageRead {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub liveliness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElementPsm,
    pub writer_id: EntityIdSubmessageElementPsm,
    pub first_sn: SequenceNumberSubmessageElementPsm,
    pub last_sn: SequenceNumberSubmessageElementPsm,
    pub count: CountSubmessageElementPsm,
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}

impl HeartbeatSubmessageRead {
    pub fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        liveliness_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElementPsm,
        writer_id: EntityIdSubmessageElementPsm,
        first_sn: SequenceNumberSubmessageElementPsm,
        last_sn: SequenceNumberSubmessageElementPsm,
        count: CountSubmessageElementPsm,
    ) -> Self {
        Self {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HeartbeatFragSubmessageWrite();

impl HeartbeatFragSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct HeartbeatFragSubmessageRead();

impl HeartbeatFragSubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoDestinationSubmessageWrite();

impl InfoDestinationSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoDestinationSubmessageRead();

impl InfoDestinationSubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]

pub struct InfoReplySubmessageWrite();

impl InfoReplySubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoReplySubmessageRead();

impl InfoReplySubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoSourceSubmessageWrite();

impl InfoSourceSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoSourceSubmessageRead();

impl InfoSourceSubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoTimestampSubmessageWrite();

impl InfoTimestampSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoTimestampSubmessageRead {
    endianness_flag: SubmessageFlag,
    invalidate_flag: SubmessageFlag,
    timestamp: TimestampSubmessageElementPsm,
}

impl InfoTimestampSubmessageAttributes for InfoTimestampSubmessageRead {
    type TimestampSubmessageElementType = TimestampSubmessageElementPsm;

    fn endianness_flag(&self) -> &SubmessageFlag {
        &self.endianness_flag
    }

    fn invalidate_flag(&self) -> &SubmessageFlag {
        &self.invalidate_flag
    }

    fn timestamp(&self) -> &Self::TimestampSubmessageElementType {
        &self.timestamp
    }
}

impl InfoTimestampSubmessageRead {
    pub fn new(
        endianness_flag: SubmessageFlag,
        invalidate_flag: SubmessageFlag,
        timestamp: TimestampSubmessageElementPsm,
    ) -> Self {
        Self {
            endianness_flag,
            invalidate_flag,
            timestamp,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct NackFragSubmessageWrite();

impl NackFragSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct NackFragSubmessageRead();

impl NackFragSubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct PadSubmessageWrite;

impl PadSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct PadSubmessageRead;

impl PadSubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}
