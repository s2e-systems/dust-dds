use rust_dds_api::{
    dcps_psm::{InstanceHandle, StatusMask},
    infrastructure::qos::PublisherQos,
    publication::publisher_listener::PublisherListener,
    return_type::DDSResult,
};
use rust_rtps_pim::{
    behavior::types::DurationPIM,
    messages::submessage_elements::ParameterListSubmessageElementPIM,
    structure::types::{
        DataPIM, InstanceHandlePIM, LocatorPIM, GUIDPIM,
    },
};

use crate::utils::shared_object::RtpsShared;

use super::rtps_writer_impl::RTPSWriterImpl;

pub struct RTPSWriterGroupImpl<PSM>
where
    PSM: GUIDPIM
        + LocatorPIM
        + DurationPIM
        + InstanceHandlePIM
        + DataPIM
        + ParameterListSubmessageElementPIM,
{
    guid: PSM::GUIDType,
    qos: PublisherQos,
    listener: Option<&'static dyn PublisherListener>,
    status_mask: StatusMask,
    writer_list: Vec<RtpsShared<RTPSWriterImpl<PSM>>>,
}

impl<PSM> RTPSWriterGroupImpl<PSM>
where
    PSM: GUIDPIM
        + LocatorPIM
        + DurationPIM
        + InstanceHandlePIM
        + DataPIM
        + ParameterListSubmessageElementPIM,
{
    pub fn new(
        guid: PSM::GUIDType,
        qos: PublisherQos,
        listener: Option<&'static dyn PublisherListener>,
        status_mask: StatusMask,
    ) -> Self {
        Self {
            guid,
            qos,
            listener,
            status_mask,
            writer_list: Vec::new(),
        }
    }

    pub fn writer_list(&self) -> &[RtpsShared<RTPSWriterImpl<PSM>>] {
        &self.writer_list
    }

    pub fn add_writer(&mut self, writer: RtpsShared<RTPSWriterImpl<PSM>>) {
        self.writer_list.push(writer)
    }

    pub fn delete_writer(&mut self, _writer: InstanceHandle) -> DDSResult<()> {
        todo!()
        // let index = self
        //     .writer_list
        //     .iter()
        //     .position(|x| crate::utils::instance_handle_from_guid(&x.lock().guid()) == writer)
        //     .ok_or(DDSError::PreconditionNotMet("RTPS writer not found"))?;
        // self.writer_list.swap_remove(index);
        // Ok(())
    }
}

impl<PSM> rust_rtps_pim::structure::RTPSGroup<PSM> for RTPSWriterGroupImpl<PSM> where
    PSM: GUIDPIM
        + LocatorPIM
        + DurationPIM
        + InstanceHandlePIM
        + DataPIM
        + ParameterListSubmessageElementPIM
{
}

impl<PSM> rust_rtps_pim::structure::RTPSEntity<PSM> for RTPSWriterGroupImpl<PSM>
where
    PSM: GUIDPIM
        + LocatorPIM
        + DurationPIM
        + InstanceHandlePIM
        + DataPIM
        + ParameterListSubmessageElementPIM,
{
    fn guid(&self) -> &PSM::GUIDType {
        &self.guid
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
