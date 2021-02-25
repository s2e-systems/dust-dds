use std::sync::{atomic, Arc, Mutex, Weak};

use rust_dds_api::{
    dcps_psm::StatusMask,
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
    types::{
        constants::{
            ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY, ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY,
        },
        EntityId, ReliabilityKind, TopicKind, GUID,
    },
};

use super::rtps_datawriter_impl::{RtpsDataWriterImpl, RtpsWriterFlavor};

struct AtomicPublisherQos {}

pub struct RtpsPublisherImpl {
    group: Group,
    writer_list: Mutex<Vec<Arc<RtpsDataWriterImpl>>>,
    writer_count: atomic::AtomicU8,
    default_datawriter_qos: DataWriterQos,
    qos: Mutex<PublisherQos>,
    listener: Option<Box<dyn PublisherListener>>,
    status_mask: StatusMask,
}

impl RtpsPublisherImpl {
    pub fn new(
        group: Group,
        qos: PublisherQos,
        listener: Option<Box<dyn PublisherListener>>,
        status_mask: StatusMask,
    ) -> Self {
        Self {
            group,
            writer_list: Default::default(),
            writer_count: atomic::AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            qos: Mutex::new(qos),
            listener,
            status_mask,
        }
    }

    pub fn create_datawriter<'a, T: DDSType>(
        &'a self,
        qos: Option<DataWriterQos>,
        a_listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
        mask: StatusMask,
    ) -> Option<Weak<RtpsDataWriterImpl>> {
        let qos = qos.unwrap_or(self.default_datawriter_qos.clone());
        qos.is_consistent().ok()?;

        let entity_key = [
            0,
            self.writer_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let guid_prefix = self.group.entity.guid.prefix();
        let entity_kind = match T::has_key() {
            true => ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY,
            false => ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
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

        let data_writer = Arc::new(RtpsDataWriterImpl::new(
            RtpsWriterFlavor::Stateful(stateful_writer),
            qos,
            a_listener,
            mask,
        ));

        self.writer_list.lock().unwrap().push(data_writer.clone());

        Some(Arc::downgrade(&data_writer))
    }

    pub fn delete_datawriter(&self, a_datawriter: &Weak<RtpsDataWriterImpl>) -> DDSResult<()> {
        let datawriter_impl = a_datawriter.upgrade().ok_or(DDSError::AlreadyDeleted)?;
        let mut writer_list = self.writer_list.lock().unwrap();
        writer_list.retain(|x| !std::ptr::eq(x.as_ref(), datawriter_impl.as_ref()));
        Ok(())
    }

    pub fn get_qos(&self) -> PublisherQos {
        self.qos.lock().unwrap().clone()
    }

    pub fn set_qos(&self, qos: Option<PublisherQos>) {
        let qos = qos.unwrap_or_default();
        *self.qos.lock().unwrap() = qos;
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use rust_dds_api::{domain::domain_participant::DomainParticipant, infrastructure::qos_policy::ReliabilityQosPolicyKind};
    // use crate::utils::node::Node;

    // struct TestType;

    // impl DDSType for TestType {
    //     fn type_name() -> &'static str {
    //         todo!()
    //     }

    //     fn has_key() -> bool {
    //         todo!()
    //     }

    //     fn key(&self) -> Vec<u8> {
    //         todo!()
    //     }

    //     fn serialize(&self) -> Vec<u8> {
    //         todo!()
    //     }

    //     fn deserialize(_data: Vec<u8>) -> Self {
    //         todo!()
    //     }
    // }

    // #[test]
    // fn create_datawriter_simple() {
    //     let publisher_impl = RtpsPublisherImpl::new();
    //     let a_topic = RtpsTopicImpl::new();
    //     let qos = None;
    //     let a_listener = None;
    //     let mask = 0;
    //     let data_writer =
    //         publisher_impl.create_datawriter::<TestType>(&a_topic, qos, a_listener, mask);
    //     assert!(data_writer.is_some());
    // }
    // #[test]
    // fn set_and_get_qos() {
    //     let publisher_list = MaybeValidList::default();
    //     let guid_prefix = [1; 12];
    //     let entity_key = [1, 2, 3];
    //     let qos = PublisherQos::default();
    //     let listener = None;
    //     let status_mask = 0;
    //     let publisher = publisher_list
    //         .add(Box::new(RtpsPublisherInner::new_user_defined(
    //             guid_prefix,
    //             entity_key,
    //             qos,
    //             listener,
    //             status_mask,
    //         )))
    //         .expect("Error creating publisher");

    //     let mut new_qos = PublisherQos::default();
    //     new_qos.partition.name = "ABCD".to_string();
    //     new_qos.presentation.coherent_access = true;
    //     publisher
    //         .set_qos(Some(new_qos.clone()))
    //         .expect("Error setting publisher QoS");
    //     assert_eq!(
    //         publisher.get_qos().expect("Error getting publisher QoS"),
    //         new_qos
    //     );
    // }

    // #[test]
    // fn set_default_qos() {
    //     let publisher_list = MaybeValidList::default();
    //     let guid_prefix = [1; 12];
    //     let entity_key = [1, 2, 3];
    //     let mut qos = PublisherQos::default();
    //     qos.partition.name = "ABCD".to_string();
    //     qos.presentation.coherent_access = true;
    //     let listener = None;
    //     let status_mask = 0;
    //     let publisher = publisher_list
    //         .add(Box::new(RtpsPublisherInner::new_user_defined(
    //             guid_prefix,
    //             entity_key,
    //             qos,
    //             listener,
    //             status_mask,
    //         )))
    //         .expect("Error creating publisher");

    //     publisher
    //         .set_qos(None)
    //         .expect("Error setting publisher QoS");
    //     assert_eq!(
    //         publisher.get_qos().expect("Error getting publisher QoS"),
    //         PublisherQos::default()
    //     );
    // }

    // #[test]
    // fn set_and_get_default_datawriter_qos() {
    //     let publisher_list = MaybeValidList::default();
    //     let guid_prefix = [1; 12];
    //     let entity_key = [1, 2, 3];
    //     let qos = PublisherQos::default();
    //     let listener = None;
    //     let status_mask = 0;
    //     let publisher = publisher_list
    //         .add(Box::new(RtpsPublisherInner::new_user_defined(
    //             guid_prefix,
    //             entity_key,
    //             qos,
    //             listener,
    //             status_mask,
    //         )))
    //         .expect("Error creating publisher");

    //     let mut datawriter_qos = DataWriterQos::default();
    //     datawriter_qos.user_data.value = vec![1, 2, 3, 4, 5];
    //     datawriter_qos.reliability.kind = ReliabilityQosPolicyKind::BestEffortReliabilityQos;
    //     publisher
    //         .set_default_datawriter_qos(Some(datawriter_qos.clone()))
    //         .expect("Error setting default datawriter QoS");
    //     assert_eq!(
    //         publisher
    //             .get_default_datawriter_qos()
    //             .expect("Error getting publisher QoS"),
    //         datawriter_qos
    //     );
    // }

    // #[test]
    // fn set_inconsistent_default_datawriter_qos() {
    //     let publisher_list = MaybeValidList::default();
    //     let guid_prefix = [1; 12];
    //     let entity_key = [1, 2, 3];
    //     let qos = PublisherQos::default();
    //     let listener = None;
    //     let status_mask = 0;
    //     let publisher = publisher_list
    //         .add(Box::new(RtpsPublisherInner::new_user_defined(
    //             guid_prefix,
    //             entity_key,
    //             qos,
    //             listener,
    //             status_mask,
    //         )))
    //         .expect("Error creating publisher");

    //     let mut datawriter_qos = DataWriterQos::default();
    //     datawriter_qos.resource_limits.max_samples_per_instance = 10;
    //     datawriter_qos.resource_limits.max_samples = 2;
    //     let result = publisher.set_default_datawriter_qos(Some(datawriter_qos.clone()));

    //     match result {
    //         Err(DDSError::InconsistentPolicy) => assert!(true),
    //         _ => assert!(false),
    //     }
    // }
}
