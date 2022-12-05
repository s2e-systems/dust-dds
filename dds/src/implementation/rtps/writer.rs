use std::collections::HashMap;

use crate::{
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataWriterQos,
        qos_policy::LENGTH_UNLIMITED,
        time::{Duration, Time},
    },
    topic_definition::type_support::{DdsSerialize, DdsType, LittleEndian},
};

use super::{
    endpoint::RtpsEndpoint,
    history_cache::{RtpsParameter, RtpsWriterCacheChange, WriterHistoryCache},
    types::{ChangeKind, Guid, Locator, SequenceNumber},
};

pub struct RtpsWriter {
    endpoint: RtpsEndpoint,
    push_mode: bool,
    heartbeat_period: Duration,
    _nack_response_delay: Duration,
    _nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    _data_max_size_serialized: Option<i32>,
    writer_cache: WriterHistoryCache,
    qos: DataWriterQos,
    registered_instance_list: HashMap<InstanceHandle, Vec<u8>>,
}

impl RtpsWriter {
    pub fn new(
        endpoint: RtpsEndpoint,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
        qos: DataWriterQos,
    ) -> Self {
        Self {
            endpoint,
            push_mode,
            heartbeat_period,
            _nack_response_delay: nack_response_delay,
            _nack_suppression_duration: nack_suppression_duration,
            last_change_sequence_number: 0,
            _data_max_size_serialized: data_max_size_serialized,
            writer_cache: WriterHistoryCache::new(),
            qos,
            registered_instance_list: HashMap::new(),
        }
    }

    pub fn guid(&self) -> Guid {
        self.endpoint.guid()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.endpoint.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.endpoint.multicast_locator_list()
    }

    pub fn push_mode(&self) -> bool {
        self.push_mode
    }

    pub fn heartbeat_period(&self) -> Duration {
        self.heartbeat_period
    }

    pub fn writer_cache(&self) -> &WriterHistoryCache {
        &self.writer_cache
    }

    pub fn writer_cache_mut(&mut self) -> &mut WriterHistoryCache {
        &mut self.writer_cache
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: Vec<RtpsParameter>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> RtpsWriterCacheChange {
        self.last_change_sequence_number += 1;
        RtpsWriterCacheChange::new(
            kind,
            self.guid(),
            handle,
            self.last_change_sequence_number,
            timestamp,
            data,
            inline_qos,
        )
    }

    pub fn get_qos(&self) -> &DataWriterQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: DataWriterQos) -> DdsResult<()> {
        qos.is_consistent()?;
        self.qos = qos;
        Ok(())
    }

    pub fn register_instance_w_timestamp<Foo>(
        &mut self,
        instance: &Foo,
        _timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>>
    where
        Foo: DdsType + DdsSerialize,
    {
        let serialized_key = instance.get_serialized_key::<LittleEndian>();
        let instance_handle = serialized_key.as_slice().into();

        if !self.registered_instance_list.contains_key(&instance_handle) {
            if self.qos.resource_limits.max_instances == LENGTH_UNLIMITED
                || (self.registered_instance_list.len() as i32)
                    < self.qos.resource_limits.max_instances
            {
                self.registered_instance_list
                    .insert(instance_handle, serialized_key);
            } else {
                return Err(DdsError::OutOfResources);
            }
        }
        Ok(Some(instance_handle))
    }

    pub fn get_key_value<Foo>(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        let serialized_key = self
            .registered_instance_list
            .get(&handle)
            .ok_or(DdsError::BadParameter)?;

        key_holder.set_key_fields_from_serialized_key::<LittleEndian>(serialized_key.as_ref())
    }

    pub fn lookup_instance<Foo>(&self, instance: &Foo) -> Option<InstanceHandle>
    where
        Foo: DdsType,
    {
        let serialized_key = instance.get_serialized_key::<LittleEndian>();
        let instance_handle = serialized_key.as_slice().into();
        if self.registered_instance_list.contains_key(&instance_handle) {
            Some(instance_handle)
        } else {
            None
        }
    }
}
