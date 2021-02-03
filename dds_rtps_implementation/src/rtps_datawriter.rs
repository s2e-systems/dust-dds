use std::{
    any::Any,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard},
};

use rust_dds_api::{
    builtin_topics::SubscriptionBuiltinTopicData,
    domain::domain_participant::TopicGAT,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::DataWriterQos,
        qos_policy::ReliabilityQosPolicyKind,
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, StatusMask,
        },
    },
    publication::{
        data_writer::{AnyDataWriter, DataWriter},
        data_writer_listener::DataWriterListener,
        publisher::{Publisher, PublisherChild},
    },
    topic::topic::Topic,
};
use rust_dds_types::{DDSType, Duration, InstanceHandle, ReturnCode, ReturnCodes, Time, TopicKind};
use rust_rtps::{
    behavior::{self, endpoint_traits::CacheChangeSender, StatefulWriter, StatelessWriter, Writer},
    types::{
        constants::{
            ENTITY_KIND_BUILT_IN_WRITER_NO_KEY, ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
            ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY, ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY,
        },
        EntityId, EntityKey, GuidPrefix, ReliabilityKind, GUID,
    },
};

use crate::{
    inner::rtps_datawriter_inner::RtpsAnyDataWriterInnerRef,
    rtps_publisher::RtpsPublisher,
    rtps_topic::RtpsTopic,
    utils::{as_any::AsAny, maybe_valid::MaybeValidRef},
};

pub struct RtpsDataWriter<'a, T: DDSType> {
    pub(crate) parent_publisher: &'a RtpsPublisher<'a>,
    pub(crate) data_writer_ref: RtpsAnyDataWriterInnerRef<'a>,
    pub(crate) phantom_data: PhantomData<T>,
}

impl<'a, T: DDSType> RtpsDataWriter<'a, T> {
    pub fn new(
        parent_publisher: &'a RtpsPublisher<'a>,
        data_writer_ref: RtpsAnyDataWriterInnerRef<'a>,
    ) -> Self {
        Self{
            parent_publisher,
            data_writer_ref,
            phantom_data: PhantomData,
        }
    }
}

impl<'a, T: DDSType> PublisherChild<'a> for RtpsDataWriter<'a, T> {
    type PublisherType = RtpsPublisher<'a>;
}

impl<'a, T: DDSType> TopicGAT<'a, T> for RtpsDataWriter<'a, T> {
    type TopicType = RtpsTopic<'a, T>;
}

impl<'a, T: DDSType> DataWriter<'a, T> for RtpsDataWriter<'a, T> {
    fn register_instance(&self, _instance: T) -> ReturnCode<Option<InstanceHandle>> {
        todo!()
    }

    fn register_instance_w_timestamp(
        &self,
        _instance: T,
        _timestamp: Time,
    ) -> ReturnCode<Option<InstanceHandle>> {
        todo!()
    }

    fn unregister_instance(&self, _instance: T, _handle: Option<InstanceHandle>) -> ReturnCode<()> {
        todo!()
    }

    fn unregister_instance_w_timestamp(
        &self,
        _instance: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut T, _handle: InstanceHandle) -> ReturnCode<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &T) -> ReturnCode<Option<InstanceHandle>> {
        todo!()
    }

    fn write(&self, _data: T, _handle: Option<InstanceHandle>) -> ReturnCode<()> {
        todo!()
    }

    fn write_w_timestamp(
        &self,
        _data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn dispose(&self, _data: T, _handle: Option<InstanceHandle>) -> ReturnCode<()> {
        todo!()
    }

    fn dispose_w_timestamp(
        &self,
        _data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn wait_for_acknowledgments(&self, _max_wait: Duration) -> ReturnCode<()> {
        todo!()
    }

    fn get_liveliness_lost_status(&self, _status: &mut LivelinessLostStatus) -> ReturnCode<()> {
        todo!()
    }

    fn get_offered_deadline_missed_status(
        &self,
        _status: &mut OfferedDeadlineMissedStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_offered_incompatible_qos_status(
        &self,
        _status: &mut OfferedIncompatibleQosStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_publication_matched_status(
        &self,
        _status: &mut PublicationMatchedStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    /// This operation returns the Topic associated with the DataWriter. This is the same Topic that was used to create the DataWriter.
    fn get_topic(&self) -> &<Self as TopicGAT<'a, T>>::TopicType {
        todo!()
    }

    /// This operation returns the Publisher to which the publisher child object belongs.
    fn get_publisher(&self) -> &<Self as PublisherChild<'a>>::PublisherType {
        todo!()
    }

    fn assert_liveliness(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_matched_subscription_data(
        &self,
        _subscription_data: SubscriptionBuiltinTopicData,
        _subscription_handle: InstanceHandle,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_matched_subscriptions(
        &self,
        _subscription_handles: &mut [InstanceHandle],
    ) -> ReturnCode<()> {
        todo!()
    }
}

impl<'a, T: DDSType> Entity for RtpsDataWriter<'a, T> {
    type Qos = DataWriterQos;

    type Listener = Box<dyn DataWriterListener<T>>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> ReturnCode<()> {
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

impl<'a, T: DDSType> AnyDataWriter for RtpsDataWriter<'a, T> {}
