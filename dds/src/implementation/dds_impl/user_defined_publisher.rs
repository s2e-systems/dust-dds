use std::sync::atomic::{self, AtomicU8};

use crate::implementation::rtps::endpoint::RtpsEndpoint;
use crate::implementation::rtps::messages::submessages::AckNackSubmessage;
use crate::implementation::rtps::transport::TransportWrite;
use crate::implementation::rtps::types::{EntityId, EntityKind, Guid, TopicKind};
use crate::implementation::rtps::writer::RtpsWriter;
use crate::implementation::rtps::{group::RtpsGroupImpl, stateful_writer::RtpsStatefulWriter};
use crate::implementation::utils::condvar::DdsCondvar;
use crate::infrastructure::condition::StatusCondition;
use crate::infrastructure::error::{DdsError, DdsResult};
use crate::infrastructure::instance::InstanceHandle;
use crate::infrastructure::qos::QosKind;
use crate::infrastructure::status::StatusKind;
use crate::infrastructure::time::{Duration, DURATION_ZERO};
use crate::topic_definition::type_support::DdsType;
use crate::{
    infrastructure::qos::{DataWriterQos, PublisherQos, TopicQos},
    publication::publisher_listener::PublisherListener,
};

use crate::implementation::{
    data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
    utils::shared_object::{DdsRwLock, DdsShared},
};

use super::message_receiver::{MessageReceiver, PublisherMessageReceiver};
use super::user_defined_data_writer::AnyDataWriterListener;
use super::{
    domain_participant_impl::DomainParticipantImpl, topic_impl::TopicImpl,
    user_defined_data_writer::UserDefinedDataWriter,
};

pub struct UserDefinedPublisher {
    qos: DdsRwLock<PublisherQos>,
    rtps_group: RtpsGroupImpl,
    data_writer_list: DdsRwLock<Vec<DdsShared<UserDefinedDataWriter>>>,
    user_defined_data_writer_counter: AtomicU8,
    default_datawriter_qos: DataWriterQos,
    enabled: DdsRwLock<bool>,
    user_defined_data_send_condvar: DdsCondvar,
}

impl UserDefinedPublisher {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroupImpl,
        user_defined_data_send_condvar: DdsCondvar,
    ) -> DdsShared<Self> {
        DdsShared::new(UserDefinedPublisher {
            qos: DdsRwLock::new(qos),
            rtps_group,
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            enabled: DdsRwLock::new(false),
            user_defined_data_send_condvar,
        })
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }
}

impl DdsShared<UserDefinedPublisher> {
    pub fn is_empty(&self) -> bool {
        self.data_writer_list.read_lock().is_empty()
    }
}

impl DdsShared<UserDefinedPublisher> {
    pub fn create_datawriter<Foo>(
        &self,
        a_topic: &DdsShared<TopicImpl>,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        _mask: &[StatusKind],
        parent_participant: &DdsShared<DomainParticipantImpl>,
    ) -> DdsResult<DdsShared<UserDefinedDataWriter>>
    where
        Foo: DdsType,
    {
        let topic_shared = a_topic;

        // /////// Build the GUID
        let guid = {
            let user_defined_data_writer_counter = self
                .user_defined_data_writer_counter
                .fetch_add(1, atomic::Ordering::SeqCst);

            let entity_kind = match Foo::has_key() {
                true => EntityKind::UserDefinedWriterWithKey,
                false => EntityKind::UserDefinedWriterNoKey,
            };

            Guid::new(
                self.rtps_group.guid().prefix(),
                EntityId::new(
                    [
                        self.rtps_group.guid().entity_id().entity_key()[0],
                        user_defined_data_writer_counter,
                        0,
                    ],
                    entity_kind,
                ),
            )
        };

        // /////// Create data writer
        let data_writer_shared = {
            let qos = match qos {
                QosKind::Default => self.default_datawriter_qos.clone(),
                QosKind::Specific(q) => q,
            };
            qos.is_consistent()?;

            let topic_kind = match Foo::has_key() {
                true => TopicKind::WithKey,
                false => TopicKind::NoKey,
            };

            let rtps_writer_impl = RtpsStatefulWriter::new(RtpsWriter::new(
                RtpsEndpoint::new(
                    guid,
                    topic_kind,
                    parent_participant.default_unicast_locator_list(),
                    parent_participant.default_multicast_locator_list(),
                ),
                true,
                Duration::new(0, 200_000_000),
                DURATION_ZERO,
                DURATION_ZERO,
                None,
                qos,
            ));

            let data_writer_shared = UserDefinedDataWriter::new(
                rtps_writer_impl,
                a_listener,
                topic_shared.clone(),
                self.downgrade(),
                self.user_defined_data_send_condvar.clone(),
            );

            self.data_writer_list
                .write_lock()
                .push(data_writer_shared.clone());

            data_writer_shared
        };

        if *self.enabled.read_lock()
            && self
                .qos
                .read_lock()
                .entity_factory
                .autoenable_created_entities
        {
            data_writer_shared.enable(parent_participant)?;
        }

        Ok(data_writer_shared)
    }

    pub fn delete_datawriter(&self, data_writer_handle: InstanceHandle) -> DdsResult<()> {
        let data_writer_list = &mut self.data_writer_list.write_lock();
        let data_writer_list_position = data_writer_list
            .iter()
            .position(|x| x.get_instance_handle() == data_writer_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Data writer can only be deleted from its parent publisher".to_string(),
                )
            })?;
        data_writer_list.remove(data_writer_list_position);

        Ok(())
    }

    pub fn lookup_datawriter<Foo>(
        &self,
        topic: &DdsShared<TopicImpl>,
    ) -> DdsResult<DdsShared<UserDefinedDataWriter>>
    where
        Foo: DdsType,
    {
        let data_writer_list = &self.data_writer_list.write_lock();

        data_writer_list
            .iter()
            .find_map(|data_writer_shared| {
                let data_writer_topic = data_writer_shared.get_topic();

                if data_writer_topic.get_name().ok()? == topic.get_name().ok()?
                    && data_writer_topic.get_type_name().ok()? == Foo::type_name()
                {
                    Some(data_writer_shared.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| DdsError::PreconditionNotMet("Not found".to_string()))
    }

    pub fn suspend_publications(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn resume_publications(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn begin_coherent_changes(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn end_coherent_changes(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn set_default_datawriter_qos(&self, _qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        todo!()
    }

    pub fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        todo!()
    }

    pub fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }
}

impl DdsShared<UserDefinedPublisher> {
    pub fn set_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        if *self.enabled.read_lock() {
            self.qos.read_lock().check_immutability(&qos)?;
        }

        *self.qos.write_lock() = qos;

        Ok(())
    }

    pub fn get_qos(&self) -> PublisherQos {
        self.qos.read_lock().clone()
    }

    pub fn set_listener(
        &self,
        _a_listener: Option<Box<dyn PublisherListener>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
    }

    pub fn get_listener(&self) -> DdsResult<Option<Box<dyn PublisherListener>>> {
        todo!()
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        todo!()
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    pub fn enable(&self, parent_participant: &DdsShared<DomainParticipantImpl>) -> DdsResult<()> {
        if !parent_participant.is_enabled() {
            return Err(DdsError::PreconditionNotMet(
                "Parent participant is disabled".to_string(),
            ));
        }

        *self.enabled.write_lock() = true;

        if self
            .qos
            .read_lock()
            .entity_factory
            .autoenable_created_entities
        {
            for data_writer in self.data_writer_list.read_lock().iter() {
                data_writer.enable(parent_participant)?;
            }
        }

        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }
}

impl DdsShared<UserDefinedPublisher> {
    pub fn add_matched_reader(&self, discovered_reader_data: &DiscoveredReaderData) {
        for data_writer in self.data_writer_list.read_lock().iter() {
            data_writer.add_matched_reader(discovered_reader_data)
        }
    }
}

impl PublisherMessageReceiver for DdsShared<UserDefinedPublisher> {
    fn on_acknack_submessage_received(
        &self,
        acknack_submessage: &AckNackSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        for data_writer in self.data_writer_list.read_lock().iter() {
            data_writer.on_acknack_submessage_received(acknack_submessage, message_receiver);
        }
    }
}

impl DdsShared<UserDefinedPublisher> {
    pub fn send_message(&self, transport: &mut impl TransportWrite) {
        for data_writer in self.data_writer_list.read_lock().iter() {
            data_writer.send_message(transport);
        }
    }
}
