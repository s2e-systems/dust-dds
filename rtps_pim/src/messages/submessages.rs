use crate::{
    messages::Submessage,
    structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, LocatorPIM, ProtocolVersionPIM, SequenceNumberPIM,
        VendorIdPIM,
    },
};

use super::{
    submessage_elements::{
        CountSubmessageElementPIM, EntityIdSubmessageElementPIM,
        FragmentNumberSetSubmessageElementPIM, FragmentNumberSubmessageElementPIM,
        GuidPrefixSubmessageElementPIM, LocatorListSubmessageElementPIM,
        ParameterListSubmessageElementPIM, ProtocolVersionSubmessageElementPIM,
        SequenceNumberSetSubmessageElementPIM, SequenceNumberSubmessageElementPIM,
        SerializedDataFragmentSubmessageElementPIM, SerializedDataSubmessageElementPIM,
        TimestampSubmessageElementPIM, ULongSubmessageElementPIM, UShortSubmessageElementPIM,
        VendorIdSubmessageElementPIM,
    },
    types::{
        CountPIM, FragmentNumberPIM, ParameterIdPIM, SubmessageFlag, SubmessageKindPIM, TimePIM,
    },
    RtpsSubmessageHeaderPIM,
};

pub enum RtpsSubmessageType<
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + CountPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSetSubmessageElementPIM<PSM>
        + CountSubmessageElementPIM<PSM>
        + AckNackSubmessagePIM<PSM>,
> {
    AckNack(PSM::AckNackSubmessageType),
}

pub trait AckNackSubmessagePIM<
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + CountPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSetSubmessageElementPIM<PSM>
        + CountSubmessageElementPIM<PSM>,
>
{
    type AckNackSubmessageType: AckNackSubmessage<PSM>;
}

pub trait AckNackSubmessage<
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + CountPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSetSubmessageElementPIM<PSM>
        + CountSubmessageElementPIM<PSM>,
>: Submessage<PSM>
{
    fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        reader_id: PSM::EntityIdSubmessageElementType,
        writer_id: PSM::EntityIdSubmessageElementType,
        reader_sn_state: PSM::SequenceNumberSetSubmessageElementType,
        count: PSM::CountSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn final_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn reader_sn_state(&self) -> &PSM::SequenceNumberSetSubmessageElementType;
    fn count(&self) -> &PSM::CountSubmessageElementType;
}

pub trait DataSubmessagePIM<
    'a,
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + ParameterIdPIM
        + ParameterListSubmessageElementPIM<PSM>
        + DataPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + ParameterListSubmessageElementPIM<PSM>
        + SerializedDataSubmessageElementPIM<'a>
        + DataSubmessagePIM<'a, PSM>,
>
{
    type DataSubmessageType: DataSubmessage<'a, PSM>;
}

pub trait DataSubmessage<
    'a,
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + ParameterIdPIM
        + ParameterListSubmessageElementPIM<PSM>
        + DataPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + ParameterListSubmessageElementPIM<PSM>
        + SerializedDataSubmessageElementPIM<'a>
        + DataSubmessagePIM<'a, PSM>,
>: Submessage<PSM>
{
    fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: PSM::EntityIdSubmessageElementType,
        writer_id: PSM::EntityIdSubmessageElementType,
        writer_sn: PSM::SequenceNumberSubmessageElementType,
        inline_qos: PSM::ParameterListSubmessageElementType,
        serialized_payload: PSM::SerializedDataSubmessageElementType,
    ) -> <PSM as DataSubmessagePIM<'a, PSM>>::DataSubmessageType;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn inline_qos_flag(&self) -> SubmessageFlag;
    fn data_flag(&self) -> SubmessageFlag;
    fn key_flag(&self) -> SubmessageFlag;
    fn non_standard_payload_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn writer_sn(&self) -> &PSM::SequenceNumberSubmessageElementType;
    fn inline_qos(&self) -> &PSM::ParameterListSubmessageElementType;
    fn serialized_payload(&'a self) -> &'a PSM::SerializedDataSubmessageElementType;
}

pub trait DataFragSubmessagePIM<
    'a,
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + DataPIM
        + ParameterIdPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + FragmentNumberSubmessageElementPIM<PSM>
        + UShortSubmessageElementPIM
        + ULongSubmessageElementPIM
        + ParameterListSubmessageElementPIM<PSM>
        + SerializedDataFragmentSubmessageElementPIM<'a>,
>
{
    type DataFragSubmessageType: DataFragSubmessage<'a, PSM>;
}

pub trait DataFragSubmessage<
    'a,
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + DataPIM
        + ParameterIdPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + FragmentNumberSubmessageElementPIM<PSM>
        + UShortSubmessageElementPIM
        + ULongSubmessageElementPIM
        + ParameterListSubmessageElementPIM<PSM>
        + SerializedDataFragmentSubmessageElementPIM<'a>,
>: Submessage<PSM>
{
    fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        reader_id: PSM::EntityIdSubmessageElementType,
        writer_id: PSM::EntityIdSubmessageElementType,
        writer_sn: PSM::SequenceNumberSubmessageElementType,
        fragment_starting_num: PSM::FragmentNumberSubmessageElementType,
        fragments_in_submessage: PSM::UShortSubmessageElementType,
        data_size: PSM::ULongSubmessageElementType,
        fragment_size: PSM::UShortSubmessageElementType,
        inline_qos: PSM::ParameterListSubmessageElementType,
        serialized_payload: PSM::SerializedDataFragmentSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn inline_qos_flag(&self) -> SubmessageFlag;
    fn non_standard_payload_flag(&self) -> SubmessageFlag;
    fn key_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn writer_sn(&self) -> &PSM::SequenceNumberSubmessageElementType;
    fn fragment_starting_num(&self) -> &PSM::FragmentNumberSubmessageElementType;
    fn fragments_in_submessage(&self) -> &PSM::UShortSubmessageElementType;
    fn data_size(&self) -> &PSM::ULongSubmessageElementType;
    fn fragment_size(&self) -> &PSM::UShortSubmessageElementType;
    fn inline_qos(&self) -> &PSM::ParameterListSubmessageElementType;
    fn serialized_payload(&self) -> &PSM::SerializedDataFragmentSubmessageElementType;
}

pub trait GapSubmessagePIM<
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + SequenceNumberSetSubmessageElementPIM<PSM>,
>
{
    type GapSubmessageType: GapSubmessage<PSM>;
}

pub trait GapSubmessage<
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + SequenceNumberSetSubmessageElementPIM<PSM>,
>: Submessage<PSM>
{
    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: PSM::EntityIdSubmessageElementType,
        writer_id: PSM::EntityIdSubmessageElementType,
        gap_start: PSM::SequenceNumberSubmessageElementType,
        gap_list: PSM::SequenceNumberSetSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn gap_start(&self) -> &PSM::SequenceNumberSubmessageElementType;
    fn gap_list(&self) -> &PSM::SequenceNumberSetSubmessageElementType;
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}

pub trait HeartbeatSubmessagePIM<
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + CountPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + CountSubmessageElementPIM<PSM>,
>
{
    type HeartbeatSubmessageType: HeartbeatSubmessage<PSM>;
}

pub trait HeartbeatSubmessage<
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + CountPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + CountSubmessageElementPIM<PSM>,
>: Submessage<PSM>
{
    fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        liveliness_flag: SubmessageFlag,
        reader_id: PSM::EntityIdSubmessageElementType,
        writer_id: PSM::EntityIdSubmessageElementType,
        first_sn: PSM::SequenceNumberSubmessageElementType,
        last_sn: PSM::SequenceNumberSubmessageElementType,
        count: PSM::CountSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn final_flag(&self) -> SubmessageFlag;
    fn liveliness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn first_sn(&self) -> &PSM::SequenceNumberSubmessageElementType;
    fn last_sn(&self) -> &PSM::SequenceNumberSubmessageElementType;
    fn count(&self) -> &PSM::CountSubmessageElementType;
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}

pub trait HeartbeatFragSubmessagePIM<
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + CountPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + FragmentNumberSubmessageElementPIM<PSM>
        + CountSubmessageElementPIM<PSM>,
>
{
    type HeartbeatFragSubmessageType: HeartbeatFragSubmessage<PSM>;
}

pub trait HeartbeatFragSubmessage<
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + CountPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + FragmentNumberSubmessageElementPIM<PSM>
        + CountSubmessageElementPIM<PSM>,
>: Submessage<PSM>
{
    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: PSM::EntityIdSubmessageElementType,
        writer_id: PSM::EntityIdSubmessageElementType,
        writer_sn: PSM::SequenceNumberSubmessageElementType,
        last_fragment_num: PSM::FragmentNumberSubmessageElementType,
        count: PSM::CountSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn writer_sn(&self) -> &PSM::SequenceNumberSubmessageElementType;
    fn last_fragment_num(&self) -> &PSM::FragmentNumberSubmessageElementType;
    fn count(&self) -> &PSM::CountSubmessageElementType;
}

pub trait InfoDestinationSubmessagePIM<
    PSM: SubmessageKindPIM
        + GuidPrefixPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + GuidPrefixSubmessageElementPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + EntityIdPIM,
>
{
    type InfoDestinationSubmessageType: InfoDestinationSubmessage<PSM>;
}

pub trait InfoDestinationSubmessage<
    PSM: SubmessageKindPIM
        + GuidPrefixPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + GuidPrefixSubmessageElementPIM<PSM>,
>: Submessage<PSM>
{
    fn new(
        endianness_flag: SubmessageFlag,
        guid_prefix: PSM::GuidPrefixSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn guid_prefix(&self) -> &PSM::GuidPrefixSubmessageElementType;
}

pub trait InfoReplySubmessagePIM<
    PSM: SubmessageKindPIM
        + LocatorPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + LocatorListSubmessageElementPIM<PSM>,
>
{
    type InfoReplySubmessageType: InfoReplySubmessage<PSM>;
}

pub trait InfoReplySubmessage<
    PSM: SubmessageKindPIM
        + LocatorPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + LocatorListSubmessageElementPIM<PSM>,
>: Submessage<PSM>
{
    fn new(
        endianness_flag: SubmessageFlag,
        multicast_flag: SubmessageFlag,
        unicast_locator_list: PSM::LocatorListSubmessageElementType,
        multicast_locator_list: PSM::LocatorListSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn multicast_flag(&self) -> SubmessageFlag;
    fn unicast_locator_list(&self) -> &PSM::LocatorListSubmessageElementType;
    fn multicast_locator_list(&self) -> &PSM::LocatorListSubmessageElementType;
}

pub trait InfoSourceSubmessagePIM<
    PSM: SubmessageKindPIM
        + ProtocolVersionPIM
        + VendorIdPIM
        + GuidPrefixPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + ProtocolVersionSubmessageElementPIM<PSM>
        + VendorIdSubmessageElementPIM<PSM>
        + GuidPrefixSubmessageElementPIM<PSM>,
>
{
    type InfoSourceSubmessageType: InfoSourceSubmessage<PSM>;
}

pub trait InfoSourceSubmessage<
    PSM: SubmessageKindPIM
        + ProtocolVersionPIM
        + VendorIdPIM
        + GuidPrefixPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + ProtocolVersionSubmessageElementPIM<PSM>
        + VendorIdSubmessageElementPIM<PSM>
        + GuidPrefixSubmessageElementPIM<PSM>,
>: Submessage<PSM>
{
    fn new(
        endianness_flag: SubmessageFlag,
        protocol_version: PSM::ProtocolVersionSubmessageElementType,
        vendor_id: PSM::VendorIdSubmessageElementType,
        guid_prefix: PSM::GuidPrefixSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn protocol_version(&self) -> &PSM::ProtocolVersionSubmessageElementType;
    fn vendor_id(&self) -> &PSM::VendorIdSubmessageElementType;
    fn guid_prefix(&self) -> &PSM::GuidPrefixSubmessageElementType;
}

pub trait InfoTimestampSubmessagePIM<
    'a,
    PSM: SubmessageKindPIM + TimePIM + RtpsSubmessageHeaderPIM<PSM> + TimestampSubmessageElementPIM<PSM>,
>
{
    type InfoTimestampSubmessageType: InfoTimestampSubmessage<PSM>;
}

pub trait InfoTimestampSubmessage<
    PSM: SubmessageKindPIM + TimePIM + RtpsSubmessageHeaderPIM<PSM> + TimestampSubmessageElementPIM<PSM>,
>: Submessage<PSM>
{
    fn new(
        endianness_flag: SubmessageFlag,
        invalidate_flag: SubmessageFlag,
        timestamp: PSM::TimestampSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn invalidate_flag(&self) -> SubmessageFlag;
    fn timestamp(&self) -> &PSM::TimestampSubmessageElementType;
}

pub trait NackFragSubmessagePIM<
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + CountPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + FragmentNumberSetSubmessageElementPIM<PSM>
        + CountSubmessageElementPIM<PSM>,
>
{
    type NackFragSubmessageType: NackFragSubmessage<PSM>;
}

pub trait NackFragSubmessage<
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + CountPIM
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + FragmentNumberSetSubmessageElementPIM<PSM>
        + CountSubmessageElementPIM<PSM>,
>: Submessage<PSM>
{
    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: PSM::EntityIdSubmessageElementType,
        writer_id: PSM::EntityIdSubmessageElementType,
        writer_sn: PSM::SequenceNumberSubmessageElementType,
        fragment_number_state: PSM::FragmentNumberSetSubmessageElementType,
        count: PSM::CountSubmessageElementType,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &PSM::EntityIdSubmessageElementType;
    fn writer_sn(&self) -> &PSM::SequenceNumberSubmessageElementType;
    fn fragment_number_state(&self) -> &PSM::FragmentNumberSetSubmessageElementType;
    fn count(&self) -> &PSM::CountSubmessageElementType;
}

pub trait PadSubmessagePIM<PSM: SubmessageKindPIM + RtpsSubmessageHeaderPIM<PSM>> {
    type PadSubmessageType: PadSubmessage<PSM>;
}

pub trait PadSubmessage<PSM: SubmessageKindPIM + RtpsSubmessageHeaderPIM<PSM>>:
    Submessage<PSM>
{
}
