use crate::{
    domain::domain_participant::DomainParticipant,
    implementation::{
        data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData,
        dds::{
            dds_data_writer,
            dds_domain_participant::{self, DdsDomainParticipant},
            dds_publisher, dds_topic,
            nodes::{DomainParticipantNode, TopicNode},
        },
        utils::actor::ActorAddress,
    },
    infrastructure::{
        condition::StatusCondition,
        error::DdsResult,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
};

use super::{
    topic_listener::TopicListener,
    type_support::{dds_serialize_key_to_bytes, dds_serialize_to_bytes},
};

/// The [`Topic`] represents the fact that both publications and subscriptions are tied to a single data-type. Its attributes
/// `type_name` defines a unique resulting type for the publication or the subscription. It has also a `name` that allows it to
/// be retrieved locally.
#[derive(PartialEq, Eq)]
pub struct Topic(TopicNode);

impl Topic {
    pub(crate) fn new(node: TopicNode) -> Self {
        Self(node)
    }

    pub(crate) fn node(&self) -> &TopicNode {
        &self.0
    }
}

// impl<Foo> Drop for Topic<Foo> {
//     fn drop(&mut self) {
//         todo!()
//         // match &self.node {
//         //     TopicNodeKind::Listener(_) => (),
//         //     TopicNodeKind::UserDefined(t) => THE_DDS_DOMAIN_PARTICIPANT_FACTORY
//         //         .get_participant_mut(&t.guid().prefix(), |dp| {
//         //             if let Some(dp) = dp {
//         //                 crate::implementation::behavior::domain_participant::delete_topic(
//         //                     dp,
//         //                     t.guid(),
//         //                 )
//         //                 .ok();
//         //             }
//         //         }),
//         // }
//     }
// }

impl Topic {
    /// This method allows the application to retrieve the [`InconsistentTopicStatus`] of the [`Topic`].
    #[tracing::instrument(skip(self))]
    pub fn get_inconsistent_topic_status(&self) -> DdsResult<InconsistentTopicStatus> {
        self.0
            .topic_address()
            .send_mail_and_await_reply_blocking(dds_topic::get_inconsistent_topic_status::new())?
    }
}

/// This implementation block represents the TopicDescription operations for the [`Topic`].
impl Topic {
    /// This operation returns the [`DomainParticipant`] to which the [`Topic`] belongs.
    #[tracing::instrument(skip(self))]
    pub fn get_participant(&self) -> DdsResult<DomainParticipant> {
        Ok(DomainParticipant::new(DomainParticipantNode::new(
            self.0.participant_address().clone(),
        )))
    }

    /// The name of the type used to create the [`Topic`]
    #[tracing::instrument(skip(self))]
    pub fn get_type_name(&self) -> DdsResult<String> {
        self.0
            .topic_address()
            .send_mail_and_await_reply_blocking(dds_topic::get_type_name::new())
    }

    /// The name used to create the [`Topic`]
    #[tracing::instrument(skip(self))]
    pub fn get_name(&self) -> DdsResult<String> {
        self.0
            .topic_address()
            .send_mail_and_await_reply_blocking(dds_topic::get_name::new())
    }
}

/// This implementation block contains the Entity operations for the [`Topic`].
impl Topic {
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
    #[tracing::instrument(skip(self))]
    pub fn set_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => self
                .0
                .participant_address()
                .send_mail_and_await_reply_blocking(
                    dds_domain_participant::default_topic_qos::new(),
                )?,
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        if self
            .0
            .topic_address()
            .send_mail_and_await_reply_blocking(dds_topic::is_enabled::new())?
        {
            self.0
                .topic_address()
                .send_mail_and_await_reply_blocking(dds_topic::get_qos::new())?
                .check_immutability(&qos)?
        }

        self.0
            .topic_address()
            .send_mail_and_await_reply_blocking(dds_topic::set_qos::new(qos))
    }

    /// This operation allows access to the existing set of [`TopicQos`] policies.
    #[tracing::instrument(skip(self))]
    pub fn get_qos(&self) -> DdsResult<TopicQos> {
        self.0
            .topic_address()
            .send_mail_and_await_reply_blocking(dds_topic::get_qos::new())
    }

    /// This operation allows access to the [`StatusCondition`] associated with the Entity. The returned
    /// condition can then be added to a [`WaitSet`](crate::infrastructure::wait_set::WaitSet) so that the application can wait for specific status changes
    /// that affect the Entity.
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        self.0
            .topic_address()
            .send_mail_and_await_reply_blocking(dds_topic::get_statuscondition::new())
            .map(StatusCondition::new)
    }

    /// This operation retrieves the list of communication statuses in the Entity that are ‘triggered.’ That is, the list of statuses whose
    /// value has changed since the last time the application read the status.
    /// When the entity is first created or if the entity is not enabled, all communication statuses are in the “untriggered” state so the
    /// list returned by the [`Self::get_status_changes`] operation will be empty.
    /// The list of statuses returned by the [`Self::get_status_changes`] operation refers to the status that are triggered on the Entity itself
    /// and does not include statuses that apply to contained entities.
    #[tracing::instrument(skip(self))]
    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
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
    #[tracing::instrument(skip(self))]
    pub fn enable(&self) -> DdsResult<()> {
        if !self
            .0
            .topic_address()
            .send_mail_and_await_reply_blocking(dds_topic::is_enabled::new())?
        {
            self.0
                .topic_address()
                .send_mail_and_await_reply_blocking(dds_topic::enable::new())?;

            announce_topic(
                self.0.participant_address(),
                self.0.topic_address().send_mail_and_await_reply_blocking(
                    dds_topic::as_discovered_topic_data::new(),
                )?,
            )?;
        }

        Ok(())
    }

    /// This operation returns the [`InstanceHandle`] that represents the Entity.
    #[tracing::instrument(skip(self))]
    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.0
            .topic_address()
            .send_mail_and_await_reply_blocking(dds_topic::get_instance_handle::new())
    }

    /// This operation installs a Listener on the Entity. The listener will only be invoked on the changes of communication status
    /// indicated by the specified mask. It is permitted to use [`None`] as the value of the listener. The [`None`] listener behaves
    /// as a Listener whose operations perform no action.
    /// Only one listener can be attached to each Entity. If a listener was already set, the operation [`Self::set_listener()`] will replace it with the
    /// new one. Consequently if the value [`None`] is passed for the listener parameter to the [`Self::set_listener()`] operation, any existing listener
    /// will be removed.
    #[tracing::instrument(skip(self, _a_listener), fields(with_listener = _a_listener.is_some()))]
    pub fn set_listener(
        &self,
        _a_listener: Option<Box<dyn TopicListener + Send + Sync>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
    }
}

fn announce_topic(
    domain_participant: &ActorAddress<DdsDomainParticipant>,
    discovered_topic_data: DiscoveredTopicData,
) -> DdsResult<()> {
    let serialized_data = dds_serialize_to_bytes(&discovered_topic_data)?;
    let timestamp = domain_participant
        .send_mail_and_await_reply_blocking(dds_domain_participant::get_current_time::new())?;
    let builtin_publisher = domain_participant
        .send_mail_and_await_reply_blocking(dds_domain_participant::get_builtin_publisher::new())?;
    let data_writer_list = builtin_publisher
        .send_mail_and_await_reply_blocking(dds_publisher::data_writer_list::new())?;
    for data_writer in data_writer_list {
        if data_writer.send_mail_and_await_reply_blocking(dds_data_writer::get_type_name::new())
            == Ok("DiscoveredTopicData".to_string())
        {
            data_writer.send_mail_and_await_reply_blocking(
                dds_data_writer::write_w_timestamp::new(
                    serialized_data,
                    dds_serialize_key_to_bytes(&discovered_topic_data)?,
                    None,
                    timestamp,
                ),
            )??;

            domain_participant.send_mail_blocking(dds_domain_participant::send_message::new())?;
            break;
        }
    }

    Ok(())
}
