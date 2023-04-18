use crate::{
    implementation::dds_impl::message_receiver::MessageReceiver,
    infrastructure::{
        error::DdsResult, instance::InstanceHandle, qos::DataReaderQos,
        qos_policy::ReliabilityQosPolicyKind, status::SampleRejectedStatusKind, time::Time,
    },
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::DdsDeserialize,
};

use super::{
    messages::submessages::DataSubmessage,
    reader::{RtpsReader, RtpsReaderCacheChange, RtpsReaderError, RtpsReaderResult},
    types::{Guid, GuidPrefix, ENTITYID_UNKNOWN},
};

pub struct RtpsStatelessReader(RtpsReader);

pub enum StatelessReaderDataReceivedResult {
    NotForThisReader,
    NewSampleAdded(InstanceHandle),
    SampleRejected(InstanceHandle, SampleRejectedStatusKind),
    InvalidData(&'static str),
}

impl RtpsStatelessReader {
    pub fn new(reader: RtpsReader) -> Self {
        if reader.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            panic!("Reliable stateless reader is not supported");
        }

        Self(reader)
    }

    pub fn guid(&self) -> Guid {
        self.0.guid()
    }

    pub fn _convert_data_to_cache_change(
        &self,
        data_submessage: &DataSubmessage,
        source_timestamp: Option<Time>,
        source_guid_prefix: GuidPrefix,
        reception_timestamp: Time,
    ) -> RtpsReaderResult<RtpsReaderCacheChange> {
        self.0.convert_data_to_cache_change(
            data_submessage,
            source_timestamp,
            source_guid_prefix,
            reception_timestamp,
        )
    }

    pub fn _add_change(
        &mut self,
        change: RtpsReaderCacheChange,
    ) -> RtpsReaderResult<InstanceHandle> {
        self.0.add_change(change)
    }

    pub fn get_qos(&self) -> &DataReaderQos {
        self.0.get_qos()
    }

    pub fn _set_qos(&mut self, qos: DataReaderQos) -> DdsResult<()> {
        self.0.set_qos(qos)
    }

    pub fn read<Foo>(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.0.read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn _take<Foo>(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.0.take(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn read_next_instance<Foo>(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.0.read_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn _take_next_instance<Foo>(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.0.take_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) -> StatelessReaderDataReceivedResult {
        if data_submessage.reader_id == ENTITYID_UNKNOWN
            || data_submessage.reader_id == self.0.guid().entity_id()
        {
            let change_result = self.0.convert_data_to_cache_change(
                data_submessage,
                Some(message_receiver.timestamp()),
                message_receiver.source_guid_prefix(),
                message_receiver.reception_timestamp(),
            );
            match change_result {
                Ok(change) => {
                    let add_change_result = self.0.add_change(change);

                    match add_change_result {
                        Ok(h) => StatelessReaderDataReceivedResult::NewSampleAdded(h),
                        Err(e) => match e {
                            RtpsReaderError::InvalidData(s) => {
                                StatelessReaderDataReceivedResult::InvalidData(s)
                            }
                            RtpsReaderError::Rejected(h, k) => {
                                StatelessReaderDataReceivedResult::SampleRejected(h, k)
                            }
                        },
                    }
                }
                Err(_) => StatelessReaderDataReceivedResult::InvalidData("Invalid data submessage"), // Change is ignored,
            }
        } else {
            StatelessReaderDataReceivedResult::NotForThisReader
        }
    }
}
