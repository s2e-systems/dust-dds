use super::{
    submessage_elements::{
        CountSubmessageElement, EntityIdSubmessageElement, FragmentNumberSetSubmessageElement,
        FragmentNumberSubmessageElement, GuidPrefixSubmessageElement, LocatorListSubmessageElement,
        ParameterListSubmessageElement, ProtocolVersionSubmessageElement,
        SequenceNumberSetSubmessageElement, SequenceNumberSubmessageElement,
        SerializedDataFragmentSubmessageElement, SerializedDataSubmessageElement,
        TimestampSubmessageElement, TimestampSubmessageElementType, ULongSubmessageElement,
        UShortSubmessageElement, VendorIdSubmessageElement,
    },
    types::SubmessageFlag,
};

pub trait RtpsSubmessagePIM<'a> {
    type AckNackSubmessageType;
    type DataSubmessageType;
    type DataFragSubmessageType;
    type GapSubmessageType;
    type HeartbeatSubmessageType;
    type HeartbeatFragSubmessageType;
    type InfoDestinationSubmessageType;
    type InfoReplySubmessageType;
    type InfoSourceSubmessageType;
    type InfoTimestampSubmessageType;
    type NackFragSubmessageType;
    type PadSubmessageType;
}

#[derive(Debug, PartialEq)]
pub enum RtpsSubmessageType<'a, S, P, L, F> {
    AckNack(AckNackSubmessage<S>),
    Data(DataSubmessage<'a, P>),
    DataFrag(DataFragSubmessage<'a, P>),
    Gap(GapSubmessage<S>),
    Heartbeat(HeartbeatSubmessage),
    HeartbeatFrag(HeartbeatFragSubmessage),
    InfoDestination(InfoDestinationSubmessage),
    InfoReply(InfoReplySubmessage<L>),
    InfoSource(InfoSourceSubmessage),
    InfoTimestamp(InfoTimestampSubmessage),
    NackFrag(NackFragSubmessage<F>),
    Pad(PadSubmessage),
}

#[derive(Debug, PartialEq)]
pub struct AckNackSubmessage<S> {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub reader_sn_state: SequenceNumberSetSubmessageElement<S>,
    pub count: CountSubmessageElement,
}

#[derive(Debug, PartialEq)]
pub struct DataSubmessage<'a, P> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub data_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub writer_sn: SequenceNumberSubmessageElement,
    pub inline_qos: ParameterListSubmessageElement<'a, P>,
    pub serialized_payload: SerializedDataSubmessageElement<'a>,
}
pub trait AckNackSubmessageTrait {
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
    fn endianness_flag(&self) -> SubmessageFlag;
    fn final_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn reader_sn_state(&self) -> &Self::SequenceNumberSetSubmessageElementType;
    fn count(&self) -> &Self::CountSubmessageElementType;
}

#[derive(Debug, PartialEq)]
pub struct DataFragSubmessage<'a, P> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub writer_sn: SequenceNumberSubmessageElement,
    pub fragment_starting_num: FragmentNumberSubmessageElement,
    pub fragments_in_submessage: UShortSubmessageElement,
    pub data_size: ULongSubmessageElement,
    pub fragment_size: UShortSubmessageElement,
    pub inline_qos: ParameterListSubmessageElement<'a, P>,
    pub serialized_payload: SerializedDataFragmentSubmessageElement<'a>,
}

#[derive(Debug, PartialEq)]
pub struct GapSubmessage<S> {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub gap_start: SequenceNumberSubmessageElement,
    pub gap_list: SequenceNumberSetSubmessageElement<S>,
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct PadSubmessage {}

pub trait DataFragSubmessageTrait {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type FragmentNumberSubmessageElementType;
    type UShortSubmessageElementType;
    type ULongSubmessageElementType;
    type ParameterListSubmessageElementType;
    type SerializedDataFragmentSubmessageElementType;

    fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        reader_id: Self::EntityIdSubmessageElementType,
        writer_id: Self::EntityIdSubmessageElementType,
        writer_sn: Self::SequenceNumberSubmessageElementType,
        fragment_starting_num: Self::FragmentNumberSubmessageElementType,
        fragments_in_submessage: Self::UShortSubmessageElementType,
        data_size: Self::ULongSubmessageElementType,
        fragment_size: Self::UShortSubmessageElementType,
        inline_qos: Self::ParameterListSubmessageElementType,
        serialized_payload: Self::SerializedDataFragmentSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn inline_qos_flag(&self) -> SubmessageFlag;
    fn non_standard_payload_flag(&self) -> SubmessageFlag;
    fn key_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType;
    fn fragment_starting_num(&self) -> &Self::FragmentNumberSubmessageElementType;
    fn fragments_in_submessage(&self) -> &Self::UShortSubmessageElementType;
    fn data_size(&self) -> &Self::ULongSubmessageElementType;
    fn fragment_size(&self) -> &Self::UShortSubmessageElementType;
    fn inline_qos(&self) -> &Self::ParameterListSubmessageElementType;
    fn serialized_payload(&self) -> &Self::SerializedDataFragmentSubmessageElementType;
}

pub trait HeartbeatSubmessageTrait {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type CountSubmessageElementType;

    fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        liveliness_flag: SubmessageFlag,
        reader_id: Self::EntityIdSubmessageElementType,
        writer_id: Self::EntityIdSubmessageElementType,
        first_sn: Self::SequenceNumberSubmessageElementType,
        last_sn: Self::SequenceNumberSubmessageElementType,
        count: Self::CountSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn final_flag(&self) -> SubmessageFlag;
    fn liveliness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn first_sn(&self) -> &Self::SequenceNumberSubmessageElementType;
    fn last_sn(&self) -> &Self::SequenceNumberSubmessageElementType;
    fn count(&self) -> &Self::CountSubmessageElementType;
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}

pub trait HeartbeatFragSubmessageTrait {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type FragmentNumberSubmessageElementType;
    type CountSubmessageElementType;

    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: Self::EntityIdSubmessageElementType,
        writer_id: Self::EntityIdSubmessageElementType,
        writer_sn: Self::SequenceNumberSubmessageElementType,
        last_fragment_num: Self::FragmentNumberSubmessageElementType,
        count: Self::CountSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType;
    fn last_fragment_num(&self) -> &Self::FragmentNumberSubmessageElementType;
    fn count(&self) -> &Self::CountSubmessageElementType;
}

pub trait InfoDestinationSubmessageTrait {
    type GuidPrefixSubmessageElementType;
    fn new(
        endianness_flag: SubmessageFlag,
        guid_prefix: Self::GuidPrefixSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn guid_prefix(&self) -> &Self::GuidPrefixSubmessageElementType;
}

pub trait InfoReplySubmessageTrait {
    type LocatorListSubmessageElementType;

    fn new(
        endianness_flag: SubmessageFlag,
        multicast_flag: SubmessageFlag,
        unicast_locator_list: Self::LocatorListSubmessageElementType,
        multicast_locator_list: Self::LocatorListSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn multicast_flag(&self) -> SubmessageFlag;
    fn unicast_locator_list(&self) -> &Self::LocatorListSubmessageElementType;
    fn multicast_locator_list(&self) -> &Self::LocatorListSubmessageElementType;
}

pub trait InfoSourceSubmessageTrait {
    type ProtocolVersionSubmessageElementType;
    type VendorIdSubmessageElementType;
    type GuidPrefixSubmessageElementType;

    fn new(
        endianness_flag: SubmessageFlag,
        protocol_version: Self::ProtocolVersionSubmessageElementType,
        vendor_id: Self::VendorIdSubmessageElementType,
        guid_prefix: Self::GuidPrefixSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn protocol_version(&self) -> &Self::ProtocolVersionSubmessageElementType;
    fn vendor_id(&self) -> &Self::VendorIdSubmessageElementType;
    fn guid_prefix(&self) -> &Self::GuidPrefixSubmessageElementType;
}

pub trait InfoTimestampSubmessageTrait {
    type TimestampSubmessageElementType: TimestampSubmessageElementType;
    fn new(
        endianness_flag: SubmessageFlag,
        invalidate_flag: SubmessageFlag,
        timestamp: Self::TimestampSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn invalidate_flag(&self) -> SubmessageFlag;
    fn timestamp(&self) -> &Self::TimestampSubmessageElementType;
}

pub trait NackFragSubmessageTrait {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type FragmentNumberSetSubmessageElementType;
    type CountSubmessageElementType;

    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: Self::EntityIdSubmessageElementType,
        writer_id: Self::EntityIdSubmessageElementType,
        writer_sn: Self::SequenceNumberSubmessageElementType,
        fragment_number_state: Self::FragmentNumberSetSubmessageElementType,
        count: Self::CountSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType;
    fn fragment_number_state(&self) -> &Self::FragmentNumberSetSubmessageElementType;
    fn count(&self) -> &Self::CountSubmessageElementType;
}

pub trait PadSubmessageTrait {}
