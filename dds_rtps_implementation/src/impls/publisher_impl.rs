use std::sync::atomic;

use rust_dds_api::{
    dcps_psm::StatusMask,
    infrastructure::qos::{DataWriterQos, PublisherQos},
    publication::publisher_listener::PublisherListener,
};
use rust_rtps::{
    structure::{RTPSEntity, RTPSGroup},
    types::GUID,
};

pub struct PublisherImpl {
    // writer_list: Vec<Arc<Mutex<StatefulDataWriterImpl<Writer<HistoryCache<CacheChange>>>>>>,
    writer_count: atomic::AtomicU8,
    default_datawriter_qos: DataWriterQos,
    qos: PublisherQos,
    listener: Option<Box<dyn PublisherListener>>,
    status_mask: StatusMask,
}

impl PublisherImpl {
    pub fn new(
        qos: PublisherQos,
        listener: Option<Box<dyn PublisherListener>>,
        status_mask: StatusMask,
    ) -> Self {
        Self {
            // writer_list: Default::default(),
            writer_count: atomic::AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            qos,
            listener,
            status_mask,
        }
    }

    // pub fn writer_list(
    //     &self,
    // ) -> &Vec<Arc<Mutex<StatefulDataWriterImpl<Writer<HistoryCache<CacheChange>>>>>> {
    //     todo!()
    //     // &self.writer_list
    // }

    // pub fn create_datawriter<'a, T: DDSType>(
    //     &'a mut self,
    //     _topic: Arc<Mutex<TopicImpl>>,
    //     qos: Option<DataWriterQos>,
    //     _a_listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
    //     _mask: StatusMask,
    // ) -> Option<Weak<Mutex<StatefulDataWriterImpl<Writer<HistoryCache<CacheChange>>>>>> {
    //     let qos = qos.unwrap_or(self.default_datawriter_qos.clone());
    //     qos.is_consistent().ok()?;

    //     todo!()

    //     // let data_writer = Arc::new(Mutex::new(StatefulDataWriterImpl::new(
    //     //     topic, qos, a_listener, mask,
    //     // )));

    //     // self.writer_list.push(data_writer.clone());

    //     // Some(Arc::downgrade(&data_writer))
    // }

    // pub fn delete_datawriter(
    //     &mut self,
    //     a_datawriter: &Weak<Mutex<StatefulDataWriterImpl<Writer<HistoryCache<CacheChange>>>>>,
    // ) -> DDSResult<()> {
    //     todo!()
    //     // let datawriter_impl = a_datawriter.upgrade().ok_or(DDSError::AlreadyDeleted)?;
    //     // self.writer_list
    //     //     .retain(|x| !std::ptr::eq(x.as_ref(), datawriter_impl.as_ref()));
    //     // Ok(())
    // }

    pub fn get_qos(&self) -> PublisherQos {
        self.qos.clone()
    }

    pub fn set_qos(&mut self, qos: Option<PublisherQos>) {
        let qos = qos.unwrap_or_default();
        self.qos = qos;
    }
}

impl RTPSEntity for PublisherImpl {
    fn guid(&self) -> GUID {
        todo!()
    }
}

impl RTPSGroup for PublisherImpl {}

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
