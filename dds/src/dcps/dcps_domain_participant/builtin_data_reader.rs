use crate::{
    dcps::{
        dcps_domain_participant::{
            data_reader_entity::DataReaderEntity, participant_entity::DiscoveredParticipantInfo,
            reader_methods::deserialize_topic_type, rtps_traits::RtpsReader,
        },
        xtypes_glue::key_and_instance_handle::{
            KeyHolderType, get_instance_handle_from_dynamic_data,
        },
    },
    infrastructure::{instance::InstanceHandle, qos::DataReaderQos, time::Time},
    runtime::{Clock, DdsRuntime},
    transport::types::ChangeKind,
    xtypes::{deserializer::deserialize_top_level_type, type_support::Type},
};
use alloc::{string::String, vec::Vec};
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub struct BuiltinDataReader<R, T> {
    pub reader: DataReaderEntity<R>,
    phantom: PhantomData<fn() -> T>,
}

impl<R, T> BuiltinDataReader<R, T> {
    pub fn new(
        instance_handle: InstanceHandle,
        qos: DataReaderQos,
        topic_name: String,
        transport_reader: R,
    ) -> Self {
        Self {
            reader: DataReaderEntity::new(instance_handle, qos, topic_name, transport_reader),
            phantom: PhantomData,
        }
    }
}

impl<R, T> Deref for BuiltinDataReader<R, T> {
    type Target = DataReaderEntity<R>;

    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}

impl<R, T> DerefMut for BuiltinDataReader<R, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}

impl<R: RtpsReader, T: Type> BuiltinDataReader<R, T> {
    pub fn process_cache_changes(
        &mut self,
        discovered_participant_list: &mut [DiscoveredParticipantInfo],
        reception_timestamp: Time,
        runtime: &impl DdsRuntime,
    ) {
        let changes = core::mem::take(self.reader.transport_reader.changes_mut());
        let data_reader_handle = &self.reader.instance_handle.clone();
        tracing::trace!(data_reader_handle=?data_reader_handle, "Processing {} reader cache changes", changes.len());

        for cache_change in changes {
            if let Some(matched_participant) = discovered_participant_list
                .iter_mut()
                .find(|x| x.guid_prefix == cache_change.writer_guid.prefix())
            {
                matched_participant.last_communication_timestamp = runtime.clock().now();
            }

            let change_instance_handle = if let Some(i) = cache_change.instance_handle {
                InstanceHandle::new(i)
            } else {
                match cache_change.kind {
                    ChangeKind::Alive | ChangeKind::AliveFiltered => {
                        let Some(data_value) = deserialize_topic_type(
                            &self.reader.topic_name,
                            T::TYPE,
                            cache_change.data_value.as_ref(),
                        ) else {
                            tracing::warn!("Failed to deserialize user defined data");
                            return;
                        };
                        let Ok(instance_handle) =
                            get_instance_handle_from_dynamic_data(&data_value)
                        else {
                            tracing::warn!("Failed to get instance handle from dynamic_data");
                            return;
                        };
                        instance_handle
                    }
                    ChangeKind::NotAliveDisposed
                    | ChangeKind::NotAliveUnregistered
                    | ChangeKind::NotAliveDisposedUnregistered => {
                        let mut dynamic_members = Vec::new();
                        let Ok(key_holder) =
                            KeyHolderType::from_dynamic_type(&T::TYPE, &mut dynamic_members)
                        else {
                            tracing::warn!("Failed to create key holder");
                            return;
                        };

                        let Ok(data_value) = deserialize_top_level_type(
                            *key_holder.as_dynamic_type(),
                            cache_change.data_value.as_ref(),
                        ) else {
                            tracing::warn!("Failed to deserialize disposed user defined data");
                            return;
                        };

                        let Ok(instance_handle) =
                            get_instance_handle_from_dynamic_data(&data_value)
                        else {
                            tracing::warn!("Failed to deserialize disposed key user defined data");
                            return;
                        };
                        instance_handle
                    }
                }
            };

            self.reader
                .add_reader_change(
                    cache_change.writer_guid,
                    cache_change.data_value,
                    cache_change.kind,
                    change_instance_handle.into(),
                    cache_change.source_timestamp.map(Into::into),
                    reception_timestamp,
                )
                .ok();
        }
    }
}
