use std::sync::{Arc, Mutex};

use rust_dds_api::{
    dcps_psm::{InstanceHandle, StatusMask},
    infrastructure::qos::{DataWriterQos, PublisherQos},
    publication::publisher_listener::PublisherListener,
    return_type::DDSResult,
};
use rust_rtps_pim::structure::types::GUID;

use crate::dds_impl::writer_factory::WriterFactory;

use super::rtps_writer_impl::RTPSWriterImpl;

pub struct RTPSWriterGroupImpl<'a, PSM: rust_rtps_pim::PIM> {
    guid: GUID<PSM>,
    qos: PublisherQos<'a>,
    listener: Option<&'a (dyn PublisherListener + 'a)>,
    status_mask: StatusMask,
    writer_list: Vec<Arc<Mutex<RTPSWriterImpl<PSM>>>>,
    writer_factory: WriterFactory<'a, PSM>,
    default_datawriter_qos: Mutex<DataWriterQos<'a>>,
}

impl<'a, PSM: rust_rtps_pim::PIM> RTPSWriterGroupImpl<'a, PSM> {
    pub fn new(
        guid: GUID<PSM>,
        qos: PublisherQos<'a>,
        listener: Option<&'a (dyn PublisherListener + 'a)>,
        status_mask: StatusMask,
    ) -> Self {
        Self {
            guid,
            qos,
            listener,
            status_mask,
            writer_list: Vec::new(),
            writer_factory: WriterFactory::new(*guid.prefix()),
            default_datawriter_qos: Mutex::new(DataWriterQos::default()),
        }
    }

    pub fn delete_datawriter(&self, handle: &InstanceHandle) -> DDSResult<()> {
        Ok(())
    }

    // pub fn create_datawriter(&self) -> RtpsWeak<RTPSWriterImpl<PSM>>{
    //     todo!()
    // }

    //     pub fn produce_messages(
    //         &self,
    //         send_data_to: &mut impl FnMut(&Locator<RtpsUdpPsm>, submessages::Data),
    //         send_gap_to: &mut impl FnMut(&Locator<RtpsUdpPsm>, submessages::Gap),
    //     ) -> () {
    //         for stateless_writer in &self.stateless_writer_list {
    //             let mut stateless_writer_lock = stateless_writer.lock().unwrap();
    //             stateless_writer_lock.produce_messages(send_data_to, send_gap_to);
    //         }
    //     }

    //     pub fn stateless_writer_add(&mut self, stateless_writer: StatelessDataWriterImpl) {
    //         self.stateless_writer_list
    //             .push(Arc::new(Mutex::new(stateless_writer)))
    //     }

    //     pub fn create_datawriter(&self) -> Option<Weak<Mutex<StatefulDataWriterImpl>>> {
    //         // To be called for user-defined entity creation

    //         // let entity_id = EntityId::new([0,0,0], ENTITYKIN);
    //         // let guid = GUID::new(self.guid().prefix(), entity_id);
    //         // let topic_kind = ();
    //         // let reliability_level = ();
    //         // let unicast_locator_list = ();
    //         // let multicast_locator_list = ();
    //         // let push_mode = ();
    //         // let heartbeat_period = ();
    //         // let nack_response_delay = ();
    //         // let nack_suppression_duration = ();
    //         // let data_max_sized_serialized = ();

    //         // let writer = Writer::new(
    //         //     guid,
    //         //     topic_kind,
    //         //     reliability_level,
    //         //     unicast_locator_list,
    //         //     multicast_locator_list,
    //         //     push_mode,
    //         //     heartbeat_period,
    //         //     nack_response_delay,
    //         //     nack_suppression_duration,
    //         //     data_max_sized_serialized,
    //         // );
    //         // let stateless_writer = StatelessWriter::new(writer);
    //         // let datawriter_impl = DataWriterImpl::new(stateless_writer);
    //         todo!()
    //     }

    //     // pub fn create_datawriter<'a, T: DDSType>(
    //     //     &'a mut self,
    //     //     _topic: Arc<Mutex<TopicImpl>>,
    //     //     qos: Option<DataWriterQos>,
    //     //     _a_listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
    //     //     _mask: StatusMask,
    //     // ) -> Option<Weak<Mutex<StatefulDataWriterImpl<Writer<HistoryCache<CacheChange>>>>>> {
    //     //     let qos = qos.unwrap_or(self.default_datawriter_qos.clone());
    //     //     qos.is_consistent().ok()?;

    //     //     todo!()

    //     //     // let data_writer = Arc::new(Mutex::new(StatefulDataWriterImpl::new(
    //     //     //     topic, qos, a_listener, mask,
    //     //     // )));

    //     //     // self.writer_list.push(data_writer.clone());

    //     //     // Some(Arc::downgrade(&data_writer))
    //     // }

    //     // pub fn delete_datawriter(
    //     //     &mut self,
    //     //     a_datawriter: &Weak<Mutex<StatefulDataWriterImpl<Writer<HistoryCache<CacheChange>>>>>,
    //     // ) -> DDSResult<()> {
    //     //     todo!()
    //     //     // let datawriter_impl = a_datawriter.upgrade().ok_or(DDSError::AlreadyDeleted)?;
    //     //     // self.writer_list
    //     //     //     .retain(|x| !std::ptr::eq(x.as_ref(), datawriter_impl.as_ref()));
    //     //     // Ok(())
    //     // }

    //     pub fn get_qos(&self) -> PublisherQos {
    //         self.qos.clone()
    //     }

    //     pub fn set_qos(&mut self, qos: Option<PublisherQos>) {
    //         let qos = qos.unwrap_or_default();
    //         self.qos = qos;
    //     }
}

impl<'p, 'dp: 'p, PSM: rust_rtps_pim::PIM> rust_dds_api::publication::publisher::Publisher<'p, 'dp>
    for RTPSWriterGroupImpl<'dp, PSM>
{
    fn suspend_publications(&self) -> DDSResult<()> {
        todo!()
    }

    fn resume_publications(&self) -> DDSResult<()> {
        todo!()
    }

    fn begin_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(
        &self,
        max_wait: rust_dds_api::dcps_psm::Duration,
    ) -> DDSResult<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datawriter_qos(&self, qos: Option<DataWriterQos<'dp>>) -> DDSResult<()> {
        let datawriter_qos = qos.unwrap_or_default();
        datawriter_qos.is_consistent()?;
        *self.default_datawriter_qos.lock().unwrap() = datawriter_qos;
        Ok(())
    }

    fn get_default_datawriter_qos(&self) -> DataWriterQos<'dp> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos<'dp>,
        _a_topic_qos: &rust_dds_api::infrastructure::qos::TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'a, PSM: rust_rtps_pim::PIM> rust_dds_api::infrastructure::entity::Entity
    for RTPSWriterGroupImpl<'a, PSM>
{
    type Qos = PublisherQos<'a>;
    type Listener = &'a (dyn PublisherListener + 'a);

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        todo!()
    }

    fn set_listener(&self, _a_listener: Option<Self::Listener>, _mask: StatusMask) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> rust_dds_api::infrastructure::entity::StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        Ok(crate::utils::instance_handle_from_guid(&self.guid))
    }
}

impl<'a, 'b: 'a, PSM: rust_rtps_pim::PIM> rust_rtps_pim::structure::RTPSEntity<PSM>
    for RTPSWriterGroupImpl<'a, PSM>
{
    fn guid(&self) -> GUID<PSM> {
        self.guid
    }
}

// #[cfg(test)]
// mod tests {
//     // use super::*;
//     // use rust_dds_api::{domain::domain_participant::DomainParticipant, infrastructure::qos_policy::ReliabilityQosPolicyKind};
//     // use crate::utils::node::Node;

//     // struct TestType;

//     // impl DDSType for TestType {
//     //     fn type_name() -> &'static str {
//     //         todo!()
//     //     }

//     //     fn has_key() -> bool {
//     //         todo!()
//     //     }

//     //     fn key(&self) -> Vec<u8> {
//     //         todo!()
//     //     }

//     //     fn serialize(&self) -> Vec<u8> {
//     //         todo!()
//     //     }

//     //     fn deserialize(_data: Vec<u8>) -> Self {
//     //         todo!()
//     //     }
//     // }

//     // #[test]
//     // fn create_datawriter_simple() {
//     //     let publisher_impl = RtpsPublisherImpl::new();
//     //     let a_topic = RtpsTopicImpl::new();
//     //     let qos = None;
//     //     let a_listener = None;
//     //     let mask = 0;
//     //     let data_writer =
//     //         publisher_impl.create_datawriter::<TestType>(&a_topic, qos, a_listener, mask);
//     //     assert!(data_writer.is_some());
//     // }
//     // #[test]
//     // fn set_and_get_qos() {
//     //     let publisher_list = MaybeValidList::default();
//     //     let guid_prefix = [1; 12];
//     //     let entity_key = [1, 2, 3];
//     //     let qos = PublisherQos::default();
//     //     let listener = None;
//     //     let status_mask = 0;
//     //     let publisher = publisher_list
//     //         .add(Box::new(RtpsPublisherInner::new_user_defined(
//     //             guid_prefix,
//     //             entity_key,
//     //             qos,
//     //             listener,
//     //             status_mask,
//     //         )))
//     //         .expect("Error creating publisher");

//     //     let mut new_qos = PublisherQos::default();
//     //     new_qos.partition.name = "ABCD".to_string();
//     //     new_qos.presentation.coherent_access = true;
//     //     publisher
//     //         .set_qos(Some(new_qos.clone()))
//     //         .expect("Error setting publisher QoS");
//     //     assert_eq!(
//     //         publisher.get_qos().expect("Error getting publisher QoS"),
//     //         new_qos
//     //     );
//     // }

//     // #[test]
//     // fn set_default_qos() {
//     //     let publisher_list = MaybeValidList::default();
//     //     let guid_prefix = [1; 12];
//     //     let entity_key = [1, 2, 3];
//     //     let mut qos = PublisherQos::default();
//     //     qos.partition.name = "ABCD".to_string();
//     //     qos.presentation.coherent_access = true;
//     //     let listener = None;
//     //     let status_mask = 0;
//     //     let publisher = publisher_list
//     //         .add(Box::new(RtpsPublisherInner::new_user_defined(
//     //             guid_prefix,
//     //             entity_key,
//     //             qos,
//     //             listener,
//     //             status_mask,
//     //         )))
//     //         .expect("Error creating publisher");

//     //     publisher
//     //         .set_qos(None)
//     //         .expect("Error setting publisher QoS");
//     //     assert_eq!(
//     //         publisher.get_qos().expect("Error getting publisher QoS"),
//     //         PublisherQos::default()
//     //     );
//     // }

//     // #[test]
//     // fn set_and_get_default_datawriter_qos() {
//     //     let publisher_list = MaybeValidList::default();
//     //     let guid_prefix = [1; 12];
//     //     let entity_key = [1, 2, 3];
//     //     let qos = PublisherQos::default();
//     //     let listener = None;
//     //     let status_mask = 0;
//     //     let publisher = publisher_list
//     //         .add(Box::new(RtpsPublisherInner::new_user_defined(
//     //             guid_prefix,
//     //             entity_key,
//     //             qos,
//     //             listener,
//     //             status_mask,
//     //         )))
//     //         .expect("Error creating publisher");

//     //     let mut datawriter_qos = DataWriterQos::default();
//     //     datawriter_qos.user_data.value = vec![1, 2, 3, 4, 5];
//     //     datawriter_qos.reliability.kind = ReliabilityQosPolicyKind::BestEffortReliabilityQos;
//     //     publisher
//     //         .set_default_datawriter_qos(Some(datawriter_qos.clone()))
//     //         .expect("Error setting default datawriter QoS");
//     //     assert_eq!(
//     //         publisher
//     //             .get_default_datawriter_qos()
//     //             .expect("Error getting publisher QoS"),
//     //         datawriter_qos
//     //     );
//     // }

//     // #[test]
//     // fn set_inconsistent_default_datawriter_qos() {
//     //     let publisher_list = MaybeValidList::default();
//     //     let guid_prefix = [1; 12];
//     //     let entity_key = [1, 2, 3];
//     //     let qos = PublisherQos::default();
//     //     let listener = None;
//     //     let status_mask = 0;
//     //     let publisher = publisher_list
//     //         .add(Box::new(RtpsPublisherInner::new_user_defined(
//     //             guid_prefix,
//     //             entity_key,
//     //             qos,
//     //             listener,
//     //             status_mask,
//     //         )))
//     //         .expect("Error creating publisher");

//     //     let mut datawriter_qos = DataWriterQos::default();
//     //     datawriter_qos.resource_limits.max_samples_per_instance = 10;
//     //     datawriter_qos.resource_limits.max_samples = 2;
//     //     let result = publisher.set_default_datawriter_qos(Some(datawriter_qos.clone()));

//     //     match result {
//     //         Err(DDSError::InconsistentPolicy) => assert!(true),
//     //         _ => assert!(false),
//     //     }
//     // }
// }
