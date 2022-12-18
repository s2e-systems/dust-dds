use crate::implementation::rtps::types::{
    Count, EntityId, GuidPrefix, ProtocolVersion, SequenceNumber, VendorId,
};

use super::{
    submessage_elements::{
        FragmentNumberSet, LocatorListSubmessageElement, ParameterList, SequenceNumberSet,
        SerializedDataFragmentSubmessageElement, SerializedDataSubmessageElement,
    },
    types::{FragmentNumber, SubmessageFlag, Time, ULong, UShort},
};

#[derive(Debug, PartialEq, Eq)]
pub struct AckNackSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub reader_sn_state: SequenceNumberSet,
    pub count: Count,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataSubmessage<'a> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub data_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub inline_qos: ParameterList<'a>,
    pub serialized_payload: SerializedDataSubmessageElement<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataFragSubmessage<'a> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub fragment_starting_num: FragmentNumber,
    pub fragments_in_submessage: UShort,
    pub data_size: ULong,
    pub fragment_size: UShort,
    pub inline_qos: ParameterList<'a>,
    pub serialized_payload: SerializedDataFragmentSubmessageElement<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GapSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub gap_start: SequenceNumber,
    pub gap_list: SequenceNumberSet,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub liveliness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub first_sn: SequenceNumber,
    pub last_sn: SequenceNumber,
    pub count: Count,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatFragSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub last_fragment_num: FragmentNumber,
    pub count: Count,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoDestinationSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub guid_prefix: GuidPrefix,
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
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoTimestampSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub invalidate_flag: SubmessageFlag,
    pub timestamp: Time,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NackFragSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub fragment_number_state: FragmentNumberSet,
    pub count: Count,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PadSubmessage {}
