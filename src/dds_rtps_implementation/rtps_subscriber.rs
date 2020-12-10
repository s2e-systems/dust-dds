use crate::dds_infrastructure::entity::{Entity, StatusCondition};
use crate::dds_infrastructure::qos::{DataReaderQos, SubscriberQos, TopicQos};
use crate::dds_infrastructure::status::SampleLostStatus;
use crate::dds_infrastructure::status::StatusMask;
use crate::dds_infrastructure::subscriber_listener::SubscriberListener;
use crate::dds_rtps_implementation::rtps_data_reader::RtpsDataReader;
use crate::dds_rtps_implementation::rtps_object::RtpsObjectReference;
use crate::types::{DDSType, InstanceHandle, ReturnCode};

#[derive(Default)]
pub struct RtpsSubscriberInner;

pub type RtpsSubscriber<'a> = RtpsObjectReference<'a, RtpsSubscriberInner>;

impl<'a> RtpsSubscriber<'a> {
    pub fn create_datareader<T: DDSType>(&self) -> Option<RtpsDataReader<T>> {
        todo!()
    }

    pub fn delete_datareader<T: DDSType>(
        &self,
        _a_datareader: &RtpsDataReader<T>,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn lookup_datareader<T: DDSType>(&self, _topic_name: &str) -> Option<RtpsDataReader<T>> {
        todo!()
    }

    pub fn begin_access(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn end_access(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn notify_datareaders(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> ReturnCode<()> {
        todo!()
    }

    pub fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn set_default_datareader_qos(&self, _qos: DataReaderQos) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_default_datareader_qos(&self) -> ReturnCode<DataReaderQos> {
        todo!()
    }

    pub fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> ReturnCode<()> {
        todo!()
    }
}

impl<'a> Entity for RtpsSubscriber<'a> {
    type Qos = SubscriberQos;
    type Listener = Box<dyn SubscriberListener>;

    fn set_qos(&self, _qos: Self::Qos) -> ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self) -> ReturnCode<Self::Qos> {
        todo!()
    }

    fn set_listener(&self, _a_listener: Self::Listener, _mask: StatusMask) -> ReturnCode<()> {
        todo!()
    }

    fn get_listener(&self) -> &Self::Listener {
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        todo!()
    }
}
