use rust_rtps_pim::{
    behavior::{
        stateful_writer::{RTPSReaderProxy, RTPSStatefulWriter},
        stateless_writer::RTPSStatelessWriter,
        types::DurationType,
        RTPSWriter,
    },
    messages::types::ParameterIdType,
    structure::{
        types::{
            ChangeKind, DataType, EntityIdType, GUIDType, GuidPrefixType, InstanceHandleType,
            LocatorType, ParameterListType, ReliabilityKind, SequenceNumberType, TopicKind,
        },
        RTPSEndpoint, RTPSEntity, RTPSHistoryCache,
    },
};

use super::{
    rtps_cache_change_impl::RTPSCacheChangeImpl, rtps_history_cache_impl::RTPSHistoryCacheImpl,
    rtps_reader_locator_impl::RTPSReaderLocatorImpl,
};

pub trait RTPSWriterImplTrait:
    SequenceNumberType
    + GuidPrefixType
    + EntityIdType
    + DurationType
    + DataType
    + LocatorType
    + InstanceHandleType
    + ParameterIdType
    + GUIDType<Self>
    + ParameterListType<Self>
    + Sized
{
}

impl<
        T: SequenceNumberType
            + GuidPrefixType
            + EntityIdType
            + DurationType
            + DataType
            + LocatorType
            + InstanceHandleType
            + ParameterIdType
            + GUIDType<Self>
            + ParameterListType<Self>
            + Sized,
    > RTPSWriterImplTrait for T
{
}

pub struct RTPSWriterImpl<PSM: RTPSWriterImplTrait> {
    guid: PSM::GUID,
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    push_mode: bool,
    unicast_locator_list: Vec<PSM::Locator>,
    multicast_locator_list: Vec<PSM::Locator>,
    heartbeat_period: PSM::Duration,
    nack_response_delay: PSM::Duration,
    nack_suppression_duration: PSM::Duration,
    last_change_sequence_number: PSM::SequenceNumber,
    data_max_size_serialized: i32,
    reader_locators: Vec<RTPSReaderLocatorImpl<PSM>>,
    reader_proxies: Vec<RTPSReaderProxy<PSM>>,
    writer_cache: RTPSHistoryCacheImpl<PSM>,
}

impl<PSM: RTPSWriterImplTrait> RTPSWriterImpl<PSM> {
    pub fn new(
        guid: PSM::GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        push_mode: bool,
        unicast_locator_list: Vec<PSM::Locator>,
        multicast_locator_list: Vec<PSM::Locator>,
        heartbeat_period: PSM::Duration,
        nack_response_delay: PSM::Duration,
        nack_suppression_duration: PSM::Duration,
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
            reader_proxies: Vec::new(),
            writer_cache: RTPSHistoryCacheImpl::new(),
        }
    }
}

impl<PSM: RTPSWriterImplTrait> RTPSEntity<PSM> for RTPSWriterImpl<PSM> {
    fn guid(&self) -> &PSM::GUID {
        &self.guid
    }
}

impl<PSM: RTPSWriterImplTrait> RTPSWriter<PSM> for RTPSWriterImpl<PSM> {
    type HistoryCacheType = RTPSHistoryCacheImpl<PSM>;

    fn push_mode(&self) -> bool {
        self.push_mode
    }

    fn heartbeat_period(&self) -> &PSM::Duration {
        &self.heartbeat_period
    }

    fn nack_response_delay(&self) -> &PSM::Duration {
        &self.nack_response_delay
    }

    fn nack_suppression_duration(&self) -> &PSM::Duration {
        &self.nack_suppression_duration
    }

    fn last_change_sequence_number(&self) -> &PSM::SequenceNumber {
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
        data: PSM::Data,
        inline_qos: PSM::ParameterList,
        handle: PSM::InstanceHandle,
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

    fn unicast_locator_list(&self) -> &[PSM::Locator] {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[PSM::Locator] {
        &self.multicast_locator_list
    }
}

impl<PSM: RTPSWriterImplTrait> RTPSStatelessWriter<PSM> for RTPSWriterImpl<PSM> {
    fn reader_locator_add(&mut self, a_locator: PSM::Locator) {
        todo!()
    }

    fn reader_locator_remove(&mut self, a_locator: &PSM::Locator) {
        todo!()
    }

    fn unsent_changes_reset(&mut self) {
        todo!()
    }
}

impl<PSM: RTPSWriterImplTrait> RTPSStatefulWriter<PSM> for RTPSWriterImpl<PSM> {
    fn matched_reader_add(&mut self, guid: PSM::GUID) {
        todo!()
    }

    fn matched_reader_remove(&mut self, reader_proxy_guid: &PSM::GUID) {
        todo!()
    }

    fn matched_reader_lookup(&self, a_reader_guid: PSM::GUID) -> Option<&RTPSReaderProxy<PSM>> {
        todo!()
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

    impl rust_rtps_pim::structure::types::InstanceHandleType for MockPSM {
        type InstanceHandle = ();
    }

    impl rust_rtps_pim::structure::types::SequenceNumberType for MockPSM {
        type SequenceNumber = i64;
        const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber = -1;
    }

    impl rust_rtps_pim::structure::types::DataType for MockPSM {
        type Data = ();
    }

    impl rust_rtps_pim::structure::types::EntityIdType for MockPSM {
        type EntityId = [u8; 4];

        const ENTITYID_UNKNOWN: Self::EntityId = [0; 4];
        const ENTITYID_PARTICIPANT: Self::EntityId = [1; 4];
    }

    impl rust_rtps_pim::messages::types::ParameterIdType for MockPSM {
        type ParameterId = u16;
    }

    impl rust_rtps_pim::structure::types::GuidPrefixType for MockPSM {
        type GuidPrefix = [u8; 12];
        const GUIDPREFIX_UNKNOWN: Self::GuidPrefix = [0; 12];
    }

    #[derive(Clone, Copy)]
    struct MockGUID;

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

    impl rust_rtps_pim::structure::types::GUIDType<MockPSM> for MockPSM {
        type GUID = MockGUID;
        const GUID_UNKNOWN: Self::GUID = MockGUID;
    }

    impl rust_rtps_pim::structure::types::ParameterListType<MockPSM> for MockPSM {
        type ParameterList = MockParameterList;
    }

    pub struct MockParameterList;

    impl rust_rtps_pim::messages::submessage_elements::ParameterList<MockPSM> for MockParameterList {
        type Parameter = MockParameter;

        fn parameter(&self) -> &[Self::Parameter] {
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

    impl rust_rtps_pim::behavior::types::DurationType for MockPSM {
        type Duration = i64;
    }

    #[derive(Clone, Copy, PartialEq)]
    pub struct MockLocator;

    impl rust_rtps_pim::structure::types::LocatorSubTypes for MockLocator {
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
        const LOCATOR_INVALID: Self = MockLocator;

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

    impl rust_rtps_pim::structure::types::LocatorType for MockPSM {
        type Locator = MockLocator;
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
            MockGUID,
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
        let change1 = writer.new_change(ChangeKind::Alive, (), MockParameterList, ());
        let change2 = writer.new_change(ChangeKind::Alive, (), MockParameterList, ());

        assert_eq!(change1.sequence_number(), &1);
        assert_eq!(change2.sequence_number(), &2);
    }

    // #[test]
    // fn send_data() {
    //     let mut rtps_writer_impl: RTPSWriterImpl<RtpsUdpPsm> = RTPSWriterImpl::new(
    //         DataWriterQos::default(),
    //         GUID::new(
    //             [1; 12],
    //             EntityId {
    //                 entity_key: [1; 3],
    //                 entity_kind: 1,
    //             },
    //         ),
    //     );

    //     let cc = rtps_writer_impl.new_change(ChangeKind::Alive, vec![0, 1, 2, 3], &[], 0);
    //     rtps_writer_impl.writer_cache_mut().add_change(cc);
    //     rtps_writer_impl.reader_locator_add(Locator::new(1, 2, [0; 16]));
    //     {
    //         println!("First");
    //         // let mut data_vec = Vec::new();
    //         // rtps_writer_impl.produce_messages(
    //         //     &mut |x, y| {
    //         //         println!("Locator: {:?}, Data: {}", x.address(), y.endianness_flag);
    //         //         data_vec.push(y);
    //         //     },
    //         //     &mut |_, _| (),
    //         // );
    //         // for data in &data_vec {
    //         //     println!("{}", data.endianness_flag);
    //         // }
    //     }
    //     println!("Second");
    //     // rtps_writer_impl.reader_locator_add(Locator::new(1, 3, [1; 16]));
    //     // rtps_writer_impl.produce_messages(
    //     //     &mut |x, y| println!("Locator: {:?}, Data: {}", x.address(), y.endianness_flag),
    //     //     &mut |_, _| (),
    //     // );
    //     // println!("After reset");
    //     // rtps_writer_impl.unsent_changes_reset();
    //     // rtps_writer_impl.produce_messages(
    //     //     &mut |x, y| println!("Locator: {:?}, Data: {}", x.address(), y.endianness_flag),
    //     //     &mut |_, _| (),
    //     // );
    // }
}
