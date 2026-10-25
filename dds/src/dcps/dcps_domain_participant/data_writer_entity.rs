use crate::{
    dcps::xtypes_glue::key_and_instance_handle::{
        KeyHolderData, KeyHolderType, get_instance_handle_from_key_holder_data,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataWriterQos,
        qos_policy::{
            DataRepresentationQosPolicy, HistoryQosPolicyKind, Length, XCDR_DATA_REPRESENTATION,
            XCDR2_DATA_REPRESENTATION,
        },
        status::OfferedIncompatibleQosStatus,
        time::{Duration, DurationKind, Time},
    },
    rtps::{
        message_creator::write_submessages_into_transport, stateful_writer::RtpsStatefulWriter,
    },
    rtps_messages::{
        submessage_elements::SequenceNumberSet,
        submessages::{
            gap::GapSubmessage, info_destination::InfoDestinationSubmessage,
            info_timestamp::InfoTimestampSubmessage,
        },
        types::TIME_INVALID,
    },
    transport::{
        interface::{RtpsParticipant, Write},
        types::{CacheChange, ChangeKind, ENTITYID_UNKNOWN, ReliabilityKind, TopicKind},
    },
    xtypes::{
        dynamic_type::{DynamicData, DynamicType},
        serializer::{serialize_cdr1_be, serialize_cdr1_le, serialize_cdr2_be, serialize_cdr2_le},
    },
};
use alloc::{collections::VecDeque, sync::Arc, vec::Vec};

use super::rtps_traits::RtpsWriter;

pub struct RegisteredInstanceInfo {
    pub instance_handle: InstanceHandle,
    pub last_write_time: Option<Time>,
    pub samples: VecDeque<i64>,
}

#[derive(Default)]
pub struct IncompatibleSubscriptions {
    pub incompatible_subscription_list: Vec<InstanceHandle>,
    pub offered_incompatible_qos_status: OfferedIncompatibleQosStatus,
}

pub struct DataWriterEntity<T> {
    pub instance_handle: InstanceHandle,
    pub transport_writer: T,
    pub topic_name: Arc<str>,
    pub enabled: bool,
    pub last_change_sequence_number: i64,
    pub qos: DataWriterQos,
    pub registered_instance_info: Vec<RegisteredInstanceInfo>,
    pub key_holder_type: KeyHolderType,
}

impl<T: RtpsWriter> DataWriterEntity<T> {
    pub fn new(
        instance_handle: InstanceHandle,
        transport_writer: T,
        topic_name: Arc<str>,
        qos: DataWriterQos,
        key_holder_type: KeyHolderType,
    ) -> Self {
        Self {
            instance_handle,
            transport_writer,
            topic_name,
            enabled: false,
            last_change_sequence_number: 0,
            qos,
            registered_instance_info: Vec::new(),
            key_holder_type,
        }
    }

    pub fn write_w_timestamp(
        &mut self,
        sample_instance_handle: InstanceHandle,
        serialized_data: Vec<u8>,
        sample_timestamp: Time,
        now: Time,
    ) -> DdsResult<()> {
        if !self
            .registered_instance_info
            .iter()
            .any(|x| x.instance_handle == sample_instance_handle)
        {
            if self.registered_instance_info.len() < self.qos.resource_limits.max_instances {
                self.registered_instance_info.push(RegisteredInstanceInfo {
                    instance_handle: sample_instance_handle,
                    last_write_time: None,
                    samples: VecDeque::new(),
                });
            } else {
                return Err(DdsError::OutOfResources);
            }
        }

        if let Length::Limited(max_samples_per_instance) =
            self.qos.resource_limits.max_samples_per_instance
        {
            // If the history Qos guarantess that the number of samples
            // is below the limit there is no need to check
            match self.qos.history.kind {
                HistoryQosPolicyKind::KeepLast(depth)
                    if depth as i32 <= max_samples_per_instance => {}
                _ => {
                    if let Some(s) = self
                        .registered_instance_info
                        .iter()
                        .find(|x| x.instance_handle == sample_instance_handle)
                    {
                        // Only Alive changes count towards the resource limits
                        if s.samples.len() >= max_samples_per_instance as usize {
                            return Err(DdsError::OutOfResources);
                        }
                    }
                }
            }
        }

        if let Length::Limited(max_samples) = self.qos.resource_limits.max_samples {
            let total_samples = self
                .registered_instance_info
                .iter()
                .fold(0, |acc, x| acc + x.samples.len());

            if total_samples >= max_samples as usize {
                return Err(DdsError::OutOfResources);
            }
        }

        self.last_change_sequence_number += 1;
        let change = CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: self.transport_writer.guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(sample_timestamp.into()),
            instance_handle: if self.key_holder_type.is_empty() {
                None
            } else {
                Some(sample_instance_handle.into())
            },
            data_value: serialized_data.into(),
        };

        let instance_info = self
            .registered_instance_info
            .iter_mut()
            .find(|x| x.instance_handle == sample_instance_handle)
            .expect("Instance info must exist");

        match &mut instance_info.last_write_time {
            Some(last_write_time) => {
                if *last_write_time < sample_timestamp {
                    *last_write_time = sample_timestamp;
                }
            }
            None => {
                instance_info.last_write_time = Some(sample_timestamp);
            }
        }

        instance_info.samples.push_back(change.sequence_number);

        if let DurationKind::Finite(lifespan_duration) = self.qos.lifespan.duration {
            let duration_until_expired = sample_timestamp - now + lifespan_duration;
            if duration_until_expired <= Duration::new(0, 0) {
                return Ok(());
            }
        }

        self.transport_writer.add_change(change);

        Ok(())
    }

    pub fn dispose_w_timestamp(
        &mut self,
        instance_handle: InstanceHandle,
        serialized_key: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let Some(instance_info) = self
            .registered_instance_info
            .iter_mut()
            .find(|x| x.instance_handle == instance_handle)
        else {
            return Err(DdsError::BadParameter);
        };

        instance_info.last_write_time = None;

        self.last_change_sequence_number += 1;
        let cache_change = CacheChange {
            kind: ChangeKind::NotAliveDisposed,
            writer_guid: self.transport_writer.guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_key.into(),
        };
        self.transport_writer.add_change(cache_change);

        Ok(())
    }

    pub fn register_w_timestamp(
        &mut self,
        dynamic_data: &DynamicData<'static>,
        type_support: &DynamicType<'static>,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let key_holder_type = KeyHolderType::new(&dynamic_data.r#type());
        let key_holder_data = KeyHolderData::from_dynamic_data(dynamic_data, &key_holder_type)?;

        if TopicKind::from(type_support) == TopicKind::NoKey {
            return Err(DdsError::IllegalOperation);
        }

        let instance_handle = get_instance_handle_from_key_holder_data(&key_holder_data)?;

        if let Some(instance_info) = self
            .registered_instance_info
            .iter_mut()
            .find(|x| x.instance_handle == instance_handle)
        {
            instance_info.last_write_time = Some(timestamp);
        } else if self.registered_instance_info.len() < self.qos.resource_limits.max_instances {
            self.registered_instance_info.push(RegisteredInstanceInfo {
                instance_handle,
                last_write_time: Some(timestamp),
                samples: VecDeque::new(),
            });
        } else {
            return Err(DdsError::OutOfResources);
        }

        Ok(Some(instance_handle))
    }

    pub fn unregister_w_timestamp(
        &mut self,
        instance_handle: InstanceHandle,
        serialized_key: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let Some(instance_info) = self
            .registered_instance_info
            .iter_mut()
            .find(|x| x.instance_handle == instance_handle)
        else {
            return Err(DdsError::BadParameter);
        };

        instance_info.last_write_time = None;

        self.last_change_sequence_number += 1;
        let kind = if self
            .qos
            .writer_data_lifecycle
            .autodispose_unregistered_instances
        {
            ChangeKind::NotAliveDisposedUnregistered
        } else {
            ChangeKind::NotAliveUnregistered
        };
        let cache_change = CacheChange {
            kind,
            writer_guid: self.transport_writer.guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_key.into(),
        };
        self.transport_writer.add_change(cache_change);
        Ok(())
    }
}

impl DataWriterEntity<RtpsStatefulWriter> {
    pub fn send_heartbeat(&mut self, transport_participant: &mut impl RtpsParticipant, now: Time) {
        let seq_num_min = self
            .transport_writer
            .changes()
            .first()
            .map(|cc| cc.sequence_number);
        let seq_num_max = self
            .transport_writer
            .changes()
            .last()
            .map(|cc| cc.sequence_number);
        let writer_id = self.transport_writer.guid().entity_id();
        let guid_prefix = self.transport_writer.guid().prefix();
        let heartbeat_period = self.transport_writer.heartbeat_period();

        for reader_proxy in self.transport_writer.matched_readers_mut() {
            if reader_proxy.reliability() == ReliabilityKind::Reliable
                && reader_proxy.unacked_changes(seq_num_max)
                && reader_proxy
                    .heartbeat_machine()
                    .is_time_for_heartbeat(now, heartbeat_period.into())
            {
                let first_sn = seq_num_min
                    .unwrap_or(1)
                    .max(reader_proxy.first_relevant_sample_seq_num());
                let last_sn = seq_num_max.unwrap_or(0).max(first_sn - 1);
                let heartbeat_submessage = reader_proxy
                    .heartbeat_machine()
                    .generate_new_heartbeat(writer_id, first_sn, last_sn, now, false);

                let info_dst =
                    InfoDestinationSubmessage::new(reader_proxy.remote_reader_guid().prefix());

                let mut writer = transport_participant.writer(reader_proxy.unicast_locator_list());
                write_submessages_into_transport(
                    &[&info_dst, &heartbeat_submessage],
                    guid_prefix,
                    &mut writer,
                );
                writer.flush();
            }
        }
    }

    pub fn send_requested_changes(
        &mut self,
        transport_participant: &mut impl RtpsParticipant,
        now: Time,
    ) {
        let seq_num_min = self
            .transport_writer
            .changes()
            .first()
            .map(|cc| cc.sequence_number);
        let seq_num_max = self
            .transport_writer
            .changes()
            .last()
            .map(|cc| cc.sequence_number);
        let writer_id = self.transport_writer.guid().entity_id();
        let guid_prefix = self.transport_writer.guid().prefix();
        let data_max_size_serialized = self.transport_writer.data_max_size_serialized();

        let (changes, matched_readers) = self.transport_writer.changes_and_matched_readers_mut();

        for reader_proxy in matched_readers {
            while let Some(next_requested_change_seq_num) = reader_proxy.next_requested_change() {
                if let Some(cache_change) = changes.iter().find(|cc| {
                    cc.sequence_number == next_requested_change_seq_num
                        && next_requested_change_seq_num
                            >= reader_proxy.first_relevant_sample_seq_num()
                }) {
                    let number_of_fragments = cache_change
                        .data_value
                        .len()
                        .div_ceil(data_max_size_serialized);

                    if number_of_fragments > 1 && cache_change.kind == ChangeKind::Alive {
                        for fragment_number in 0..number_of_fragments {
                            let data_frag = cache_change.as_data_frag_submessage(
                                reader_proxy.remote_reader_guid().entity_id(),
                                writer_id,
                                data_max_size_serialized,
                                fragment_number,
                            );

                            let info_dst = InfoDestinationSubmessage::new(
                                reader_proxy.remote_reader_guid().prefix(),
                            );
                            let info_timestamp =
                                if let Some(timestamp) = cache_change.source_timestamp {
                                    InfoTimestampSubmessage::new(false, timestamp.into())
                                } else {
                                    InfoTimestampSubmessage::new(true, TIME_INVALID)
                                };
                            let heartbeat = if fragment_number == number_of_fragments - 1 {
                                let first_sn = seq_num_min
                                    .unwrap_or(1)
                                    .max(reader_proxy.first_relevant_sample_seq_num());
                                let last_sn = seq_num_max.unwrap_or(0).max(first_sn - 1);
                                Some(reader_proxy.heartbeat_machine().generate_new_heartbeat(
                                    writer_id, first_sn, last_sn, now, false,
                                ))
                            } else {
                                None
                            };
                            let mut writer =
                                transport_participant.writer(reader_proxy.unicast_locator_list());
                            if let Some(heartbeat) = &heartbeat {
                                write_submessages_into_transport(
                                    &[&info_dst, &info_timestamp, &data_frag, heartbeat],
                                    guid_prefix,
                                    &mut writer,
                                );
                            } else {
                                write_submessages_into_transport(
                                    &[&info_dst, &info_timestamp, &data_frag],
                                    guid_prefix,
                                    &mut writer,
                                );
                            }
                            writer.flush();
                        }
                    } else {
                        let info_dst = InfoDestinationSubmessage::new(
                            reader_proxy.remote_reader_guid().prefix(),
                        );

                        let info_timestamp = if let Some(timestamp) = cache_change.source_timestamp
                        {
                            InfoTimestampSubmessage::new(false, timestamp.into())
                        } else {
                            InfoTimestampSubmessage::new(true, TIME_INVALID)
                        };

                        let inline_qos = match (
                            cache_change.status_info_parameter(),
                            cache_change.key_hash_parameter(),
                        ) {
                            (Some(s), Some(k)) => &[s, k][..],
                            (Some(s), None) => &[s][..],
                            (None, Some(k)) => &[k][..],
                            (None, None) => &[],
                        };
                        let data_submessage = cache_change.as_data_submessage(
                            reader_proxy.remote_reader_guid().entity_id(),
                            writer_id,
                            inline_qos,
                        );

                        let first_sn = seq_num_min
                            .unwrap_or(1)
                            .max(reader_proxy.first_relevant_sample_seq_num());
                        let last_sn = seq_num_max.unwrap_or(0).max(first_sn - 1);
                        let heartbeat = reader_proxy
                            .heartbeat_machine()
                            .generate_new_heartbeat(writer_id, first_sn, last_sn, now, false);

                        let mut writer =
                            transport_participant.writer(reader_proxy.unicast_locator_list());
                        write_submessages_into_transport(
                            &[&info_dst, &info_timestamp, &data_submessage, &heartbeat],
                            guid_prefix,
                            &mut writer,
                        );
                        writer.flush();
                    }
                } else {
                    let info_dst =
                        InfoDestinationSubmessage::new(reader_proxy.remote_reader_guid().prefix());

                    let gap_submessage = GapSubmessage::new(
                        ENTITYID_UNKNOWN,
                        writer_id,
                        next_requested_change_seq_num,
                        SequenceNumberSet::new(next_requested_change_seq_num + 1, []),
                    );

                    let mut writer =
                        transport_participant.writer(reader_proxy.unicast_locator_list());
                    write_submessages_into_transport(
                        &[&info_dst, &gap_submessage],
                        guid_prefix,
                        &mut writer,
                    );
                    writer.flush();
                }
            }
        }
    }

    pub fn send_requested_fragments(&mut self, transport_participant: &mut impl RtpsParticipant) {
        let writer_id = self.transport_writer.guid().entity_id();
        let guid_prefix = self.transport_writer.guid().prefix();
        let data_max_size_serialized = self.transport_writer.data_max_size_serialized();

        let (changes, matched_readers) = self.transport_writer.changes_and_matched_readers_mut();

        for reader_proxy in matched_readers {
            while let Some((change_seq_num, request_fragment_number)) =
                reader_proxy.next_requested_fragment()
            {
                if let Some(cache_change) = changes.iter().find(|cc| {
                    cc.sequence_number == change_seq_num
                        && change_seq_num >= reader_proxy.first_relevant_sample_seq_num()
                }) {
                    let number_of_fragments = cache_change
                        .data_value
                        .len()
                        .div_ceil(data_max_size_serialized);
                    let request_fragment_number = request_fragment_number as usize;
                    if (1..=number_of_fragments).contains(&request_fragment_number)
                        && cache_change.kind == ChangeKind::Alive
                    {
                        let reader_id = reader_proxy.remote_reader_guid().entity_id();
                        let data_frag = cache_change.as_data_frag_submessage(
                            reader_id,
                            writer_id,
                            data_max_size_serialized,
                            request_fragment_number - 1,
                        );

                        let info_dst = InfoDestinationSubmessage::new(
                            reader_proxy.remote_reader_guid().prefix(),
                        );
                        let info_timestamp = if let Some(timestamp) = cache_change.source_timestamp
                        {
                            InfoTimestampSubmessage::new(false, timestamp.into())
                        } else {
                            InfoTimestampSubmessage::new(true, TIME_INVALID)
                        };

                        let mut writer =
                            transport_participant.writer(reader_proxy.unicast_locator_list());
                        write_submessages_into_transport(
                            &[&info_dst, &info_timestamp, &data_frag],
                            guid_prefix,
                            &mut writer,
                        );
                        writer.flush();
                    }
                } else {
                    let info_dst =
                        InfoDestinationSubmessage::new(reader_proxy.remote_reader_guid().prefix());
                    let gap_submessage = GapSubmessage::new(
                        ENTITYID_UNKNOWN,
                        writer_id,
                        change_seq_num,
                        SequenceNumberSet::new(change_seq_num + 1, []),
                    );

                    let mut writer =
                        transport_participant.writer(reader_proxy.unicast_locator_list());
                    write_submessages_into_transport(
                        &[&info_dst, &gap_submessage],
                        guid_prefix,
                        &mut writer,
                    );
                    writer.flush();
                }
            }
        }
    }
}

pub fn serialize<'a>(
    dynamic_data: &DynamicData<'a>,
    representation: &DataRepresentationQosPolicy,
) -> DdsResult<Vec<u8>> {
    Ok(
        if representation.value.is_empty() || representation.value[0] == XCDR_DATA_REPRESENTATION {
            if cfg!(target_endian = "big") {
                serialize_cdr1_be(dynamic_data)
            } else {
                serialize_cdr1_le(dynamic_data)
            }
        } else if representation.value[0] == XCDR2_DATA_REPRESENTATION {
            if cfg!(target_endian = "big") {
                serialize_cdr2_be(dynamic_data)
            } else {
                serialize_cdr2_le(dynamic_data)
            }
        } else {
            panic!("Invalid data representation")
        }?,
    )
}
