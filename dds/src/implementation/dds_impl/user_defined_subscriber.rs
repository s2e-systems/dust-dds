use crate::implementation::rtps::endpoint::RtpsEndpoint;
use crate::implementation::rtps::messages::submessages::{DataSubmessage, HeartbeatSubmessage};
use crate::implementation::rtps::reader::RtpsReader;
use crate::implementation::rtps::transport::TransportWrite;
use crate::implementation::rtps::types::{
    EntityId, Guid, GuidPrefix, TopicKind, USER_DEFINED_READER_NO_KEY, USER_DEFINED_READER_WITH_KEY,
};
use crate::implementation::rtps::{group::RtpsGroupImpl, stateful_reader::RtpsStatefulReader};
use crate::implementation::utils::condvar::DdsCondvar;
use crate::infrastructure::error::{DdsError, DdsResult};
use crate::infrastructure::instance::InstanceHandle;
use crate::infrastructure::qos::QosKind;
use crate::infrastructure::status::{SampleLostStatus, StatusKind};
use crate::infrastructure::time::DURATION_ZERO;
use crate::subscription::subscriber_listener::SubscriberListener;
use crate::topic_definition::type_support::DdsDeserialize;
use crate::{
    infrastructure::{
        condition::StatusCondition,
        qos::{DataReaderQos, SubscriberQos, TopicQos},
    },
    topic_definition::type_support::DdsType,
};

use crate::implementation::{
    data_representation_builtin_endpoints::discovered_writer_data::DiscoveredWriterData,
    utils::shared_object::{DdsRwLock, DdsShared},
};

use super::message_receiver::{MessageReceiver, SubscriberSubmessageReceiver};
use super::user_defined_data_reader::AnyDataReaderListener;
use super::{
    domain_participant_impl::DomainParticipantImpl, topic_impl::TopicImpl,
    user_defined_data_reader::UserDefinedDataReader,
};

pub struct UserDefinedSubscriber {
    qos: DdsRwLock<SubscriberQos>,
    rtps_group: RtpsGroupImpl,
    data_reader_list: DdsRwLock<Vec<DdsShared<UserDefinedDataReader>>>,
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
    enabled: DdsRwLock<bool>,
    user_defined_data_send_condvar: DdsCondvar,
}

impl UserDefinedSubscriber {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroupImpl,
        user_defined_data_send_condvar: DdsCondvar,
    ) -> DdsShared<Self> {
        DdsShared::new(UserDefinedSubscriber {
            qos: DdsRwLock::new(qos),
            rtps_group,
            data_reader_list: DdsRwLock::new(Vec::new()),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            enabled: DdsRwLock::new(false),
            user_defined_data_send_condvar,
        })
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }
}

impl DdsShared<UserDefinedSubscriber> {
    pub fn is_empty(&self) -> bool {
        self.data_reader_list.read_lock().is_empty()
    }
}

impl DdsShared<UserDefinedSubscriber> {
    pub fn create_datareader<Foo>(
        &self,
        a_topic: &DdsShared<TopicImpl>,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        _mask: &[StatusKind],
        parent_participant: &DdsShared<DomainParticipantImpl>,
    ) -> DdsResult<DdsShared<UserDefinedDataReader>>
    where
        Foo: DdsType + for<'de> DdsDeserialize<'de>,
    {
        // /////// Build the GUID
        let entity_id = {
            let entity_kind = match Foo::has_key() {
                true => USER_DEFINED_READER_WITH_KEY,
                false => USER_DEFINED_READER_NO_KEY,
            };

            EntityId::new(
                [
                    self.rtps_group.guid().entity_id().entity_key()[0],
                    self.user_defined_data_reader_counter,
                    0,
                ],
                entity_kind,
            )
        };

        let guid = Guid::new(self.rtps_group.guid().prefix(), entity_id);

        // /////// Create data reader
        let data_reader_shared = {
            let qos = match qos {
                QosKind::Default => self.default_data_reader_qos.clone(),
                QosKind::Specific(q) => q,
            };
            qos.is_consistent()?;

            let topic_kind = match Foo::has_key() {
                true => TopicKind::WithKey,
                false => TopicKind::NoKey,
            };

            let rtps_reader = RtpsStatefulReader::new(RtpsReader::new::<Foo>(
                RtpsEndpoint::new(
                    guid,
                    topic_kind,
                    parent_participant.default_unicast_locator_list(),
                    parent_participant.default_multicast_locator_list(),
                ),
                DURATION_ZERO,
                DURATION_ZERO,
                false,
                qos,
            ));

            let data_reader_shared = UserDefinedDataReader::new(
                rtps_reader,
                a_topic.clone(),
                a_listener,
                self.downgrade(),
                self.user_defined_data_send_condvar.clone(),
            );

            self.data_reader_list
                .write_lock()
                .push(data_reader_shared.clone());

            data_reader_shared
        };

        if *self.enabled.read_lock()
            && self
                .qos
                .read_lock()
                .entity_factory
                .autoenable_created_entities
        {
            data_reader_shared.enable(parent_participant)?;
        }

        Ok(data_reader_shared)
    }

    pub fn delete_datareader(&self, a_datareader_handle: InstanceHandle) -> DdsResult<()> {
        let data_reader_list = &mut self.data_reader_list.write_lock();
        let data_reader_list_position = data_reader_list
            .iter()
            .position(|x| x.get_instance_handle() == a_datareader_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Data reader can only be deleted from its parent subscriber".to_string(),
                )
            })?;
        data_reader_list.remove(data_reader_list_position);

        Ok(())
    }

    pub fn lookup_datareader<Foo>(
        &self,
        topic: &DdsShared<TopicImpl>,
    ) -> DdsResult<DdsShared<UserDefinedDataReader>>
    where
        Foo: DdsType,
    {
        let data_reader_list = &self.data_reader_list.write_lock();

        data_reader_list
            .iter()
            .find_map(|data_reader_shared| {
                let data_reader_topic = data_reader_shared.get_topicdescription();

                if data_reader_topic.get_name().ok()? == topic.get_name().ok()?
                    && data_reader_topic.get_type_name().ok()? == Foo::type_name()
                {
                    Some(data_reader_shared.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| DdsError::PreconditionNotMet("Not found".to_string()))
    }

    pub fn notify_datareaders(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        todo!()
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn set_default_datareader_qos(&self, _qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        todo!()
    }

    pub fn get_default_datareader_qos(&self) -> DdsResult<DataReaderQos> {
        todo!()
    }
}

impl UserDefinedSubscriber {
    pub fn copy_from_topic_qos(
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }
}

impl DdsShared<UserDefinedSubscriber> {
    pub fn set_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
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

    pub fn get_qos(&self) -> SubscriberQos {
        self.qos.read_lock().clone()
    }

    pub fn set_listener(
        &self,
        _a_listener: Option<Box<dyn SubscriberListener>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
    }

    pub fn get_listener(&self) -> DdsResult<Option<Box<dyn SubscriberListener>>> {
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
            for data_reader in self.data_reader_list.read_lock().iter() {
                data_reader.enable(parent_participant)?;
            }
        }

        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }
}

impl DdsShared<UserDefinedSubscriber> {
    pub fn add_matched_writer(&self, discovered_writer_data: &DiscoveredWriterData) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            data_reader.add_matched_writer(discovered_writer_data)
        }
    }
}

impl SubscriberSubmessageReceiver for DdsShared<UserDefinedSubscriber> {
    fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            data_reader.on_data_submessage_received(data_submessage, message_receiver)
        }
    }

    fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            data_reader.on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix)
        }
    }
}

impl DdsShared<UserDefinedSubscriber> {
    pub fn send_message(&self, transport: &mut impl TransportWrite) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            data_reader.send_message(transport);
        }
    }
}
