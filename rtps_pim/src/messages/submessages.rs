use crate::{
    messages::{self, submessage_elements, Submessage},
    structure,
};

pub trait AckNack<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn final_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn reader_sn_state(&self) -> submessage_elements::SequenceNumberSet<PSM>;
    fn count(&self) -> submessage_elements::Count<PSM>;
}

pub trait Data<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn new(
        endianness_flag: PSM::SubmessageFlag,
        inline_qos_flag: PSM::SubmessageFlag,
        data_flag: PSM::SubmessageFlag,
        key_flag: PSM::SubmessageFlag,
        non_standard_payload_flag: PSM::SubmessageFlag,
        reader_id: PSM::EntityId,
        writer_id: PSM::EntityId,
        writer_sn: PSM::SequenceNumber,
        serialized_payload: &PSM::Data,
    ) -> Self;
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn inline_qos_flag(&self) -> PSM::SubmessageFlag;
    fn data_flag(&self) -> PSM::SubmessageFlag;
    fn key_flag(&self) -> PSM::SubmessageFlag;
    fn non_standard_payload_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_sn(&self) -> submessage_elements::SequenceNumber<PSM>;
    // pub inline_qos: <Self::PSM as structure::Types>::ParameterVector,
    fn serialized_payload(&self) -> &PSM::Data;
}

pub trait DataFrag<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn inline_qos_flag(&self) -> PSM::SubmessageFlag;
    fn non_standard_payload_flag(&self) -> PSM::SubmessageFlag;
    fn key_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_sn(&self) -> submessage_elements::SequenceNumber<PSM>;
    fn fragment_starting_num(&self) -> submessage_elements::FragmentNumber<PSM>;
    fn fragments_in_submessage(&self) -> submessage_elements::UShort;
    fn data_size(&self) -> submessage_elements::ULong;
    fn fragment_size(&self) -> submessage_elements::UShort;
    fn inline_qos(&self) -> submessage_elements::ParameterList<PSM>;
    fn serialized_payload(&self) -> &PSM::Data;
}

pub trait Gap<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn new(
        endianness_flag: PSM::SubmessageFlag,
        reader_id: PSM::EntityId,
        writer_id: PSM::EntityId,
        gap_start: PSM::SequenceNumber,
        gap_list: PSM::SequenceNumberVector,
    ) -> Self;
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn gap_start(&self) -> submessage_elements::SequenceNumber<PSM>;
    fn gap_list(&self) -> submessage_elements::SequenceNumberSet<PSM>;
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}
pub trait Heartbeat<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn final_flag(&self) -> PSM::SubmessageFlag;
    fn liveliness_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn first_sn(&self) -> submessage_elements::SequenceNumber<PSM>;
    fn last_sn(&self) -> submessage_elements::SequenceNumber<PSM>;
    fn count(&self) -> submessage_elements::Count<PSM>;
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}

pub trait HeartbeatFrag<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_sn(&self) -> submessage_elements::SequenceNumber<PSM>;
    fn last_fragment_num(&self) -> submessage_elements::FragmentNumber<PSM>;
    fn count(&self) -> submessage_elements::Count<PSM>;
}

pub trait InfoDestination<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn guid_prefix(&self) -> submessage_elements::GuidPrefix<PSM>;
}

pub trait InfoReply<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn multicast_flag(&self) -> PSM::SubmessageFlag;
    fn unicast_locator_list(&self) -> submessage_elements::LocatorList<PSM>;
    fn multicast_locator_list(&self) -> submessage_elements::LocatorList<PSM>;
}

pub trait InfoSource<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn protocol_version(&self) -> submessage_elements::ProtocolVersion<PSM>;
    fn vendor_id(&self) -> submessage_elements::VendorId<PSM>;
    fn guid_prefix(&self) -> submessage_elements::GuidPrefix<PSM>;
}
pub trait InfoTimestamp<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn invalidate_flag(&self) -> PSM::SubmessageFlag;
    fn timestamp(&self) -> submessage_elements::Timestamp<PSM>;
}

pub trait NackFrag<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_sn(&self) -> submessage_elements::SequenceNumber<PSM>;
    fn fragment_number_state(&self) -> submessage_elements::FragmentNumberSet<PSM>;
    fn count(&self) -> submessage_elements::Count<PSM>;
}

pub trait Pad<PSM: structure::Types + messages::Types>: Submessage<PSM> {}
