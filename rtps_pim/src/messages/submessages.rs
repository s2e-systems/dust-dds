use crate::{
    messages::{submessage_elements, Submessage},
    structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, LocatorPIM, ParameterListPIM, ProtocolVersionPIM,
        SequenceNumberPIM, VendorIdPIM,
    },
};

use super::{
    types::{
        CountPIM, FragmentNumberPIM, ParameterIdPIM, SubmessageFlag, SubmessageKindPIM, TimePIM,
    },
    RtpsSubmessageHeaderPIM,
};

pub trait AckNackSubmessagePIM<
    PSM: SubmessageKindPIM + EntityIdPIM + SequenceNumberPIM + CountPIM + RtpsSubmessageHeaderPIM<PSM>,
>
{
    type AckNackSubmessageType: AckNackSubmessage<PSM>;
}

pub trait AckNackSubmessage<
    PSM: SubmessageKindPIM + EntityIdPIM + SequenceNumberPIM + CountPIM + RtpsSubmessageHeaderPIM<PSM>,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumberSet: submessage_elements::SequenceNumberSet<PSM>;
    type Count: submessage_elements::Count<PSM>;

    fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        reader_id: Self::EntityId,
        writer_id: Self::EntityId,
        reader_sn_state: Self::SequenceNumberSet,
        count: Self::Count,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn final_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn reader_sn_state(&self) -> &Self::SequenceNumberSet;
    fn count(&self) -> &Self::Count;
}

pub trait DataSubmessagePIM<
    'a,
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + ParameterIdPIM
        + ParameterListPIM<PSM>
        + DataPIM
        + RtpsSubmessageHeaderPIM<PSM>,
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
        + ParameterListPIM<PSM>
        + DataPIM
        + RtpsSubmessageHeaderPIM<PSM>,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumber: submessage_elements::SequenceNumber<PSM>;
    type SerializedData: submessage_elements::SerializedData<'a>;

    fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: Self::EntityId,
        writer_id: Self::EntityId,
        writer_sn: Self::SequenceNumber,
        inline_qos: &'a PSM::ParameterListType,
        serialized_payload: Self::SerializedData,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn inline_qos_flag(&self) -> SubmessageFlag;
    fn data_flag(&self) -> SubmessageFlag;
    fn key_flag(&self) -> SubmessageFlag;
    fn non_standard_payload_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn writer_sn(&self) -> &Self::SequenceNumber;
    fn inline_qos(&self) -> &PSM::ParameterListType;
    fn serialized_payload(&self) -> &Self::SerializedData;
}

pub trait DataFragSubmessagePIM<
    'a,
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + DataPIM
        + ParameterIdPIM
        + RtpsSubmessageHeaderPIM<PSM>,
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
        + RtpsSubmessageHeaderPIM<PSM>,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumber: submessage_elements::SequenceNumber<PSM>;
    type FragmentNumber: submessage_elements::FragmentNumber<PSM>;
    type UShort: submessage_elements::UShort;
    type ULong: submessage_elements::ULong;
    type SerializedData: submessage_elements::SerializedDataFragment<'a>;
    type ParameterList: submessage_elements::ParameterList<PSM>;

    fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        reader_id: Self::EntityId,
        writer_id: Self::EntityId,
        writer_sn: Self::SequenceNumber,
        fragment_starting_num: Self::FragmentNumber,
        fragments_in_submessage: Self::UShort,
        data_size: Self::ULong,
        fragment_size: Self::UShort,
        inline_qos: Self::ParameterList,
        serialized_payload: Self::SerializedData,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn inline_qos_flag(&self) -> SubmessageFlag;
    fn non_standard_payload_flag(&self) -> SubmessageFlag;
    fn key_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn writer_sn(&self) -> &Self::SequenceNumber;
    fn fragment_starting_num(&self) -> &Self::FragmentNumber;
    fn fragments_in_submessage(&self) -> &Self::UShort;
    fn data_size(&self) -> &Self::ULong;
    fn fragment_size(&self) -> &Self::UShort;
    fn inline_qos(&self) -> &Self::ParameterList;
    fn serialized_payload(&self) -> &Self::SerializedData;
}

pub trait GapSubmessagePIM<
    PSM: SubmessageKindPIM + EntityIdPIM + SequenceNumberPIM + RtpsSubmessageHeaderPIM<PSM>,
>
{
    type GapSubmessageType: GapSubmessage<PSM>;
}

pub trait GapSubmessage<
    PSM: SubmessageKindPIM + EntityIdPIM + SequenceNumberPIM + RtpsSubmessageHeaderPIM<PSM>,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumber: submessage_elements::SequenceNumber<PSM>;
    type SequenceNumberSet: submessage_elements::SequenceNumberSet<PSM>;

    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: Self::EntityId,
        writer_id: Self::EntityId,
        gap_start: Self::SequenceNumber,
        gap_list: Self::SequenceNumberSet,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn gap_start(&self) -> &Self::SequenceNumber;
    fn gap_list(&self) -> &Self::SequenceNumberSet;
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}

pub trait HeartbeatSubmessagePIM<
    PSM: SubmessageKindPIM + EntityIdPIM + SequenceNumberPIM + CountPIM + RtpsSubmessageHeaderPIM<PSM>,
>
{
    type HeartbeatSubmessageType: HeartbeatSubmessage<PSM>;
}

pub trait HeartbeatSubmessage<
    PSM: SubmessageKindPIM + EntityIdPIM + SequenceNumberPIM + CountPIM + RtpsSubmessageHeaderPIM<PSM>,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumber: submessage_elements::SequenceNumber<PSM>;
    type Count: submessage_elements::Count<PSM>;

    fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        liveliness_flag: SubmessageFlag,
        reader_id: Self::EntityId,
        writer_id: Self::EntityId,
        first_sn: Self::SequenceNumber,
        last_sn: Self::SequenceNumber,
        count: Self::Count,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn final_flag(&self) -> SubmessageFlag;
    fn liveliness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn first_sn(&self) -> &Self::SequenceNumber;
    fn last_sn(&self) -> &Self::SequenceNumber;
    fn count(&self) -> &Self::Count;
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
        + RtpsSubmessageHeaderPIM<PSM>,
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
        + RtpsSubmessageHeaderPIM<PSM>,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumber: submessage_elements::SequenceNumber<PSM>;
    type FragmentNumber: submessage_elements::FragmentNumber<PSM>;
    type Count: submessage_elements::Count<PSM>;

    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: Self::EntityId,
        writer_id: Self::EntityId,
        writer_sn: Self::SequenceNumber,
        last_fragment_num: Self::FragmentNumber,
        count: Self::Count,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn writer_sn(&self) -> &Self::SequenceNumber;
    fn last_fragment_num(&self) -> &Self::FragmentNumber;
    fn count(&self) -> &Self::Count;
}

pub trait InfoDestinationSubmessagePIM<
    PSM: SubmessageKindPIM + GuidPrefixPIM + RtpsSubmessageHeaderPIM<PSM>,
>
{
    type InfoDestinationSubmessageType: InfoDestinationSubmessage<PSM>;
}

pub trait InfoDestinationSubmessage<
    PSM: SubmessageKindPIM + GuidPrefixPIM + RtpsSubmessageHeaderPIM<PSM>,
>: Submessage<PSM>
{
    type GuidPrefix: submessage_elements::GuidPrefix<PSM>;
    fn new(endianness_flag: SubmessageFlag, guid_prefix: Self::GuidPrefix) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn guid_prefix(&self) -> &Self::GuidPrefix;
}

pub trait InfoReplySubmessagePIM<PSM: SubmessageKindPIM + LocatorPIM + RtpsSubmessageHeaderPIM<PSM>>
{
    type InfoReplySubmessageType: InfoReplySubmessage<PSM>;
}

pub trait InfoReplySubmessage<PSM: SubmessageKindPIM + LocatorPIM + RtpsSubmessageHeaderPIM<PSM>>:
    Submessage<PSM>
{
    type LocatorList: submessage_elements::LocatorList<PSM>;

    fn new(
        endianness_flag: SubmessageFlag,
        multicast_flag: SubmessageFlag,
        unicast_locator_list: Self::LocatorList,
        multicast_locator_list: Self::LocatorList,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn multicast_flag(&self) -> SubmessageFlag;
    fn unicast_locator_list(&self) -> &Self::LocatorList;
    fn multicast_locator_list(&self) -> &Self::LocatorList;
}

pub trait InfoSourceSubmessagePIM<
    PSM: SubmessageKindPIM
        + ProtocolVersionPIM
        + VendorIdPIM
        + GuidPrefixPIM
        + RtpsSubmessageHeaderPIM<PSM>,
>
{
    type InfoSourceSubmessageType: InfoSourceSubmessage<PSM>;
}

pub trait InfoSourceSubmessage<
    PSM: SubmessageKindPIM
        + ProtocolVersionPIM
        + VendorIdPIM
        + GuidPrefixPIM
        + RtpsSubmessageHeaderPIM<PSM>,
>: Submessage<PSM>
{
    type GuidPrefix: submessage_elements::GuidPrefix<PSM>;
    type ProtocolVersion: submessage_elements::ProtocolVersion<PSM>;
    type VendorId: submessage_elements::VendorId<PSM>;

    fn new(
        endianness_flag: SubmessageFlag,
        protocol_version: Self::ProtocolVersion,
        vendor_id: Self::VendorId,
        guid_prefix: Self::GuidPrefix,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn protocol_version(&self) -> &Self::ProtocolVersion;
    fn vendor_id(&self) -> &Self::VendorId;
    fn guid_prefix(&self) -> &Self::GuidPrefix;
}

pub trait InfoTimestampSubmessagePIM<
    PSM: SubmessageKindPIM + TimePIM + RtpsSubmessageHeaderPIM<PSM>,
>
{
    type InfoTimestampSubmessageType: InfoTimestampSubmessage<PSM>;
}

pub trait InfoTimestampSubmessage<PSM: SubmessageKindPIM + TimePIM + RtpsSubmessageHeaderPIM<PSM>>:
    Submessage<PSM>
{
    type Timestamp: submessage_elements::Timestamp<PSM>;

    fn new(
        endianness_flag: SubmessageFlag,
        invalidate_flag: SubmessageFlag,
        timestamp: Self::Timestamp,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn invalidate_flag(&self) -> SubmessageFlag;
    fn timestamp(&self) -> &Self::Timestamp;
}

pub trait NackFragSubmessagePIM<
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + CountPIM
        + RtpsSubmessageHeaderPIM<PSM>,
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
        + RtpsSubmessageHeaderPIM<PSM>,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumber: submessage_elements::SequenceNumber<PSM>;
    type FragmentNumberSet: submessage_elements::FragmentNumberSet<PSM>;
    type Count: submessage_elements::Count<PSM>;

    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: Self::EntityId,
        writer_id: Self::EntityId,
        writer_sn: Self::SequenceNumber,
        fragment_number_state: Self::FragmentNumberSet,
        count: Self::Count,
    ) -> Self;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn writer_sn(&self) -> &Self::SequenceNumber;
    fn fragment_number_state(&self) -> &Self::FragmentNumberSet;
    fn count(&self) -> &Self::Count;
}

pub trait PadSubmessagePIM<PSM: SubmessageKindPIM + RtpsSubmessageHeaderPIM<PSM>> {
    type PadSubmessageType: PadSubmessage<PSM>;
}

pub trait PadSubmessage<PSM: SubmessageKindPIM + RtpsSubmessageHeaderPIM<PSM>>:
    Submessage<PSM>
{
}
