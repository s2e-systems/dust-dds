use std::{cell::RefCell, collections::HashMap};

use dds_api::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps_psm::{
        Duration, InstanceHandle, LivelinessLostStatus, OfferedDeadlineMissedStatus,
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, StatusMask, Time,
    },
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::DataWriterQos,
    },
    publication::{
        data_writer::{DataWriter, DataWriterGetPublisher, DataWriterGetTopic},
        data_writer_listener::DataWriterListener,
        publisher::Publisher,
    },
    return_type::{DdsError, DdsResult},
    topic::topic_description::TopicDescription,
};
use rtps_pim::{
    behavior::{
        stateful_writer_behavior::{
            RtpsStatefulWriterReceiveAckNackSubmessage, RtpsStatefulWriterSendSubmessages,
        },
        stateless_writer_behavior::{
            RtpsStatelessWriterReceiveAckNackSubmessage, RtpsStatelessWriterSendSubmessages,
        },
        writer::{
            reader_locator::RtpsReaderLocatorAttributes,
            reader_proxy::{RtpsReaderProxyAttributes, RtpsReaderProxyConstructor},
            stateful_writer::{RtpsStatefulWriterAttributes, RtpsStatefulWriterOperations},
            writer::RtpsWriterOperations,
        },
    },
    discovery::{
        participant_discovery::ParticipantDiscovery,
        spdp::spdp_discovered_participant_data::RtpsSpdpDiscoveredParticipantDataAttributes,
    },
    messages::{
        overall_structure::{RtpsMessage, RtpsMessageHeader, RtpsSubmessageType},
        submessage_elements::{Parameter, TimestampSubmessageElement},
        submessages::{AckNackSubmessage, InfoTimestampSubmessage},
        types::{FragmentNumber, TIME_INVALID},
    },
    structure::{
        cache_change::RtpsCacheChangeAttributes,
        entity::RtpsEntityAttributes,
        history_cache::RtpsHistoryCacheOperations,
        types::{
            ChangeKind, Guid, GuidPrefix, Locator, SequenceNumber, PROTOCOLVERSION, VENDOR_ID_S2E,
        },
    },
    transport::TransportWrite,
};

use crate::{
    data_representation_builtin_endpoints::{
        discovered_reader_data::DiscoveredReaderData, discovered_topic_data::DiscoveredTopicData,
        discovered_writer_data::DiscoveredWriterData,
        spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    },
    dds_type::{DdsSerialize, DdsType, LittleEndian},
    utils::{
        discovery_traits::AddMatchedReader,
        rtps_communication_traits::{ReceiveRtpsAckNackSubmessage, SendRtpsMessage},
        rtps_structure::RtpsStructure,
        shared_object::{DdsRwLock, DdsShared, DdsWeak},
    },
};

use super::{publisher_attributes::PublisherAttributes, topic_attributes::TopicAttributes};

pub trait AnyDataWriterListener<DW> {
    fn trigger_on_liveliness_lost(&mut self, _the_writer: DW, _status: LivelinessLostStatus);
    fn trigger_on_offered_deadline_missed(
        &mut self,
        _the_writer: DW,
        _status: OfferedDeadlineMissedStatus,
    );
    fn trigger_on_offered_incompatible_qos(
        &mut self,
        _the_writer: DW,
        _status: OfferedIncompatibleQosStatus,
    );
    fn trigger_on_publication_matched(
        &mut self,
        _the_writer: DW,
        _status: PublicationMatchedStatus,
    );
}

impl<Foo, DW> AnyDataWriterListener<DW> for Box<dyn DataWriterListener<Foo = Foo> + Send + Sync>
where
    DW: DataWriter<Foo>,
{
    fn trigger_on_liveliness_lost(&mut self, the_writer: DW, status: LivelinessLostStatus) {
        self.on_liveliness_lost(&the_writer, status);
    }

    fn trigger_on_offered_deadline_missed(
        &mut self,
        the_writer: DW,
        status: OfferedDeadlineMissedStatus,
    ) {
        self.on_offered_deadline_missed(&the_writer, status);
    }

    fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: DW,
        status: OfferedIncompatibleQosStatus,
    ) {
        self.on_offered_incompatible_qos(&the_writer, status);
    }

    fn trigger_on_publication_matched(&mut self, the_writer: DW, status: PublicationMatchedStatus) {
        self.on_publication_matched(&the_writer, status)
    }
}

pub enum RtpsWriter<Rtps>
where
    Rtps: RtpsStructure,
{
    Stateless(Rtps::StatelessWriter),
    Stateful(Rtps::StatefulWriter),
}

impl<Rtps> RtpsEntityAttributes for RtpsWriter<Rtps>
where
    Rtps: RtpsStructure,
    Rtps::StatelessWriter: RtpsEntityAttributes,
    Rtps::StatefulWriter: RtpsEntityAttributes,
{
    fn guid(&self) -> Guid {
        match self {
            RtpsWriter::Stateless(w) => w.guid(),
            RtpsWriter::Stateful(w) => w.guid(),
        }
    }
}

impl<Rtps> RtpsWriter<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn try_as_stateless_writer(&mut self) -> DdsResult<&mut Rtps::StatelessWriter> {
        match self {
            RtpsWriter::Stateless(x) => Ok(x),
            RtpsWriter::Stateful(_) => Err(DdsError::PreconditionNotMet(
                "Not a stateless writer".to_string(),
            )),
        }
    }
    pub fn try_as_stateful_writer(&mut self) -> DdsResult<&mut Rtps::StatefulWriter> {
        match self {
            RtpsWriter::Stateless(_) => Err(DdsError::PreconditionNotMet(
                "Not a stateful writer".to_string(),
            )),
            RtpsWriter::Stateful(x) => Ok(x),
        }
    }
}

pub struct DataWriterAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    _qos: DataWriterQos,
    rtps_writer: DdsRwLock<RtpsWriter<Rtps>>,
    sample_info: DdsRwLock<HashMap<SequenceNumber, Time>>,
    listener: DdsRwLock<Option<Box<dyn AnyDataWriterListener<DdsShared<Self>> + Send + Sync>>>,
    topic: DdsShared<TopicAttributes<Rtps>>,
    publisher: DdsWeak<PublisherAttributes<Rtps>>,
    status: DdsRwLock<PublicationMatchedStatus>,
}

impl<Rtps> DataWriterAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(
        qos: DataWriterQos,
        rtps_writer: RtpsWriter<Rtps>,
        listener: Option<<DdsShared<Self> as Entity>::Listener>,
        topic: DdsShared<TopicAttributes<Rtps>>,
        publisher: DdsWeak<PublisherAttributes<Rtps>>,
    ) -> DdsShared<Self> {
        DdsShared::new(Self {
            _qos: qos,
            rtps_writer: DdsRwLock::new(rtps_writer),
            sample_info: DdsRwLock::new(HashMap::new()),
            listener: DdsRwLock::new(listener),
            topic,
            publisher,
            status: DdsRwLock::new(PublicationMatchedStatus {
                total_count: 0,
                total_count_change: 0,
                last_subscription_handle: 0,
                current_count: 0,
                current_count_change: 0,
            }),
        })
    }
}

impl<Rtps> DataWriterAttributes<Rtps>
where
    Rtps: RtpsStructure,
    Rtps::StatefulWriter: for<'a> RtpsStatefulWriterAttributes<'a> + RtpsStatefulWriterOperations,
    for<'a> <Rtps::StatefulWriter as RtpsStatefulWriterAttributes<'a>>::ReaderProxyListType:
        IntoIterator,
    for<'a> <<Rtps::StatefulWriter as RtpsStatefulWriterAttributes<'a>>::ReaderProxyListType as IntoIterator>::Item:
        RtpsReaderProxyAttributes,
    <Rtps::StatefulWriter as RtpsStatefulWriterOperations>::ReaderProxyType: RtpsReaderProxyConstructor,
{
    pub fn add_matched_participant(
        &self,
        participant_discovery: &ParticipantDiscovery<SpdpDiscoveredParticipantData>,
    ) {
        let mut rtps_writer_lock = self.rtps_writer.write_lock();
        if let RtpsWriter::Stateful(rtps_writer) = &mut *rtps_writer_lock {
            if !rtps_writer
                .matched_readers()
                .into_iter()
                .any(|r| r.remote_reader_guid().prefix == participant_discovery.guid_prefix())
            {
                let type_name = self.topic.get_type_name().unwrap();
                if type_name == DiscoveredWriterData::type_name() {
                    participant_discovery.discovered_participant_add_publications_writer(rtps_writer);
                } else if type_name == DiscoveredReaderData::type_name() {
                    participant_discovery.discovered_participant_add_subscriptions_writer(rtps_writer);
                } else if type_name == DiscoveredTopicData::type_name() {
                    participant_discovery.discovered_participant_add_topics_writer(rtps_writer);
                }
            }
        }
    }
}

impl<Rtps> ReceiveRtpsAckNackSubmessage for DdsShared<DataWriterAttributes<Rtps>>
where
    Rtps: RtpsStructure,
    Rtps::StatelessWriter: RtpsStatelessWriterReceiveAckNackSubmessage<Vec<SequenceNumber>>,
    Rtps::StatefulWriter: RtpsStatefulWriterReceiveAckNackSubmessage<Vec<SequenceNumber>>,
{
    fn on_acknack_submessage_received(
        &self,
        acknack_submessage: &AckNackSubmessage<Vec<SequenceNumber>>,
        source_guid_prefix: GuidPrefix,
    ) {
        match &mut *self.rtps_writer.write_lock() {
            RtpsWriter::Stateless(stateless_rtps_writer) => {
                stateless_rtps_writer.on_acknack_submessage_received(&acknack_submessage)
            }
            RtpsWriter::Stateful(stateful_rtps_writer) => stateful_rtps_writer
                .on_acknack_submessage_received(&acknack_submessage, source_guid_prefix),
        }
    }
}

impl<Rtps> AddMatchedReader for DdsShared<DataWriterAttributes<Rtps>>
where
    Rtps: RtpsStructure,
    Rtps::StatefulWriter: RtpsStatefulWriterOperations,
    <Rtps::StatefulWriter as RtpsStatefulWriterOperations>::ReaderProxyType:
        RtpsReaderProxyConstructor,
{
    fn add_matched_reader(&self, discovered_reader_data: &DiscoveredReaderData) {
        let topic_name = &discovered_reader_data
            .subscription_builtin_topic_data
            .topic_name;
        let type_name = &discovered_reader_data
            .subscription_builtin_topic_data
            .type_name;
        let writer_topic_name = &self.topic.get_name().unwrap();
        let writer_type_name = self.topic.get_type_name().unwrap();
        if topic_name == writer_topic_name && type_name == writer_type_name {
            let reader_proxy =
                <Rtps::StatefulWriter as RtpsStatefulWriterOperations>::ReaderProxyType::new(
                    discovered_reader_data.reader_proxy.remote_reader_guid,
                    discovered_reader_data.reader_proxy.remote_group_entity_id,
                    discovered_reader_data
                        .reader_proxy
                        .unicast_locator_list
                        .as_ref(),
                    discovered_reader_data
                        .reader_proxy
                        .multicast_locator_list
                        .as_ref(),
                    discovered_reader_data.reader_proxy.expects_inline_qos,
                    true, // ???
                );
            match &mut *self.rtps_writer.write_lock() {
                RtpsWriter::Stateless(_) => (),
                RtpsWriter::Stateful(rtps_stateful_writer) => {
                    rtps_stateful_writer.matched_reader_add(reader_proxy);

                    let mut status = self.status.write_lock();
                    1;
                    status.total_count += 1;

                    self.listener
                        .write_lock()
                        .as_mut()
                        .map(|l| l.trigger_on_publication_matched(self.clone(), *status));
                }
            };
        }
    }
}

impl<Rtps, Foo> DataWriter<Foo> for DdsShared<DataWriterAttributes<Rtps>>
where
    Rtps: RtpsStructure,
    Rtps::StatefulWriter: for<'a> RtpsStatefulWriterAttributes<'a>,
    for<'a> <Rtps::StatefulWriter as RtpsStatefulWriterAttributes<'a>>::ReaderProxyListType:
        IntoIterator,
    Foo: DdsSerialize,
{
    fn register_instance(&self, _instance: Foo) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }

    fn register_instance_w_timestamp(
        &self,
        _instance: Foo,
        _timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }

    fn unregister_instance(
        &self,
        _instance: Foo,
        _handle: Option<InstanceHandle>,
    ) -> DdsResult<()> {
        todo!()
    }

    fn unregister_instance_w_timestamp(
        &self,
        _instance: Foo,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut Foo, _handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }

    fn write(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = self
            .publisher
            .upgrade()?
            .get_participant()?
            .upgrade()?
            .get_current_time()?;
        self.write_w_timestamp(data, handle, timestamp)
    }

    fn write_w_timestamp(
        &self,
        data: &Foo,
        _handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let mut serialized_data = Vec::new();
        data.serialize::<_, LittleEndian>(&mut serialized_data)?;
        let mut rtps_writer_lock = self.rtps_writer.write_lock();
        let mut sample_info_lock = self.sample_info.write_lock();
        let sequence_number;
        match &mut *rtps_writer_lock {
            RtpsWriter::Stateless(rtps_writer) => {
                let change = rtps_writer.new_change(ChangeKind::Alive, serialized_data, vec![], 0);
                sequence_number = change.sequence_number();
                rtps_writer.add_change(change);
            }
            RtpsWriter::Stateful(rtps_writer) => {
                let change = rtps_writer.new_change(ChangeKind::Alive, serialized_data, vec![], 0);
                sequence_number = change.sequence_number();
                rtps_writer.add_change(change);
            }
        }
        sample_info_lock.insert(sequence_number, timestamp);

        Ok(())
    }

    fn dispose(&self, _data: Foo, _handle: Option<InstanceHandle>) -> DdsResult<()> {
        todo!()
    }

    fn dispose_w_timestamp(
        &self,
        _data: Foo,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DdsResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DdsResult<()> {
        todo!()
    }

    fn get_liveliness_lost_status(&self, _status: &mut LivelinessLostStatus) -> DdsResult<()> {
        todo!()
    }

    fn get_offered_deadline_missed_status(
        &self,
        _status: &mut OfferedDeadlineMissedStatus,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_offered_incompatible_qos_status(
        &self,
        _status: &mut OfferedIncompatibleQosStatus,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_publication_matched_status(
        &self,
        _status: &mut PublicationMatchedStatus,
    ) -> DdsResult<()> {
        todo!()
    }

    fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
    }

    fn get_matched_subscription_data(
        &self,
        _subscription_data: SubscriptionBuiltinTopicData,
        _subscription_handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_matched_subscriptions(&self) -> DdsResult<Vec<InstanceHandle>> {
        let mut rpts_writer_lock = self.rtps_writer.write_lock();
        let matched_subscriptions = match &mut *rpts_writer_lock {
            RtpsWriter::Stateless(_) => vec![],
            RtpsWriter::Stateful(w) => w
                .matched_readers()
                .into_iter()
                .enumerate()
                .map(|x| x.0 as i32)
                .collect(),
        };

        Ok(matched_subscriptions)
    }
}

impl<Rtps> DataWriterGetPublisher for DdsShared<DataWriterAttributes<Rtps>>
where
    Rtps: RtpsStructure,
{
    type PublisherType = DdsShared<PublisherAttributes<Rtps>>;

    fn datawriter_get_publisher(&self) -> DdsResult<Self::PublisherType> {
        Ok(self.publisher.upgrade()?.clone())
    }
}

impl<Rtps> DataWriterGetTopic for DdsShared<DataWriterAttributes<Rtps>>
where
    Rtps: RtpsStructure,
{
    type TopicType = DdsShared<TopicAttributes<Rtps>>;

    fn datawriter_get_topic(&self) -> DdsResult<Self::TopicType> {
        Ok(self.topic.clone())
    }
}

impl<Rtps> Entity for DdsShared<DataWriterAttributes<Rtps>>
where
    Rtps: RtpsStructure,
{
    type Qos = DataWriterQos;
    type Listener = Box<dyn AnyDataWriterListener<Self> + Send + Sync>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DdsResult<()> {
        todo!()
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        todo!()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, _mask: StatusMask) -> DdsResult<()> {
        *self.listener.write_lock() = a_listener;
        Ok(())
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        todo!()
    }

    fn enable(&self) -> DdsResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        todo!()
    }
}

impl<Rtps> SendRtpsMessage for DdsShared<DataWriterAttributes<Rtps>>
where
    Rtps: RtpsStructure,
    Rtps::StatelessWriter: RtpsEntityAttributes
        + for<'a> RtpsStatelessWriterSendSubmessages<
            'a,
            Vec<Parameter<'a>>,
            &'a [u8],
            Vec<SequenceNumber>,
        >,
    for<'a> <Rtps::StatelessWriter as RtpsStatelessWriterSendSubmessages<
        'a,
        Vec<Parameter<'a>>,
        &'a [u8],
        Vec<SequenceNumber>,
    >>::ReaderLocatorType: RtpsReaderLocatorAttributes,
    Rtps::StatefulWriter: RtpsEntityAttributes
        + for<'a> RtpsStatefulWriterSendSubmessages<
            'a,
            Vec<Parameter<'a>>,
            &'a [u8],
            Vec<SequenceNumber>,
        >,
    for<'a> <Rtps::StatefulWriter as RtpsStatefulWriterSendSubmessages<
        'a,
        Vec<Parameter<'a>>,
        &'a [u8],
        Vec<SequenceNumber>,
    >>::ReaderProxyType: RtpsReaderProxyAttributes,
{
    fn send_message(
        &self,
        transport: &mut impl for<'a> TransportWrite<
            Vec<
                RtpsSubmessageType<
                    Vec<SequenceNumber>,
                    Vec<Parameter<'a>>,
                    &'a [u8],
                    Vec<Locator>,
                    Vec<FragmentNumber>,
                >,
            >,
        >,
    ) {
        let destined_submessages = RefCell::new(Vec::new());

        let mut rtps_writer_lock = self.rtps_writer.write_lock();
        let sample_info_lock = self.sample_info.read_lock();
        let guid_prefix = rtps_writer_lock.guid().prefix();
        match &mut *rtps_writer_lock {
            RtpsWriter::Stateless(stateless_rtps_writer) => {
                stateless_rtps_writer.send_submessages(
                    |reader_locator, data| {
                        let info_ts =
                            if let Some(time) = sample_info_lock.get(&data.writer_sn.value) {
                                InfoTimestampSubmessage {
                                    endianness_flag: true,
                                    invalidate_flag: false,
                                    timestamp: TimestampSubmessageElement {
                                        value: rtps_pim::messages::types::Time(
                                            ((time.sec as u64) << 32) + time.nanosec as u64,
                                        ),
                                    },
                                }
                            } else {
                                InfoTimestampSubmessage {
                                    endianness_flag: true,
                                    invalidate_flag: true,
                                    timestamp: TimestampSubmessageElement {
                                        value: TIME_INVALID,
                                    },
                                }
                            };
                        destined_submessages.borrow_mut().push((
                            vec![reader_locator.locator()],
                            vec![
                                RtpsSubmessageType::InfoTimestamp(info_ts),
                                RtpsSubmessageType::Data(data),
                            ],
                        ));
                    },
                    |reader_locator, gap| {
                        destined_submessages.borrow_mut().push((
                            vec![reader_locator.locator()],
                            vec![RtpsSubmessageType::Gap(gap)],
                        ));
                    },
                    |_, _| (),
                );
            }
            RtpsWriter::Stateful(stateful_rtps_writer) => {
                stateful_rtps_writer.send_submessages(
                    |reader_proxy, data| {
                        let info_ts =
                            if let Some(time) = sample_info_lock.get(&data.writer_sn.value) {
                                InfoTimestampSubmessage {
                                    endianness_flag: true,
                                    invalidate_flag: false,
                                    timestamp: TimestampSubmessageElement {
                                        value: rtps_pim::messages::types::Time(
                                            ((time.sec as u64) << 32) + time.nanosec as u64,
                                        ),
                                    },
                                }
                            } else {
                                InfoTimestampSubmessage {
                                    endianness_flag: true,
                                    invalidate_flag: true,
                                    timestamp: TimestampSubmessageElement {
                                        value: TIME_INVALID,
                                    },
                                }
                            };
                        destined_submessages.borrow_mut().push((
                            reader_proxy.unicast_locator_list().to_vec(),
                            vec![
                                RtpsSubmessageType::InfoTimestamp(info_ts),
                                RtpsSubmessageType::Data(data),
                            ],
                        ));
                    },
                    |reader_proxy, gap| {
                        destined_submessages.borrow_mut().push((
                            reader_proxy.unicast_locator_list().to_vec(),
                            vec![RtpsSubmessageType::Gap(gap)],
                        ));
                    },
                    |reader_proxy, heartbeat| {
                        destined_submessages.borrow_mut().push((
                            reader_proxy.unicast_locator_list().to_vec(),
                            vec![RtpsSubmessageType::Heartbeat(heartbeat)],
                        ));
                    },
                );
            }
        }
        let writer_destined_submessages = destined_submessages.take();

        for (locator_list, submessages) in writer_destined_submessages {
            let header = RtpsMessageHeader {
                protocol: rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
                version: PROTOCOLVERSION,
                vendor_id: VENDOR_ID_S2E,
                guid_prefix,
            };

            let rtps_message = RtpsMessage {
                header,
                submessages,
            };
            for locator in locator_list {
                transport.write(&rtps_message, locator);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use dds_api::infrastructure::qos::TopicQos;

    use crate::{
        dds_type::Endianness,
        test_utils::{
            mock_rtps::MockRtps, mock_rtps_cache_change::MockRtpsCacheChange,
            mock_rtps_history_cache::MockRtpsHistoryCache,
            mock_rtps_stateful_writer::MockRtpsStatefulWriter,
            mock_rtps_stateless_writer::MockRtpsStatelessWriter,
        },
    };

    use super::*;

    struct MockFoo {}

    impl DdsSerialize for MockFoo {
        fn serialize<W: Write, E: Endianness>(&self, _writer: W) -> DdsResult<()> {
            Ok(())
        }
    }

    #[test]
    fn try_as_stateful_writer_on_stateful_is_ok() {
        assert!(
            RtpsWriter::<MockRtps>::Stateful(MockRtpsStatefulWriter::new())
                .try_as_stateful_writer()
                .is_ok()
        );
    }

    #[test]
    fn try_as_stateful_writer_on_stateless_is_err() {
        assert!(
            RtpsWriter::<MockRtps>::Stateless(MockRtpsStatelessWriter::new())
                .try_as_stateful_writer()
                .is_err()
        );
    }

    #[test]
    fn try_as_stateless_writer_on_stateless_is_ok() {
        assert!(
            RtpsWriter::<MockRtps>::Stateless(MockRtpsStatelessWriter::new())
                .try_as_stateless_writer()
                .is_ok()
        );
    }

    #[test]
    fn try_as_stateless_writer_on_stateful_is_err() {
        assert!(
            RtpsWriter::<MockRtps>::Stateful(MockRtpsStatefulWriter::new())
                .try_as_stateless_writer()
                .is_err()
        );
    }

    #[test]
    fn write_w_timestamp_stateless() {
        let mut mock_writer = MockRtpsStatelessWriter::new();
        mock_writer
            .expect_new_change()
            .once()
            .return_once(|_, _, _, _| {
                let mut mock_cache_change = MockRtpsCacheChange::new();
                mock_cache_change.expect_sequence_number().return_const(1);
                mock_cache_change
            });
        mock_writer.expect_add_change_().once().return_const(());

        let dummy_topic = TopicAttributes::new(TopicQos::default(), "", "", DdsWeak::new());

        let shared_data_writer: DdsShared<DataWriterAttributes<MockRtps>> =
            DataWriterAttributes::new(
                DataWriterQos::default(),
                RtpsWriter::Stateless(mock_writer),
                None,
                dummy_topic,
                DdsWeak::new(),
            );

        shared_data_writer
            .write_w_timestamp(&MockFoo {}, None, Time { sec: 0, nanosec: 0 })
            .unwrap();
    }

    #[test]
    fn write_w_timestamp_stateful() {
        let mock_writer_history_cache = MockRtpsHistoryCache::new();

        let mut mock_writer = MockRtpsStatefulWriter::new();
        mock_writer
            .expect_new_change()
            .once()
            .return_once(|_, _, _, _| {
                let mut mock_cache_change = MockRtpsCacheChange::new();
                mock_cache_change.expect_sequence_number().return_const(1);
                mock_cache_change
            });
        mock_writer
            .expect_writer_cache()
            .return_var(mock_writer_history_cache);
        mock_writer.expect_add_change_().once().return_const(());
        let dummy_topic = TopicAttributes::new(TopicQos::default(), "", "", DdsWeak::new());

        let shared_data_writer: DdsShared<DataWriterAttributes<MockRtps>> =
            DataWriterAttributes::new(
                DataWriterQos::default(),
                RtpsWriter::Stateful(mock_writer),
                None,
                dummy_topic,
                DdsWeak::new(),
            );

        shared_data_writer
            .write_w_timestamp(&MockFoo {}, None, Time { sec: 0, nanosec: 0 })
            .unwrap();
    }
}
