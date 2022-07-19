use crate::{
    rtps_impl::{
        rtps_group_impl::RtpsGroupImpl, rtps_stateful_reader_impl::RtpsStatefulReaderImpl,
    },
    transport::TransportWrite,
};
use dds_api::{
    dcps_psm::{
        InstanceHandle, InstanceStateMask, SampleLostStatus, SampleStateMask, StatusMask,
        ViewStateMask,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos},
        qos_policy::ReliabilityQosPolicyKind,
    },
    return_type::{DdsError, DdsResult},
    subscription::{
        data_reader::{AnyDataReader, DataReaderGetTopicDescription},
        subscriber::{Subscriber, SubscriberDataReaderFactory, SubscriberGetParticipant},
        subscriber_listener::SubscriberListener,
    },
    topic::topic_description::TopicDescription,
};
use rtps_pim::{
    behavior::reader::stateful_reader::RtpsStatefulReaderConstructor,
    messages::{
        submessage_elements::Parameter,
        submessages::{DataSubmessage, HeartbeatSubmessage},
    },
    structure::{
        entity::RtpsEntityAttributes,
        participant::RtpsParticipantAttributes,
        types::{
            EntityId, Guid, GuidPrefix, ReliabilityKind, TopicKind, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
    },
};

use crate::{
    data_representation_builtin_endpoints::{
        discovered_reader_data::{DiscoveredReaderData, RtpsReaderProxy},
        discovered_writer_data::DiscoveredWriterData,
    },
    dds_type::DdsType,
    utils::{
        discovery_traits::AddMatchedWriter,
        rtps_communication_traits::{
            ReceiveRtpsDataSubmessage, ReceiveRtpsHeartbeatSubmessage, SendRtpsMessage,
        },
        shared_object::{DdsRwLock, DdsShared, DdsWeak},
        timer::ThreadTimer,
    },
};

use super::{
    data_reader_impl::{DataReaderImpl, RtpsReader},
    domain_participant_impl::{DataReaderDiscovery, DomainParticipantImpl},
    topic_impl::TopicImpl,
};

pub struct SubscriberImpl {
    qos: DdsRwLock<SubscriberQos>,
    rtps_group: RtpsGroupImpl,
    data_reader_list: DdsRwLock<Vec<DdsShared<DataReaderImpl<ThreadTimer>>>>,
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
    parent_domain_participant: DdsWeak<DomainParticipantImpl>,
    enabled: DdsRwLock<bool>,
}

impl SubscriberImpl {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroupImpl,
        parent_domain_participant: DdsWeak<DomainParticipantImpl>,
    ) -> DdsShared<Self> {
        DdsShared::new(SubscriberImpl {
            qos: DdsRwLock::new(qos),
            rtps_group,
            data_reader_list: DdsRwLock::new(Vec::new()),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant,
            enabled: DdsRwLock::new(false),
        })
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }
}

pub trait SubscriberEmpty {
    fn is_empty(&self) -> bool;
}

impl SubscriberEmpty for DdsShared<SubscriberImpl> {
    fn is_empty(&self) -> bool {
        self.data_reader_list.read_lock().is_empty()
    }
}
pub trait AddDataReader {
    fn add_data_reader(&self, reader: DdsShared<DataReaderImpl<ThreadTimer>>);
}

impl AddDataReader for DdsShared<SubscriberImpl> {
    fn add_data_reader(&self, reader: DdsShared<DataReaderImpl<ThreadTimer>>) {
        self.data_reader_list.write_lock().push(reader);
    }
}

impl<Foo> SubscriberDataReaderFactory<Foo> for DdsShared<SubscriberImpl>
where
    Foo: DdsType,
{
    type TopicType = DdsShared<TopicImpl>;
    type DataReaderType = DdsShared<DataReaderImpl<ThreadTimer>>;

    fn datareader_factory_create_datareader(
        &self,
        a_topic: &Self::TopicType,
        qos: Option<DataReaderQos>,
        a_listener: Option<<Self::DataReaderType as Entity>::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<Self::DataReaderType>
    where
        Self::DataReaderType: Entity,
    {
        // /////// Build the GUID
        let entity_id = {
            let entity_kind = match Foo::has_key() {
                true => USER_DEFINED_WRITER_WITH_KEY,
                false => USER_DEFINED_WRITER_NO_KEY,
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
            let qos = qos.unwrap_or(self.default_data_reader_qos.clone());
            qos.is_consistent()?;

            let topic_kind = match Foo::has_key() {
                true => TopicKind::WithKey,
                false => TopicKind::NoKey,
            };

            let reliability_level = match qos.reliability.kind {
                ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
                ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
            };

            let domain_participant = self.parent_domain_participant.upgrade().ok();
            let rtps_reader = RtpsReader::Stateful(RtpsStatefulReaderImpl::new(
                guid,
                topic_kind,
                reliability_level,
                domain_participant
                    .as_ref()
                    .map(|dp| dp.default_unicast_locator_list())
                    .unwrap_or(&[]),
                domain_participant
                    .as_ref()
                    .map(|dp| dp.default_multicast_locator_list())
                    .unwrap_or(&[]),
                rtps_pim::behavior::types::DURATION_ZERO,
                rtps_pim::behavior::types::DURATION_ZERO,
                false,
            ));

            let data_reader_shared = DataReaderImpl::new(
                qos,
                rtps_reader,
                a_topic.clone(),
                a_listener,
                self.downgrade(),
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
            data_reader_shared.enable()?;
        }

        Ok(data_reader_shared)
    }

    fn datareader_factory_delete_datareader(
        &self,
        a_datareader: &Self::DataReaderType,
    ) -> DdsResult<()> {
        let data_reader_list = &mut self.data_reader_list.write_lock();
        let data_reader_list_position = data_reader_list
            .iter()
            .position(|x| x == a_datareader)
            .ok_or(DdsError::PreconditionNotMet(
                "Data reader can only be deleted from its parent subscriber".to_string(),
            ))?;
        data_reader_list.remove(data_reader_list_position);

        Ok(())
    }

    fn datareader_factory_lookup_datareader(
        &self,
        topic: &Self::TopicType,
    ) -> DdsResult<Self::DataReaderType> {
        let data_reader_list = &self.data_reader_list.write_lock();

        data_reader_list
            .iter()
            .find_map(|data_reader_shared| {
                let data_reader_topic = data_reader_shared
                    .data_reader_get_topicdescription()
                    .unwrap();

                if data_reader_topic.get_name().ok()? == topic.get_name().ok()?
                    && data_reader_topic.get_type_name().ok()? == Foo::type_name()
                {
                    Some(data_reader_shared.clone())
                } else {
                    None
                }
            })
            .ok_or(DdsError::PreconditionNotMet("Not found".to_string()))
    }
}

pub trait AnnounceDataReader {
    fn announce_datareader(&self, sedp_discovered_reader_data: DiscoveredReaderData);
}

impl AnnounceDataReader for DdsShared<SubscriberImpl> {
    fn announce_datareader(&self, sedp_discovered_reader_data: DiscoveredReaderData) {
        if let Ok(domain_participant) = self.parent_domain_participant.upgrade() {
            domain_participant.add_created_data_reader(&DiscoveredReaderData {
                reader_proxy: RtpsReaderProxy {
                    unicast_locator_list: domain_participant
                        .default_unicast_locator_list()
                        .to_vec(),
                    multicast_locator_list: domain_participant
                        .default_multicast_locator_list()
                        .to_vec(),
                    ..sedp_discovered_reader_data.reader_proxy
                },
                ..sedp_discovered_reader_data
            });
        }
    }
}

impl Subscriber for DdsShared<SubscriberImpl> {
    fn begin_access(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn end_access(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn get_datareaders(
        &self,
        _readers: &mut [&mut dyn AnyDataReader],
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn notify_datareaders(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> DdsResult<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn set_default_datareader_qos(&self, _qos: Option<DataReaderQos>) -> DdsResult<()> {
        todo!()
    }

    fn get_default_datareader_qos(&self) -> DdsResult<DataReaderQos> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }
}

impl SubscriberGetParticipant for DdsShared<SubscriberImpl> {
    type DomainParticipant = DdsShared<DomainParticipantImpl>;

    fn subscriber_get_participant(&self) -> DdsResult<Self::DomainParticipant> {
        Ok(self
            .parent_domain_participant
            .upgrade()
            .expect("Failed to get parent participant of subscriber"))
    }
}

impl Entity for DdsShared<SubscriberImpl> {
    type Qos = SubscriberQos;
    type Listener = Box<dyn SubscriberListener>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DdsResult<()> {
        let qos = qos.unwrap_or_default();

        if *self.enabled.read_lock() {
            self.qos.read_lock().check_immutability(&qos)?;
        }

        *self.qos.write_lock() = qos;

        Ok(())
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        Ok(self.qos.read_lock().clone())
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        todo!()
    }

    fn enable(&self) -> DdsResult<()> {
        if !self.parent_domain_participant.upgrade()?.is_enabled() {
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
                data_reader.enable()?;
            }
        }

        Ok(())
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self.rtps_group.guid().into())
    }
}

impl AddMatchedWriter for DdsShared<SubscriberImpl> {
    fn add_matched_writer(&self, discovered_writer_data: &DiscoveredWriterData) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            data_reader.add_matched_writer(&discovered_writer_data)
        }
    }
}

impl ReceiveRtpsDataSubmessage for DdsShared<SubscriberImpl> {
    fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<Vec<Parameter>, &[u8]>,
        source_guid_prefix: GuidPrefix,
    ) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            data_reader.on_data_submessage_received(data_submessage, source_guid_prefix)
        }
    }
}

impl ReceiveRtpsHeartbeatSubmessage for DdsShared<SubscriberImpl> {
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

impl SendRtpsMessage for DdsShared<SubscriberImpl> {
    fn send_message(&self, transport: &mut impl TransportWrite) {
        for data_reader in self.data_reader_list.read_lock().iter() {
            data_reader.send_message(transport);
        }
    }
}

#[cfg(test)]
mod tests {
    use dds_api::return_type::DdsError;
    use rtps_pim::structure::{group::RtpsGroupConstructor, types::GUID_UNKNOWN};

    use crate::dds_type::{DdsDeserialize, DdsType};

    use super::*;

    macro_rules! make_empty_dds_type {
        ($type_name:ident) => {
            struct $type_name {}

            impl<'de> DdsDeserialize<'de> for $type_name {
                fn deserialize(_buf: &mut &'de [u8]) -> DdsResult<Self> {
                    Ok($type_name {})
                }
            }

            impl DdsType for $type_name {
                fn type_name() -> &'static str {
                    stringify!($type_name)
                }

                fn has_key() -> bool {
                    false
                }
            }
        };
    }

    make_empty_dds_type!(Foo);

    #[test]
    fn create_datareader() {
        let subscriber_attributes = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
        );

        let subscriber = DdsShared::new(subscriber_attributes);

        let topic = DdsShared::new(TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let data_reader = subscriber.create_datareader::<Foo>(&topic, None, None, 0);

        assert!(data_reader.is_ok());
    }

    #[test]
    fn datareader_factory_delete_datareader() {
        let subscriber_attributes = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
        );
        let subscriber = DdsShared::new(subscriber_attributes);

        let topic = DdsShared::new(TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let data_reader = subscriber
            .create_datareader::<Foo>(&topic, None, None, 0)
            .unwrap();

        assert_eq!(1, subscriber.data_reader_list.read_lock().len());

        subscriber.delete_datareader::<Foo>(&data_reader).unwrap();
        assert_eq!(0, subscriber.data_reader_list.read_lock().len());
    }

    #[test]
    fn datareader_factory_delete_datareader_from_other_subscriber() {
        let subscriber_attributes = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
        );
        let subscriber = DdsShared::new(subscriber_attributes);

        let subscriber2_attributes = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
        );
        let subscriber2 = DdsShared::new(subscriber2_attributes);

        let topic = DdsShared::new(TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let data_reader = subscriber
            .create_datareader::<Foo>(&topic, None, None, 0)
            .unwrap();

        assert_eq!(1, subscriber.data_reader_list.read_lock().len());
        assert_eq!(0, subscriber2.data_reader_list.read_lock().len());

        assert!(matches!(
            subscriber2.delete_datareader::<Foo>(&data_reader),
            Err(DdsError::PreconditionNotMet(_))
        ));
    }

    #[test]
    fn datareader_factory_lookup_datareader_when_empty() {
        let subscriber_attributes = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
        );
        let subscriber = DdsShared::new(subscriber_attributes);

        let topic = DdsShared::new(TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        assert!(subscriber.lookup_datareader::<Foo>(&topic).is_err());
    }

    #[test]
    fn datareader_factory_lookup_datareader_when_one_datareader() {
        let subscriber_attributes = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
        );
        let subscriber = DdsShared::new(subscriber_attributes);

        let topic = DdsShared::new(TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let data_reader = subscriber
            .create_datareader::<Foo>(&topic, None, None, 0)
            .unwrap();

        assert!(subscriber.lookup_datareader::<Foo>(&topic).unwrap() == data_reader);
    }

    make_empty_dds_type!(Bar);

    #[test]
    fn datareader_factory_lookup_datareader_when_one_datareader_with_wrong_type() {
        let subscriber_attributes = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
        );
        let subscriber = DdsShared::new(subscriber_attributes);

        let topic_foo = DdsShared::new(TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let topic_bar = DdsShared::new(TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            Bar::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        subscriber
            .create_datareader::<Bar>(&topic_bar, None, None, 0)
            .unwrap();

        assert!(subscriber.lookup_datareader::<Foo>(&topic_foo).is_err());
    }

    #[test]
    fn datareader_factory_lookup_datareader_when_one_datareader_with_wrong_topic() {
        let subscriber_attributes = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
        );
        let subscriber = DdsShared::new(subscriber_attributes);

        let topic1 = DdsShared::new(TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            Foo::type_name(),
            "topic1",
            DdsWeak::new(),
        ));

        let topic2 = DdsShared::new(TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            Foo::type_name(),
            "topic2",
            DdsWeak::new(),
        ));

        subscriber
            .create_datareader::<Foo>(&topic2, None, None, 0)
            .unwrap();

        assert!(subscriber.lookup_datareader::<Foo>(&topic1).is_err());
    }

    #[test]
    fn datareader_factory_lookup_datareader_with_two_types() {
        let subscriber_attributes = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
        );
        let subscriber = DdsShared::new(subscriber_attributes);

        let topic_foo = DdsShared::new(TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let topic_bar = DdsShared::new(TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            Bar::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let data_reader_foo = subscriber
            .create_datareader::<Foo>(&topic_foo, None, None, 0)
            .unwrap();
        let data_reader_bar = subscriber
            .create_datareader::<Bar>(&topic_bar, None, None, 0)
            .unwrap();

        assert!(subscriber.lookup_datareader::<Foo>(&topic_foo).unwrap() == data_reader_foo);

        assert!(subscriber.lookup_datareader::<Bar>(&topic_bar).unwrap() == data_reader_bar);
    }

    #[test]
    fn datareader_factory_lookup_datareader_with_two_topics() {
        let subscriber_attributes = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
        );
        let subscriber = DdsShared::new(subscriber_attributes);

        let topic1 = DdsShared::new(TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            Foo::type_name(),
            "topic1",
            DdsWeak::new(),
        ));

        let topic2 = DdsShared::new(TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            Foo::type_name(),
            "topic2",
            DdsWeak::new(),
        ));

        let data_reader1 = subscriber
            .create_datareader::<Foo>(&topic1, None, None, 0)
            .unwrap();
        let data_reader2 = subscriber
            .create_datareader::<Foo>(&topic2, None, None, 0)
            .unwrap();

        assert!(subscriber.lookup_datareader::<Foo>(&topic1).unwrap() == data_reader1);

        assert!(subscriber.lookup_datareader::<Foo>(&topic2).unwrap() == data_reader2);
    }

    #[test]
    fn get_instance_handle() {
        let guid = Guid::new(
            GuidPrefix([2; 12]),
            EntityId {
                entity_key: [3; 3],
                entity_kind: 1,
            },
        );
        let subscriber = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(guid),
            DdsWeak::new(),
        );
        *subscriber.enabled.write_lock() = true;

        let expected_instance_handle: [u8; 16] = guid.into();
        let instance_handle = subscriber.get_instance_handle().unwrap();
        assert_eq!(expected_instance_handle, instance_handle);
    }
}
