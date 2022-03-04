use super::{types::SubmessageFlag, submessage_elements::{SequenceNumberSetSubmessageElement, ParameterListSubmessageElement}};

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

    fn endianness_flag(&self) -> SubmessageFlag;
    fn final_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn reader_sn_state(&self) -> &Self::SequenceNumberSetSubmessageElementType;
    fn count(&self) -> &Self::CountSubmessageElementType;
}

pub trait DataSubmessageConstructor<P> {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
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
        inline_qos: ParameterListSubmessageElement<P>,
        serialized_payload: Self::SerializedDataSubmessageElementType,
    ) -> Self;
}

pub trait DataSubmessageAttributes<P> {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type SerializedDataSubmessageElementType;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn inline_qos_flag(&self) -> SubmessageFlag;
    fn data_flag(&self) -> SubmessageFlag;
    fn key_flag(&self) -> SubmessageFlag;
    fn non_standard_payload_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType;
    fn inline_qos(&self) -> &ParameterListSubmessageElement<P>;
    fn serialized_payload(&self) -> &Self::SerializedDataSubmessageElementType;
}

pub trait DataFragSubmessageConstructor {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type FragmentNumberSubmessageElementType;
    type UShortSubmessageElementType;
    type ULongSubmessageElementType;
    type ParameterListSubmessageElementType;
    type SerializedDataSubmessageElementType;

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
        serialized_payload: Self::SerializedDataSubmessageElementType,
    ) -> Self;
}

pub trait DataFragSubmessageAttributes {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type FragmentNumberSubmessageElementType;
    type UShortSubmessageElementType;
    type ULongSubmessageElementType;
    type ParameterListSubmessageElementType;
    type SerializedDataSubmessageElementType;

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
    fn serialized_payload(&self) -> &Self::SerializedDataSubmessageElementType;
}

pub trait GapSubmessageConstructor<S> {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;

    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: Self::EntityIdSubmessageElementType,
        writer_id: Self::EntityIdSubmessageElementType,
        gap_start: Self::SequenceNumberSubmessageElementType,
        gap_list: SequenceNumberSetSubmessageElement<S>,
    ) -> Self;
}

pub trait GapSubmessageAttributes<S> {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn gap_start(&self) -> &Self::SequenceNumberSubmessageElementType;
    fn gap_list(&self) -> &SequenceNumberSetSubmessageElement<S>;
}

pub trait HeartbeatSubmessageConstructor {
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
}

pub trait HeartbeatSubmessageAttributes {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type CountSubmessageElementType;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn final_flag(&self) -> SubmessageFlag;
    fn liveliness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn first_sn(&self) -> &Self::SequenceNumberSubmessageElementType;
    fn last_sn(&self) -> &Self::SequenceNumberSubmessageElementType;
    fn count(&self) -> &Self::CountSubmessageElementType;
}

pub trait HeartbeatFragSubmessageConstructor {
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
}

pub trait HeartbeatFragSubmessageAttributes {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type FragmentNumberSubmessageElementType;
    type CountSubmessageElementType;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType;
    fn last_fragment_num(&self) -> &Self::FragmentNumberSubmessageElementType;
    fn count(&self) -> &Self::CountSubmessageElementType;
}

pub trait InfoDestinationSubmessageConstructor {
    type GuidPrefixSubmessageElementType;
    fn new(
        endianness_flag: SubmessageFlag,
        guid_prefix: Self::GuidPrefixSubmessageElementType,
    ) -> Self;
}

pub trait InfoDestinationSubmessageAttributes {
    type GuidPrefixSubmessageElementType;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn guid_prefix(&self) -> &Self::GuidPrefixSubmessageElementType;
}

pub trait InfoReplySubmessageConstructor {
    type LocatorListSubmessageElementType;

    fn new(
        endianness_flag: SubmessageFlag,
        multicast_flag: SubmessageFlag,
        unicast_locator_list: Self::LocatorListSubmessageElementType,
        multicast_locator_list: Self::LocatorListSubmessageElementType,
    );
}

pub trait InfoReplySubmessageAttributes {
    type LocatorListSubmessageElementType;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn multicast_flag(&self) -> SubmessageFlag;
    fn unicast_locator_list(&self) -> &Self::LocatorListSubmessageElementType;
    fn multicast_locator_list(&self) -> &Self::LocatorListSubmessageElementType;
}

pub trait InfoSourceSubmessageConstructor {
    type ProtocolVersionSubmessageElementType;
    type VendorIdSubmessageElementType;
    type GuidPrefixSubmessageElementType;

    fn new(
        endianness_flag: SubmessageFlag,
        protocol_version: Self::ProtocolVersionSubmessageElementType,
        vendor_id: Self::VendorIdSubmessageElementType,
        guid_prefix: Self::GuidPrefixSubmessageElementType,
    ) -> Self;
}

pub trait InfoSourceSubmessageAttributes {
    type ProtocolVersionSubmessageElementType;
    type VendorIdSubmessageElementType;
    type GuidPrefixSubmessageElementType;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn protocol_version(&self) -> &Self::ProtocolVersionSubmessageElementType;
    fn vendor_id(&self) -> &Self::VendorIdSubmessageElementType;
    fn guid_prefix(&self) -> &Self::GuidPrefixSubmessageElementType;
}

pub trait InfoTimestampSubmessageConstructor {
    type TimestampSubmessageElementType;
    fn new(
        endianness_flag: SubmessageFlag,
        invalidate_flag: SubmessageFlag,
        timestamp: Self::TimestampSubmessageElementType,
    ) -> Self;
}

pub trait InfoTimestampSubmessageAttributes {
    type TimestampSubmessageElementType;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn invalidate_flag(&self) -> SubmessageFlag;
    fn timestamp(&self) -> &Self::TimestampSubmessageElementType;
}

pub trait NackFragSubmessageConstructor {
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
}

pub trait NackFragSubmessageAttributes {
    type EntityIdSubmessageElementType;
    type SequenceNumberSubmessageElementType;
    type FragmentNumberSetSubmessageElementType;
    type CountSubmessageElementType;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_id(&self) -> &Self::EntityIdSubmessageElementType;
    fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType;
    fn fragment_number_state(&self) -> &Self::FragmentNumberSetSubmessageElementType;
    fn count(&self) -> &Self::CountSubmessageElementType;
}

pub trait PadSubmessageConstructor {
    fn new() -> Self;
}

pub trait PadSubmessageAttributes {}
