use crate::dds_infrastructure::entity::{Entity, StatusCondition};
use crate::dds_infrastructure::qos::TopicQos;
use crate::dds_infrastructure::status::StatusMask;
use crate::dds_infrastructure::topic_listener::TopicListener;
use crate::dds_rtps_implementation::rtps_object::RtpsObjectReference;
use crate::types::{DDSType, InstanceHandle, ReturnCode};

pub struct RtpsTopicInner<T: DDSType> {
    marker: std::marker::PhantomData<T>,
}

impl<T: DDSType> Default for RtpsTopicInner<T> {
    fn default() -> Self {
        Self {
            marker: std::marker::PhantomData,
        }
    }
}

pub type RtpsTopic<'a, T> = RtpsObjectReference<'a, RtpsTopicInner<T>>;

impl<'a, T: DDSType> RtpsTopic<'a, T> {}

impl<'a, T: DDSType> Entity for RtpsTopic<'a, T> {
    type Qos = TopicQos;
    type Listener = Box<dyn TopicListener<T>>;

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
