use crate::{
    implementation::rtps::{
        endpoint::RtpsEndpoint, reader::RtpsReader,
        stateless_reader::StatelessReaderDataReceivedResult, types::TopicKind,
    },
    infrastructure::qos::DataReaderQos,
    infrastructure::{
        instance::InstanceHandle,
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::StatusKind,
        time::DURATION_ZERO,
    },
    subscription::data_reader::Sample,
    topic_definition::type_support::DdsType,
};
use crate::{
    implementation::{
        rtps::{
            messages::submessages::DataSubmessage, stateless_reader::RtpsStatelessReader,
            types::Guid,
        },
        utils::shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::error::{DdsError, DdsResult},
    subscription::sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    topic_definition::type_support::DdsDeserialize,
};

use super::{
    message_receiver::MessageReceiver, status_condition_impl::StatusConditionImpl,
    topic_impl::TopicImpl,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BuiltInStatelessReaderDataSubmessageReceivedResult {
    NoChange,
    NewDataAvailable,
}

pub struct BuiltinStatelessReader {
    rtps_reader: DdsRwLock<RtpsStatelessReader>,
    _topic: DdsShared<TopicImpl>,
    enabled: DdsRwLock<bool>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
}

impl BuiltinStatelessReader {
    pub fn new<Foo>(guid: Guid, topic: DdsShared<TopicImpl>) -> DdsShared<Self>
    where
        Foo: DdsType + for<'de> DdsDeserialize<'de>,
    {
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let qos = DataReaderQos {
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast,
                depth: 1,
            },
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: DURATION_ZERO,
            },
            ..Default::default()
        };
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
            qos,
        );
        let rtps_reader = RtpsStatelessReader::new(reader);

        DdsShared::new(BuiltinStatelessReader {
            rtps_reader: DdsRwLock::new(rtps_reader),
            _topic: topic,
            enabled: DdsRwLock::new(false),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
        })
    }
}

impl DdsShared<BuiltinStatelessReader> {
    pub fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) -> BuiltInStatelessReaderDataSubmessageReceivedResult {
        let r = self
            .rtps_reader
            .write_lock()
            .on_data_submessage_received(data_submessage, message_receiver);

        match r {
            StatelessReaderDataReceivedResult::NotForThisReader
            | StatelessReaderDataReceivedResult::SampleRejected(_, _)
            | StatelessReaderDataReceivedResult::InvalidData(_) => {
                BuiltInStatelessReaderDataSubmessageReceivedResult::NoChange
            }
            StatelessReaderDataReceivedResult::NewSampleAdded(_) => {
                BuiltInStatelessReaderDataSubmessageReceivedResult::NewDataAvailable
            }
        }
    }

    pub fn read<Foo>(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader.write_lock().reader_mut().read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn read_next_instance<Foo>(
        &self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
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

        self.rtps_reader
            .write_lock()
            .reader_mut()
            .read_next_instance(
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            )
    }

    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        Ok(())
    }

    pub fn get_qos(&self) -> DataReaderQos {
        self.rtps_reader.read_lock().reader().get_qos().clone()
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_reader.read_lock().reader().guid().into()
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn on_data_available(&self) {
        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::DataAvailable);
    }
}
