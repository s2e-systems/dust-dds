use rust_dds_api::publication::Publisher;
use rust_dds_api::infrastructure::entity::{Entity, DomainEntity};
use rust_dds_api::infrastructure::qos::PublisherQos;
use rust_dds_api::publication::PublisherListener;

use rust_dds_api::types::{TopicKind, ReturnCode};
pub struct RtpsPublisher {
   
}

impl Entity for RtpsPublisher {
    type Qos = PublisherQos;

    type Listener = Box<dyn PublisherListener>;

    fn set_qos(&self, qos_list: Self::Qos) -> ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self, qos_list: &mut Self::Qos) -> ReturnCode<()> {
        todo!()
    }

    fn set_listener(&self, a_listener: Self::Listener, mask: rust_dds_api::infrastructure::status::StatusMask) -> ReturnCode<()> {
        todo!()
    }

    fn get_listener(&self, ) -> Self::Listener {
        todo!()
    }

    fn get_statuscondition(&self, ) -> rust_dds_api::infrastructure::entity::StatusCondition {
        todo!()
    }

    fn get_status_changes(&self, ) -> rust_dds_api::infrastructure::status::StatusMask {
        todo!()
    }

    fn enable(&self, ) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self, ) -> ReturnCode<rust_dds_api::types::InstanceHandle> {
        todo!()
    }
}

impl DomainEntity for RtpsPublisher {

}

impl Publisher for RtpsPublisher {
    fn create_datawriter<T: rust_dds_api::types::DDSType>(
        &self,
        a_topic: &dyn rust_dds_api::topic::Topic<T, Qos=rust_dds_api::infrastructure::qos::TopicQos, Listener=dyn rust_dds_api::topic::TopicListener<T>>,
        qos: Option<&rust_dds_api::infrastructure::qos::DataWriterQos>,
        _a_listener: impl rust_dds_api::publication::DataWriterListener<T>,
        _mask: rust_dds_api::infrastructure::status::StatusMask
    ) -> Option<Box<dyn rust_dds_api::publication::DataWriter<T>>> {
        todo!()
    }

    fn delete_datawriter<T: rust_dds_api::types::DDSType>(
        &self,
        a_datawriter: &mut dyn rust_dds_api::publication::DataWriter<T>
    ) -> ReturnCode<()> {
        todo!()
    }

    fn lookup_datawriter<T: rust_dds_api::types::DDSType>(
        &self,
        topic_name: &String,
    ) -> Option<Box<dyn rust_dds_api::publication::DataWriter<T>>> {
        todo!()
    }

    fn suspend_publications(&self) -> ReturnCode<()> {
        todo!()
    }

    fn resume_publications(&self) -> ReturnCode<()> {
        todo!()
    }

    fn begin_coherent_changes(&self) -> ReturnCode<()> {
        todo!()
    }

    fn end_coherent_changes(&self) -> ReturnCode<()> {
        todo!()
    }

    fn wait_for_acknowledgments(
        &self,
        _max_wait: rust_dds_api::types::Duration
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_participant<T: rust_dds_api::domain::DomainParticipant>(&self) -> &T {
        todo!()
    }

    fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    fn set_default_datawriter_qos(
        &self,
        _qos_list: rust_dds_api::infrastructure::qos::DataWriterQos,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_default_datawriter_qos (
        &self,
        _qos_list: &mut rust_dds_api::infrastructure::qos::DataWriterQos,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut rust_dds_api::infrastructure::qos::DataWriterQos,
        _a_topic_qos: &rust_dds_api::infrastructure::qos::TopicQos,
    ) -> ReturnCode<()> {
        todo!()
    }
}
