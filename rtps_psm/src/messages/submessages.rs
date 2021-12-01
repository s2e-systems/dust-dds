use std::ops::Deref;

use rust_rtps_pim::{
    messages::{
        submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement, Parameter,
            ParameterListSubmessageElement, SequenceNumberSetSubmessageElement,
            SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
            TimestampSubmessageElement,
        },
        submessages::{
            AckNackSubmessage, DataFragSubmessage, DataSubmessage, GapSubmessage,
            HeartbeatFragSubmessage, HeartbeatSubmessage, InfoDestinationSubmessage,
            InfoReplySubmessage, InfoSourceSubmessage, InfoTimestampSubmessage, NackFragSubmessage,
            PadSubmessage,
        },
        types::SubmessageFlag,
    },
    structure::types::SequenceNumber,
};

use super::overall_structure::RtpsSubmessageTypeWrite;

#[derive(Debug, PartialEq)]
pub struct AckNackSubmessageWrite(AckNackSubmessage<Vec<SequenceNumber>>);

impl AckNackSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for AckNackSubmessageWrite {
    type Target = AckNackSubmessage<Vec<SequenceNumber>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<<AckNackSubmessageWrite as Deref>::Target> for RtpsSubmessageTypeWrite<'a> {
    fn from(_: <AckNackSubmessageWrite as Deref>::Target) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct AckNackSubmessageRead(AckNackSubmessage<Vec<SequenceNumber>>);

impl AckNackSubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for AckNackSubmessageRead {
    type Target = AckNackSubmessage<Vec<SequenceNumber>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct DataSubmessageWrite<'a>(<Self as Deref>::Target);

impl<'a> DataSubmessageWrite<'a> {
    pub fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        writer_sn: SequenceNumberSubmessageElement,
        inline_qos: ParameterListSubmessageElement<Vec<Parameter<Vec<u8>>>>,
        serialized_payload: SerializedDataSubmessageElement<&'a [u8]>,
    ) -> Self {
        Self(DataSubmessage {
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
        })
    }
}

impl<'a> Deref for DataSubmessageWrite<'a> {
    type Target = DataSubmessage<Vec<Parameter<Vec<u8>>>, &'a [u8]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<<DataSubmessageWrite<'a> as Deref>::Target> for RtpsSubmessageTypeWrite<'a> {
    fn from(s: <DataSubmessageWrite<'a> as Deref>::Target) -> Self {
        Self::Data(DataSubmessageWrite::new(
            s.endianness_flag,
            s.inline_qos_flag,
            s.data_flag,
            s.key_flag,
            s.non_standard_payload_flag,
            s.reader_id,
            s.writer_id,
            s.writer_sn,
            s.inline_qos,
            s.serialized_payload,
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct DataSubmessageRead<'a>(<Self as Deref>::Target);

impl<'a> DataSubmessageRead<'a> {
    pub fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        writer_sn: SequenceNumberSubmessageElement,
        inline_qos: ParameterListSubmessageElement<Vec<Parameter<&'a [u8]>>>,
        serialized_payload: SerializedDataSubmessageElement<&'a [u8]>,
    ) -> Self {
        Self(DataSubmessage {
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
        })
    }
}

impl<'a> Deref for DataSubmessageRead<'a> {
    type Target = DataSubmessage<Vec<Parameter<&'a [u8]>>, &'a [u8]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct DataFragSubmessageWrite(<Self as Deref>::Target);

impl DataFragSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for DataFragSubmessageWrite {
    type Target = DataFragSubmessage<(), ()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<<DataFragSubmessageWrite as Deref>::Target> for RtpsSubmessageTypeWrite<'a> {
    fn from(_: <DataFragSubmessageWrite as Deref>::Target) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct DataFragSubmessageRead(<Self as Deref>::Target);

impl DataFragSubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for DataFragSubmessageRead {
    type Target = DataFragSubmessage<(), ()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct GapSubmessageWrite(<Self as Deref>::Target);

impl GapSubmessageWrite {
    pub fn new(
        endianness_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        gap_start: SequenceNumberSubmessageElement,
        gap_list: SequenceNumberSetSubmessageElement<Vec<SequenceNumber>>,
    ) -> Self {
        Self(GapSubmessage {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        })
    }
}

impl Deref for GapSubmessageWrite {
    type Target = GapSubmessage<Vec<SequenceNumber>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a> From<<GapSubmessageWrite as Deref>::Target> for RtpsSubmessageTypeWrite<'a> {
    fn from(s: <GapSubmessageWrite as Deref>::Target) -> Self {
        Self::Gap(GapSubmessageWrite::new(
            s.endianness_flag,
            s.reader_id,
            s.writer_id,
            s.gap_start,
            s.gap_list,
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct GapSubmessageRead(<Self as Deref>::Target);

impl GapSubmessageRead {
    pub fn new(
        endianness_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        gap_start: SequenceNumberSubmessageElement,
        gap_list: SequenceNumberSetSubmessageElement<Vec<SequenceNumber>>,
    ) -> Self {
        Self(GapSubmessage {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        })
    }
}

impl Deref for GapSubmessageRead {
    type Target = GapSubmessage<Vec<SequenceNumber>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct HeartbeatSubmessageWrite(<Self as Deref>::Target);

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
        Self(HeartbeatSubmessage {
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

impl Deref for HeartbeatSubmessageWrite {
    type Target = HeartbeatSubmessage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<<HeartbeatSubmessageWrite as Deref>::Target> for RtpsSubmessageTypeWrite<'a> {
    fn from(s: <HeartbeatSubmessageWrite as Deref>::Target) -> Self {
        Self::Heartbeat(HeartbeatSubmessageWrite::new(
            s.endianness_flag,
            s.final_flag,
            s.liveliness_flag,
            s.reader_id,
            s.writer_id,
            s.first_sn,
            s.last_sn,
            s.count,
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct HeartbeatSubmessageRead(<Self as Deref>::Target);

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
        Self(HeartbeatSubmessage {
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

impl Deref for HeartbeatSubmessageRead {
    type Target = HeartbeatSubmessage;

    fn deref(&self) -> &Self::Target {
        &self.0
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
pub struct PadSubmessageWrite(<Self as Deref>::Target);

impl PadSubmessageWrite {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for PadSubmessageWrite {
    type Target = PadSubmessage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<<PadSubmessageWrite as Deref>::Target> for RtpsSubmessageTypeWrite<'a> {
    fn from(_: <PadSubmessageWrite as Deref>::Target) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct PadSubmessageRead(<Self as Deref>::Target);

impl PadSubmessageRead {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for PadSubmessageRead {
    type Target = PadSubmessage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
