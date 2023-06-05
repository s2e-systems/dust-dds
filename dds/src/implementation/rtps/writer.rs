use std::collections::HashMap;

use crate::{
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataWriterQos,
        time::{Duration, Time},
    },
    topic_definition::type_support::DdsSerializedKey,
};

use super::{
    endpoint::RtpsEndpoint,
    history_cache::{RtpsWriterCacheChange, WriterHistoryCache, Data},
    messages::submessage_elements::ParameterList,
    types::{ChangeKind, Guid, Locator, SequenceNumber},
};

pub struct RtpsWriter {
    endpoint: RtpsEndpoint,
    push_mode: bool,
    heartbeat_period: Duration,
    _nack_response_delay: Duration,
    _nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    data_max_size_serialized: usize,
    writer_cache: WriterHistoryCache,
    qos: DataWriterQos,
    registered_instance_list: HashMap<InstanceHandle, DdsSerializedKey>,
}

impl RtpsWriter {
    pub fn new(
        endpoint: RtpsEndpoint,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: usize,
        qos: DataWriterQos,
    ) -> Self {
        Self {
            endpoint,
            push_mode,
            heartbeat_period,
            _nack_response_delay: nack_response_delay,
            _nack_suppression_duration: nack_suppression_duration,
            last_change_sequence_number: SequenceNumber::new(0),
            data_max_size_serialized,
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

    pub fn data_max_size_serialized(&self) -> usize {
        self.data_max_size_serialized
    }

    pub fn writer_cache(&self) -> &WriterHistoryCache {
        &self.writer_cache
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: ParameterList,
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
            Data::new(data).chunks(self.data_max_size_serialized),
            inline_qos,
        )
    }

    pub fn change_list(&self) -> &[RtpsWriterCacheChange] {
        self.writer_cache.change_list()
    }

    pub fn add_change(&mut self, change: RtpsWriterCacheChange) {
        self.writer_cache.add_change(change)
    }

    pub fn remove_change<F>(&mut self, f: F)
    where
        F: FnMut(&RtpsWriterCacheChange) -> bool,
    {
        self.writer_cache.remove_change(f)
    }

    pub fn get_qos(&self) -> &DataWriterQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: DataWriterQos) {
        self.qos = qos;
    }

    pub fn register_instance_w_timestamp(
        &mut self,
        instance_serialized_key: DdsSerializedKey,
        _timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        let instance_handle = instance_serialized_key.clone().into();

        if !self.registered_instance_list.contains_key(&instance_handle) {
            if self.registered_instance_list.len() < self.qos.resource_limits.max_instances {
                self.registered_instance_list
                    .insert(instance_handle, instance_serialized_key);
            } else {
                return Err(DdsError::OutOfResources);
            }
        }
        Ok(Some(instance_handle))
    }

    pub fn get_key_value(&self, handle: InstanceHandle) -> Option<&DdsSerializedKey> {
        self.registered_instance_list.get(&handle)
    }

    pub fn lookup_instance(
        &self,
        instance_serialized_key: DdsSerializedKey,
    ) -> Option<InstanceHandle> {
        let instance_handle = instance_serialized_key.into();
        if self.registered_instance_list.contains_key(&instance_handle) {
            Some(instance_handle)
        } else {
            None
        }
    }
}
