use super::{
    submessage_elements::{
        CountSubmessageElement, EntityIdSubmessageElement, FragmentNumberSetSubmessageElement,
        FragmentNumberSubmessageElement, GuidPrefixSubmessageElement, LocatorListSubmessageElement,
        ProtocolVersionSubmessageElement, SequenceNumberSubmessageElement,
        TimestampSubmessageElement, VendorIdSubmessageElement,
    },
    types::SubmessageFlag,
};

pub trait AckNackSubmessageConstructor {
    type EntityIdSubmessageElementType;
    type SequenceNumberSetSubmessageElementType;
    type CountSubmessageElementType;

    fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        reader_id: Self::EntityIdSubmessageElementType,
        writer_id: Self::EntityIdSubmessageElementType,
        reader_sn_state: Self::SequenceNumberSetSubmessageElementType,
        count: Self::CountSubmessageElementType,
    ) -> Self;
}

pub trait AckNackSubmessageAttributes {
    type EntityIdSubmessageElementType;
    type SequenceNumberSetSubmessageElementType;
    type CountSubmessageElementType;

    fn endianness_flag(&self) -> &SubmessageFlag;
    fn final_flag(&self) -> &SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn reader_sn_state(&self) -> &Self::SequenceNumberSetSubmessageElementType;
    fn count(&self) -> &Self::CountSubmessageElementType;
}

pub trait DataSubmessageConstructor {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type ParameterListSubmessageElementType;
    type SerializedDataSubmessageElementType;

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
    ) -> Self;
}

pub trait DataSubmessageAttributes {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type ParameterListSubmessageElementType;
    type SerializedDataSubmessageElementType;

    fn endianness_flag(&self) -> &SubmessageFlag;
    fn inline_qos_flag(&self) -> &SubmessageFlag;
    fn data_flag(&self) -> &SubmessageFlag;
    fn key_flag(&self) -> &SubmessageFlag;
    fn non_standard_payload_flag(&self) -> &SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType;
    fn inline_qos(&self) -> &Self::ParameterListSubmessageElementType;
    fn serialized_payload(&self) -> &Self::SerializedDataSubmessageElementType;
}

pub trait GapSubmessageConstructor {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type SequenceNumberSetSubmessageElementType;

    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: Self::EntityIdSubmessageElementType,
        writer_id: Self::EntityIdSubmessageElementType,
        gap_start: Self::SequenceNumberSubmessageElementType,
        gap_list: Self::SequenceNumberSetSubmessageElementType,
    ) -> Self;
}

pub trait GapSubmessageAttributes {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type SequenceNumberSetSubmessageElementType;

    fn endianness_flag(&self) -> &SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn gap_start(&self) -> &Self::SequenceNumberSubmessageElementType;
    fn gap_list(&self) -> &Self::SequenceNumberSetSubmessageElementType;
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
pub struct HeartbeatFragSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub writer_sn: SequenceNumberSubmessageElement,
    pub last_fragment_num: FragmentNumberSubmessageElement,
    pub count: CountSubmessageElement,
}

#[derive(Debug, PartialEq)]
pub struct InfoDestinationSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub guid_prefix: GuidPrefixSubmessageElement,
}

#[derive(Debug, PartialEq)]
pub struct InfoReplySubmessage<L> {
    pub endianness_flag: SubmessageFlag,
    pub multicast_flag: SubmessageFlag,
    pub unicast_locator_list: LocatorListSubmessageElement<L>,
    pub multicast_locator_list: LocatorListSubmessageElement<L>,
}

#[derive(Debug, PartialEq)]
pub struct InfoSourceSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub protocol_version: ProtocolVersionSubmessageElement,
    pub vendor_id: VendorIdSubmessageElement,
    pub guid_prefix: GuidPrefixSubmessageElement,
}

#[derive(Debug, PartialEq)]
pub struct InfoTimestampSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub invalidate_flag: SubmessageFlag,
    pub timestamp: TimestampSubmessageElement,
}

#[derive(Debug, PartialEq)]
pub struct NackFragSubmessage<F> {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub writer_sn: SequenceNumberSubmessageElement,
    pub fragment_number_state: FragmentNumberSetSubmessageElement<F>,
    pub count: CountSubmessageElement,
}

pub trait PadSubmessageConstructor {
    fn new() -> Self;
}

pub trait PadSubmessageAttributes {}
