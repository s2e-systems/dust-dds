use rust_dds_api::subscription::Subscriber;
use rust_dds_api::infrastructure::entity::{Entity, DomainEntity};
use rust_dds_api::infrastructure::qos::SubscriberQos;
use rust_dds_api::subscription::SubscriberListener;

pub struct RtpsSubscriber{}

impl Entity for RtpsSubscriber {
    type Qos = SubscriberQos;

    type Listener = Box<dyn SubscriberListener>;

    fn set_qos(&self, qos_list: Self::Qos) -> rust_dds_api::types::ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self, qos_list: &mut Self::Qos) -> rust_dds_api::types::ReturnCode<()> {
        todo!()
    }

    fn set_listener(&self, a_listener: Self::Listener, mask: rust_dds_api::infrastructure::status::StatusMask) -> rust_dds_api::types::ReturnCode<()> {
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

    fn enable(&self, ) -> rust_dds_api::types::ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self, ) -> rust_dds_api::types::ReturnCode<rust_dds_api::types::InstanceHandle> {
        todo!()
    }
}

impl DomainEntity for RtpsSubscriber {

}

impl Subscriber for RtpsSubscriber {
    fn create_datareader<T: rust_dds_api::types::DDSType>(
        &self,
        a_topic: &dyn rust_dds_api::topic::Topic<T>,
        qos: Option<&rust_dds_api::infrastructure::qos::DataReaderQos>,
        _a_listener: impl rust_dds_api::subscription::DataReaderListener<T>,
        _mask: rust_dds_api::infrastructure::status::StatusMask
    ) -> Option<Box<dyn rust_dds_api::subscription::DataReader<T>>> {
        todo!()
    }

    fn delete_datareader<T: rust_dds_api::types::DDSType>(
        &self,
        a_datareader: &mut dyn rust_dds_api::subscription::DataReader<T>
    ) -> rust_dds_api::types::ReturnCode<()> {
        todo!()
    }

    fn lookup_datareader<T: rust_dds_api::types::DDSType>(
        &self,
        _topic_name: String
    ) -> Box<dyn rust_dds_api::subscription::DataReader<T>> {
        todo!()
    }

    fn begin_access(&self) -> rust_dds_api::types::ReturnCode<()> {
        todo!()
    }

    fn end_access(&self) -> rust_dds_api::types::ReturnCode<()> {
        todo!()
    }

    fn get_datareaders<T: rust_dds_api::types::DDSType>(
        &self,
        _readers: &mut [&mut dyn rust_dds_api::subscription::DataReader<T>],
        _sample_states: &[rust_dds_api::infrastructure::status::SampleStateKind],
        _view_states: &[rust_dds_api::infrastructure::status::ViewStateKind],
        _instance_states: &[rust_dds_api::infrastructure::status::InstanceStateKind],
    ) -> rust_dds_api::types::ReturnCode<()> {
        todo!()
    }

    fn notify_datareaders(&self) -> rust_dds_api::types::ReturnCode<()> {
        todo!()
    }

    fn get_sample_lost_status(&self, _status: &mut rust_dds_api::infrastructure::status::SampleLostStatus) -> rust_dds_api::types::ReturnCode<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> rust_dds_api::types::ReturnCode<()> {
        todo!()
    }

    fn set_default_datareader_qos(
        &self,
        _qos: rust_dds_api::infrastructure::qos::DataReaderQos,
    ) -> rust_dds_api::types::ReturnCode<()> {
        todo!()
    }

    fn get_default_datareader_qos(
        &self,
        _qos_list: &mut rust_dds_api::infrastructure::qos::DataReaderQos,
    ) -> rust_dds_api::types::ReturnCode<()> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut rust_dds_api::infrastructure::qos::DataReaderQos,
        _a_topic_qos: &rust_dds_api::infrastructure::qos::TopicQos,
    ) -> rust_dds_api::types::ReturnCode<()> {
        todo!()
    }
}