use crate::implementation::rtps::types::Count;

use super::{
    submessage_elements::{
        EntityIdSubmessageElement, FragmentNumberSetSubmessageElement,
        FragmentNumberSubmessageElement, GuidPrefixSubmessageElement, LocatorListSubmessageElement,
        ParameterListSubmessageElement, ProtocolVersionSubmessageElement,
        SequenceNumberSetSubmessageElement, SequenceNumberSubmessageElement,
        SerializedDataFragmentSubmessageElement, SerializedDataSubmessageElement,
        TimestampSubmessageElement, ULongSubmessageElement, UShortSubmessageElement,
        VendorIdSubmessageElement,
    },
    types::SubmessageFlag,
};

#[derive(Debug, PartialEq, Eq)]
pub struct AckNackSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub reader_sn_state: SequenceNumberSetSubmessageElement,
    pub count: Count,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataSubmessage<'a> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub data_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub writer_sn: SequenceNumberSubmessageElement,
    pub inline_qos: ParameterListSubmessageElement<'a>,
    pub serialized_payload: SerializedDataSubmessageElement<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataFragSubmessage<'a> {
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
    pub inline_qos: ParameterListSubmessageElement<'a>,
    pub serialized_payload: SerializedDataFragmentSubmessageElement<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GapSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub gap_start: SequenceNumberSubmessageElement,
    pub gap_list: SequenceNumberSetSubmessageElement,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub liveliness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub first_sn: SequenceNumberSubmessageElement,
    pub last_sn: SequenceNumberSubmessageElement,
    pub count: Count,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatFragSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub writer_sn: SequenceNumberSubmessageElement,
    pub last_fragment_num: FragmentNumberSubmessageElement,
    pub count: Count,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoDestinationSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub guid_prefix: GuidPrefixSubmessageElement,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoReplySubmessage {
    pub endianness_flag: SubmessageFlag,
    pub multicast_flag: SubmessageFlag,
    pub unicast_locator_list: LocatorListSubmessageElement,
    pub multicast_locator_list: LocatorListSubmessageElement,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoSourceSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub protocol_version: ProtocolVersionSubmessageElement,
    pub vendor_id: VendorIdSubmessageElement,
    pub guid_prefix: GuidPrefixSubmessageElement,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoTimestampSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub invalidate_flag: SubmessageFlag,
    pub timestamp: TimestampSubmessageElement,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NackFragSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityIdSubmessageElement,
    pub writer_id: EntityIdSubmessageElement,
    pub writer_sn: SequenceNumberSubmessageElement,
    pub fragment_number_state: FragmentNumberSetSubmessageElement,
    pub count: Count,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PadSubmessage {}
