use super::{
    submessage_elements::{
        CountSubmessageElement, EntityIdSubmessageElement, FragmentNumberSetSubmessageElement,
        FragmentNumberSubmessageElement, GuidPrefixSubmessageElement, LocatorListSubmessageElement,
        ParameterListSubmessageElement, ProtocolVersionSubmessageElement,
        SequenceNumberSetSubmessageElement, SequenceNumberSubmessageElement,
        SerializedDataFragmentSubmessageElement, SerializedDataSubmessageElement,
        TimestampSubmessageElement, ULongSubmessageElement, UShortSubmessageElement,
        VendorIdSubmessageElement,
    },
    types::SubmessageFlag,
};

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
pub struct DataSubmessage<P, D> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub data_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub writer_sn: SequenceNumberSubmessageElement,
    pub inline_qos: ParameterListSubmessageElement<P>,
    pub serialized_payload: SerializedDataSubmessageElement<D>,
}

#[derive(Debug, PartialEq)]
pub struct DataFragSubmessage<P, D> {
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
    pub inline_qos: ParameterListSubmessageElement<P>,
    pub serialized_payload: SerializedDataFragmentSubmessageElement<D>,
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
