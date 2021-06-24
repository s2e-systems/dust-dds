use rust_rtps_pim::{
    behavior::{
        stateful_writer::{RTPSReaderProxy, RTPSStatefulWriter},
        stateless_writer::{RTPSReaderLocator, RTPSStatelessWriter},
        types::DurationPIM,
        RTPSWriter,
    },
    messages::submessage_elements::ParameterListSubmessageElementPIM,
    structure::{
        types::{
            ChangeKind, DataPIM, InstanceHandlePIM, Locator, ReliabilityKind, SequenceNumber,
            TopicKind, GUID,
        },
        RTPSEndpoint, RTPSEntity, RTPSHistoryCache,
    },
};

use super::{
    rtps_cache_change_impl::RTPSCacheChangeImpl, rtps_history_cache_impl::RTPSHistoryCacheImpl,
    rtps_reader_locator_impl::RTPSReaderLocatorImpl, rtps_reader_proxy_impl::RTPSReaderProxyImpl,
};

pub struct RTPSWriterImpl<PSM>
where
    PSM: DurationPIM + InstanceHandlePIM + DataPIM + ParameterListSubmessageElementPIM,
{
    guid: GUID,
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    push_mode: bool,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    heartbeat_period: PSM::DurationType,
    nack_response_delay: PSM::DurationType,
    nack_suppression_duration: PSM::DurationType,
    last_change_sequence_number: SequenceNumber,
    data_max_size_serialized: i32,
    reader_locators: Vec<RTPSReaderLocatorImpl>,
    matched_readers: Vec<RTPSReaderProxyImpl>,
    writer_cache: RTPSHistoryCacheImpl<PSM>,
}

impl<PSM> RTPSWriterImpl<PSM>
where
    PSM: DurationPIM + InstanceHandlePIM + DataPIM + ParameterListSubmessageElementPIM,
{
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        push_mode: bool,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        heartbeat_period: PSM::DurationType,
        nack_response_delay: PSM::DurationType,
        nack_suppression_duration: PSM::DurationType,
        data_max_size_serialized: i32,
    ) -> Self
    where
        SequenceNumber: PartialEq + Ord,
    {
        Self {
            guid,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
            last_change_sequence_number: 0.into(),
            reader_locators: Vec::new(),
            matched_readers: Vec::new(),
            writer_cache: RTPSHistoryCacheImpl::new(),
        }
    }

    pub fn writer_cache_and_reader_locators(
        &mut self,
    ) -> (
        &RTPSHistoryCacheImpl<PSM>,
        &mut Vec<RTPSReaderLocatorImpl>,
    ) {
        (&self.writer_cache, &mut self.reader_locators)
    }
}

impl<PSM> RTPSEntity for RTPSWriterImpl<PSM>
where
    PSM: DurationPIM + InstanceHandlePIM + DataPIM + ParameterListSubmessageElementPIM,
{
    fn guid(&self) -> &GUID {
        &self.guid
    }
}

impl<PSM> RTPSWriter<PSM> for RTPSWriterImpl<PSM>
where
    PSM: DurationPIM + InstanceHandlePIM + DataPIM + ParameterListSubmessageElementPIM,
    SequenceNumber: Ord + Copy,
    GUID: Copy,
{
    type HistoryCacheType = RTPSHistoryCacheImpl<PSM>;

    fn push_mode(&self) -> bool {
        self.push_mode
    }

    fn heartbeat_period(&self) -> &PSM::DurationType {
        &self.heartbeat_period
    }

    fn nack_response_delay(&self) -> &PSM::DurationType {
        &self.nack_response_delay
    }

    fn nack_suppression_duration(&self) -> &PSM::DurationType {
        &self.nack_suppression_duration
    }

    fn last_change_sequence_number(&self) -> &SequenceNumber {
        &self.last_change_sequence_number
    }

    fn data_max_size_serialized(&self) -> i32 {
        self.data_max_size_serialized
    }

    fn writer_cache(&self) -> &RTPSHistoryCacheImpl<PSM> {
        &self.writer_cache
    }

    fn writer_cache_mut(&mut self) -> &mut RTPSHistoryCacheImpl<PSM> {
        &mut self.writer_cache
    }

    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: PSM::DataType,
        inline_qos: PSM::ParameterListSubmessageElementType,
        handle: PSM::InstanceHandleType,
    ) -> <Self::HistoryCacheType as RTPSHistoryCache>::CacheChange
    where
        // Self::HistoryCacheType: RTPSHistoryCache<PSM>,
        PSM: DataPIM + ParameterListSubmessageElementPIM + InstanceHandlePIM,
    {
        self.last_change_sequence_number = self.last_change_sequence_number + 1;
        RTPSCacheChangeImpl::new(
            kind,
            self.guid,
            handle,
            self.last_change_sequence_number,
            data,
            inline_qos,
        )
    }
}

impl<PSM> RTPSEndpoint for RTPSWriterImpl<PSM>
where
    PSM:  DurationPIM + InstanceHandlePIM + DataPIM + ParameterListSubmessageElementPIM,
{
    fn topic_kind(&self) -> &TopicKind {
        &self.topic_kind
    }

    fn reliability_level(&self) -> &ReliabilityKind {
        &self.reliability_level
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.multicast_locator_list
    }
}

impl<PSM> RTPSStatelessWriter for RTPSWriterImpl<PSM>
where
    PSM: DurationPIM + InstanceHandlePIM + DataPIM + ParameterListSubmessageElementPIM,
    Locator: PartialEq,
{
    type ReaderLocatorPIM = RTPSReaderLocatorImpl;

    fn reader_locators(&mut self) -> &mut [Self::ReaderLocatorPIM] {
        &mut self.reader_locators
    }

    fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorPIM) {
        self.reader_locators.push(a_locator)
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.retain(|x| x.locator() != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        todo!()
    }
}

impl<PSM> RTPSStatefulWriter<PSM> for RTPSWriterImpl<PSM>
where
    PSM: DurationPIM + InstanceHandlePIM + DataPIM + ParameterListSubmessageElementPIM,
{
    type ReaderProxyType = RTPSReaderProxyImpl;

    fn matched_readers(&self) -> &[Self::ReaderProxyType] {
        &self.matched_readers
    }

    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType) {
        self.matched_readers.push(a_reader_proxy)
    }

    fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID) {
        self.matched_readers
            .retain(|x| x.remote_reader_guid() != reader_proxy_guid)
    }

    fn matched_reader_lookup(&self, a_reader_guid: &GUID) -> Option<&Self::ReaderProxyType> {
        self.matched_readers
            .iter()
            .find(|&x| x.remote_reader_guid() == a_reader_guid)
    }

    fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::{types::GUID_UNKNOWN, RTPSCacheChange};

    use super::*;

    struct MockPSM;

    impl rust_rtps_pim::structure::types::InstanceHandlePIM for MockPSM {
        type InstanceHandleType = ();
    }

    impl rust_rtps_pim::structure::types::DataPIM for MockPSM {
        type DataType = [u8; 0];
    }


    impl rust_rtps_pim::messages::submessage_elements::ParameterListSubmessageElementPIM for MockPSM {
        type ParameterListSubmessageElementType = MockParameterList;
    }

    pub struct MockParameterList;

    impl rust_rtps_pim::messages::submessage_elements::ParameterListSubmessageElementType<MockPSM>
        for MockParameterList
    {
        type Parameter = MockParameter;

        fn new(_parameter: &[MockParameter]) -> Self {
            todo!()
        }

        fn parameter(&self) -> &[MockParameter] {
            todo!()
        }
    }

    pub struct MockParameter;
    impl rust_rtps_pim::messages::submessage_elements::ParameterType for MockParameter {
        fn parameter_id(&self) -> u16 {
            todo!()
        }

        fn length(&self) -> i16 {
            todo!()
        }

        fn value(&self) -> &[u8] {
            todo!()
        }
    }

    impl rust_rtps_pim::behavior::types::DurationPIM for MockPSM {
        type DurationType = i64;
    }

    #[test]
    fn new_change() {
        let push_mode = true;
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let heartbeat_period = 0;
        let nack_response_delay = 0;
        let nack_suppression_duration = 0;
        let data_max_size_serialized = i32::MAX;
        let mut writer: RTPSWriterImpl<MockPSM> = RTPSWriterImpl::new(
            GUID_UNKNOWN,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );
        let change1 = writer.new_change(ChangeKind::Alive, [], MockParameterList, ());
        let change2 = writer.new_change(ChangeKind::Alive, [], MockParameterList, ());

        assert_eq!(change1.sequence_number(), &1);
        assert_eq!(change2.sequence_number(), &2);
    }

    #[test]
    fn reader_locator_add() {
        let push_mode = true;
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let heartbeat_period = 0;
        let nack_response_delay = 0;
        let nack_suppression_duration = 0;
        let data_max_size_serialized = i32::MAX;
        let mut writer: RTPSWriterImpl<MockPSM> = RTPSWriterImpl::new(
            GUID_UNKNOWN,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );
        let reader_locator1 = RTPSReaderLocatorImpl::new(Locator::new([1;4],[1;4], [1;16]), false);
        let reader_locator2 = RTPSReaderLocatorImpl::new(Locator::new([2;4],[2;4], [2;16]), false);
        writer.reader_locator_add(reader_locator1);
        writer.reader_locator_add(reader_locator2);

        assert_eq!(writer.reader_locators().len(), 2)
    }

    #[test]
    fn reader_locator_remove() {
        let push_mode = true;
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let heartbeat_period = 0;
        let nack_response_delay = 0;
        let nack_suppression_duration = 0;
        let data_max_size_serialized = i32::MAX;
        let mut writer: RTPSWriterImpl<MockPSM> = RTPSWriterImpl::new(
            GUID_UNKNOWN,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );

        let reader_locator1 = RTPSReaderLocatorImpl::new(Locator::new([1;4],[1;4], [1;16]), false);
        let reader_locator2 = RTPSReaderLocatorImpl::new(Locator::new([2;4],[2;4], [2;16]), false);
        writer.reader_locator_add(reader_locator1);
        writer.reader_locator_add(reader_locator2);
        writer.reader_locator_remove(&Locator::new([1;4],[1;4], [1;16]));

        assert_eq!(writer.reader_locators().len(), 1)
    }

    #[test]
    fn matched_reader_add() {
        let push_mode = true;
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let heartbeat_period = 0;
        let nack_response_delay = 0;
        let nack_suppression_duration = 0;
        let data_max_size_serialized = i32::MAX;
        let mut writer: RTPSWriterImpl<MockPSM> = RTPSWriterImpl::new(
            GUID_UNKNOWN,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );
        let reader_proxy1 =
            RTPSReaderProxyImpl::new(GUID_UNKNOWN, [0; 4], vec![], vec![], false, true);
        let reader_proxy2 =
            RTPSReaderProxyImpl::new(GUID_UNKNOWN, [0; 4], vec![], vec![], false, true);
        writer.matched_reader_add(reader_proxy1);
        writer.matched_reader_add(reader_proxy2);
        assert_eq!(writer.matched_readers().len(), 2)
    }

    #[test]
    fn matched_reader_remove() {
        let push_mode = true;
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let heartbeat_period = 0;
        let nack_response_delay = 0;
        let nack_suppression_duration = 0;
        let data_max_size_serialized = i32::MAX;
        let mut writer: RTPSWriterImpl<MockPSM> = RTPSWriterImpl::new(
            GUID_UNKNOWN,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );

        let reader_proxy1 =
            RTPSReaderProxyImpl::new(GUID_UNKNOWN, [0; 4], vec![], vec![], false, true);
        let reader_proxy2 =
            RTPSReaderProxyImpl::new(GUID_UNKNOWN, [0; 4], vec![], vec![], false, true);
        writer.matched_reader_add(reader_proxy1);
        writer.matched_reader_add(reader_proxy2);
        writer.matched_reader_remove(&GUID_UNKNOWN);

        assert_eq!(writer.matched_readers().len(), 2)
    }

    #[test]
    fn matched_reader_lookup() {
        let push_mode = true;
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let heartbeat_period = 0;
        let nack_response_delay = 0;
        let nack_suppression_duration = 0;
        let data_max_size_serialized = i32::MAX;
        let mut writer: RTPSWriterImpl<MockPSM> = RTPSWriterImpl::new(
            GUID_UNKNOWN,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );

        let reader_proxy1 =
            RTPSReaderProxyImpl::new(GUID_UNKNOWN, [0; 4], vec![], vec![], false, true);
        let reader_proxy2 =
            RTPSReaderProxyImpl::new(GUID_UNKNOWN, [0; 4], vec![], vec![], false, true);
        writer.matched_reader_add(reader_proxy1);
        writer.matched_reader_add(reader_proxy2);

        assert!(writer.matched_reader_lookup(&GUID_UNKNOWN).is_some());
        assert!(writer.matched_reader_lookup(&GUID_UNKNOWN).is_none());
    }
}
