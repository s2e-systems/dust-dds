use alloc::string::String;
use tracing::info;

use crate::{
    dcps::{
        dcps_domain_participant::{
            AddChangeResult, DcpsDomainParticipant, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER, ENTITYID_TL_SVC_REPLY_READER,
            ENTITYID_TL_SVC_REQ_READER, RtpsReaderKind, RtpsWriterKind, TopicDescriptionKind,
        },
        dcps_mail::{DcpsMail, EventServiceMail},
        listeners::domain_participant_listener::ListenerMail,
        xtypes_glue::key_and_instance_handle::{
            KeyHolderType, get_instance_handle_from_dynamic_data,
        },
    },
    dds_async::{
        data_reader::DataReaderAsync, domain_participant::DomainParticipantAsync,
        subscriber::SubscriberAsync, topic::TopicAsync, topic_description::TopicDescriptionAsync,
    },
    infrastructure::{instance::InstanceHandle, status::StatusKind, time::DurationKind},
    rtps::message_receiver::MessageReceiver,
    rtps_messages::{
        overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
        submessages::{
            data::DataSubmessage, data_frag::DataFragSubmessage, gap::GapSubmessage,
            heartbeat::HeartbeatSubmessage,
        },
    },
    runtime::{Clock, DdsRuntime, Spawner, Timer},
    subscription::data_reader::DataReader,
    transport::types::{CacheChange, ChangeKind, Guid},
    xtypes::deserializer::deserialize_top_level_type,
};

impl DcpsDomainParticipant {
    pub fn process_received_cache_changes(
        &mut self,
        // cache_change: &CacheChange,
        // subscriber_handle: &InstanceHandle,
        // data_reader_handle: &InstanceHandle,
        runtime: &impl DdsRuntime,
    ) {
        let reception_timestamp = runtime.clock().now();
        for subscriber in self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .chain(core::iter::once(
                &mut self.domain_participant.builtin_subscriber,
            ))
        {
            for data_reader in &mut subscriber.data_reader_list {
                let changes = match &mut data_reader.transport_reader {
                    RtpsReaderKind::Stateful(r) => core::mem::take(r.changes_mut()),
                    RtpsReaderKind::Stateless(r) => core::mem::take(r.changes_mut()),
                };
                let subscriber_handle = &subscriber.instance_handle.clone();
                let data_reader_handle = &data_reader.instance_handle.clone();

                for cache_change in changes {
                    let writer_instance_handle =
                        InstanceHandle::new(cache_change.writer_guid.into());

                    if data_reader
                        .matched_publication_list
                        .iter()
                        .any(|x| &x.key().value == writer_instance_handle.as_ref())
                    {
                        let Some(reader_topic) = self
                            .domain_participant
                            .topic_description_list
                            .iter()
                            .find(|t| t.topic_name() == data_reader.topic_name)
                        else {
                            return;
                        };

                        if let TopicDescriptionKind::ContentFilteredTopic(content_filtered_topic) =
                            reader_topic
                        {
                            if cache_change.kind == ChangeKind::Alive {
                                let Ok(data) = deserialize_top_level_type(
                                    data_reader.type_support,
                                    cache_change.data_value.as_ref(),
                                ) else {
                                    return;
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
                                        return;
                                    };
                                    let Ok(member_descriptor) = data.get_descriptor(member_id)
                                    else {
                                        return;
                                    };
                                    match member_descriptor.r#type.get_kind() {
                                        crate::xtypes::dynamic_type::TypeKind::NONE => todo!(),
                                        crate::xtypes::dynamic_type::TypeKind::BOOLEAN => todo!(),
                                        crate::xtypes::dynamic_type::TypeKind::BYTE => todo!(),
                                        crate::xtypes::dynamic_type::TypeKind::INT16 => todo!(),
                                        crate::xtypes::dynamic_type::TypeKind::INT32 => {
                                            let member_value =
                                                data.get_int32_value(member_id).unwrap();
                                            if !comparison_function.compare_int32(
                                                member_value,
                                                &content_filtered_topic.expression_parameters[0]
                                                    .parse()
                                                    .expect("valid number"),
                                            ) {
                                                return;
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
                                        crate::xtypes::dynamic_type::TypeKind::STRING8 => {
                                            let member_value =
                                                data.get_string_value(member_id).unwrap();
                                            if !comparison_function.compare_string(
                                                member_value,
                                                &content_filtered_topic.expression_parameters[0],
                                            ) {
                                                return;
                                            }
                                        }
                                        crate::xtypes::dynamic_type::TypeKind::STRING16 => todo!(),
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
                                    return;
                                };
                            }
                        }

                        let participant_handle = self.domain_participant.instance_handle;
                        let (dynamic_data, change_instance_handle) = match cache_change.kind {
                            ChangeKind::Alive | ChangeKind::AliveFiltered => {
                                let Ok(data_value) = deserialize_top_level_type(
                                    data_reader.type_support,
                                    cache_change.data_value.as_ref(),
                                ) else {
                                    tracing::warn!("Failed to deserialize user defined data");
                                    return;
                                };
                                let Ok(instance_handle) =
                                    get_instance_handle_from_dynamic_data(&data_value)
                                else {
                                    tracing::warn!(
                                        "Failed to get instance handle from dynamic_data"
                                    );
                                    return;
                                };
                                (Some(data_value), instance_handle)
                            }
                            ChangeKind::NotAliveDisposed
                            | ChangeKind::NotAliveUnregistered
                            | ChangeKind::NotAliveDisposedUnregistered => {
                                match cache_change.instance_handle {
                                    Some(i) => {
                                        let instance_handle = InstanceHandle::new(i);
                                        (None, instance_handle)
                                    }
                                    None => {
                                        let Ok(key_holder) = KeyHolderType::from_dynamic_type(
                                            &data_reader.type_support,
                                        ) else {
                                            tracing::warn!("Failed to create key holder");
                                            return;
                                        };

                                        let Ok(data_value) = deserialize_top_level_type(
                                            *key_holder.as_dynamic_type(),
                                            cache_change.data_value.as_ref(),
                                        ) else {
                                            tracing::warn!(
                                                "Failed to deserialize disposed user defined data"
                                            );
                                            return;
                                        };

                                        let Ok(instance_handle) =
                                            get_instance_handle_from_dynamic_data(&data_value)
                                        else {
                                            tracing::warn!(
                                                "Failed to deserialize disposed key user defined data"
                                            );
                                            return;
                                        };
                                        (None, instance_handle)
                                    }
                                }
                            }
                        };

                        let the_participant = DomainParticipantAsync::new(
                            self.dcps_sender.clone(),
                            self.domain_participant.domain_id,
                            self.domain_participant.instance_handle,
                        );
                        let the_subscriber =
                            SubscriberAsync::new(*subscriber_handle, the_participant.clone());
                        let the_topic = match reader_topic {
                            TopicDescriptionKind::Topic(topic_entity) => {
                                TopicDescriptionAsync::Topic(TopicAsync::new(
                                    topic_entity.instance_handle,
                                    topic_entity.type_name.clone(),
                                    topic_entity.topic_name.clone(),
                                    the_participant.clone(),
                                ))
                            }
                            TopicDescriptionKind::ContentFilteredTopic(_) => todo!(),
                        };
                        let the_reader = DataReaderAsync::new(
                            *data_reader_handle,
                            the_subscriber.clone(),
                            the_topic,
                        );

                        match data_reader.add_reader_change(
                            cache_change.writer_guid,
                            dynamic_data,
                            cache_change.kind,
                            change_instance_handle.into(),
                            cache_change.source_timestamp.map(Into::into),
                            reception_timestamp,
                        ) {
                            Ok(AddChangeResult::Added(change_instance_handle)) => {
                                info!("New change added");
                                if let DurationKind::Finite(deadline_missed_period) =
                                    data_reader.qos.deadline.period
                                {
                                    let dcps_sender = self.dcps_sender;

                                    let mut timer_handle = runtime.timer();
                                    let subscriber_handle = *subscriber_handle;
                                    let data_reader_handle = *data_reader_handle;
                                    runtime.spawner().spawn(async move {
                                        loop {
                                            timer_handle.delay(deadline_missed_period.into()).await;
                                            dcps_sender
                                                .send(DcpsMail::Event(
                                                    EventServiceMail::RequestedDeadlineMissed {
                                                        participant_handle,
                                                        subscriber_handle,
                                                        data_reader_handle,
                                                        change_instance_handle,
                                                    },
                                                ))
                                                .await;
                                        }
                                    });
                                }
                                let data_reader_on_data_available_active = data_reader
                                    .listener_mask
                                    .is_enabled(&StatusKind::DataAvailable);

                                if subscriber
                                    .listener_mask
                                    .is_enabled(&StatusKind::DataOnReaders)
                                {
                                    if let Some(l) = &subscriber.listener_sender {
                                        l.send(ListenerMail::DataOnReaders { the_subscriber }).ok();
                                    }
                                } else if data_reader_on_data_available_active {
                                    if let Some(l) = &data_reader.listener_sender {
                                        info!("Triggering data reader DataAvailable listener");
                                        l.send(ListenerMail::DataAvailable { the_reader }).ok();
                                    }
                                }

                                subscriber
                                    .status_condition
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
                                info!("Change rejected");
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
                                } else if subscriber
                                    .listener_mask
                                    .is_enabled(&StatusKind::SampleRejected)
                                {
                                    let status = data_reader.get_sample_rejected_status();
                                    if let Some(l) = &subscriber.listener_sender {
                                        l.send(ListenerMail::SampleRejected { status, the_reader })
                                            .ok();
                                    }
                                } else if self
                                    .domain_participant
                                    .listener_mask
                                    .is_enabled(&StatusKind::SampleRejected)
                                {
                                    let status = data_reader.get_sample_rejected_status();
                                    if let Some(l) = &self.domain_participant.listener_sender {
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
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn remove_writer_change(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        sequence_number: i64,
    ) {
        if let Some(p) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        {
            if let Some(dw) = p
                .data_writer_list
                .iter_mut()
                .find(|x| x.instance_handle == data_writer_handle)
            {
                dw.transport_writer.remove_change(sequence_number);
            }
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
                        for subscriber in self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .chain(core::iter::once(
                                &mut self.domain_participant.builtin_subscriber,
                            ))
                        {
                            for dr in &mut subscriber.data_reader_list {
                                match &mut dr.transport_reader {
                                    RtpsReaderKind::Stateful(r) => {
                                        let writer_guid = Guid::new(
                                            message_receiver.source_guid_prefix(),
                                            heartbeat_frag_submessage.writer_id(),
                                        );
                                        if let Some(writer_proxy) =
                                            r.matched_writer_lookup(writer_guid)
                                        {
                                            if writer_proxy.last_received_heartbeat_count()
                                                < heartbeat_frag_submessage.count()
                                            {
                                                writer_proxy
                                                    .set_last_received_heartbeat_frag_count(
                                                        heartbeat_frag_submessage.count(),
                                                    );
                                            }
                                        }
                                    }
                                    RtpsReaderKind::Stateless(_) => (),
                                }
                            }
                        }
                    }
                    RtpsSubmessageReadKind::AckNack(ack_nack_submessage) => {
                        for publisher in self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .chain(core::iter::once(
                                &mut self.domain_participant.builtin_publisher,
                            ))
                        {
                            for dw in &mut publisher.data_writer_list {
                                match &mut dw.transport_writer {
                                    RtpsWriterKind::Stateful(w) => {
                                        if w.on_acknack_submessage_received(
                                            ack_nack_submessage,
                                            message_receiver.source_guid_prefix(),
                                            self.transport.message_writer.as_ref(),
                                            &runtime.clock(),
                                        )
                                        .is_some()
                                        {
                                            if let Some(x) = dw.acknowledgement_notification.take()
                                            {
                                                x.send(());
                                            }

                                            if w.is_change_acknowledged(
                                                dw.last_change_sequence_number,
                                            ) {
                                                for n in dw
                                                    .wait_for_acknowledgments_notification
                                                    .drain(..)
                                                {
                                                    n.send(Ok(()));
                                                }
                                            }
                                        }
                                    }
                                    RtpsWriterKind::Stateless(_) => (),
                                }
                            }
                        }
                    }
                    RtpsSubmessageReadKind::NackFrag(nack_frag_submessage) => {
                        for publisher in self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .chain(core::iter::once(
                                &mut self.domain_participant.builtin_publisher,
                            ))
                        {
                            for dw in &mut publisher.data_writer_list {
                                match &mut dw.transport_writer {
                                    RtpsWriterKind::Stateful(w) => w
                                        .on_nack_frag_submessage_received(
                                            nack_frag_submessage,
                                            message_receiver.source_guid_prefix(),
                                            self.transport.message_writer.as_ref(),
                                        ),
                                    RtpsWriterKind::Stateless(_) => (),
                                }
                            }
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
        for subscriber in self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .chain(core::iter::once(
                &mut self.domain_participant.builtin_subscriber,
            ))
        {
            for dr in &mut subscriber.data_reader_list {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => {
                        r.on_data_submessage(
                            data_submessage,
                            message_receiver.source_guid_prefix(),
                            message_receiver.source_timestamp(),
                        );
                    }
                    RtpsReaderKind::Stateless(r) => {
                        r.on_data_submessage(
                            data_submessage,
                            message_receiver.source_guid_prefix(),
                            message_receiver.source_timestamp(),
                        );
                    }
                }
            }
        }
    }

    #[inline]
    fn handle_gap_submessage(
        &mut self,
        message_receiver: &MessageReceiver<'_>,
        gap_submessage: &GapSubmessage,
    ) {
        for subscriber in self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .chain(core::iter::once(
                &mut self.domain_participant.builtin_subscriber,
            ))
        {
            for dr in &mut subscriber.data_reader_list {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => {
                        let writer_guid = Guid::new(
                            message_receiver.source_guid_prefix(),
                            gap_submessage.writer_id(),
                        );
                        if let Some(writer_proxy) = r.matched_writer_lookup(writer_guid) {
                            for seq_num in
                                gap_submessage.gap_start()..gap_submessage.gap_list().base()
                            {
                                writer_proxy.irrelevant_change_set(seq_num)
                            }

                            for seq_num in gap_submessage.gap_list().set() {
                                writer_proxy.irrelevant_change_set(seq_num)
                            }
                        }
                    }
                    RtpsReaderKind::Stateless(_) => (),
                }
            }
        }
    }

    fn handle_heartbeat_submessage(
        &mut self,
        message_receiver: &MessageReceiver<'_>,
        heartbeat_submessage: &HeartbeatSubmessage,
    ) {
        for subscriber in self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .chain(core::iter::once(
                &mut self.domain_participant.builtin_subscriber,
            ))
        {
            for dr in &mut subscriber.data_reader_list {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => {
                        let writer_guid = Guid::new(
                            message_receiver.source_guid_prefix(),
                            heartbeat_submessage.writer_id(),
                        );
                        let reader_guid = r.guid();
                        if let Some(writer_proxy) = r.matched_writer_lookup(writer_guid) {
                            if writer_proxy.last_received_heartbeat_count()
                                < heartbeat_submessage.count()
                            {
                                writer_proxy.set_last_received_heartbeat_count(
                                    heartbeat_submessage.count(),
                                );
                                writer_proxy.missing_changes_update(heartbeat_submessage.last_sn());
                                writer_proxy.lost_changes_update(heartbeat_submessage.first_sn());

                                let must_send_acknacks = !heartbeat_submessage.final_flag()
                                    || (!heartbeat_submessage.liveliness_flag()
                                        && writer_proxy.missing_changes().count() > 0);
                                writer_proxy.set_must_send_acknacks(must_send_acknacks);

                                writer_proxy.write_message(
                                    &reader_guid,
                                    self.transport.message_writer.as_ref(),
                                );
                            }
                        }
                    }
                    RtpsReaderKind::Stateless(_) => (),
                }
            }
        }
    }

    fn handle_data_frag_submessage(
        &mut self,
        message_receiver: &MessageReceiver<'_>,
        data_frag_submessage: &DataFragSubmessage,
    ) {
        for subscriber in self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .chain(core::iter::once(
                &mut self.domain_participant.builtin_subscriber,
            ))
        {
            for dr in &mut subscriber.data_reader_list {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => {
                        r.on_data_frag_submessage(
                            data_frag_submessage,
                            message_receiver.source_guid_prefix(),
                            message_receiver.source_timestamp(),
                        );
                    }
                    RtpsReaderKind::Stateless(_) => (),
                }
            }
        }
    }

    pub fn poke(&mut self, clock: &impl Clock) {
        for publisher in self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .chain(core::iter::once(
                &mut self.domain_participant.builtin_publisher,
            ))
        {
            for dw in &mut publisher.data_writer_list {
                match &mut dw.transport_writer {
                    RtpsWriterKind::Stateful(writer) => {
                        writer.write_message(self.transport.message_writer.as_ref(), clock)
                    }
                    RtpsWriterKind::Stateless(_writer) => {}
                }
            }
        }
    }
}
