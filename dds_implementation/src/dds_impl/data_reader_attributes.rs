use std::collections::HashSet;

use dds_api::{
    builtin_topics::PublicationBuiltinTopicData,
    dcps_psm::{
        InstanceHandle, InstanceStateMask, LivelinessChangedStatus, RequestedDeadlineMissedStatus,
        RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus, SampleStateMask,
        StatusMask, SubscriptionMatchedStatus, Time, ViewStateMask, ALIVE_INSTANCE_STATE,
        DATA_AVAILABLE_STATUS, NEW_VIEW_STATE, NOT_READ_SAMPLE_STATE, READ_SAMPLE_STATE,
        REQUESTED_DEADLINE_MISSED_STATUS, SUBSCRIPTION_MATCHED_STATUS,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::DataReaderQos,
        qos_policy::HistoryQosPolicyKind,
        read_condition::ReadCondition,
        sample_info::SampleInfo,
    },
    return_type::{DdsError, DdsResult},
    subscription::{
        data_reader::{DataReader, DataReaderGetSubscriber, DataReaderGetTopicDescription},
        data_reader_listener::DataReaderListener,
        query_condition::QueryCondition,
    },
    topic::topic_description::TopicDescription,
};
use rtps_implementation::{
    rtps_history_cache_impl::{RtpsCacheChangeImpl, RtpsHistoryCacheImpl},
    rtps_stateful_reader_impl::RtpsStatefulReaderImpl,
    rtps_stateless_reader_impl::RtpsStatelessReaderImpl,
};
use rtps_pim::{
    behavior::{
        reader::{
            reader::RtpsReaderAttributes,
            stateful_reader::{RtpsStatefulReaderAttributes, RtpsStatefulReaderOperations},
            writer_proxy::{RtpsWriterProxyAttributes, RtpsWriterProxyConstructor},
        },
        stateful_reader_behavior::{
            RtpsStatefulReaderReceiveDataSubmessage, RtpsStatefulReaderReceiveHeartbeatSubmessage,
            RtpsStatefulReaderSendSubmessages,
        },
        stateless_reader_behavior::RtpsStatelessReaderReceiveDataSubmessage,
    },
    discovery::{
        participant_discovery::ParticipantDiscovery,
        spdp::spdp_discovered_participant_data::RtpsSpdpDiscoveredParticipantDataAttributes,
    },
    messages::{
        overall_structure::{RtpsMessage, RtpsMessageHeader, RtpsSubmessageType},
        submessage_elements::Parameter,
        submessages::{DataSubmessage, HeartbeatSubmessage},
        types::FragmentNumber,
    },
    structure::{
        cache_change::RtpsCacheChangeAttributes,
        entity::RtpsEntityAttributes,
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
        types::{GuidPrefix, Locator, SequenceNumber, PROTOCOLVERSION, VENDOR_ID_S2E},
    },
    transport::TransportWrite,
};

use crate::{
    data_representation_builtin_endpoints::{
        discovered_reader_data::DiscoveredReaderData, discovered_topic_data::DiscoveredTopicData,
        discovered_writer_data::DiscoveredWriterData,
        spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    },
    dds_type::{DdsDeserialize, DdsType},
    utils::{
        discovery_traits::AddMatchedWriter,
        rtps_communication_traits::{
            ReceiveRtpsDataSubmessage, ReceiveRtpsHeartbeatSubmessage, SendRtpsMessage,
        },
        shared_object::{DdsRwLock, DdsShared, DdsWeak},
        timer::Timer,
    },
};

use super::{subscriber_attributes::SubscriberAttributes, topic_attributes::TopicAttributes};

pub trait AnyDataReaderListener<DR> {
    fn trigger_on_data_available(&mut self, reader: DR);
    fn trigger_on_sample_rejected(&mut self, reader: DR, status: SampleRejectedStatus);
    fn trigger_on_liveliness_changed(&mut self, reader: DR, status: LivelinessChangedStatus);
    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: DR,
        status: RequestedDeadlineMissedStatus,
    );
    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: DR,
        status: RequestedIncompatibleQosStatus,
    );
    fn trigger_on_subscription_matched(&mut self, reader: DR, status: SubscriptionMatchedStatus);
    fn trigger_on_sample_lost(&mut self, reader: DR, status: SampleLostStatus);
}

impl<Foo, DR> AnyDataReaderListener<DR> for Box<dyn DataReaderListener<Foo = Foo> + Send + Sync>
where
    DR: DataReader<Foo>,
{
    fn trigger_on_data_available(&mut self, reader: DR) {
        self.on_data_available(&reader)
    }

    fn trigger_on_sample_rejected(&mut self, reader: DR, status: SampleRejectedStatus) {
        self.on_sample_rejected(&reader, status)
    }

    fn trigger_on_liveliness_changed(&mut self, reader: DR, status: LivelinessChangedStatus) {
        self.on_liveliness_changed(&reader, status)
    }

    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: DR,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.on_requested_deadline_missed(&reader, status)
    }

    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: DR,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.on_requested_incompatible_qos(&reader, status)
    }

    fn trigger_on_subscription_matched(&mut self, reader: DR, status: SubscriptionMatchedStatus) {
        self.on_subscription_matched(&reader, status)
    }

    fn trigger_on_sample_lost(&mut self, reader: DR, status: SampleLostStatus) {
        self.on_sample_lost(&reader, status)
    }
}

pub enum RtpsReader {
    Stateless(RtpsStatelessReaderImpl),
    Stateful(RtpsStatefulReaderImpl),
}

impl RtpsReaderAttributes for RtpsReader {
    type HistoryCacheType = RtpsHistoryCacheImpl;

    fn heartbeat_response_delay(&self) -> rtps_pim::behavior::types::Duration {
        match self {
            RtpsReader::Stateless(reader) => reader.heartbeat_response_delay(),
            RtpsReader::Stateful(reader) => reader.heartbeat_response_delay(),
        }
    }

    fn heartbeat_suppression_duration(&self) -> rtps_pim::behavior::types::Duration {
        match self {
            RtpsReader::Stateless(reader) => reader.heartbeat_suppression_duration(),
            RtpsReader::Stateful(reader) => reader.heartbeat_suppression_duration(),
        }
    }

    fn reader_cache(&mut self) -> &mut Self::HistoryCacheType {
        match self {
            RtpsReader::Stateless(reader) => reader.reader_cache(),
            RtpsReader::Stateful(reader) => reader.reader_cache(),
        }
    }

    fn expects_inline_qos(&self) -> bool {
        match self {
            RtpsReader::Stateless(reader) => reader.expects_inline_qos(),
            RtpsReader::Stateful(reader) => reader.expects_inline_qos(),
        }
    }
}

pub struct DataReaderAttributes<T> {
    rtps_reader: DdsRwLock<RtpsReader>,
    qos: DdsRwLock<DataReaderQos>,
    topic: DdsShared<TopicAttributes>,
    listener: DdsRwLock<Option<<DdsShared<Self> as Entity>::Listener>>,
    parent_subscriber: DdsWeak<SubscriberAttributes>,
    samples_read: DdsRwLock<HashSet<SequenceNumber>>,
    deadline_timer: DdsRwLock<T>,
    status_change: DdsRwLock<StatusMask>,
    subscription_matched_status: DdsRwLock<SubscriptionMatchedStatus>,
    requested_deadline_missed_status: DdsRwLock<RequestedDeadlineMissedStatus>,
}

pub trait DataReaderConstructor
where
    Self: Entity,
{
    fn new(
        qos: DataReaderQos,
        rtps_reader: RtpsReader,
        topic: DdsShared<TopicAttributes>,
        listener: Option<<Self as Entity>::Listener>,
        parent_subscriber: DdsWeak<SubscriberAttributes>,
    ) -> Self;
}

impl<T> DataReaderConstructor for DdsShared<DataReaderAttributes<T>>
where
    T: Timer,
{
    fn new(
        qos: DataReaderQos,
        rtps_reader: RtpsReader,
        topic: DdsShared<TopicAttributes>,
        listener: Option<<Self as Entity>::Listener>,
        parent_subscriber: DdsWeak<SubscriberAttributes>,
    ) -> Self {
        let deadline_duration = std::time::Duration::from_secs(*qos.deadline.period.sec() as u64)
            + std::time::Duration::from_nanos(*qos.deadline.period.nanosec() as u64);

        DdsShared::new(DataReaderAttributes {
            rtps_reader: DdsRwLock::new(rtps_reader),
            qos: DdsRwLock::new(qos),
            topic,
            listener: DdsRwLock::new(listener),
            parent_subscriber,
            samples_read: DdsRwLock::new(HashSet::new()),
            deadline_timer: DdsRwLock::new(T::new(deadline_duration)),
            status_change: DdsRwLock::new(0),
            subscription_matched_status: DdsRwLock::new(SubscriptionMatchedStatus {
                total_count: 0,
                total_count_change: 0,
                last_publication_handle: 0,
                current_count: 0,
                current_count_change: 0,
            }),
            requested_deadline_missed_status: DdsRwLock::new(RequestedDeadlineMissedStatus {
                total_count: 0,
                total_count_change: 0,
                last_instance_handle: 0,
            }),
        })
    }
}

fn read_sample<'a, T>(
    data_reader_attributes: &DataReaderAttributes<T>,
    cache_change: &'a RtpsCacheChangeImpl,
) -> (&'a [u8], SampleInfo) {
    *data_reader_attributes.status_change.write_lock() &= !DATA_AVAILABLE_STATUS;

    let mut samples_read = data_reader_attributes.samples_read.write_lock();
    let data_value = cache_change.data_value();

    let sample_state = {
        let sn = cache_change.sequence_number();
        if samples_read.contains(&sn) {
            READ_SAMPLE_STATE
        } else {
            samples_read.insert(sn);
            NOT_READ_SAMPLE_STATE
        }
    };

    let sample_info = SampleInfo {
        sample_state,
        view_state: NEW_VIEW_STATE,
        instance_state: ALIVE_INSTANCE_STATE,
        disposed_generation_count: 0,
        no_writers_generation_count: 0,
        sample_rank: 0,
        generation_rank: 0,
        absolute_generation_rank: 0,
        source_timestamp: Time { sec: 0, nanosec: 0 },
        instance_handle: 0,
        publication_handle: 0,
        valid_data: true,
    };

    (data_value, sample_info)
}

impl<T> DataReaderAttributes<T> {
    pub fn add_matched_participant(
        &self,
        participant_discovery: &ParticipantDiscovery<SpdpDiscoveredParticipantData>,
    ) {
        let mut rtps_reader_lock = self.rtps_reader.write_lock();
        if let RtpsReader::Stateful(rtps_reader) = &mut *rtps_reader_lock {
            if !rtps_reader
                .matched_writers()
                .into_iter()
                .any(|r| r.remote_writer_guid().prefix == participant_discovery.guid_prefix())
            {
                let type_name = self.topic.get_type_name().unwrap();
                if type_name == DiscoveredWriterData::type_name() {
                    participant_discovery
                        .discovered_participant_add_publications_reader(rtps_reader);
                } else if type_name == DiscoveredReaderData::type_name() {
                    participant_discovery
                        .discovered_participant_add_subscriptions_reader(rtps_reader);
                } else if type_name == DiscoveredTopicData::type_name() {
                    participant_discovery.discovered_participant_add_topics_reader(rtps_reader);
                }
            }
        }
    }
}

impl<T> ReceiveRtpsDataSubmessage for DdsShared<DataReaderAttributes<T>>
where
    T: Timer + Send + Sync + 'static,
{
    fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<Vec<Parameter>, &[u8]>,
        source_guid_prefix: GuidPrefix,
    ) {
        let before_data_cache_len;
        let after_data_cache_len;
        let mut rtps_reader = self.rtps_reader.write_lock();
        match &mut *rtps_reader {
            RtpsReader::Stateless(stateless_rtps_reader) => {
                before_data_cache_len = stateless_rtps_reader.reader_cache().changes().len();

                stateless_rtps_reader
                    .on_data_submessage_received(data_submessage, source_guid_prefix);

                after_data_cache_len = stateless_rtps_reader.reader_cache().changes().len();
            }
            RtpsReader::Stateful(stateful_rtps_reader) => {
                before_data_cache_len = stateful_rtps_reader.reader_cache().changes().len();

                stateful_rtps_reader
                    .on_data_submessage_received(data_submessage, source_guid_prefix);

                after_data_cache_len = stateful_rtps_reader.reader_cache().changes().len();
            }
        }
        // Call the listener after dropping the rtps_reader lock to avoid deadlock
        drop(rtps_reader);
        if before_data_cache_len < after_data_cache_len {
            DataReaderAttributes::on_data_received(self.clone()).unwrap();
        }
    }
}

impl<T> ReceiveRtpsHeartbeatSubmessage for DdsShared<DataReaderAttributes<T>> {
    fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        let mut rtps_reader = self.rtps_reader.write_lock();
        if let RtpsReader::Stateful(stateful_rtps_reader) = &mut *rtps_reader {
            stateful_rtps_reader
                .on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
        }
    }
}

impl<T> AddMatchedWriter for DdsShared<DataReaderAttributes<T>>
where
    T: Timer,
{
    fn add_matched_writer(&self, discovered_writer_data: &DiscoveredWriterData) {
        let topic_name = &discovered_writer_data
            .publication_builtin_topic_data
            .topic_name;
        let type_name = &discovered_writer_data
            .publication_builtin_topic_data
            .type_name;
        let reader_topic_name = &self.topic.get_name().unwrap();
        let reader_type_name = self.topic.get_type_name().unwrap();
        if topic_name == reader_topic_name && type_name == reader_type_name {
            let writer_proxy =
                <RtpsStatefulReaderImpl as RtpsStatefulReaderOperations>::WriterProxyType::new(
                    discovered_writer_data.writer_proxy.remote_writer_guid,
                    discovered_writer_data
                        .writer_proxy
                        .unicast_locator_list
                        .as_ref(),
                    discovered_writer_data
                        .writer_proxy
                        .multicast_locator_list
                        .as_ref(),
                    discovered_writer_data.writer_proxy.data_max_size_serialized,
                    discovered_writer_data.writer_proxy.remote_group_entity_id,
                );
            let mut rtps_reader = self.rtps_reader.write_lock();
            match &mut *rtps_reader {
                RtpsReader::Stateless(_) => (),
                RtpsReader::Stateful(rtps_stateful_reader) => {
                    rtps_stateful_reader.matched_writer_add(writer_proxy);
                    let mut status = self.subscription_matched_status.write_lock();
                    status.total_count += 1;
                    status.total_count_change += 1;
                    status.current_count += 1;
                    status.current_count_change += 1;

                    if let Some(l) = self.listener.write_lock().as_mut() {
                        *self.status_change.write_lock() &= !SUBSCRIPTION_MATCHED_STATUS;
                        l.trigger_on_subscription_matched(self.clone(), *status)
                    };

                    status.total_count_change = 0;
                    status.current_count_change = 0;
                }
            };
        }
    }
}

impl<T> DataReaderAttributes<T>
where
    T: Timer + Send + Sync + 'static,
{
    pub fn on_data_received(reader: DdsShared<Self>) -> DdsResult<()> {
        if reader.qos.read_lock().history.kind == HistoryQosPolicyKind::KeepLastHistoryQoS {
            let mut rtps_reader = reader.rtps_reader.write_lock();

            let cache_len = rtps_reader.reader_cache().changes().len() as i32;
            if cache_len > reader.qos.read_lock().history.depth {
                let mut seq_nums: Vec<_> = rtps_reader
                    .reader_cache()
                    .changes()
                    .iter()
                    .map(|c| c.sequence_number())
                    .collect();
                seq_nums.sort();

                let to_delete = &seq_nums
                    [0..(cache_len as usize - reader.qos.read_lock().history.depth as usize)];
                rtps_reader
                    .reader_cache()
                    .remove_change(|c| to_delete.contains(&c.sequence_number()));
            }
        }

        let reader_shared = reader.clone();
        reader.deadline_timer.write_lock().on_deadline(move || {
            reader_shared
                .requested_deadline_missed_status
                .write_lock()
                .total_count += 1;
            reader_shared
                .requested_deadline_missed_status
                .write_lock()
                .total_count_change += 1;

            *reader_shared.status_change.write_lock() |= REQUESTED_DEADLINE_MISSED_STATUS;
            reader_shared.listener.write_lock().as_mut().map(|l| {
                *reader_shared.status_change.write_lock() &= !REQUESTED_DEADLINE_MISSED_STATUS;
                l.trigger_on_requested_deadline_missed(
                    reader_shared.clone(),
                    reader_shared
                        .requested_deadline_missed_status
                        .read_lock()
                        .clone(),
                )
            });
        });

        *reader.status_change.write_lock() |= DATA_AVAILABLE_STATUS;
        if let Some(l) = reader.listener.write_lock().as_mut() {
            *reader.status_change.write_lock() &= !DATA_AVAILABLE_STATUS;
            l.trigger_on_data_available(reader.clone())
        };

        Ok(())
    }
}

impl<T> DataReaderGetSubscriber for DdsShared<DataReaderAttributes<T>>
where
    T: Timer,
{
    type Subscriber = DdsShared<SubscriberAttributes>;

    fn data_reader_get_subscriber(&self) -> DdsResult<Self::Subscriber> {
        Ok(self.parent_subscriber.upgrade()?.clone())
    }
}

impl<T> DataReaderGetTopicDescription for DdsShared<DataReaderAttributes<T>>
where
    T: Timer,
{
    type TopicDescription = DdsShared<TopicAttributes>;

    fn data_reader_get_topicdescription(&self) -> DdsResult<Self::TopicDescription> {
        Ok(self.topic.clone())
    }
}

impl<T, Foo> DataReader<Foo> for DdsShared<DataReaderAttributes<T>>
where
    T: Timer,
    Foo: for<'de> DdsDeserialize<'de>,
{
    fn read(
        &self,
        max_samples: i32,
        sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<Vec<(Foo, SampleInfo)>> {
        let mut rtps_reader = self.rtps_reader.write_lock();

        let samples = rtps_reader
            .reader_cache()
            .changes()
            .iter()
            .map(|sample| {
                let (mut data_value, sample_info) = read_sample(self, sample);
                let foo = DdsDeserialize::deserialize(&mut data_value)?;
                Ok((foo, sample_info))
            })
            .filter(|result| {
                if let Ok((_, info)) = result {
                    info.sample_state & sample_states != 0
                } else {
                    true
                }
            })
            .take(max_samples as usize)
            .collect::<DdsResult<Vec<_>>>()?;

        if samples.is_empty() {
            Err(DdsError::NoData)
        } else {
            Ok(samples)
        }
    }

    fn take(
        &self,
        _max_samples: i32,
        sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<Vec<(Foo, SampleInfo)>> {
        let mut rtps_reader = self.rtps_reader.write_lock();

        let (samples, to_delete): (Vec<_>, Vec<_>) = rtps_reader
            .reader_cache()
            .changes()
            .iter()
            .map(|sample| {
                let (mut data_value, sample_info) = read_sample(self, sample);
                let foo = DdsDeserialize::deserialize(&mut data_value)?;

                Ok(((foo, sample_info), sample.sequence_number()))
            })
            .filter(|result| {
                if let Ok(((_, info), _)) = result {
                    info.sample_state & sample_states != 0
                } else {
                    true
                }
            })
            .collect::<DdsResult<Vec<_>>>()?
            .into_iter()
            .unzip();

        rtps_reader
            .reader_cache()
            .remove_change(|x| to_delete.contains(&x.sequence_number()));

        Ok(samples)
    }

    fn read_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_condition: ReadCondition,
    ) -> DdsResult<()> {
        todo!()
    }

    fn take_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_condition: ReadCondition,
    ) -> DdsResult<()> {
        todo!()
    }

    fn read_next_sample(
        &self,
        _data_value: &mut [Foo],
        _sample_info: &mut [SampleInfo],
    ) -> DdsResult<()> {
        todo!()
    }

    fn take_next_sample(
        &self,
        _data_value: &mut [Foo],
        _sample_info: &mut [SampleInfo],
    ) -> DdsResult<()> {
        todo!()
    }

    fn read_instance(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_handle: InstanceHandle,
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        todo!()
    }

    fn take_instance(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_handle: InstanceHandle,
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        todo!()
    }

    fn read_next_instance(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        todo!()
    }

    fn take_next_instance(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        todo!()
    }

    fn read_next_instance_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> DdsResult<()> {
        todo!()
    }

    fn take_next_instance_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> DdsResult<()> {
        todo!()
    }

    fn return_loan(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut Foo, _handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &Foo) -> DdsResult<InstanceHandle> {
        todo!()
    }

    fn create_readcondition(
        &self,
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<ReadCondition> {
        todo!()
    }

    fn create_querycondition(
        &self,
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
        _query_expression: &'static str,
        _query_parameters: &[&'static str],
    ) -> DdsResult<QueryCondition> {
        todo!()
    }

    fn delete_readcondition(&self, _a_condition: ReadCondition) -> DdsResult<()> {
        todo!()
    }

    fn get_liveliness_changed_status(
        &self,
        _status: &mut LivelinessChangedStatus,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_requested_deadline_missed_status(&self) -> DdsResult<RequestedDeadlineMissedStatus> {
        let status = self.requested_deadline_missed_status.read_lock().clone();

        self.requested_deadline_missed_status
            .write_lock()
            .total_count_change = 0;

        Ok(status)
    }

    fn get_requested_incompatible_qos_status(
        &self,
        _status: &mut RequestedIncompatibleQosStatus,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> DdsResult<()> {
        todo!()
    }

    fn get_sample_rejected_status(&self, _status: &mut SampleRejectedStatus) -> DdsResult<()> {
        todo!()
    }

    fn get_subscription_matched_status(&self) -> DdsResult<SubscriptionMatchedStatus> {
        Ok(*self.subscription_matched_status.read_lock())
    }

    fn delete_contained_entities(&self) -> DdsResult<()> {
        todo!()
    }

    fn wait_for_historical_data(&self) -> DdsResult<()> {
        todo!()
    }

    fn get_matched_publication_data(
        &self,
        _publication_data: &mut PublicationBuiltinTopicData,
        _publication_handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_matched_publications(&self) -> DdsResult<Vec<InstanceHandle>> {
        let mut rtps_reader_lock = self.rtps_reader.write_lock();
        let matched_publications = match &mut *rtps_reader_lock {
            RtpsReader::Stateless(_) => vec![],
            RtpsReader::Stateful(r) => r
                .matched_writers()
                .into_iter()
                .enumerate()
                .map(|x| x.0 as i32)
                .collect(),
        };

        Ok(matched_publications)
    }
}

impl<T> Entity for DdsShared<DataReaderAttributes<T>> {
    type Qos = DataReaderQos;
    type Listener = Box<dyn AnyDataReaderListener<Self> + Send + Sync>;

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
        Ok(*self.status_change.read_lock())
    }

    fn enable(&self) -> DdsResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        todo!()
    }
}

impl<T> SendRtpsMessage for DdsShared<DataReaderAttributes<T>> {
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
        if let RtpsReader::Stateful(stateful_rtps_reader) = &mut *self.rtps_reader.write_lock() {
            let mut acknacks = Vec::new();
            stateful_rtps_reader.send_submessages(|wp, acknack| {
                acknacks.push((
                    wp.unicast_locator_list().to_vec(),
                    vec![RtpsSubmessageType::AckNack(acknack)],
                ))
            });

            for (locator_list, acknacks) in acknacks {
                let header = RtpsMessageHeader {
                    protocol: rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
                    version: PROTOCOLVERSION,
                    vendor_id: VENDOR_ID_S2E,
                    guid_prefix: stateful_rtps_reader.guid().prefix(),
                };

                let message = RtpsMessage {
                    header,
                    submessages: acknacks,
                };

                for &locator in &locator_list {
                    transport.write(&message, locator);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dds_impl::{data_reader_attributes::RtpsReader, topic_attributes::TopicAttributes},
        dds_type::{DdsSerialize, DdsType, Endianness},
        utils::shared_object::DdsShared,
    };
    use dds_api::{
        dcps_psm::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        infrastructure::qos_policy::{DeadlineQosPolicy, HistoryQosPolicy},
        return_type::DdsResult,
    };
    use mockall::mock;
    use rtps_pim::{
        behavior::{reader::stateful_reader::RtpsStatefulReaderConstructor, types::DURATION_ZERO},
        structure::{
            cache_change::RtpsCacheChangeConstructor, history_cache::RtpsHistoryCacheConstructor,
            types::GUID_UNKNOWN,
        },
    };
    use std::{io::Write, time::Duration};

    pub struct ManualTimer {
        on_deadline: Option<Box<dyn FnMut() + Send + Sync>>,
    }

    impl ManualTimer {
        pub fn trigger(&mut self) {
            if let Some(f) = &mut self.on_deadline {
                f()
            }
            self.on_deadline = None;
        }
    }

    impl Timer for ManualTimer {
        fn new(_duration: Duration) -> Self {
            ManualTimer { on_deadline: None }
        }

        fn reset(&mut self) {
            self.on_deadline = None;
        }

        fn on_deadline<F>(&mut self, f: F)
        where
            F: FnMut() + Send + Sync + 'static,
        {
            self.on_deadline = Some(Box::new(f));
        }
    }

    struct UserData(u8);

    impl DdsType for UserData {
        fn type_name() -> &'static str {
            "UserData"
        }

        fn has_key() -> bool {
            false
        }
    }

    impl<'de> DdsDeserialize<'de> for UserData {
        fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
            Ok(UserData(buf[0]))
        }
    }

    impl DdsSerialize for UserData {
        fn serialize<W: Write, E: Endianness>(&self, mut writer: W) -> DdsResult<()> {
            writer
                .write(&[self.0])
                .map(|_| ())
                .map_err(|e| DdsError::PreconditionNotMet(format!("{}", e)))
        }
    }

    fn cache_change(value: u8, sn: SequenceNumber) -> RtpsCacheChangeImpl {
        let cache_change = RtpsCacheChangeImpl::new(
            rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            sn,
            vec![value],
            vec![],
        );

        cache_change
    }

    fn reader_with_changes<T: Timer>(
        changes: Vec<RtpsCacheChangeImpl>,
    ) -> DdsShared<DataReaderAttributes<T>> {
        let mut stateful_reader = RtpsStatefulReaderImpl::new(
            GUID_UNKNOWN,
            rtps_pim::structure::types::TopicKind::NoKey,
            rtps_pim::structure::types::ReliabilityKind::BestEffort,
            &[],
            &[],
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        );
        for change in changes {
            stateful_reader.reader_cache().add_change(change);
        }

        DataReaderConstructor::new(
            DataReaderQos {
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepAllHistoryQos,
                    depth: 0,
                },
                ..Default::default()
            },
            RtpsReader::Stateful(stateful_reader),
            TopicAttributes::new(
                Default::default(),
                "type_name",
                "topic_name",
                DdsWeak::new(),
            ),
            None,
            DdsWeak::new(),
        )
    }

    #[test]
    fn read_all_samples() {
        let reader = DdsShared::new(reader_with_changes::<ManualTimer>(vec![
            cache_change(1, 1),
            cache_change(0, 2),
            cache_change(2, 3),
            cache_change(5, 4),
        ]));

        let all_samples: Vec<(UserData, _)> = reader
            .read(
                i32::MAX,
                ANY_SAMPLE_STATE,
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
            )
            .unwrap();
        assert_eq!(4, all_samples.len());
        assert_eq!(
            vec![1, 0, 2, 5],
            all_samples.into_iter().map(|s| s.0 .0).collect::<Vec<_>>()
        );
    }

    #[test]
    fn read_only_unread() {
        let reader = reader_with_changes::<ManualTimer>(vec![cache_change(1, 1)]);

        let unread_samples = DataReader::<UserData>::read(
            &reader,
            i32::MAX,
            NOT_READ_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .unwrap();

        assert_eq!(1, unread_samples.len());

        assert!(DataReader::<UserData>::read(
            &reader,
            i32::MAX,
            NOT_READ_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .is_err());
    }

    #[test]
    fn on_missed_deadline_increases_total_count() {
        let reader = {
            let reader = reader_with_changes::<ManualTimer>(vec![]);
            *reader.qos.write_lock() = DataReaderQos {
                deadline: DeadlineQosPolicy {
                    period: dds_api::dcps_psm::Duration::new(1, 0),
                },
                ..Default::default()
            };
            reader
        };

        assert_eq!(
            0,
            DataReader::<UserData>::get_requested_deadline_missed_status(&reader)
                .unwrap()
                .total_count
        );

        DataReaderAttributes::on_data_received(reader.clone()).unwrap();

        assert_eq!(
            0,
            DataReader::<UserData>::get_requested_deadline_missed_status(&reader)
                .unwrap()
                .total_count
        );

        reader.deadline_timer.write_lock().trigger();

        assert_eq!(
            1,
            DataReader::<UserData>::get_requested_deadline_missed_status(&reader)
                .unwrap()
                .total_count
        );
    }

    mock! {
        Listener {}
        impl AnyDataReaderListener<DdsShared<DataReaderAttributes<ManualTimer>>> for Listener {
            fn trigger_on_data_available(&mut self, reader: DdsShared<DataReaderAttributes<ManualTimer>>);
            fn trigger_on_sample_rejected(
                &mut self,
                reader: DdsShared<DataReaderAttributes<ManualTimer>>,
                status: SampleRejectedStatus,
            );
            fn trigger_on_liveliness_changed(
                &mut self,
                reader: DdsShared<DataReaderAttributes<ManualTimer>>,
                status: LivelinessChangedStatus,
            );
            fn trigger_on_requested_deadline_missed(
                &mut self,
                reader: DdsShared<DataReaderAttributes<ManualTimer>>,
                status: RequestedDeadlineMissedStatus,
            );
            fn trigger_on_requested_incompatible_qos(
                &mut self,
                reader: DdsShared<DataReaderAttributes<ManualTimer>>,
                status: RequestedIncompatibleQosStatus,
            );
            fn trigger_on_subscription_matched(
                &mut self,
                reader: DdsShared<DataReaderAttributes<ManualTimer>>,
                status: SubscriptionMatchedStatus,
            );
            fn trigger_on_sample_lost(
                &mut self,
                reader: DdsShared<DataReaderAttributes<ManualTimer>>,
                status: SampleLostStatus,
            );
        }
    }

    #[test]
    fn on_deadline_missed_calls_listener() {
        let reader = {
            let reader = reader_with_changes::<ManualTimer>(vec![]);
            *reader.qos.write_lock() = DataReaderQos {
                deadline: DeadlineQosPolicy {
                    period: dds_api::dcps_psm::Duration::new(1, 0),
                },
                ..Default::default()
            };
            reader
        };

        DataReaderAttributes::on_data_received(reader.clone()).unwrap();

        let mut listener = MockListener::new();
        listener
            .expect_trigger_on_requested_deadline_missed()
            .once()
            .return_const(());
        reader.set_listener(Some(Box::new(listener)), 0).unwrap();

        reader.deadline_timer.write_lock().trigger();
    }

    #[test]
    fn receiving_data_triggers_status_change() {
        let reader = {
            let reader = reader_with_changes::<ManualTimer>(vec![]);
            *reader.qos.write_lock() = DataReaderQos {
                deadline: DeadlineQosPolicy {
                    period: dds_api::dcps_psm::Duration::new(1, 0),
                },
                ..Default::default()
            };
            reader
        };

        DataReaderAttributes::on_data_received(reader.clone()).unwrap();

        assert!(reader.get_status_changes().unwrap() & DATA_AVAILABLE_STATUS > 0);
    }

    #[test]
    fn on_data_available_listener_resets_status_change() {
        let reader = {
            let reader = reader_with_changes::<ManualTimer>(vec![]);
            *reader.qos.write_lock() = DataReaderQos {
                deadline: DeadlineQosPolicy {
                    period: dds_api::dcps_psm::Duration::new(1, 0),
                },
                ..Default::default()
            };
            reader
        };

        let listener = {
            let mut listener = MockListener::new();
            listener
                .expect_trigger_on_data_available()
                .once()
                .return_const(());
            listener
        };
        reader.set_listener(Some(Box::new(listener)), 0).unwrap();

        DataReaderAttributes::on_data_received(reader.clone()).unwrap();

        assert_eq!(
            0,
            reader.get_status_changes().unwrap() & DATA_AVAILABLE_STATUS
        );
    }

    #[test]
    fn deadline_missed_triggers_status_change() {
        let reader = {
            let reader = reader_with_changes::<ManualTimer>(vec![]);
            *reader.qos.write_lock() = DataReaderQos {
                deadline: DeadlineQosPolicy {
                    period: dds_api::dcps_psm::Duration::new(1, 0),
                },
                ..Default::default()
            };
            reader
        };

        DataReaderAttributes::on_data_received(reader.clone()).unwrap();
        reader.deadline_timer.write_lock().trigger();

        assert!(reader.get_status_changes().unwrap() & REQUESTED_DEADLINE_MISSED_STATUS > 0);
    }

    #[test]
    fn on_deadline_missed_listener_resets_status_changed() {
        let reader = {
            let reader = reader_with_changes::<ManualTimer>(vec![]);
            *reader.qos.write_lock() = DataReaderQos {
                deadline: DeadlineQosPolicy {
                    period: dds_api::dcps_psm::Duration::new(1, 0),
                },
                ..Default::default()
            };
            reader
        };

        let listener = {
            let mut listener = Box::new(MockListener::new());
            listener
                .expect_trigger_on_requested_deadline_missed()
                .once()
                .return_const(());
            listener
                .expect_trigger_on_data_available()
                .once()
                .return_const(());
            listener
        };

        reader.set_listener(Some(listener), 0).unwrap();

        DataReaderAttributes::on_data_received(reader.clone()).unwrap();
        reader.deadline_timer.write_lock().trigger();

        assert_eq!(
            0,
            reader.get_status_changes().unwrap() & REQUESTED_DEADLINE_MISSED_STATUS
        );
    }

    fn reader_with_max_depth<T: Timer>(
        max_depth: i32,
        changes: Vec<RtpsCacheChangeImpl>,
    ) -> DdsShared<DataReaderAttributes<T>> {
        let mut history_cache = RtpsHistoryCacheImpl::new();
        for change in changes {
            history_cache.add_change(change);
        }

        let stateful_reader = RtpsStatefulReaderImpl::new(
            GUID_UNKNOWN,
            rtps_pim::structure::types::TopicKind::NoKey,
            rtps_pim::structure::types::ReliabilityKind::BestEffort,
            &[],
            &[],
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        );

        DataReaderConstructor::new(
            DataReaderQos {
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepLastHistoryQoS,
                    depth: max_depth,
                },
                ..Default::default()
            },
            RtpsReader::Stateful(stateful_reader),
            TopicAttributes::new(
                Default::default(),
                "type_name",
                "topic_name",
                DdsWeak::new(),
            ),
            None,
            DdsWeak::new(),
        )
    }

    #[test]
    fn keep_last_qos() {
        let reader = {
            let reader = reader_with_max_depth::<ManualTimer>(
                2,
                vec![
                    cache_change(1, 1),
                    cache_change(2, 2),
                    cache_change(3, 3),
                    cache_change(4, 4),
                ],
            );

            reader
        };

        DataReaderAttributes::on_data_received(reader.clone()).unwrap();
    }
}
