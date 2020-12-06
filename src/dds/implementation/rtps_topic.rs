use crate::types::DDSType;

pub struct RtpsTopic<T: DDSType>{
    marker: std::marker::PhantomData<T>
}

impl<T: DDSType> RtpsTopic<T>
{
    // /// This method allows the application to retrieve the INCONSISTENT_TOPIC status of the Topic.
    // /// Each DomainEntity has a set of relevant communication statuses. A change of status causes the corresponding Listener to be
    // /// invoked and can also be monitored by means of the associated StatusCondition.
    // /// The complete list of communication status, their values, and the DomainEntities they apply to is provided in 2.2.4.1,
    // /// Communication Status.
    // fn get_inconsistent_topic_status(
    //     &self, _status: &mut InconsistentTopicStatus,
    // ) -> ReturnCode<()>;
}
