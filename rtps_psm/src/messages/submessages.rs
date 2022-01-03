use std::ops::Deref;

use rust_rtps_pim::{
    messages::{
        submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement, SequenceNumberSubmessageElement,
            TimestampSubmessageElement,
        },
        submessages::{
            DataSubmessageAttributes, DataSubmessageConstructor, GapSubmessageConstructor,
            HeartbeatFragSubmessage, HeartbeatSubmessageConstructor, InfoDestinationSubmessage,
            InfoReplySubmessage, InfoSourceSubmessage, InfoTimestampSubmessage, NackFragSubmessage,
        },
        types::SubmessageFlag,
    },
    structure::types::SequenceNumber,
};

use super::{
    overall_structure::RtpsSubmessageTypeWrite,
    submessage_elements::{
        CountSubmessageElementPsm, EntityIdSubmessageElementPsm, Parameter,
        ParameterListSubmessageElementWrite,
        SequenceNumberSetSubmessageElementPsm, SequenceNumberSubmessageElementPsm,
        SerializedDataSubmessageElementPsm, ParameterListSubmessageElementRead,
    },
};

#[derive(Debug, PartialEq)]
pub struct AckNackSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub reader_sn_state: SequenceNumberSetSubmessageElementPsm,
    pub count: CountSubmessageElement,
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
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub reader_sn_state: SequenceNumberSetSubmessageElementPsm,
    pub count: CountSubmessageElement,
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
    type SequenceNumberSubmessageElementType = SequenceNumber;
    type ParameterListSubmessageElementType = &'a [Parameter<'a>];
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
            writer_sn: SequenceNumberSubmessageElementPsm { value: writer_sn },
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
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub gap_start: SequenceNumberSubmessageElement,
    pub gap_list: SequenceNumberSetSubmessageElementPsm,
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}

impl GapSubmessageConstructor for GapSubmessageWrite {
    type EntityIdSubmessageElementType = EntityIdSubmessageElementPsm;
    type SequenceNumberSubmessageElementType = SequenceNumber;
    type SequenceNumberSetSubmessageElementType = SequenceNumberSetSubmessageElementPsm;

    fn new(
        _endianness_flag: SubmessageFlag,
        _reader_id: Self::EntityIdSubmessageElementType,
        _writer_id: Self::EntityIdSubmessageElementType,
        _gap_start: Self::SequenceNumberSubmessageElementType,
        _gap_list: Self::SequenceNumberSetSubmessageElementType,
    ) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct GapSubmessageRead {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub gap_start: SequenceNumberSubmessageElement,
    pub gap_list: SequenceNumberSetSubmessageElementPsm,
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}

impl GapSubmessageRead {
    pub fn new(
        endianness_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        gap_start: SequenceNumberSubmessageElement,
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
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub first_sn: SequenceNumberSubmessageElement,
    pub last_sn: SequenceNumberSubmessageElement,
    pub count: CountSubmessageElement,
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
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub first_sn: SequenceNumberSubmessageElement,
    pub last_sn: SequenceNumberSubmessageElement,
    pub count: CountSubmessageElement,
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
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        first_sn: SequenceNumberSubmessageElement,
        last_sn: SequenceNumberSubmessageElement,
        count: CountSubmessageElement,
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

    type SequenceNumberSubmessageElementType = SequenceNumber;

    type CountSubmessageElementType = CountSubmessageElementPsm;

    fn new(
        _endianness_flag: SubmessageFlag,
        _final_flag: SubmessageFlag,
        _liveliness_flag: SubmessageFlag,
        _reader_id: Self::EntityIdSubmessageElementType,
        _writer_id: Self::EntityIdSubmessageElementType,
        _first_sn: Self::SequenceNumberSubmessageElementType,
        _last_sn: Self::SequenceNumberSubmessageElementType,
        _count: Self::CountSubmessageElementType,
    ) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct HeartbeatSubmessageRead {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub liveliness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub first_sn: SequenceNumberSubmessageElement,
    pub last_sn: SequenceNumberSubmessageElement,
    pub count: CountSubmessageElement,
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
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        first_sn: SequenceNumberSubmessageElement,
        last_sn: SequenceNumberSubmessageElement,
        count: CountSubmessageElement,
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
pub struct HeartbeatFragSubmessageWrite(<Self as Deref>::Target);

impl HeartbeatFragSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for HeartbeatFragSubmessageWrite {
    type Target = HeartbeatFragSubmessage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a> From<<HeartbeatFragSubmessageWrite as Deref>::Target> for RtpsSubmessageTypeWrite<'a> {
    fn from(_: <HeartbeatFragSubmessageWrite as Deref>::Target) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct HeartbeatFragSubmessageRead(<Self as Deref>::Target);

impl HeartbeatFragSubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for HeartbeatFragSubmessageRead {
    type Target = HeartbeatFragSubmessage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoDestinationSubmessageWrite(<Self as Deref>::Target);

impl InfoDestinationSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for InfoDestinationSubmessageWrite {
    type Target = InfoDestinationSubmessage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<<InfoDestinationSubmessageWrite as Deref>::Target> for RtpsSubmessageTypeWrite<'a> {
    fn from(_: <InfoDestinationSubmessageWrite as Deref>::Target) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoDestinationSubmessageRead(<Self as Deref>::Target);

impl InfoDestinationSubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for InfoDestinationSubmessageRead {
    type Target = InfoDestinationSubmessage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq)]

pub struct InfoReplySubmessageWrite(<Self as Deref>::Target);

impl InfoReplySubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for InfoReplySubmessageWrite {
    type Target = InfoReplySubmessage<()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<<InfoReplySubmessageWrite as Deref>::Target> for RtpsSubmessageTypeWrite<'a> {
    fn from(_: <InfoReplySubmessageWrite as Deref>::Target) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoReplySubmessageRead(<Self as Deref>::Target);

impl InfoReplySubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for InfoReplySubmessageRead {
    type Target = InfoReplySubmessage<()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoSourceSubmessageWrite(<Self as Deref>::Target);

impl InfoSourceSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for InfoSourceSubmessageWrite {
    type Target = InfoSourceSubmessage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<<InfoSourceSubmessageWrite as Deref>::Target> for RtpsSubmessageTypeWrite<'a> {
    fn from(_: <InfoSourceSubmessageWrite as Deref>::Target) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoSourceSubmessageRead(<Self as Deref>::Target);

impl InfoSourceSubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for InfoSourceSubmessageRead {
    type Target = InfoSourceSubmessage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoTimestampSubmessageWrite(<Self as Deref>::Target);

impl InfoTimestampSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for InfoTimestampSubmessageWrite {
    type Target = InfoTimestampSubmessage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<<InfoTimestampSubmessageWrite as Deref>::Target> for RtpsSubmessageTypeWrite<'a> {
    fn from(_: <InfoTimestampSubmessageWrite as Deref>::Target) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoTimestampSubmessageRead(<Self as Deref>::Target);

impl InfoTimestampSubmessageRead {
    pub fn new(
        endianness_flag: SubmessageFlag,
        invalidate_flag: SubmessageFlag,
        timestamp: TimestampSubmessageElement,
    ) -> Self {
        Self(InfoTimestampSubmessage {
            endianness_flag,
            invalidate_flag,
            timestamp,
        })
    }
}

impl Deref for InfoTimestampSubmessageRead {
    type Target = InfoTimestampSubmessage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct NackFragSubmessageWrite(<Self as Deref>::Target);

impl NackFragSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for NackFragSubmessageWrite {
    type Target = NackFragSubmessage<()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<<NackFragSubmessageWrite as Deref>::Target> for RtpsSubmessageTypeWrite<'a> {
    fn from(_: <NackFragSubmessageWrite as Deref>::Target) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct NackFragSubmessageRead(<Self as Deref>::Target);

impl NackFragSubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for NackFragSubmessageRead {
    type Target = NackFragSubmessage<()>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
