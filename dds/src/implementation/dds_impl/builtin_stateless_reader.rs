use std::sync::mpsc::SyncSender;

use crate::{
    implementation::{
        rtps::{
            endpoint::RtpsEndpoint, reader::RtpsReader, reader_cache_change::RtpsReaderCacheChange,
            types::TopicKind,
        },
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
            stateless_reader::RtpsStatelessReader,
            types::{EntityId, Guid, ENTITYID_UNKNOWN},
        },
        utils::{
            shared_object::{DdsRwLock, DdsShared},
            timer::Timer,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::HANDLE_NIL,
        status::{RequestedDeadlineMissedStatus, StatusKind},
    },
    subscription::sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    topic_definition::type_support::DdsDeserialize,
};

use super::{
    message_receiver::MessageReceiver, status_condition_impl::StatusConditionImpl,
    topic_impl::TopicImpl,
};

pub struct BuiltinStatelessReader<Tim> {
    rtps_reader: DdsRwLock<RtpsStatelessReader>,
    _topic: DdsShared<TopicImpl>,
    deadline_timer: DdsRwLock<Tim>,
    requested_deadline_missed_status: DdsRwLock<RequestedDeadlineMissedStatus>,
    enabled: DdsRwLock<bool>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
}

impl<Tim> BuiltinStatelessReader<Tim>
where
    Tim: Timer,
{
    pub fn new<Foo>(
        guid: Guid,
        topic: DdsShared<TopicImpl>,
        notifications_sender: SyncSender<(Guid, StatusKind)>,
    ) -> DdsShared<Self>
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
            notifications_sender,
        );
        let rtps_reader = RtpsStatelessReader::new(reader);
        let qos = rtps_reader.reader().get_qos();
        let deadline_duration = std::time::Duration::from_secs(qos.deadline.period.sec() as u64)
            + std::time::Duration::from_nanos(qos.deadline.period.nanosec() as u64);

        DdsShared::new(BuiltinStatelessReader {
            rtps_reader: DdsRwLock::new(rtps_reader),
            _topic: topic,
            deadline_timer: DdsRwLock::new(Tim::new(deadline_duration)),

            requested_deadline_missed_status: DdsRwLock::new(RequestedDeadlineMissedStatus {
                total_count: 0,
                total_count_change: 0,
                last_instance_handle: HANDLE_NIL,
            }),

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
            let a_change = match RtpsReaderCacheChange::try_from_data_submessage(
                data_submessage,
                Some(message_receiver.timestamp()),
                message_receiver.source_guid_prefix(),
            ) {
                Ok(a_change) => a_change,
                Err(_) => return,
            };

            rtps_reader.reader_mut().add_change(a_change).ok();
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
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader.write_lock().reader_mut().take(
            max_samples,
            sample_states,
            view_states,
            instance_states,
        )
    }
}

impl<Tim> DdsShared<BuiltinStatelessReader<Tim>> {
    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        Ok(())
    }
}
