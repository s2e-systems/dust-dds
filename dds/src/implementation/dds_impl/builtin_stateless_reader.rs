use crate::{
    implementation::rtps::{endpoint::RtpsEndpoint, reader::RtpsReader, types::TopicKind},
    infrastructure::qos::DataReaderQos,
    infrastructure::{
        instance::InstanceHandle,
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
        utils::shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::error::{DdsError, DdsResult},
    subscription::sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    topic_definition::type_support::DdsDeserialize,
};

use super::{message_receiver::MessageReceiver, topic_impl::TopicImpl};

pub struct BuiltinStatelessReader {
    rtps_reader: DdsRwLock<RtpsStatelessReader>,
    _topic: DdsShared<TopicImpl>,
    enabled: DdsRwLock<bool>,
}

impl BuiltinStatelessReader {
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

        DdsShared::new(BuiltinStatelessReader {
            rtps_reader: DdsRwLock::new(rtps_reader),
            _topic: topic,
            enabled: DdsRwLock::new(false),
        })
    }
}

impl DdsShared<BuiltinStatelessReader> {
    pub fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) {
        let mut rtps_reader = self.rtps_reader.write_lock();

        let data_reader_id: EntityId = data_submessage.reader_id;
        if data_reader_id == ENTITYID_UNKNOWN
            || data_reader_id == rtps_reader.reader().guid().entity_id()
        {
            rtps_reader
                .reader_mut()
                .add_change(
                    data_submessage,
                    Some(message_receiver.timestamp()),
                    message_receiver.source_guid_prefix(),
                    message_receiver.reception_timestamp(),
                )
                .ok();
        }
    }
}

impl DdsShared<BuiltinStatelessReader> {
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
            None,
        )
    }
}

impl DdsShared<BuiltinStatelessReader> {
    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        Ok(())
    }
}
