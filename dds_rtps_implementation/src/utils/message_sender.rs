use rust_rtps_pim::{
    behavior::{
        stateless_writer::{best_effort_send_unsent_data, RTPSReaderLocator},
        types::DurationPIM,
    },
    messages::{
        submessage_elements::{
            EntityIdSubmessageElementPIM, EntityIdSubmessageElementType,
            ParameterListSubmessageElementPIM, SequenceNumberSetSubmessageElementPIM,
            SequenceNumberSetSubmessageElementType, SequenceNumberSubmessageElementPIM,
            SequenceNumberSubmessageElementType, SerializedDataSubmessageElementPIM,
            SerializedDataSubmessageElementType,
        },
        submessages::{
            AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessage, DataSubmessagePIM,
            GapSubmessage, GapSubmessagePIM, HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM,
            InfoDestinationSubmessagePIM, InfoReplySubmessagePIM, InfoSourceSubmessagePIM,
            InfoTimestampSubmessagePIM, NackFragSubmessagePIM, PadSubmessagePIM,
            RtpsSubmessageType,
        },
        types::ProtocolIdPIM,
        RtpsSubmessageHeaderPIM,
    },
    structure::{types::SequenceNumber, RTPSCacheChange, RTPSHistoryCache},
};

use crate::transport::Transport;

pub fn send_data<PSM, HistoryCache, ReaderLocator>(
    writer_cache: &HistoryCache,
    reader_locators: &mut [ReaderLocator],
    last_change_sequence_number: SequenceNumber,
    transport: &mut dyn Transport<PSM>,
) where
    PSM: DurationPIM
        + ParameterListSubmessageElementPIM
        + AckNackSubmessagePIM
        + for<'a> DataSubmessagePIM<'a, PSM>
        + for<'a> DataFragSubmessagePIM<'a>
        + GapSubmessagePIM
        + HeartbeatSubmessagePIM
        + HeartbeatFragSubmessagePIM
        + InfoDestinationSubmessagePIM
        + InfoReplySubmessagePIM
        + InfoSourceSubmessagePIM
        + InfoTimestampSubmessagePIM
        + NackFragSubmessagePIM
        + PadSubmessagePIM
        + EntityIdSubmessageElementPIM
        + SequenceNumberSubmessageElementPIM
        + for<'a> SerializedDataSubmessageElementPIM<'a>
        + RtpsSubmessageHeaderPIM
        + SequenceNumberSetSubmessageElementPIM
        + ProtocolIdPIM,
    PSM::EntityIdSubmessageElementType: EntityIdSubmessageElementType,
    PSM::SequenceNumberSubmessageElementType: SequenceNumberSubmessageElementType,
    PSM::SequenceNumberSetSubmessageElementType: SequenceNumberSetSubmessageElementType,
    PSM::GapSubmessageType: GapSubmessage<PSM>,
    PSM::ParameterListSubmessageElementType: Clone,
    HistoryCache: RTPSHistoryCache,
    ReaderLocator: RTPSReaderLocator,
    <HistoryCache as RTPSHistoryCache>::CacheChange: RTPSCacheChange<PSM>,
{
    for reader_locator in reader_locators {
        let mut data_submessage_list: Vec<RtpsSubmessageType<'_, PSM>> = vec![];
        let mut gap_submessage_list: Vec<RtpsSubmessageType<'_, PSM>> = vec![];

        let mut submessages = vec![];
        best_effort_send_unsent_data(
            reader_locator,
            &last_change_sequence_number,
            writer_cache,
            |data_submessage| data_submessage_list.push(RtpsSubmessageType::Data(data_submessage)),
            |gap_submessage| gap_submessage_list.push(RtpsSubmessageType::Gap(gap_submessage)),
        );

        for data_submessage in data_submessage_list {
            submessages.push(data_submessage)
        }
        for gap_submessage in gap_submessage_list {
            submessages.push(gap_submessage);
        }

        transport.write(&submessages, reader_locator.locator());
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use rust_rtps_pim::{messages::Submessage, structure::types::Locator};

    use super::*;

    #[test]
    fn send_data_message() {
        let mut transport: MockTransport<MockPSM> = MockTransport(PhantomData);
        let writer_cache = MockHistoryCache;
        let mut reader_locators = [MockReaderLocator];
        let last_change_sequence_number = 0;
        send_data(
            &writer_cache,
            &mut reader_locators,
            last_change_sequence_number,
            &mut transport,
        );
    }

    struct MockPSM;

    impl DurationPIM for MockPSM {
        type DurationType = ();
    }

    impl ParameterListSubmessageElementPIM for MockPSM {
        type ParameterListSubmessageElementType = ();
    }

    impl AckNackSubmessagePIM for MockPSM {
        type AckNackSubmessageType = ();
    }

    impl<'a, PSM> DataSubmessagePIM<'a, PSM> for MockPSM
    where
        PSM: RtpsSubmessageHeaderPIM
            + EntityIdSubmessageElementPIM
            + SequenceNumberSubmessageElementPIM
            + ParameterListSubmessageElementPIM
            + SerializedDataSubmessageElementPIM<'a>,
    {
        type DataSubmessageType = MockDataSubmessage;
    }

    impl<'a> DataFragSubmessagePIM<'a> for MockPSM {
        type DataFragSubmessageType = ();
    }

    impl GapSubmessagePIM for MockPSM {
        type GapSubmessageType = MockGapSubmessage;
    }

    impl HeartbeatSubmessagePIM for MockPSM {
        type HeartbeatSubmessageType = ();
    }

    impl HeartbeatFragSubmessagePIM for MockPSM {
        type HeartbeatFragSubmessageType = ();
    }

    impl InfoDestinationSubmessagePIM for MockPSM {
        type InfoDestinationSubmessageType = ();
    }

    impl InfoReplySubmessagePIM for MockPSM {
        type InfoReplySubmessageType = ();
    }

    impl InfoSourceSubmessagePIM for MockPSM {
        type InfoSourceSubmessageType = ();
    }

    impl InfoTimestampSubmessagePIM for MockPSM {
        type InfoTimestampSubmessageType = ();
    }

    impl NackFragSubmessagePIM for MockPSM {
        type NackFragSubmessageType = ();
    }

    impl PadSubmessagePIM for MockPSM {
        type PadSubmessageType = ();
    }

    impl EntityIdSubmessageElementPIM for MockPSM {
        type EntityIdSubmessageElementType = MockEntityId;
    }

    impl SequenceNumberSubmessageElementPIM for MockPSM {
        type SequenceNumberSubmessageElementType = MockSequenceNumber;
    }

    impl<'a> SerializedDataSubmessageElementPIM<'a> for MockPSM {
        type SerializedDataSubmessageElementType = MockSerializedData;
    }

    impl RtpsSubmessageHeaderPIM for MockPSM {
        type RtpsSubmessageHeaderType = ();
    }

    impl SequenceNumberSetSubmessageElementPIM for MockPSM {
        type SequenceNumberSetSubmessageElementType = MockSequenceNumberSet;
    }

    impl ProtocolIdPIM for MockPSM {
        type ProtocolIdType = ();

        const PROTOCOL_RTPS: Self::ProtocolIdType = ();
    }

    struct MockEntityId;
    impl EntityIdSubmessageElementType for MockEntityId {
        fn new(value: &rust_rtps_pim::structure::types::EntityId) -> Self {
            todo!()
        }

        fn value(&self) -> &rust_rtps_pim::structure::types::EntityId {
            todo!()
        }
    }

    struct MockSequenceNumber;

    impl SequenceNumberSubmessageElementType for MockSequenceNumber {
        fn new(value: SequenceNumber) -> Self {
            todo!()
        }

        fn value(&self) -> SequenceNumber {
            todo!()
        }
    }

    struct MockSerializedData;
    impl<'a> SerializedDataSubmessageElementType<'a> for MockSerializedData {
        fn new(value: &'a [u8]) -> Self {
            todo!()
        }

        fn value(&self) -> &[u8] {
            todo!()
        }
    }

    struct MockSequenceNumberSet;

    impl SequenceNumberSetSubmessageElementType for MockSequenceNumberSet {
        type IntoIter = std::vec::IntoIter<SequenceNumber>;

        fn new(base: SequenceNumber, set: &[SequenceNumber]) -> Self {
            todo!()
        }

        fn base(&self) -> SequenceNumber {
            todo!()
        }

        fn set(&self) -> Self::IntoIter {
            todo!()
        }
    }

    struct MockDataSubmessage;

    impl<'a, PSM> DataSubmessage<'a, PSM> for MockDataSubmessage
    where
        PSM: RtpsSubmessageHeaderPIM
            + EntityIdSubmessageElementPIM
            + SequenceNumberSubmessageElementPIM
            + ParameterListSubmessageElementPIM
            + SerializedDataSubmessageElementPIM<'a>,
    {
        fn new(
            endianness_flag: rust_rtps_pim::messages::types::SubmessageFlag,
            inline_qos_flag: rust_rtps_pim::messages::types::SubmessageFlag,
            data_flag: rust_rtps_pim::messages::types::SubmessageFlag,
            key_flag: rust_rtps_pim::messages::types::SubmessageFlag,
            non_standard_payload_flag: rust_rtps_pim::messages::types::SubmessageFlag,
            reader_id: PSM::EntityIdSubmessageElementType,
            writer_id: PSM::EntityIdSubmessageElementType,
            writer_sn: PSM::SequenceNumberSubmessageElementType,
            inline_qos: PSM::ParameterListSubmessageElementType,
            serialized_payload: PSM::SerializedDataSubmessageElementType,
        ) -> Self {
            todo!()
        }

        fn endianness_flag(&self) -> rust_rtps_pim::messages::types::SubmessageFlag {
            todo!()
        }

        fn inline_qos_flag(&self) -> rust_rtps_pim::messages::types::SubmessageFlag {
            todo!()
        }

        fn data_flag(&self) -> rust_rtps_pim::messages::types::SubmessageFlag {
            todo!()
        }

        fn key_flag(&self) -> rust_rtps_pim::messages::types::SubmessageFlag {
            todo!()
        }

        fn non_standard_payload_flag(&self) -> rust_rtps_pim::messages::types::SubmessageFlag {
            todo!()
        }

        fn reader_id(&self) -> &PSM::EntityIdSubmessageElementType {
            todo!()
        }

        fn writer_id(&self) -> &PSM::EntityIdSubmessageElementType {
            todo!()
        }

        fn writer_sn(&self) -> &PSM::SequenceNumberSubmessageElementType {
            todo!()
        }

        fn inline_qos(&self) -> &PSM::ParameterListSubmessageElementType {
            todo!()
        }

        fn serialized_payload(&'a self) -> &'a PSM::SerializedDataSubmessageElementType {
            todo!()
        }
    }

    impl<PSM> Submessage<PSM> for MockDataSubmessage
    where
        PSM: RtpsSubmessageHeaderPIM,
    {
        fn submessage_header(&self) -> PSM::RtpsSubmessageHeaderType {
            todo!()
        }
    }

    struct MockGapSubmessage;

    impl<PSM> GapSubmessage<PSM> for MockGapSubmessage
    where
        PSM: RtpsSubmessageHeaderPIM
            + EntityIdSubmessageElementPIM
            + SequenceNumberSubmessageElementPIM
            + SequenceNumberSetSubmessageElementPIM,
    {
        fn new(
            endianness_flag: rust_rtps_pim::messages::types::SubmessageFlag,
            reader_id: PSM::EntityIdSubmessageElementType,
            writer_id: PSM::EntityIdSubmessageElementType,
            gap_start: PSM::SequenceNumberSubmessageElementType,
            gap_list: PSM::SequenceNumberSetSubmessageElementType,
        ) -> Self {
            todo!()
        }

        fn endianness_flag(&self) -> rust_rtps_pim::messages::types::SubmessageFlag {
            todo!()
        }

        fn reader_id(&self) -> &PSM::EntityIdSubmessageElementType {
            todo!()
        }

        fn writer_id(&self) -> &PSM::EntityIdSubmessageElementType {
            todo!()
        }

        fn gap_start(&self) -> &PSM::SequenceNumberSubmessageElementType {
            todo!()
        }

        fn gap_list(&self) -> &PSM::SequenceNumberSetSubmessageElementType {
            todo!()
        }
    }

    impl<PSM> Submessage<PSM> for MockGapSubmessage
    where
        PSM: RtpsSubmessageHeaderPIM,
    {
        fn submessage_header(&self) -> PSM::RtpsSubmessageHeaderType {
            todo!()
        }
    }

    struct MockTransport<PSM>(PhantomData<PSM>);

    impl<PSM> Transport<PSM> for MockTransport<PSM> {
        fn write<'a>(
            &mut self,
            message: &[RtpsSubmessageType<'a, PSM>],
            destination_locator: &Locator,
        ) where
            PSM: AckNackSubmessagePIM
                + DataSubmessagePIM<'a, PSM>
                + DataFragSubmessagePIM<'a>
                + GapSubmessagePIM
                + HeartbeatSubmessagePIM
                + HeartbeatFragSubmessagePIM
                + InfoDestinationSubmessagePIM
                + InfoReplySubmessagePIM
                + InfoSourceSubmessagePIM
                + InfoTimestampSubmessagePIM
                + NackFragSubmessagePIM
                + PadSubmessagePIM
                + RtpsSubmessageHeaderPIM
                + EntityIdSubmessageElementPIM
                + SequenceNumberSubmessageElementPIM
                + ParameterListSubmessageElementPIM
                + SerializedDataSubmessageElementPIM<'a>,
        {
            todo!()
        }

        fn read<'a>(
            &'a self,
        ) -> Option<(
            PSM::RTPSMessageType,
            rust_rtps_pim::structure::types::Locator,
        )>
        where
            PSM: rust_rtps_pim::messages::RTPSMessagePIM<'a, PSM>,
        {
            todo!()
        }

        fn unicast_locator_list(&self) -> &[rust_rtps_pim::structure::types::Locator] {
            todo!()
        }

        fn multicast_locator_list(&self) -> &[rust_rtps_pim::structure::types::Locator] {
            todo!()
        }
    }

    struct MockHistoryCache;

    impl RTPSHistoryCache for MockHistoryCache {
        type CacheChange = MockCacheChange;

        fn new() -> Self
        where
            Self: Sized,
        {
            todo!()
        }

        fn add_change(&mut self, change: Self::CacheChange) {
            todo!()
        }

        fn remove_change(&mut self, seq_num: &SequenceNumber) {
            todo!()
        }

        fn get_change(&self, seq_num: &SequenceNumber) -> Option<&Self::CacheChange> {
            todo!()
        }

        fn get_seq_num_min(&self) -> Option<&SequenceNumber> {
            todo!()
        }

        fn get_seq_num_max(&self) -> Option<&SequenceNumber> {
            todo!()
        }
    }

    struct MockCacheChange;

    impl<PSM> RTPSCacheChange<PSM> for MockCacheChange
    where
        PSM: ParameterListSubmessageElementPIM,
    {
        type DataType = Vec<u8>;
        type InstanceHandleType = ();

        fn kind(&self) -> rust_rtps_pim::structure::types::ChangeKind {
            todo!()
        }

        fn writer_guid(&self) -> &rust_rtps_pim::structure::types::GUID {
            todo!()
        }

        fn instance_handle(&self) -> &Self::InstanceHandleType {
            todo!()
        }

        fn sequence_number(&self) -> &SequenceNumber {
            todo!()
        }

        fn data_value(&self) -> &Self::DataType {
            todo!()
        }

        fn inline_qos(&self) -> &PSM::ParameterListSubmessageElementType {
            todo!()
        }
    }

    struct MockReaderLocator;

    impl RTPSReaderLocator for MockReaderLocator {
        type SequenceNumberVector = Vec<SequenceNumber>;

        fn locator(&self) -> &Locator {
            todo!()
        }

        fn expects_inline_qos(&self) -> bool {
            todo!()
        }

        fn next_requested_change(&mut self) -> Option<SequenceNumber> {
            todo!()
        }

        fn next_unsent_change(
            &mut self,
            last_change_sequence_number: &SequenceNumber,
        ) -> Option<SequenceNumber> {
            todo!()
        }

        fn requested_changes(&self) -> Self::SequenceNumberVector {
            todo!()
        }

        fn requested_changes_set(
            &mut self,
            req_seq_num_set: &[SequenceNumber],
            last_change_sequence_number: &SequenceNumber,
        ) {
            todo!()
        }

        fn unsent_changes(
            &self,
            last_change_sequence_number: SequenceNumber,
        ) -> Self::SequenceNumberVector {
            todo!()
        }
    }
}
