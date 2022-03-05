use super::{
    submessage_elements::{
        CountSubmessageElement, EntityIdSubmessageElement, FragmentNumberSubmessageElement,
        GuidPrefixSubmessageElement, LocatorListSubmessageElement, ParameterListSubmessageElement,
        ProtocolVersionSubmessageElement, SequenceNumberSetSubmessageElement,
        SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
        TimestampSubmessageElement, ULongSubmessageElement, UShortSubmessageElement,
        VendorIdSubmessageElement, FragmentNumberSetSubmessageElement,
    },
    types::SubmessageFlag,
};

pub trait AckNackSubmessageConstructor<S> {
    fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        reader_sn_state: SequenceNumberSetSubmessageElement<S>,
        count: CountSubmessageElement,
    ) -> Self;
}

pub trait AckNackSubmessageAttributes<S> {
    fn endianness_flag(&self) -> SubmessageFlag;
    fn final_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &EntityIdSubmessageElement;
    fn writer_id(&self) -> &EntityIdSubmessageElement;
    fn reader_sn_state(&self) -> &SequenceNumberSetSubmessageElement<S>;
    fn count(&self) -> &CountSubmessageElement;
}

pub trait DataSubmessageConstructor<'a, P> {
    fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        writer_sn: SequenceNumberSubmessageElement,
        inline_qos: ParameterListSubmessageElement<P>,
        serialized_payload: SerializedDataSubmessageElement<'a>,
    ) -> Self;
}

pub trait DataSubmessageAttributes<P> {
    fn endianness_flag(&self) -> SubmessageFlag;
    fn inline_qos_flag(&self) -> SubmessageFlag;
    fn data_flag(&self) -> SubmessageFlag;
    fn key_flag(&self) -> SubmessageFlag;
    fn non_standard_payload_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &EntityIdSubmessageElement;
    fn writer_id(&self) -> &EntityIdSubmessageElement;
    fn writer_sn(&self) -> &SequenceNumberSubmessageElement;
    fn inline_qos(&self) -> &ParameterListSubmessageElement<P>;
    fn serialized_payload(&self) -> &SerializedDataSubmessageElement<'_>;
}

pub trait DataFragSubmessageConstructor<P> {
    fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        writer_sn: SequenceNumberSubmessageElement,
        fragment_starting_num: FragmentNumberSubmessageElement,
        fragments_in_submessage: UShortSubmessageElement,
        data_size: ULongSubmessageElement,
        fragment_size: UShortSubmessageElement,
        inline_qos: ParameterListSubmessageElement<P>,
        serialized_payload: SerializedDataSubmessageElement,
    ) -> Self;
}

pub trait DataFragSubmessageAttributes<P> {
    fn endianness_flag(&self) -> SubmessageFlag;
    fn inline_qos_flag(&self) -> SubmessageFlag;
    fn non_standard_payload_flag(&self) -> SubmessageFlag;
    fn key_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &EntityIdSubmessageElement;
    fn writer_id(&self) -> &EntityIdSubmessageElement;
    fn writer_sn(&self) -> &SequenceNumberSubmessageElement;
    fn fragment_starting_num(&self) -> &FragmentNumberSubmessageElement;
    fn fragments_in_submessage(&self) -> &UShortSubmessageElement;
    fn data_size(&self) -> &ULongSubmessageElement;
    fn fragment_size(&self) -> &UShortSubmessageElement;
    fn inline_qos(&self) -> &ParameterListSubmessageElement<P>;
    fn serialized_payload(&self) -> &SerializedDataSubmessageElement;
}

pub trait GapSubmessageConstructor<S> {
    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        gap_start: SequenceNumberSubmessageElement,
        gap_list: SequenceNumberSetSubmessageElement<S>,
    ) -> Self;
}

pub trait GapSubmessageAttributes<S> {
    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &EntityIdSubmessageElement;
    fn writer_id(&self) -> &EntityIdSubmessageElement;
    fn gap_start(&self) -> &SequenceNumberSubmessageElement;
    fn gap_list(&self) -> &SequenceNumberSetSubmessageElement<S>;
}

pub trait HeartbeatSubmessageConstructor {
    fn new(
        endianness_flag: SubmessageFlag,
        final_flag: SubmessageFlag,
        liveliness_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        first_sn: SequenceNumberSubmessageElement,
        last_sn: SequenceNumberSubmessageElement,
        count: CountSubmessageElement,
    ) -> Self;
}

pub trait HeartbeatSubmessageAttributes {
    fn endianness_flag(&self) -> SubmessageFlag;
    fn final_flag(&self) -> SubmessageFlag;
    fn liveliness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &EntityIdSubmessageElement;
    fn writer_id(&self) -> &EntityIdSubmessageElement;
    fn first_sn(&self) -> &SequenceNumberSubmessageElement;
    fn last_sn(&self) -> &SequenceNumberSubmessageElement;
    fn count(&self) -> &CountSubmessageElement;
}

pub trait HeartbeatFragSubmessageConstructor {
    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        writer_sn: SequenceNumberSubmessageElement,
        last_fragment_num: FragmentNumberSubmessageElement,
        count: CountSubmessageElement,
    ) -> Self;
}

pub trait HeartbeatFragSubmessageAttributes {
    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &EntityIdSubmessageElement;
    fn writer_id(&self) -> &EntityIdSubmessageElement;
    fn writer_sn(&self) -> &SequenceNumberSubmessageElement;
    fn last_fragment_num(&self) -> &FragmentNumberSubmessageElement;
    fn count(&self) -> &CountSubmessageElement;
}

pub trait InfoDestinationSubmessageConstructor {
    fn new(endianness_flag: SubmessageFlag, guid_prefix: GuidPrefixSubmessageElement) -> Self;
}

pub trait InfoDestinationSubmessageAttributes {
    fn endianness_flag(&self) -> SubmessageFlag;
    fn guid_prefix(&self) -> &GuidPrefixSubmessageElement;
}

pub trait InfoReplySubmessageConstructor<L> {
    fn new(
        endianness_flag: SubmessageFlag,
        multicast_flag: SubmessageFlag,
        unicast_locator_list: LocatorListSubmessageElement<L>,
        multicast_locator_list: LocatorListSubmessageElement<L>,
    );
}

pub trait InfoReplySubmessageAttributes<L> {
    fn endianness_flag(&self) -> SubmessageFlag;
    fn multicast_flag(&self) -> SubmessageFlag;
    fn unicast_locator_list(&self) -> &LocatorListSubmessageElement<L>;
    fn multicast_locator_list(&self) -> &LocatorListSubmessageElement<L>;
}

pub trait InfoSourceSubmessageConstructor {
    fn new(
        endianness_flag: SubmessageFlag,
        protocol_version: ProtocolVersionSubmessageElement,
        vendor_id: VendorIdSubmessageElement,
        guid_prefix: GuidPrefixSubmessageElement,
    ) -> Self;
}

pub trait InfoSourceSubmessageAttributes {
    fn endianness_flag(&self) -> SubmessageFlag;
    fn protocol_version(&self) -> &ProtocolVersionSubmessageElement;
    fn vendor_id(&self) -> &VendorIdSubmessageElement;
    fn guid_prefix(&self) -> &GuidPrefixSubmessageElement;
}

pub trait InfoTimestampSubmessageConstructor {
    fn new(
        endianness_flag: SubmessageFlag,
        invalidate_flag: SubmessageFlag,
        timestamp: TimestampSubmessageElement,
    ) -> Self;
}

pub trait InfoTimestampSubmessageAttributes {
    fn endianness_flag(&self) -> SubmessageFlag;
    fn invalidate_flag(&self) -> SubmessageFlag;
    fn timestamp(&self) -> &TimestampSubmessageElement;
}

pub trait NackFragSubmessageConstructor<F> {
    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: EntityIdSubmessageElement,
        writer_id: EntityIdSubmessageElement,
        writer_sn: SequenceNumberSubmessageElement,
        fragment_number_state: FragmentNumberSetSubmessageElement<F>,
        count: CountSubmessageElement,
    ) -> Self;
}

pub trait NackFragSubmessageAttributes<F> {
    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &EntityIdSubmessageElement;
    fn writer_id(&self) -> &EntityIdSubmessageElement;
    fn writer_sn(&self) -> &SequenceNumberSubmessageElement;
    fn fragment_number_state(&self) -> &FragmentNumberSetSubmessageElement<F>;
    fn count(&self) -> &CountSubmessageElement;
}

pub trait PadSubmessageConstructor {
    fn new() -> Self;
}

pub trait PadSubmessageAttributes {}
