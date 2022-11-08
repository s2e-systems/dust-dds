use crate::{
    implementation::{
        rtps::{endpoint::RtpsEndpoint, reader::RtpsReader, types::TopicKind},
        utils::timer::ThreadTimer,
    },
    infrastructure::qos::DataReaderQos,
    infrastructure::{
        qos_policy::{HistoryQosPolicy, HistoryQosPolicyKind},
        time::DURATION_ZERO,
    },
    subscription::data_reader::Sample,
    topic_definition::type_support::DdsType,
};
use crate::{
    implementation::{
        rtps::{
            messages::submessages::DataSubmessage,
            reader_cache_change::RtpsReaderCacheChange,
            stateless_reader::RtpsStatelessReader,
            types::{ChangeKind, EntityId, Guid, SequenceNumber, ENTITYID_UNKNOWN},
        },
        utils::{
            shared_object::{DdsRwLock, DdsShared},
            timer::Timer,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::{InstanceHandle, HANDLE_NIL},
        qos_policy::DestinationOrderQosPolicyKind,
        status::{RequestedDeadlineMissedStatus, StatusKind},
    },
    subscription::sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
    topic_definition::type_support::DdsDeserialize,
};
use std::collections::HashSet;

use super::{
    message_receiver::MessageReceiver, status_condition_impl::StatusConditionImpl,
    topic_impl::TopicImpl,
};

pub struct BuiltinStatelessReader<Tim> {
    rtps_reader: DdsRwLock<RtpsStatelessReader>,
    _topic: DdsShared<TopicImpl>,
    samples_read: DdsRwLock<HashSet<SequenceNumber>>,
    deadline_timer: DdsRwLock<Tim>,
    requested_deadline_missed_status: DdsRwLock<RequestedDeadlineMissedStatus>,
    samples_viewed: DdsRwLock<HashSet<InstanceHandle>>,
    enabled: DdsRwLock<bool>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
}

impl<Tim> BuiltinStatelessReader<Tim>
where
    Tim: Timer,
{
    pub fn new<Foo>(guid: Guid, topic: DdsShared<TopicImpl>) -> DdsShared<Self>
    where
        Foo: DdsType + for<'de> DdsDeserialize<'de>,
    {
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];

        let reader = RtpsReader::new::<Foo>(
            RtpsEndpoint::new(
                guid,
                TopicKind::WithKey,
                unicast_locator_list,
                multicast_locator_list,
            ),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
            DataReaderQos {
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepAll,
                    depth: 0,
                },
                ..Default::default()
            },
        );
        let rtps_reader = RtpsStatelessReader::new(reader);
        let qos = rtps_reader.reader().get_qos();
        let deadline_duration = std::time::Duration::from_secs(qos.deadline.period.sec() as u64)
            + std::time::Duration::from_nanos(qos.deadline.period.nanosec() as u64);

        DdsShared::new(BuiltinStatelessReader {
            rtps_reader: DdsRwLock::new(rtps_reader),
            _topic: topic,
            samples_read: DdsRwLock::new(HashSet::new()),
            deadline_timer: DdsRwLock::new(Tim::new(deadline_duration)),

            requested_deadline_missed_status: DdsRwLock::new(RequestedDeadlineMissedStatus {
                total_count: 0,
                total_count_change: 0,
                last_instance_handle: HANDLE_NIL,
            }),

            samples_viewed: DdsRwLock::new(HashSet::new()),
            enabled: DdsRwLock::new(false),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
        })
    }
}

impl DdsShared<BuiltinStatelessReader<ThreadTimer>> {
    pub fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) {
        let mut rtps_reader = self.rtps_reader.write_lock();

        let before_data_cache_len = rtps_reader.reader_mut().changes().len();

        let data_reader_id: EntityId = data_submessage.reader_id.value.into();
        if data_reader_id == ENTITYID_UNKNOWN
            || data_reader_id == rtps_reader.reader().guid().entity_id()
        {
            rtps_reader
                .reader_mut()
                .on_data_submessage_received(data_submessage, message_receiver);
        }

        let after_data_cache_len = rtps_reader.reader_mut().changes().len();

        // Call the listener after dropping the rtps_reader lock to avoid deadlock
        drop(rtps_reader);
        if before_data_cache_len < after_data_cache_len {
            let reader_shared = self.clone();
            self.deadline_timer.write_lock().on_deadline(move || {
                reader_shared
                    .requested_deadline_missed_status
                    .write_lock()
                    .total_count += 1;
                reader_shared
                    .requested_deadline_missed_status
                    .write_lock()
                    .total_count_change += 1;

                reader_shared
                    .status_condition
                    .write_lock()
                    .add_communication_state(StatusKind::RequestedDeadlineMissed);
            });

            self.status_condition
                .write_lock()
                .add_communication_state(StatusKind::DataAvailable);
        }
    }
}

impl<Tim> DdsShared<BuiltinStatelessReader<Tim>>
where
    Tim: Timer,
{
    pub fn take<Foo>(
        &self,
        _max_samples: i32,
        sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut rtps_reader = self.rtps_reader.write_lock();

        let (mut samples, to_delete): (Vec<_>, Vec<_>) = rtps_reader
            .reader_mut()
            .changes()
            .iter()
            .map(|cache_change| match cache_change.kind() {
                ChangeKind::Alive => {
                    let (mut data_value, sample_info) = self.read_sample(cache_change);
                    let value = DdsDeserialize::deserialize(&mut data_value)?;
                    let sample = Sample {
                        data: Some(value),
                        sample_info,
                    };
                    Ok((sample, cache_change.sequence_number()))
                }
                ChangeKind::AliveFiltered => todo!(),
                ChangeKind::NotAliveDisposed => {
                    let (_, sample_info) = self.read_sample(cache_change);
                    let sample = Sample {
                        data: None,
                        sample_info,
                    };
                    Ok((sample, cache_change.sequence_number()))
                }
                ChangeKind::NotAliveUnregistered => todo!(),
            })
            .filter(|result| {
                if let Ok((sample, _)) = result {
                    sample_states.contains(&sample.sample_info.sample_state)
                } else {
                    true
                }
            })
            .collect::<DdsResult<Vec<_>>>()?
            .into_iter()
            .unzip();

        rtps_reader
            .reader_mut()
            .remove_change(|x| to_delete.contains(&x.sequence_number()));

        if rtps_reader.reader().get_qos().destination_order.kind
            == DestinationOrderQosPolicyKind::BySourceTimestamp
        {
            samples.sort_by(|a, b| {
                a.sample_info
                    .source_timestamp
                    .as_ref()
                    .unwrap()
                    .cmp(b.sample_info.source_timestamp.as_ref().unwrap())
            });
        }

        for sample in samples.iter() {
            self.samples_viewed
                .write_lock()
                .insert(sample.sample_info.instance_handle);
        }

        Ok(samples)
    }

    fn read_sample<'a>(&self, cache_change: &'a RtpsReaderCacheChange) -> (&'a [u8], SampleInfo) {
        self.status_condition
            .write_lock()
            .remove_communication_state(StatusKind::DataAvailable);

        let mut samples_read = self.samples_read.write_lock();
        let data_value = cache_change.data_value();

        let sample_state = {
            let sn = cache_change.sequence_number();
            if samples_read.contains(&sn) {
                SampleStateKind::Read
            } else {
                samples_read.insert(sn);
                SampleStateKind::NotRead
            }
        };

        let (instance_state, valid_data) = match cache_change.kind() {
            ChangeKind::Alive => (InstanceStateKind::Alive, true),
            ChangeKind::NotAliveDisposed => (InstanceStateKind::NotAliveDisposed, false),
            _ => unimplemented!(),
        };

        let view_state = if self
            .samples_viewed
            .read_lock()
            .contains(&cache_change.instance_handle())
        {
            ViewStateKind::NotNew
        } else {
            ViewStateKind::New
        };

        let sample_info = SampleInfo {
            sample_state,
            view_state,
            instance_state,
            disposed_generation_count: 0,
            no_writers_generation_count: 0,
            sample_rank: 0,
            generation_rank: 0,
            absolute_generation_rank: 0,
            source_timestamp: *cache_change.source_timestamp(),
            instance_handle: cache_change.instance_handle(),
            publication_handle: <[u8; 16]>::from(cache_change.writer_guid()).into(),
            valid_data,
        };

        (data_value, sample_info)
    }
}

impl<Tim> DdsShared<BuiltinStatelessReader<Tim>> {
    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        Ok(())
    }
}
