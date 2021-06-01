use crate::{
    messages::{submessage_elements, Submessage},
    structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, LocatorPIM, ProtocolVersionPIM, SequenceNumberPIM,
        VendorIdPIM,
    },
};

use super::types::{
    CountPIM, FragmentNumberPIM, ParameterIdPIM, SubmessageFlagPIM, SubmessageKindPIM, TimePIM,
};

pub trait AckNackSubmessagePIM<
    PSM: SubmessageKindPIM + SubmessageFlagPIM + EntityIdPIM + SequenceNumberPIM + CountPIM,
>
{
    type AckNackSubmessageType: AckNackSubmessage<PSM>;
}

pub trait AckNackSubmessage<
    PSM: SubmessageKindPIM + SubmessageFlagPIM + EntityIdPIM + SequenceNumberPIM + CountPIM,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumberSet: submessage_elements::SequenceNumberSet<PSM>;
    type Count: submessage_elements::Count<PSM>;

    fn endianness_flag(&self) -> PSM::SubmessageFlagType;
    fn final_flag(&self) -> PSM::SubmessageFlagType;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn reader_sn_state(&self) -> &Self::SequenceNumberSet;
    fn count(&self) -> &Self::Count;
}

pub trait DataSubmessagePIM<
    PSM: SubmessageKindPIM + SubmessageFlagPIM + EntityIdPIM + SequenceNumberPIM + DataPIM,
>
{
    type DataSubmessageType: DataSubmessage<PSM>;
}

pub trait DataSubmessage<
    PSM: SubmessageKindPIM + SubmessageFlagPIM + EntityIdPIM + SequenceNumberPIM + DataPIM,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumber: submessage_elements::SequenceNumber<PSM>;
    type SerializedData: submessage_elements::SerializedData;

    fn endianness_flag(&self) -> PSM::SubmessageFlagType;
    fn inline_qos_flag(&self) -> PSM::SubmessageFlagType;
    fn data_flag(&self) -> PSM::SubmessageFlagType;
    fn key_flag(&self) -> PSM::SubmessageFlagType;
    fn non_standard_payload_flag(&self) -> PSM::SubmessageFlagType;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn writer_sn(&self) -> &Self::SequenceNumber;
    // pub inline_qos: <Self::PSM as structure::Types>::ParameterVector,
    fn serialized_payload(&self) -> &Self::SerializedData;
}

pub trait DataFragSubmessagePIM<
    PSM: SubmessageKindPIM
        + SubmessageFlagPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + DataPIM
        + ParameterIdPIM,
>
{
    type DataFragSubmessageType: DataFragSubmessage<PSM>;
}

pub trait DataFragSubmessage<
    PSM: SubmessageKindPIM
        + SubmessageFlagPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + DataPIM
        + ParameterIdPIM,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumber: submessage_elements::SequenceNumber<PSM>;
    type FragmentNumber: submessage_elements::FragmentNumber<PSM>;
    type UShort: submessage_elements::UShort;
    type ULong: submessage_elements::ULong;
    type SerializedData: submessage_elements::SerializedDataFragment;
    type ParameterList: submessage_elements::ParameterList<PSM>;

    fn endianness_flag(&self) -> PSM::SubmessageFlagType;
    fn inline_qos_flag(&self) -> PSM::SubmessageFlagType;
    fn non_standard_payload_flag(&self) -> PSM::SubmessageFlagType;
    fn key_flag(&self) -> PSM::SubmessageFlagType;
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
    PSM: SubmessageKindPIM + SubmessageFlagPIM + EntityIdPIM + SequenceNumberPIM,
>
{
    type GapSubmessageType: GapSubmessage<PSM>;
}

pub trait GapSubmessage<
    PSM: SubmessageKindPIM + SubmessageFlagPIM + EntityIdPIM + SequenceNumberPIM,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumber: submessage_elements::SequenceNumber<PSM>;
    type SequenceNumberSet: submessage_elements::SequenceNumberSet<PSM>;

    fn endianness_flag(&self) -> PSM::SubmessageFlagType;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn gap_start(&self) -> &Self::SequenceNumber;
    fn gap_list(&self) -> &Self::SequenceNumberSet;
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}

pub trait HeartbeatSubmessagePIM<
    PSM: SubmessageKindPIM + SubmessageFlagPIM + EntityIdPIM + SequenceNumberPIM + CountPIM,
>
{
    type HeartbeatSubmessageType: HeartbeatSubmessage<PSM>;
}

pub trait HeartbeatSubmessage<
    PSM: SubmessageKindPIM + SubmessageFlagPIM + EntityIdPIM + SequenceNumberPIM + CountPIM,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumber: submessage_elements::SequenceNumber<PSM>;
    type Count: submessage_elements::Count<PSM>;

    fn endianness_flag(&self) -> PSM::SubmessageFlagType;
    fn final_flag(&self) -> PSM::SubmessageFlagType;
    fn liveliness_flag(&self) -> PSM::SubmessageFlagType;
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
        + SubmessageFlagPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + CountPIM,
>
{
    type HeartbeatFragSubmessageType: HeartbeatFragSubmessage<PSM>;
}

pub trait HeartbeatFragSubmessage<
    PSM: SubmessageKindPIM
        + SubmessageFlagPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + CountPIM,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumber: submessage_elements::SequenceNumber<PSM>;
    type FragmentNumber: submessage_elements::FragmentNumber<PSM>;
    type Count: submessage_elements::Count<PSM>;

    fn endianness_flag(&self) -> PSM::SubmessageFlagType;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn writer_sn(&self) -> &Self::SequenceNumber;
    fn last_fragment_num(&self) -> &Self::FragmentNumber;
    fn count(&self) -> &Self::Count;
}

pub trait InfoDestinationSubmessagePIM<PSM: SubmessageKindPIM + SubmessageFlagPIM + GuidPrefixPIM> {
    type InfoDestinationSubmessageType: InfoDestinationSubmessage<PSM>;
}

pub trait InfoDestinationSubmessage<PSM: SubmessageKindPIM + SubmessageFlagPIM + GuidPrefixPIM>:
    Submessage<PSM>
{
    type GuidPrefix: submessage_elements::GuidPrefix<PSM>;

    fn endianness_flag(&self) -> PSM::SubmessageFlagType;
    fn guid_prefix(&self) -> &Self::GuidPrefix;
}

pub trait InfoReplySubmessagePIM<PSM: SubmessageKindPIM + SubmessageFlagPIM + LocatorPIM> {
    type InfoReplySubmessageType: InfoReplySubmessage<PSM>;
}

pub trait InfoReplySubmessage<PSM: SubmessageKindPIM + SubmessageFlagPIM + LocatorPIM>:
    Submessage<PSM>
{
    type LocatorList: submessage_elements::LocatorList<PSM>;

    fn endianness_flag(&self) -> PSM::SubmessageFlagType;
    fn multicast_flag(&self) -> PSM::SubmessageFlagType;
    fn unicast_locator_list(&self) -> &Self::LocatorList;
    fn multicast_locator_list(&self) -> &Self::LocatorList;
}

pub trait InfoSourceSubmessagePIM<
    PSM: SubmessageKindPIM + SubmessageFlagPIM + ProtocolVersionPIM + VendorIdPIM + GuidPrefixPIM,
>
{
    type InfoSourceSubmessageType: InfoSourceSubmessage<PSM>;
}

pub trait InfoSourceSubmessage<
    PSM: SubmessageKindPIM + SubmessageFlagPIM + ProtocolVersionPIM + VendorIdPIM + GuidPrefixPIM,
>: Submessage<PSM>
{
    type GuidPrefix: submessage_elements::GuidPrefix<PSM>;
    type ProtocolVersion: submessage_elements::ProtocolVersion<PSM>;
    type VendorId: submessage_elements::VendorId<PSM>;

    fn endianness_flag(&self) -> PSM::SubmessageFlagType;
    fn protocol_version(&self) -> Self::ProtocolVersion;
    fn vendor_id(&self) -> &Self::VendorId;
    fn guid_prefix(&self) -> &Self::GuidPrefix;
}

pub trait InfoTimestampSubmessagePIM<PSM: SubmessageKindPIM + SubmessageFlagPIM + TimePIM> {
    type InfoTimestampSubmessageType: InfoTimestampSubmessage<PSM>;
}

pub trait InfoTimestampSubmessage<PSM: SubmessageKindPIM + SubmessageFlagPIM + TimePIM>:
    Submessage<PSM>
{
    type Timestamp: submessage_elements::Timestamp<PSM>;

    fn endianness_flag(&self) -> PSM::SubmessageFlagType;
    fn invalidate_flag(&self) -> PSM::SubmessageFlagType;
    fn timestamp(&self) -> &Self::Timestamp;
}

pub trait NackFragSubmessagePIM<
    PSM: SubmessageKindPIM
        + SubmessageFlagPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + CountPIM,
>
{
    type NackFragSubmessageType: NackFragSubmessage<PSM>;
}

pub trait NackFragSubmessage<
    PSM: SubmessageKindPIM
        + SubmessageFlagPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + FragmentNumberPIM
        + CountPIM,
>: Submessage<PSM>
{
    type EntityId: submessage_elements::EntityId<PSM>;
    type SequenceNumber: submessage_elements::SequenceNumber<PSM>;
    type FragmentNumberSet: submessage_elements::FragmentNumberSet<PSM>;
    type Count: submessage_elements::Count<PSM>;

    fn endianness_flag(&self) -> PSM::SubmessageFlagType;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn writer_sn(&self) -> &Self::SequenceNumber;
    fn fragment_number_state(&self) -> &Self::FragmentNumberSet;
    fn count(&self) -> &Self::Count;
}

pub trait PadSubmessagePIM<PSM: SubmessageKindPIM + SubmessageFlagPIM> {
    type PadSubmessageType: PadSubmessage<PSM>;
}

pub trait PadSubmessage<PSM: SubmessageKindPIM + SubmessageFlagPIM>: Submessage<PSM> {}
