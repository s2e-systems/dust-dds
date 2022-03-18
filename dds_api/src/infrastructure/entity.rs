use crate::{
    dcps_psm::{InstanceHandle, StatusMask},
    return_type::DdsResult,
};

pub struct StatusCondition;

pub trait DomainEntity {}

pub trait Entity {
    type Qos;
    type Listener;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DdsResult<()>;

    fn get_qos(&self) -> DdsResult<Self::Qos>;

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DdsResult<()>;

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>>;

    fn get_statuscondition(&self) -> DdsResult<StatusCondition>;

    fn get_status_changes(&self) -> DdsResult<StatusMask>;

    fn enable(&self) -> DdsResult<()>;

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle>;
}
