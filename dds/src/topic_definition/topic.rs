use std::marker::PhantomData;

use crate::{
    domain::domain_participant::DomainParticipant,
    implementation::{
        dds_impl::topic_impl::{AnyTopicListener, TopicImpl},
        utils::shared_object::DdsWeak,
    },
    infrastructure::{
        condition::StatusCondition,
        error::DdsResult,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
};

use super::topic_listener::TopicListener;

/// The [`Topic`] represents the fact that both publications and subscriptions are tied to a single data-type. Its attributes
/// `type_name` defines a unique resulting type for the publication or the subscription. It has also a `name` that allows it to
/// be retrieved locally.
pub struct Topic<Foo>(pub(crate) DdsWeak<TopicImpl>, PhantomData<Foo>);

impl<Foo> Topic<Foo> {
    pub(crate) fn new(topic_attributes: DdsWeak<TopicImpl>) -> Self {
        Self(topic_attributes, PhantomData)
    }
}

impl<Foo> Topic<Foo> {
    /// This method allows the application to retrieve the [`InconsistentTopicStatus`] of the [`Topic`].
    pub fn get_inconsistent_topic_status(&self) -> DdsResult<InconsistentTopicStatus> {
        self.0.upgrade()?.get_inconsistent_topic_status()
    }
}

/// This implementation block represents the TopicDescription operations for the [`Topic`].
impl<Foo> Topic<Foo> {
    /// This operation returns the [`DomainParticipant`] to which the [`Topic`] belongs.
    pub fn get_participant(&self) -> DdsResult<DomainParticipant> {
        self.0
            .upgrade()?
            .get_participant()
            .map(|x| DomainParticipant::new(x.downgrade()))
    }

    /// The name of the type used to create the [`Topic`]
    pub fn get_type_name(&self) -> DdsResult<&'static str> {
        Ok(self.0.upgrade()?.get_type_name())
    }

    /// The name used to create the [`Topic`]
    pub fn get_name(&self) -> DdsResult<String> {
        Ok(self.0.upgrade()?.get_name())
    }
}

/// This implementation block contains the Entity operations for the [`Topic`].
impl<Foo> Topic<Foo>
where
    Foo: 'static,
{
    /// This operation is used to set the QoS policies of the Entity and replacing the values of any policies previously set.
    /// Certain policies are “immutable;” they can only be set at Entity creation time, or before the entity is made enabled.
    /// If [`Self::set_qos()`] is invoked after the Entity is enabled and it attempts to change the value of an “immutable” policy, the operation will
    /// fail and returns [`DdsError::ImmutablePolicy`](crate::infrastructure::error::DdsError).
    /// Certain values of QoS policies can be incompatible with the settings of the other policies. This operation will also fail if it specifies
    /// a set of values that once combined with the existing values would result in an inconsistent set of policies. In this case,
    /// the return value is [`DdsError::InconsistentPolicy`](crate::infrastructure::error::DdsError).
    /// The existing set of policies are only changed if the [`Self::set_qos()`] operation succeeds. This is indicated by the [`Ok`] return value. In all
    /// other cases, none of the policies is modified.
    /// The parameter `qos` can be set to [`QosKind::Default`] to indicate that the QoS of the Entity should be changed to match the current default QoS set in the Entity’s factory.
    /// The operation [`Self::set_qos()`] cannot modify the immutable QoS so a successful return of the operation indicates that the mutable QoS for the Entity has been
    /// modified to match the current default for the Entity’s factory.
    pub fn set_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        self.0.upgrade()?.set_qos(qos)
    }

    /// This operation allows access to the existing set of [`TopicQos`] policies.
    pub fn get_qos(&self) -> DdsResult<TopicQos> {
        Ok(self.0.upgrade()?.get_qos())
    }

    /// This operation installs a Listener on the Entity. The listener will only be invoked on the changes of communication status
    /// indicated by the specified mask. It is permitted to use [`None`] as the value of the listener. The [`None`] listener behaves
    /// as a Listener whose operations perform no action.
    /// Only one listener can be attached to each Entity. If a listener was already set, the operation [`Self::set_listener()`] will replace it with the
    /// new one. Consequently if the value [`None`] is passed for the listener parameter to the [`Self::set_listener()`] operation, any existing listener
    /// will be removed.
    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn TopicListener<Foo = Foo>>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        #[allow(clippy::redundant_closure)]
        self.0.upgrade()?.set_listener(
            a_listener.map::<Box<dyn AnyTopicListener>, _>(|l| Box::new(l)),
            mask,
        )
    }

    /// This operation allows access to the existing Listener attached to the Entity.
    pub fn get_listener(&self) -> DdsResult<Option<Box<dyn TopicListener<Foo = Foo>>>> {
        todo!()
    }

    /// This operation allows access to the [`StatusCondition`] associated with the Entity. The returned
    /// condition can then be added to a [`WaitSet`](crate::infrastructure::wait_set::WaitSet) so that the application can wait for specific status changes
    /// that affect the Entity.
    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        self.0.upgrade()?.get_statuscondition()
    }

    /// This operation retrieves the list of communication statuses in the Entity that are ‘triggered.’ That is, the list of statuses whose
    /// value has changed since the last time the application read the status.
    /// When the entity is first created or if the entity is not enabled, all communication statuses are in the “untriggered” state so the
    /// list returned by the [`Self::get_status_changes`] operation will be empty.
    /// The list of statuses returned by the [`Self::get_status_changes`] operation refers to the status that are triggered on the Entity itself
    /// and does not include statuses that apply to contained entities.
    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        self.0.upgrade()?.get_status_changes()
    }

    /// This operation enables the Entity. Entity objects can be created either enabled or disabled. This is controlled by the value of
    /// the [`EntityFactoryQosPolicy`](crate::infrastructure::qos_policy::EntityFactoryQosPolicy) on the corresponding factory for the Entity.
    /// The default setting of [`EntityFactoryQosPolicy`](crate::infrastructure::qos_policy::EntityFactoryQosPolicy) is such that, by default, it is not necessary to explicitly call enable on newly
    /// created entities.
    /// The [`Self::enable()`] operation is idempotent. Calling [`Self::enable()`] on an already enabled Entity returns [`Ok`] and has no effect.
    /// If an Entity has not yet been enabled, the following kinds of operations may be invoked on it:
    /// - Operations to set or get an Entity’s QoS policies (including default QoS policies) and listener
    /// - [`Self::get_statuscondition()`]
    /// - Factory and lookup operations
    /// - [`Self::get_status_changes()`] and other get status operations (although the status of a disabled entity never changes)
    /// Other operations may explicitly state that they may be called on disabled entities; those that do not will return the error
    /// NotEnabled.
    /// It is legal to delete an Entity that has not been enabled by calling the proper operation on its factory.
    /// Entities created from a factory that is disabled, are created disabled regardless of the setting of the ENTITY_FACTORY Qos
    /// policy.
    /// Calling enable on an Entity whose factory is not enabled will fail and return PRECONDITION_NOT_MET.
    /// If the `autoenable_created_entities` field of [`EntityFactoryQosPolicy`](crate::infrastructure::qos_policy::EntityFactoryQosPolicy) is set to [`true`], the [`Self::enable()`] operation on the factory will
    /// automatically enable all entities created from the factory.
    /// The Listeners associated with an entity are not called until the entity is enabled. Conditions associated with an entity that is not
    /// enabled are “inactive,” that is, the operation [`StatusCondition::get_trigger_value()`] will always return `false`.
    pub fn enable(&self) -> DdsResult<()> {
        self.0.upgrade()?.enable()
    }

    /// This operation returns the [`InstanceHandle`] that represents the Entity.
    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.upgrade()?.get_instance_handle())
    }
}

pub trait AnyTopic {}
