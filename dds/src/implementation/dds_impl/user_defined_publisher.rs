use std::sync::{mpsc::SyncSender, RwLockWriteGuard};

use crate::{
    implementation::{
        rtps::{
            group::RtpsGroup,
            messages::submessages::{AckNackSubmessage, NackFragSubmessage},
            types::Locator,
        },
        utils::{
            condvar::DdsCondvar,
            iterator::DdsListIntoIterator,
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        status::StatusKind,
        time::Duration,
    },
    publication::publisher_listener::PublisherListener,
    topic_definition::type_support::DdsType,
};

use super::{
    any_data_writer_listener::AnyDataWriterListener,
    domain_participant_impl::AnnounceKind,
    message_receiver::{MessageReceiver, PublisherMessageReceiver},
    status_condition_impl::StatusConditionImpl,
    status_listener::StatusListener,
    user_defined_data_writer::UserDefinedDataWriter,
    writer_factory::WriterFactory,
};

pub struct UserDefinedPublisher {
    qos: DdsRwLock<PublisherQos>,
    rtps_group: RtpsGroup,
    data_writer_list: DdsRwLock<Vec<DdsShared<UserDefinedDataWriter>>>,
    data_writer_factory: DdsRwLock<WriterFactory>,
    enabled: DdsRwLock<bool>,
    user_defined_data_send_condvar: DdsCondvar,
    status_listener: DdsRwLock<StatusListener<dyn PublisherListener + Send + Sync>>,
    data_max_size_serialized: usize,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    announce_sender: SyncSender<AnnounceKind>,
}

impl UserDefinedPublisher {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroup,
        listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
        user_defined_data_send_condvar: DdsCondvar,
        data_max_size_serialized: usize,
        announce_sender: SyncSender<AnnounceKind>,
    ) -> DdsShared<Self> {
        DdsShared::new(UserDefinedPublisher {
            qos: DdsRwLock::new(qos),
            rtps_group,
            data_writer_list: DdsRwLock::new(Vec::new()),
            data_writer_factory: DdsRwLock::new(WriterFactory::new()),
            enabled: DdsRwLock::new(false),
            user_defined_data_send_condvar,
            status_listener: DdsRwLock::new(StatusListener::new(listener, mask)),
            data_max_size_serialized,
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            announce_sender,
        })
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_datawriter<Foo>(
        &self,
        type_name: &'static str,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        mask: &[StatusKind],
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
    ) -> DdsResult<DdsShared<UserDefinedDataWriter>>
    where
        Foo: DdsType,
    {
        let rtps_writer_impl = self.data_writer_factory.write_lock().create_writer(
            &self.rtps_group,
            Foo::has_key(),
            qos,
            default_unicast_locator_list,
            default_multicast_locator_list,
            self.data_max_size_serialized,
        )?;

        let data_writer_shared = UserDefinedDataWriter::new(
            rtps_writer_impl,
            a_listener,
            mask,
            type_name,
            topic_name,
            self.user_defined_data_send_condvar.clone(),
            self.announce_sender.clone(),
        );

        self.data_writer_list
            .write_lock()
            .push(data_writer_shared.clone());

        Ok(data_writer_shared)
    }

    pub fn delete_datawriter(&self, data_writer_handle: InstanceHandle) -> DdsResult<()> {
        let data_writer_list = &mut self.data_writer_list.write_lock();
        let data_writer_list_position = data_writer_list
            .iter()
            .position(|x| InstanceHandle::from(x.guid()) == data_writer_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Data writer can only be deleted from its parent publisher".to_string(),
                )
            })?;
        let data_writer = data_writer_list.remove(data_writer_list_position);

        // The writer creation is announced only on enabled so its deletion must be announced only if it is enabled
        if data_writer.is_enabled() {
            self.announce_sender
                .send(AnnounceKind::DeletedDataWriter(data_writer.guid().into()))
                .ok();
        }

        Ok(())
    }

    pub fn data_writer_list(&self) -> DdsListIntoIterator<DdsShared<UserDefinedDataWriter>> {
        DdsListIntoIterator::new(self.data_writer_list.read_lock())
    }

    pub fn get_status_listener_lock(
        &self,
    ) -> RwLockWriteGuard<StatusListener<dyn PublisherListener + Send + Sync>> {
        self.status_listener.write_lock()
    }
}

impl DdsShared<UserDefinedPublisher> {
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
        for data_writer in self.data_writer_list.write_lock().drain(..) {
            // The writer creation is announced only on enabled so its deletion must be announced only if it is enabled
            if data_writer.is_enabled() {
                self.announce_sender
                    .send(AnnounceKind::DeletedDataWriter(data_writer.guid().into()))
                    .ok();
            }
        }

        Ok(())
    }

    pub fn set_default_datawriter_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        self.data_writer_factory
            .write_lock()
            .set_default_datawriter_qos(qos)
    }

    pub fn get_default_datawriter_qos(&self) -> DataWriterQos {
        self.data_writer_factory
            .read_lock()
            .get_default_datawriter_qos()
            .clone()
    }

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

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_status_changes(&self) -> Vec<StatusKind> {
        self.status_condition.read_lock().get_status_changes()
    }

    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
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

    fn on_nack_frag_submessage_received(
        &self,
        nackfrag_submessage: &NackFragSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        for data_writer in self.data_writer_list.read_lock().iter() {
            data_writer.on_nack_frag_submessage_received(nackfrag_submessage, message_receiver);
        }
    }
}
