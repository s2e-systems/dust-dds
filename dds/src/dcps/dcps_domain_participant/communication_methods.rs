use alloc::{string::String, vec::Vec};
use tracing::info;

use crate::{
    dcps::{
        dcps_domain_participant::{
            AddChangeResult, ContentFilteredTopicEntity, DcpsDomainParticipant,
            DiscoveredParticipantInfo, RtpsReader, TopicEntity,
            builtin_data_reader::BuiltinDataReader, get_topic_type_support,
            reader_methods::deserialize_topic_type,
        },
        listeners::domain_participant_listener::ListenerMail,
        xtypes_glue::key_and_instance_handle::{
            KeyHolderType, get_instance_handle_from_dynamic_data,
        },
    },
    dds_async::{
        data_reader::DataReaderAsync, domain_participant::DomainParticipantAsync,
        subscriber::SubscriberAsync,
    },
    infrastructure::{instance::InstanceHandle, status::StatusKind, time::Time},
    rtps::message_receiver::MessageReceiver,
    rtps_messages::{
        overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
        submessages::{
            data::DataSubmessage, data_frag::DataFragSubmessage, gap::GapSubmessage,
            heartbeat::HeartbeatSubmessage,
        },
    },
    runtime::{Clock, DdsRuntime},
    transport::types::{ChangeKind, Guid},
    xtypes::deserializer::deserialize_top_level_type,
};

impl DcpsDomainParticipant {
    #[tracing::instrument(skip(self, runtime))]
    pub fn process_received_cache_changes(&mut self, runtime: &impl DdsRuntime) {
        let reception_timestamp = runtime.clock().now();
        let dcps_sender = self.dcps_sender;
        let domain_id = self.domain_participant.domain_id;
        let dp_instance_handle = self.domain_participant.instance_handle;
        let dp_listener_mask = self.domain_participant.listener_mask;
        let dp_listener_sender = &self.domain_participant.listener_sender;
        let discovered_participant_list = &mut self.domain_participant.discovered_participant_list;
        let content_filtered_topic_list = &self.domain_participant.content_filtered_topic_list;
        let locally_created_topic_list = &self.domain_participant.locally_created_topic_list;

        for subscriber in &mut self.domain_participant.user_defined_subscriber_list {
            let subscriber_handle = subscriber.instance_handle;
            let subscriber_listener_mask = subscriber.listener_mask;
            let subscriber_listener_sender = subscriber.listener_sender.clone();
            let (subscriber_status_condition, data_reader_list) = (
                &mut subscriber.status_condition,
                &mut subscriber.data_reader_list,
            );
            'data_readers: for data_reader in data_reader_list {
                let changes = core::mem::take(data_reader.transport_reader.changes_mut());
                let data_reader_handle = &data_reader.instance_handle.clone();
                tracing::trace!(subscriber_handle=?subscriber_handle, data_reader_handle=?data_reader_handle, "Processing {} reader cache changes", changes.len());

                for cache_change in changes {
                    if let Some(matched_participant) = discovered_participant_list
                        .iter_mut()
                        .find(|x| x.guid_prefix == cache_change.writer_guid.prefix())
                    {
                        matched_participant.last_communication_timestamp = runtime.clock().now();
                    }
                    let Some(type_support) = get_topic_type_support(
                        &data_reader.topic_name,
                        content_filtered_topic_list,
                        locally_created_topic_list,
                    ) else {
                        tracing::warn!(topic_name = ?data_reader.topic_name, "Failed to find type support for reader");
                        continue 'data_readers;
                    };

                    let (topic_name, type_name) = if let Some(content_filtered_topic) =
                        content_filtered_topic_list
                            .iter()
                            .find(|t| t.topic_name == data_reader.topic_name)
                    {
                        let Some(reader_topic) = locally_created_topic_list
                            .iter()
                            .find(|t| t.topic_name == content_filtered_topic.related_topic_name)
                        else {
                            tracing::warn!(topic_name = ?data_reader.topic_name, "Failed to find related_topic_name for reader");
                            continue 'data_readers;
                        };
                        if cache_change.kind == ChangeKind::Alive {
                            let Some(data) = deserialize_topic_type(
                                &data_reader.topic_name,
                                *type_support,
                                cache_change.data_value.as_ref(),
                            ) else {
                                continue 'data_readers;
                            };
                            enum Operator {
                                LessThan,
                                Equal,
                            }

                            impl Operator {
                                fn to_str(&self) -> &'static str {
                                    match self {
                                        Self::Equal => "=",
                                        Self::LessThan => "<=",
                                    }
                                }

                                fn compare_string(&self, lhs: &String, rhs: &String) -> bool {
                                    match self {
                                        Self::Equal => lhs == rhs,
                                        Self::LessThan => lhs <= rhs,
                                    }
                                }
                                fn compare_int32(&self, lhs: &i32, rhs: &i32) -> bool {
                                    match self {
                                        Self::Equal => lhs == rhs,
                                        Self::LessThan => lhs <= rhs,
                                    }
                                }
                            }

                            let mut operators = [Operator::LessThan, Operator::Equal].iter();
                            let filter = loop {
                                if let Some(operator) = operators.next() {
                                    if let Some((variable_name, _)) = content_filtered_topic
                                        .filter_expression
                                        .split_once(operator.to_str())
                                    {
                                        break Some((variable_name, operator));
                                    }
                                } else {
                                    break None;
                                };
                            };

                            if let Some((variable_name, comparison_function)) = filter {
                                let Some(member_id) =
                                    data.get_member_id_by_name(variable_name.trim())
                                else {
                                    continue 'data_readers;
                                };
                                let Ok(member_descriptor) = data.get_descriptor(member_id) else {
                                    continue 'data_readers;
                                };
                                match member_descriptor.r#type.get_kind() {
                                    crate::xtypes::dynamic_type::TypeKind::NONE => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::BOOLEAN => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::BYTE => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::INT16 => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::INT32 => {
                                        let member_value = data.get_int32_value(member_id).unwrap();
                                        if !comparison_function.compare_int32(
                                            member_value,
                                            &content_filtered_topic.expression_parameters[0]
                                                .parse()
                                                .expect("valid number"),
                                        ) {
                                            continue 'data_readers;
                                        }
                                    }
                                    crate::xtypes::dynamic_type::TypeKind::INT64 => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::UINT16 => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::UINT32 => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::UINT64 => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::FLOAT32 => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::FLOAT64 => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::FLOAT128 => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::INT8 => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::UINT8 => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::CHAR8 => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::CHAR16 => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::STRING8
                                    | crate::xtypes::dynamic_type::TypeKind::STRING16 => {
                                        let member_value =
                                            data.get_string_value(member_id).unwrap();
                                        if !comparison_function.compare_string(
                                            member_value,
                                            &content_filtered_topic.expression_parameters[0],
                                        ) {
                                            continue 'data_readers;
                                        }
                                    }
                                    crate::xtypes::dynamic_type::TypeKind::ALIAS => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::ENUM => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::BITMASK => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::ANNOTATION => {
                                        todo!()
                                    }
                                    crate::xtypes::dynamic_type::TypeKind::STRUCTURE => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::UNION => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::BITSET => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::SEQUENCE => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::ARRAY => todo!(),
                                    crate::xtypes::dynamic_type::TypeKind::MAP => todo!(),
                                }
                            } else {
                                continue 'data_readers;
                            };
                        }
                        (
                            reader_topic.type_name.clone(),
                            reader_topic.topic_name.clone(),
                        )
                    } else {
                        let Some(reader_topic) = locally_created_topic_list
                            .iter()
                            .find(|t| t.topic_name == data_reader.topic_name)
                        else {
                            tracing::warn!(topic_name = ?data_reader.topic_name, "Failed to find topic for reader");
                            continue 'data_readers;
                        };
                        (
                            data_reader.topic_name.clone(),
                            reader_topic.type_name.clone(),
                        )
                    };

                    let change_instance_handle = if let Some(i) = cache_change.instance_handle {
                        InstanceHandle::new(i)
                    } else {
                        match cache_change.kind {
                            ChangeKind::Alive | ChangeKind::AliveFiltered => {
                                let Some(data_value) = deserialize_topic_type(
                                    &data_reader.topic_name,
                                    *type_support,
                                    cache_change.data_value.as_ref(),
                                ) else {
                                    tracing::warn!("Failed to deserialize user defined data");
                                    continue 'data_readers;
                                };
                                let Ok(instance_handle) =
                                    get_instance_handle_from_dynamic_data(&data_value)
                                else {
                                    tracing::warn!(
                                        "Failed to get instance handle from dynamic_data"
                                    );
                                    continue 'data_readers;
                                };
                                instance_handle
                            }
                            ChangeKind::NotAliveDisposed
                            | ChangeKind::NotAliveUnregistered
                            | ChangeKind::NotAliveDisposedUnregistered => {
                                let mut dynamic_members = Vec::new();
                                let Ok(key_holder) = KeyHolderType::from_dynamic_type(
                                    type_support,
                                    &mut dynamic_members,
                                ) else {
                                    tracing::warn!("Failed to create key holder");
                                    continue 'data_readers;
                                };

                                let Ok(data_value) = deserialize_top_level_type(
                                    *key_holder.as_dynamic_type(),
                                    cache_change.data_value.as_ref(),
                                ) else {
                                    tracing::warn!(
                                        "Failed to deserialize disposed user defined data"
                                    );
                                    continue 'data_readers;
                                };

                                let Ok(instance_handle) =
                                    get_instance_handle_from_dynamic_data(&data_value)
                                else {
                                    tracing::warn!(
                                        "Failed to deserialize disposed key user defined data"
                                    );
                                    continue 'data_readers;
                                };
                                instance_handle
                            }
                        }
                    };

                    let the_participant =
                        DomainParticipantAsync::new(dcps_sender, domain_id, dp_instance_handle);
                    let the_subscriber =
                        SubscriberAsync::new(subscriber_handle, the_participant.clone());

                    let the_reader = DataReaderAsync::new(
                        *data_reader_handle,
                        the_subscriber.clone(),
                        topic_name,
                        type_name,
                    );

                    match data_reader.add_reader_change(
                        cache_change.writer_guid,
                        cache_change.data_value,
                        cache_change.kind,
                        change_instance_handle.into(),
                        cache_change.source_timestamp.map(Into::into),
                        reception_timestamp,
                    ) {
                        Ok(AddChangeResult::Added) => {
                            tracing::info!("New change added");

                            let data_reader_on_data_available_active = data_reader
                                .listener_mask
                                .is_enabled(&StatusKind::DataAvailable);

                            if subscriber_listener_mask.is_enabled(&StatusKind::DataOnReaders) {
                                if let Some(l) = &subscriber_listener_sender {
                                    l.send(ListenerMail::DataOnReaders { the_subscriber }).ok();
                                }
                            } else if data_reader_on_data_available_active {
                                if let Some(l) = &data_reader.listener_sender {
                                    info!("Triggering data reader DataAvailable listener");
                                    l.send(ListenerMail::DataAvailable { the_reader }).ok();
                                }
                            }

                            subscriber_status_condition
                                .add_communication_state(StatusKind::DataOnReaders);

                            data_reader
                                .status_condition
                                .add_communication_state(StatusKind::DataAvailable);
                        }
                        Ok(AddChangeResult::NotAdded) => (), // Do nothing
                        Ok(AddChangeResult::Rejected(
                            instance_handle,
                            sample_rejected_status_kind,
                        )) => {
                            tracing::info!("Change rejected");
                            data_reader.increment_sample_rejected_status(
                                instance_handle,
                                sample_rejected_status_kind,
                            );

                            if data_reader
                                .listener_mask
                                .is_enabled(&StatusKind::SampleRejected)
                            {
                                let status = data_reader.get_sample_rejected_status();

                                if let Some(l) = &data_reader.listener_sender {
                                    l.send(ListenerMail::SampleRejected { the_reader, status })
                                        .ok();
                                };
                            } else if subscriber_listener_mask
                                .is_enabled(&StatusKind::SampleRejected)
                            {
                                let status = data_reader.get_sample_rejected_status();
                                if let Some(l) = &subscriber_listener_sender {
                                    l.send(ListenerMail::SampleRejected { status, the_reader })
                                        .ok();
                                }
                            } else if dp_listener_mask.is_enabled(&StatusKind::SampleRejected) {
                                let status = data_reader.get_sample_rejected_status();
                                if let Some(l) = dp_listener_sender {
                                    l.send(ListenerMail::SampleRejected { status, the_reader })
                                        .ok();
                                }
                            }

                            data_reader
                                .status_condition
                                .add_communication_state(StatusKind::SampleRejected);
                        }
                        Err(_) => (),
                    }
                }
            }
        }

        let builtin_subscriber = &mut self.domain_participant.builtin_subscriber;
        Self::process_single_builtin_reader_cache_changes(
            discovered_participant_list,
            content_filtered_topic_list,
            locally_created_topic_list,
            &mut builtin_subscriber.dcps_participant_reader,
            reception_timestamp,
            runtime,
        );
        for data_reader in builtin_subscriber.stateful_data_reader_list_mut() {
            Self::process_single_builtin_reader_cache_changes(
                discovered_participant_list,
                content_filtered_topic_list,
                locally_created_topic_list,
                data_reader,
                reception_timestamp,
                runtime,
            );
        }
    }

    fn process_single_builtin_reader_cache_changes(
        discovered_participant_list: &mut [DiscoveredParticipantInfo],
        content_filtered_topic_list: &[ContentFilteredTopicEntity],
        locally_created_topic_list: &[TopicEntity],
        data_reader: &mut BuiltinDataReader<impl RtpsReader>,
        reception_timestamp: Time,
        runtime: &impl DdsRuntime,
    ) {
        let changes = core::mem::take(data_reader.transport_reader.changes_mut());
        let data_reader_handle = &data_reader.instance_handle.clone();
        tracing::trace!(data_reader_handle=?data_reader_handle, "Processing {} reader cache changes", changes.len());

        for cache_change in changes {
            if let Some(matched_participant) = discovered_participant_list
                .iter_mut()
                .find(|x| x.guid_prefix == cache_change.writer_guid.prefix())
            {
                matched_participant.last_communication_timestamp = runtime.clock().now();
            }

            let Some(type_support) = get_topic_type_support(
                &data_reader.topic_name,
                content_filtered_topic_list,
                locally_created_topic_list,
            ) else {
                tracing::warn!(topic_name = ?data_reader.topic_name, "Failed to find type support for reader");
                return;
            };

            let change_instance_handle = if let Some(i) = cache_change.instance_handle {
                InstanceHandle::new(i)
            } else {
                match cache_change.kind {
                    ChangeKind::Alive | ChangeKind::AliveFiltered => {
                        let Some(data_value) = deserialize_topic_type(
                            &data_reader.topic_name,
                            *type_support,
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
                            KeyHolderType::from_dynamic_type(type_support, &mut dynamic_members)
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

            data_reader
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

    #[tracing::instrument(skip(self, data_message, runtime))]
    pub fn handle_data(&mut self, data_message: &[u8], runtime: &impl DdsRuntime) {
        if let Ok(rtps_message) = RtpsMessageRead::try_from(data_message) {
            let mut message_receiver = MessageReceiver::new(&rtps_message);

            while let Some(submessage) = message_receiver.next() {
                match submessage {
                    RtpsSubmessageReadKind::Data(data_submessage) => {
                        self.handle_data_submessage(&message_receiver, data_submessage);
                    }
                    RtpsSubmessageReadKind::DataFrag(data_frag_submessage) => {
                        self.handle_data_frag_submessage(&message_receiver, data_frag_submessage);
                    }
                    RtpsSubmessageReadKind::Gap(gap_submessage) => {
                        self.handle_gap_submessage(&message_receiver, gap_submessage);
                    }
                    RtpsSubmessageReadKind::Heartbeat(heartbeat_submessage) => {
                        self.handle_heartbeat_submessage(&message_receiver, heartbeat_submessage);
                    }
                    RtpsSubmessageReadKind::HeartbeatFrag(heartbeat_frag_submessage) => {
                        for dr in self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .flat_map(|s| {
                                s.data_reader_list.iter_mut().map(|dr| &mut dr.rtps_reader)
                            })
                            .chain(
                                self.domain_participant
                                    .builtin_subscriber
                                    .stateful_data_reader_list_mut()
                                    .into_iter()
                                    .map(|dr| &mut dr.reader),
                            )
                        {
                            let writer_guid = Guid::new(
                                message_receiver.source_guid_prefix(),
                                heartbeat_frag_submessage.writer_id(),
                            );
                            if let Some(writer_proxy) =
                                dr.transport_reader.matched_writer_lookup(writer_guid)
                            {
                                if writer_proxy.last_received_heartbeat_count()
                                    < heartbeat_frag_submessage.count()
                                {
                                    writer_proxy.set_last_received_heartbeat_frag_count(
                                        heartbeat_frag_submessage.count(),
                                    );
                                }
                            }
                        }
                    }
                    RtpsSubmessageReadKind::AckNack(ack_nack_submessage) => {
                        for dw in self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .flat_map(|p| p.data_writer_list.iter_mut())
                        {
                            if dw
                                .transport_writer
                                .on_acknack_submessage_received(
                                    ack_nack_submessage,
                                    message_receiver.source_guid_prefix(),
                                    self.transport.message_writer.as_ref(),
                                    &runtime.clock(),
                                )
                                .is_some()
                            {
                                if let Some(x) = dw.acknowledgement_notification.take() {
                                    x.send(());
                                }

                                if dw
                                    .transport_writer
                                    .is_change_acknowledged(dw.last_change_sequence_number)
                                {
                                    for n in dw.wait_for_acknowledgments_notification.drain(..) {
                                        n.send(Ok(()));
                                    }
                                }
                            }
                        }
                        for dw in self
                            .domain_participant
                            .builtin_publisher
                            .stateful_data_writer_list_mut()
                        {
                            dw.transport_writer.on_acknack_submessage_received(
                                ack_nack_submessage,
                                message_receiver.source_guid_prefix(),
                                self.transport.message_writer.as_ref(),
                                &runtime.clock(),
                            );
                        }
                    }
                    RtpsSubmessageReadKind::NackFrag(nack_frag_submessage) => {
                        for dw in self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .flat_map(|p| p.data_writer_list.iter_mut())
                        {
                            dw.transport_writer.on_nack_frag_submessage_received(
                                nack_frag_submessage,
                                message_receiver.source_guid_prefix(),
                                self.transport.message_writer.as_ref(),
                            );
                        }
                        for dw in self
                            .domain_participant
                            .builtin_publisher
                            .stateful_data_writer_list_mut()
                        {
                            dw.transport_writer.on_nack_frag_submessage_received(
                                nack_frag_submessage,
                                message_receiver.source_guid_prefix(),
                                self.transport.message_writer.as_ref(),
                            );
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    fn handle_data_submessage(
        &mut self,
        message_receiver: &MessageReceiver<'_>,
        data_submessage: &DataSubmessage,
    ) {
        self.domain_participant
            .builtin_subscriber
            .dcps_participant_reader
            .transport_reader
            .on_data_submessage(
                data_submessage,
                message_receiver.source_guid_prefix(),
                message_receiver.source_timestamp(),
            );
        for dr in self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .flat_map(|s| s.data_reader_list.iter_mut().map(|dr| &mut dr.rtps_reader))
            .chain(
                self.domain_participant
                    .builtin_subscriber
                    .stateful_data_reader_list_mut()
                    .into_iter()
                    .map(|dr| &mut dr.reader),
            )
        {
            dr.transport_reader.on_data_submessage(
                data_submessage,
                message_receiver.source_guid_prefix(),
                message_receiver.source_timestamp(),
            );
        }
    }

    #[inline]
    fn handle_gap_submessage(
        &mut self,
        message_receiver: &MessageReceiver<'_>,
        gap_submessage: &GapSubmessage,
    ) {
        for dr in self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .flat_map(|s| s.data_reader_list.iter_mut().map(|dr| &mut dr.rtps_reader))
            .chain(
                self.domain_participant
                    .builtin_subscriber
                    .stateful_data_reader_list_mut()
                    .into_iter()
                    .map(|dr| &mut dr.reader),
            )
        {
            let writer_guid = Guid::new(
                message_receiver.source_guid_prefix(),
                gap_submessage.writer_id(),
            );
            if let Some(writer_proxy) = dr.transport_reader.matched_writer_lookup(writer_guid) {
                for seq_num in gap_submessage.gap_start()..gap_submessage.gap_list().base() {
                    writer_proxy.irrelevant_change_set(seq_num)
                }

                for seq_num in gap_submessage.gap_list().set() {
                    writer_proxy.irrelevant_change_set(seq_num)
                }
            }
        }
    }

    fn handle_heartbeat_submessage(
        &mut self,
        message_receiver: &MessageReceiver<'_>,
        heartbeat_submessage: &HeartbeatSubmessage,
    ) {
        for dr in self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .flat_map(|s| s.data_reader_list.iter_mut().map(|dr| &mut dr.rtps_reader))
            .chain(
                self.domain_participant
                    .builtin_subscriber
                    .stateful_data_reader_list_mut()
                    .into_iter()
                    .map(|dr| &mut dr.reader),
            )
        {
            let writer_guid = Guid::new(
                message_receiver.source_guid_prefix(),
                heartbeat_submessage.writer_id(),
            );
            let reader_guid = dr.transport_reader.guid();
            if let Some(writer_proxy) = dr.transport_reader.matched_writer_lookup(writer_guid) {
                if writer_proxy.last_received_heartbeat_count() < heartbeat_submessage.count() {
                    writer_proxy.set_last_received_heartbeat_count(heartbeat_submessage.count());
                    writer_proxy.missing_changes_update(heartbeat_submessage.last_sn());
                    writer_proxy.lost_changes_update(heartbeat_submessage.first_sn());

                    let must_send_acknacks = !heartbeat_submessage.final_flag()
                        || (!heartbeat_submessage.liveliness_flag()
                            && writer_proxy.missing_changes().count() > 0);
                    writer_proxy.set_must_send_acknacks(must_send_acknacks);

                    writer_proxy
                        .write_message(&reader_guid, self.transport.message_writer.as_ref());
                }
            }
        }
    }

    fn handle_data_frag_submessage(
        &mut self,
        message_receiver: &MessageReceiver<'_>,
        data_frag_submessage: &DataFragSubmessage,
    ) {
        for dr in self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .flat_map(|s| s.data_reader_list.iter_mut().map(|dr| &mut dr.rtps_reader))
            .chain(
                self.domain_participant
                    .builtin_subscriber
                    .stateful_data_reader_list_mut()
                    .into_iter()
                    .map(|dr| &mut dr.reader),
            )
        {
            dr.transport_reader.on_data_frag_submessage(
                data_frag_submessage,
                message_receiver.source_guid_prefix(),
                message_receiver.source_timestamp(),
            );
        }
    }

    pub fn poke(&mut self, clock: &impl Clock) {
        for dw in self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .flat_map(|p| p.data_writer_list.iter_mut())
        {
            dw.transport_writer
                .write_message(self.transport.message_writer.as_ref(), clock);
        }
        for dw in self
            .domain_participant
            .builtin_publisher
            .stateful_data_writer_list_mut()
        {
            dw.transport_writer
                .write_message(self.transport.message_writer.as_ref(), clock);
        }
    }
}
