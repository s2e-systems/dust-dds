use rust_rtps_pim::{
    behavior::{
        stateful_writer::{RTPSReaderProxy, RTPSStatefulWriter},
        stateless_writer::{RTPSReaderLocator, RTPSStatelessWriter},
        types::DurationPIM,
        RTPSWriter,
    },
    messages::types::ParameterIdPIM,
    structure::{
        types::{
            ChangeKind, DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM,
            ParameterListPIM, ReliabilityKind, SequenceNumberPIM, TopicKind, GUIDPIM,
        },
        RTPSEndpoint, RTPSEntity, RTPSHistoryCache,
    },
};

use super::{
    rtps_cache_change_impl::RTPSCacheChangeImpl, rtps_history_cache_impl::RTPSHistoryCacheImpl,
    rtps_reader_locator_impl::RTPSReaderLocatorImpl, rtps_reader_proxy_impl::RTPSReaderProxyImpl,
};

pub trait RTPSWriterImplTrait:
    SequenceNumberPIM
    + GuidPrefixPIM
    + EntityIdPIM
    + DurationPIM
    + DataPIM
    + LocatorPIM
    + InstanceHandlePIM
    + ParameterIdPIM
    + GUIDPIM<Self>
    + ParameterListPIM<Self>
    + Sized
{
}

impl<
        T: SequenceNumberPIM
            + GuidPrefixPIM
            + EntityIdPIM
            + DurationPIM
            + DataPIM
            + LocatorPIM
            + InstanceHandlePIM
            + ParameterIdPIM
            + GUIDPIM<Self>
            + ParameterListPIM<Self>
            + Sized,
    > RTPSWriterImplTrait for T
{
}

pub struct RTPSWriterImpl<PSM: RTPSWriterImplTrait> {
    guid: PSM::GUIDType,
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    push_mode: bool,
    unicast_locator_list: Vec<PSM::LocatorType>,
    multicast_locator_list: Vec<PSM::LocatorType>,
    heartbeat_period: PSM::DurationType,
    nack_response_delay: PSM::DurationType,
    nack_suppression_duration: PSM::DurationType,
    last_change_sequence_number: PSM::SequenceNumberType,
    data_max_size_serialized: i32,
    reader_locators: Vec<RTPSReaderLocatorImpl<PSM>>,
    matched_readers: Vec<RTPSReaderProxyImpl<PSM>>,
    writer_cache: RTPSHistoryCacheImpl<PSM>,
}

impl<PSM: RTPSWriterImplTrait> RTPSWriterImpl<PSM> {
    pub fn new(
        guid: PSM::GUIDType,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        push_mode: bool,
        unicast_locator_list: Vec<PSM::LocatorType>,
        multicast_locator_list: Vec<PSM::LocatorType>,
        heartbeat_period: PSM::DurationType,
        nack_response_delay: PSM::DurationType,
        nack_suppression_duration: PSM::DurationType,
        data_max_size_serialized: i32,
    ) -> Self {
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
}

impl<PSM: RTPSWriterImplTrait> RTPSEntity<PSM> for RTPSWriterImpl<PSM> {
    fn guid(&self) -> &PSM::GUIDType {
        &self.guid
    }
}

impl<PSM: RTPSWriterImplTrait> RTPSWriter<PSM> for RTPSWriterImpl<PSM> {
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

    fn last_change_sequence_number(&self) -> &PSM::SequenceNumberType {
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
        inline_qos: PSM::ParameterListType,
        handle: PSM::InstanceHandleType,
    ) -> <Self::HistoryCacheType as RTPSHistoryCache<PSM>>::CacheChange {
        self.last_change_sequence_number = (self.last_change_sequence_number.into() + 1).into();
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

impl<PSM: RTPSWriterImplTrait> RTPSEndpoint<PSM> for RTPSWriterImpl<PSM> {
    fn topic_kind(&self) -> TopicKind {
        self.topic_kind
    }

    fn reliability_level(&self) -> ReliabilityKind {
        self.reliability_level
    }

    fn unicast_locator_list(&self) -> &[PSM::LocatorType] {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[PSM::LocatorType] {
        &self.multicast_locator_list
    }
}

impl<PSM: RTPSWriterImplTrait> RTPSStatelessWriter<PSM> for RTPSWriterImpl<PSM> {
    type ReaderLocatorPIM = RTPSReaderLocatorImpl<PSM>;

    fn reader_locators(&self) -> &[Self::ReaderLocatorPIM] {
        &self.reader_locators
    }

    fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorPIM) {
        self.reader_locators.push(a_locator)
    }

    fn reader_locator_remove(&mut self, a_locator: &PSM::LocatorType) {
        self.reader_locators.retain(|x| x.locator() != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        todo!()
    }
}

impl<PSM: RTPSWriterImplTrait> RTPSStatefulWriter<PSM> for RTPSWriterImpl<PSM> {
    type ReaderProxyType = RTPSReaderProxyImpl<PSM>;

    fn matched_readers(&self) -> &[Self::ReaderProxyType] {
        &self.matched_readers
    }

    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType) {
        self.matched_readers.push(a_reader_proxy)
    }

    fn matched_reader_remove(&mut self, reader_proxy_guid: &PSM::GUIDType) {
        self.matched_readers
            .retain(|x| x.remote_reader_guid() != reader_proxy_guid)
    }

    fn matched_reader_lookup(
        &self,
        a_reader_guid: &PSM::GUIDType,
    ) -> Option<&Self::ReaderProxyType> {
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
    use rust_rtps_pim::structure::RTPSCacheChange;

    use super::*;

    struct MockPSM;

    impl rust_rtps_pim::structure::types::InstanceHandlePIM for MockPSM {
        type InstanceHandleType = ();
    }

    impl rust_rtps_pim::structure::types::SequenceNumberPIM for MockPSM {
        type SequenceNumberType = i64;
        const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumberType = -1;
    }

    impl rust_rtps_pim::structure::types::DataPIM for MockPSM {
        type DataType = [u8;0];
    }

    impl rust_rtps_pim::structure::types::EntityIdPIM for MockPSM {
        type EntityIdType = [u8; 4];

        const ENTITYID_UNKNOWN: Self::EntityIdType = [0; 4];
        const ENTITYID_PARTICIPANT: Self::EntityIdType = [1; 4];
    }

    impl rust_rtps_pim::messages::types::ParameterIdPIM for MockPSM {
        type ParameterIdType = u16;
    }

    impl rust_rtps_pim::structure::types::GuidPrefixPIM for MockPSM {
        type GuidPrefixType = [u8; 12];
        const GUIDPREFIX_UNKNOWN: Self::GuidPrefixType = [0; 12];
    }

    #[derive(Clone, Copy, PartialEq)]
    struct MockGUID(u8);

    impl rust_rtps_pim::structure::types::GUID<MockPSM> for MockGUID {
        fn new(_prefix: [u8; 12], _entity_id: [u8; 4]) -> Self {
            todo!()
        }

        fn prefix(&self) -> &[u8; 12] {
            todo!()
        }

        fn entity_id(&self) -> &[u8; 4] {
            todo!()
        }
    }

    impl rust_rtps_pim::structure::types::GUIDPIM<MockPSM> for MockPSM {
        type GUIDType = MockGUID;
        const GUID_UNKNOWN: Self::GUIDType = MockGUID(0);
    }

    impl rust_rtps_pim::structure::types::ParameterListPIM<MockPSM> for MockPSM {
        type ParameterListType = MockParameterList;
    }

    pub struct MockParameterList;

    impl rust_rtps_pim::messages::submessage_elements::ParameterList<MockPSM> for MockParameterList {
        type Parameter = MockParameter;
        type ParameterList = MockParameterList;

        fn new(_parameter: Self::ParameterList) -> Self {
            todo!()
        }

        fn parameter(&self) -> &Self::ParameterList {
            todo!()
        }
    }

    pub struct MockParameter;
    impl rust_rtps_pim::messages::submessage_elements::Parameter<MockPSM> for MockParameter {
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

    #[derive(Clone, Copy, PartialEq)]
    pub struct MockLocator(u8);

    impl rust_rtps_pim::structure::types::Locator for MockLocator {
        type LocatorKind = [u8; 4];

        const LOCATOR_KIND_INVALID: Self::LocatorKind = [0; 4];
        const LOCATOR_KIND_RESERVED: Self::LocatorKind = [1; 4];
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv4: Self::LocatorKind = [2; 4];
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv6: Self::LocatorKind = [3; 4];

        type LocatorPort = [u8; 4];
        const LOCATOR_PORT_INVALID: Self::LocatorPort = [0; 4];

        type LocatorAddress = [u8; 16];

        const LOCATOR_ADDRESS_INVALID: Self::LocatorAddress = [0; 16];
        const LOCATOR_INVALID: Self = MockLocator(0);

        fn kind(&self) -> &Self::LocatorKind {
            todo!()
        }

        fn port(&self) -> &Self::LocatorPort {
            todo!()
        }

        fn address(&self) -> &Self::LocatorAddress {
            todo!()
        }
    }

    impl rust_rtps_pim::structure::types::LocatorPIM for MockPSM {
        type LocatorType = MockLocator;
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
            MockGUID(1),
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
            MockGUID(1),
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
        let reader_locator1 = RTPSReaderLocatorImpl::new(MockLocator(1), false);
        let reader_locator2 = RTPSReaderLocatorImpl::new(MockLocator(2), false);
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
            MockGUID(1),
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

        let reader_locator1 = RTPSReaderLocatorImpl::new(MockLocator(1), false);
        let reader_locator2 = RTPSReaderLocatorImpl::new(MockLocator(2), false);
        writer.reader_locator_add(reader_locator1);
        writer.reader_locator_add(reader_locator2);
        writer.reader_locator_remove(&MockLocator(1));

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
            MockGUID(1),
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
            RTPSReaderProxyImpl::new(MockGUID(2), [0; 4], vec![], vec![], false, true);
        let reader_proxy2 =
            RTPSReaderProxyImpl::new(MockGUID(3), [0; 4], vec![], vec![], false, true);
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
            MockGUID(1),
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
            RTPSReaderProxyImpl::new(MockGUID(2), [0; 4], vec![], vec![], false, true);
        let reader_proxy2 =
            RTPSReaderProxyImpl::new(MockGUID(3), [0; 4], vec![], vec![], false, true);
        writer.matched_reader_add(reader_proxy1);
        writer.matched_reader_add(reader_proxy2);
        writer.matched_reader_remove(&MockGUID(1));

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
            MockGUID(1),
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
            RTPSReaderProxyImpl::new(MockGUID(2), [0; 4], vec![], vec![], false, true);
        let reader_proxy2 =
            RTPSReaderProxyImpl::new(MockGUID(3), [0; 4], vec![], vec![], false, true);
        writer.matched_reader_add(reader_proxy1);
        writer.matched_reader_add(reader_proxy2);

        assert!(writer.matched_reader_lookup(&MockGUID(3)).is_some());
        assert!(writer.matched_reader_lookup(&MockGUID(4)).is_none());
    }
}
