use std::collections::HashMap;

use serde::Serialize;

use crate::{
    implementation::data_representation_inline_qos::{
        parameter_id_values::PID_STATUS_INFO,
        types::{
            STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED, STATUS_INFO_UNREGISTERED,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::{InstanceHandle, HANDLE_NIL},
        qos::DataWriterQos,
        time::{Duration, DurationKind, Time},
    },
    topic_definition::type_support::{DdsSerializedKey, DdsType},
};

use super::{
    endpoint::RtpsEndpoint,
    history_cache::{RtpsParameter, RtpsWriterCacheChange, WriterHistoryCache},
    messages::types::ParameterId,
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

    pub fn writer_cache(&self) -> &WriterHistoryCache {
        &self.writer_cache
    }

    pub fn writer_cache_mut(&mut self) -> &mut WriterHistoryCache {
        &mut self.writer_cache
    }

    pub fn new_write_change(
        &mut self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        _handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<RtpsWriterCacheChange> {
        let handle = self
            .register_instance_w_timestamp(instance_serialized_key, timestamp)?
            .unwrap_or(HANDLE_NIL);
        let change = self.new_change(
            ChangeKind::Alive,
            serialized_data,
            vec![],
            handle,
            timestamp,
        );

        Ok(change)
    }

    pub fn new_dispose_change(
        &mut self,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<RtpsWriterCacheChange> {
        let instance_handle = match handle {
            Some(h) => {
                if let Some(stored_handle) = self.lookup_instance(instance_serialized_key.clone()) {
                    if stored_handle == h {
                        Ok(h)
                    } else {
                        Err(DdsError::PreconditionNotMet(
                            "Handle does not match instance".to_string(),
                        ))
                    }
                } else {
                    Err(DdsError::BadParameter)
                }
            }
            None => {
                if let Some(stored_handle) = self.lookup_instance(instance_serialized_key.clone()) {
                    Ok(stored_handle)
                } else {
                    Err(DdsError::PreconditionNotMet(
                        "Instance not registered with this DataWriter".to_string(),
                    ))
                }
            }
        }?;

        let mut serialized_status_info = Vec::new();
        let mut serializer =
            cdr::Serializer::<_, cdr::LittleEndian>::new(&mut serialized_status_info);
        STATUS_INFO_DISPOSED.serialize(&mut serializer).unwrap();

        let inline_qos = vec![RtpsParameter::new(
            ParameterId(PID_STATUS_INFO),
            serialized_status_info,
        )];

        Ok(self.new_change(
            ChangeKind::NotAliveDisposed,
            instance_serialized_key.as_ref().to_vec(),
            inline_qos,
            instance_handle,
            timestamp,
        ))
    }

    pub fn new_unregister_change(
        &mut self,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<RtpsWriterCacheChange> {
        let instance_handle = match handle {
            Some(h) => {
                if let Some(stored_handle) = self.lookup_instance(instance_serialized_key.clone()) {
                    if stored_handle == h {
                        Ok(h)
                    } else {
                        Err(DdsError::PreconditionNotMet(
                            "Handle does not match instance".to_string(),
                        ))
                    }
                } else {
                    Err(DdsError::BadParameter)
                }
            }
            None => {
                if let Some(stored_handle) = self.lookup_instance(instance_serialized_key.clone()) {
                    Ok(stored_handle)
                } else {
                    Err(DdsError::PreconditionNotMet(
                        "Instance not registered with this DataWriter".to_string(),
                    ))
                }
            }
        }?;

        let mut serialized_status_info = Vec::new();
        let mut serializer =
            cdr::Serializer::<_, cdr::LittleEndian>::new(&mut serialized_status_info);
        if self
            .qos
            .writer_data_lifecycle
            .autodispose_unregistered_instances
        {
            STATUS_INFO_DISPOSED_UNREGISTERED
                .serialize(&mut serializer)
                .unwrap();
        } else {
            STATUS_INFO_UNREGISTERED.serialize(&mut serializer).unwrap();
        }

        let inline_qos = vec![RtpsParameter::new(
            ParameterId(PID_STATUS_INFO),
            serialized_status_info,
        )];

        Ok(self.new_change(
            ChangeKind::NotAliveUnregistered,
            instance_serialized_key.as_ref().to_vec(),
            inline_qos,
            instance_handle,
            timestamp,
        ))
    }

    fn new_change(
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

    pub fn get_key_value<Foo>(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        let serialized_key = self
            .registered_instance_list
            .get(&handle)
            .ok_or(DdsError::BadParameter)?;

        key_holder.set_key_fields_from_serialized_key(serialized_key)
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

    pub fn data_max_size_serialized(&self) -> usize {
        self.data_max_size_serialized
    }

    pub fn remove_stale_changes(&mut self, now: Time) {
        let timespan_duration = self.qos.lifespan.duration;
        self.writer_cache
            .remove_change(|cc| DurationKind::Finite(now - cc.timestamp()) > timespan_duration)
    }
}
