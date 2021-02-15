use std::sync::{atomic, Mutex};

use rust_dds_api::{
    dcps_psm::{InstanceHandle, StatusMask},
    dds_type::DDSType,
    infrastructure::{
        qos::{DataWriterQos, PublisherQos},
        qos_policy::ReliabilityQosPolicyKind,
    },
    publication::{
        data_writer_listener::DataWriterListener, publisher_listener::PublisherListener,
    },
    return_type::{DDSError, DDSResult},
};
use rust_rtps::{
    behavior::StatefulWriter,
    structure::Group,
    transport::Transport,
    types::{
        constants::{
            ENTITY_KIND_BUILT_IN_WRITER_GROUP, ENTITY_KIND_USER_DEFINED_WRITER_GROUP,
            ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY, ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY,
        },
        EntityId, GuidPrefix, ReliabilityKind, TopicKind, GUID,
    },
};

use crate::utils::maybe_valid::{MaybeValid, MaybeValidList, MaybeValidRef};

use super::{
    message_sender::RtpsMessageSender,
    rtps_datawriter_impl::{RtpsAnyDataWriterImplRef, RtpsDataWriterImpl, RtpsWriterFlavor},
    rtps_topic_inner::RtpsTopicInnerRef,
};

enum EntityType {
    BuiltIn,
    UserDefined,
}

pub struct RtpsPublisherInner {
    group: Group,
    entity_type: EntityType,
    writer_list: MaybeValidList<RtpsDataWriterImpl>,
    writer_count: atomic::AtomicU8,
    default_datawriter_qos: Mutex<DataWriterQos>,
    qos: Mutex<PublisherQos>,
    listener: Option<Box<dyn PublisherListener>>,
    status_mask: StatusMask,
}

impl RtpsPublisherInner {
    pub fn new_builtin(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        qos: PublisherQos,
        listener: Option<Box<dyn PublisherListener>>,
        status_mask: StatusMask,
    ) -> Self {
        Self::new(
            guid_prefix,
            entity_key,
            qos,
            listener,
            status_mask,
            EntityType::BuiltIn,
        )
    }

    pub fn new_user_defined(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        qos: PublisherQos,
        listener: Option<Box<dyn PublisherListener>>,
        status_mask: StatusMask,
    ) -> Self {
        Self::new(
            guid_prefix,
            entity_key,
            qos,
            listener,
            status_mask,
            EntityType::UserDefined,
        )
    }

    fn new(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        qos: PublisherQos,
        listener: Option<Box<dyn PublisherListener>>,
        status_mask: StatusMask,
        entity_type: EntityType,
    ) -> Self {
        let entity_id = match entity_type {
            EntityType::BuiltIn => EntityId::new(entity_key, ENTITY_KIND_BUILT_IN_WRITER_GROUP),
            EntityType::UserDefined => {
                EntityId::new(entity_key, ENTITY_KIND_USER_DEFINED_WRITER_GROUP)
            }
        };
        let guid = GUID::new(guid_prefix, entity_id);

        Self {
            group: Group::new(guid),
            entity_type,
            writer_list: Default::default(),
            writer_count: atomic::AtomicU8::new(0),
            default_datawriter_qos: Mutex::new(DataWriterQos::default()),
            qos: Mutex::new(qos),
            listener,
            status_mask,
        }
    }

    pub fn writer_list(&self) -> &MaybeValidList<RtpsDataWriterImpl> {
        &self.writer_list
    }
}

pub type RtpsPublisherInnerRef<'a> = MaybeValidRef<'a, Box<RtpsPublisherInner>>;

impl<'a> RtpsPublisherInnerRef<'a> {
    fn get(&self) -> DDSResult<&Box<RtpsPublisherInner>> {
        MaybeValid::get(self).ok_or(DDSError::AlreadyDeleted)
    }

    pub fn delete(&self) -> DDSResult<()> {
        if self.get()?.writer_list.is_empty() {
            MaybeValid::delete(self);
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher still contains data writers",
            ))
        }
    }

    pub fn create_datawriter<T: DDSType>(
        &self,
        a_topic: &RtpsTopicInnerRef,
        qos: Option<DataWriterQos>,
        a_listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Option<RtpsAnyDataWriterImplRef> {
        let this = self.get().ok()?;
        let topic = a_topic.get().ok()?;
        let qos = qos.unwrap_or(this.default_datawriter_qos.lock().unwrap().clone());
        qos.is_consistent().ok()?;

        let entity_key = [
            0,
            this.writer_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let guid_prefix = this.group.entity.guid.prefix();
        let entity_kind = match T::has_key() {
            true => ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY,
            false => ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY,
        };
        let guid = GUID::new(guid_prefix, EntityId::new(entity_key, entity_kind));
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let topic_kind = match T::has_key() {
            true => TopicKind::WithKey,
            false => TopicKind::NoKey,
        };
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let push_mode = true;
        let heartbeat_period = rust_rtps::behavior::types::Duration::from_millis(500);
        let nack_response_delay = rust_rtps::behavior::types::constants::DURATION_ZERO;
        let nack_suppression_duration = rust_rtps::behavior::types::constants::DURATION_ZERO;
        let data_max_sized_serialized = None;
        let stateful_writer = StatefulWriter::new(
            guid,
            unicast_locator_list,
            multicast_locator_list,
            topic_kind,
            reliability_level,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_sized_serialized,
        );

        let data_writer_impl = RtpsDataWriterImpl::new(
            RtpsWriterFlavor::Stateful(stateful_writer),
            topic,
            qos,
            a_listener,
            status_mask,
        );

        this.writer_list.add(data_writer_impl)
    }

    pub fn set_default_datawriter_qos(&self, qos: Option<DataWriterQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        *self.get()?.default_datawriter_qos.lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_default_datawriter_qos(&self) -> DDSResult<DataWriterQos> {
        Ok(self.get()?.default_datawriter_qos.lock().unwrap().clone())
    }

    pub fn get_qos(&self) -> DDSResult<PublisherQos> {
        Ok(self.get()?.qos.lock().unwrap().clone())
    }

    pub fn set_qos(&self, qos: Option<PublisherQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        *self.get()?.qos.lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }

    pub fn send_data(&self, transport: &dyn Transport) {
        if let Some(publisher) = self.get().ok() {
            for writer in publisher.writer_list.into_iter() {
                let destined_messages = writer.produce_messages();
                let participant_guid_prefix = publisher.group.entity.guid.prefix();
                RtpsMessageSender::send_cache_change_messages(
                    participant_guid_prefix,
                    transport,
                    destined_messages,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_dds_api::infrastructure::qos_policy::ReliabilityQosPolicyKind;

    #[test]
    fn set_and_get_qos() {
        let publisher_list = MaybeValidList::default();
        let guid_prefix = [1; 12];
        let entity_key = [1, 2, 3];
        let qos = PublisherQos::default();
        let listener = None;
        let status_mask = 0;
        let publisher = publisher_list
            .add(Box::new(RtpsPublisherInner::new_user_defined(
                guid_prefix,
                entity_key,
                qos,
                listener,
                status_mask,
            )))
            .expect("Error creating publisher");

        let mut new_qos = PublisherQos::default();
        new_qos.partition.name = "ABCD".to_string();
        new_qos.presentation.coherent_access = true;
        publisher
            .set_qos(Some(new_qos.clone()))
            .expect("Error setting publisher QoS");
        assert_eq!(
            publisher.get_qos().expect("Error getting publisher QoS"),
            new_qos
        );
    }

    #[test]
    fn set_default_qos() {
        let publisher_list = MaybeValidList::default();
        let guid_prefix = [1; 12];
        let entity_key = [1, 2, 3];
        let mut qos = PublisherQos::default();
        qos.partition.name = "ABCD".to_string();
        qos.presentation.coherent_access = true;
        let listener = None;
        let status_mask = 0;
        let publisher = publisher_list
            .add(Box::new(RtpsPublisherInner::new_user_defined(
                guid_prefix,
                entity_key,
                qos,
                listener,
                status_mask,
            )))
            .expect("Error creating publisher");

        publisher
            .set_qos(None)
            .expect("Error setting publisher QoS");
        assert_eq!(
            publisher.get_qos().expect("Error getting publisher QoS"),
            PublisherQos::default()
        );
    }

    #[test]
    fn set_and_get_default_datawriter_qos() {
        let publisher_list = MaybeValidList::default();
        let guid_prefix = [1; 12];
        let entity_key = [1, 2, 3];
        let qos = PublisherQos::default();
        let listener = None;
        let status_mask = 0;
        let publisher = publisher_list
            .add(Box::new(RtpsPublisherInner::new_user_defined(
                guid_prefix,
                entity_key,
                qos,
                listener,
                status_mask,
            )))
            .expect("Error creating publisher");

        let mut datawriter_qos = DataWriterQos::default();
        datawriter_qos.user_data.value = vec![1, 2, 3, 4, 5];
        datawriter_qos.reliability.kind = ReliabilityQosPolicyKind::BestEffortReliabilityQos;
        publisher
            .set_default_datawriter_qos(Some(datawriter_qos.clone()))
            .expect("Error setting default datawriter QoS");
        assert_eq!(
            publisher
                .get_default_datawriter_qos()
                .expect("Error getting publisher QoS"),
            datawriter_qos
        );
    }

    #[test]
    fn set_inconsistent_default_datawriter_qos() {
        let publisher_list = MaybeValidList::default();
        let guid_prefix = [1; 12];
        let entity_key = [1, 2, 3];
        let qos = PublisherQos::default();
        let listener = None;
        let status_mask = 0;
        let publisher = publisher_list
            .add(Box::new(RtpsPublisherInner::new_user_defined(
                guid_prefix,
                entity_key,
                qos,
                listener,
                status_mask,
            )))
            .expect("Error creating publisher");

        let mut datawriter_qos = DataWriterQos::default();
        datawriter_qos.resource_limits.max_samples_per_instance = 10;
        datawriter_qos.resource_limits.max_samples = 2;
        let result = publisher.set_default_datawriter_qos(Some(datawriter_qos.clone()));

        match result {
            Err(DDSError::InconsistentPolicy) => assert!(true),
            _ => assert!(false),
        }
    }
}
