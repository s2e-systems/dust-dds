use std::sync::Mutex;

use rust_dds_api::{
    dcps_psm::{
        InstanceHandle, InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask,
        ViewStateKind,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos},
    },
    return_type::{DDSError, DDSResult},
    subscription::{
        data_reader::{AnyDataReader, DataReader},
        data_reader_listener::DataReaderListener,
        subscriber::{Subscriber, SubscriberDataReaderFactory},
        subscriber_listener::SubscriberListener,
    },
};
use rust_rtps_pim::{
    behavior::stateful_reader_behavior::StatefulReaderBehavior, structure::types::GuidPrefix,
};
use rust_rtps_psm::messages::submessages::DataSubmessageRead;

use crate::{
    dds_type::{DdsDeserialize, DdsType},
    rtps_impl::rtps_group_impl::RtpsGroupImpl,
    utils::{
        message_receiver::ProcessDataSubmessage,
        shared_object::{
            rtps_shared_downgrade, rtps_shared_read_lock, rtps_shared_write_lock,
            rtps_weak_upgrade, RtpsShared, RtpsWeak,
        },
    },
};

use super::{
    data_reader_impl::{DataReaderImpl, RtpsReader},
    data_reader_proxy::DataReaderProxy,
    domain_participant_proxy::DomainParticipantProxy,
    topic_proxy::TopicProxy,
};

pub struct SubscriberAttributes {
    qos: SubscriberQos,
    rtps_group: RtpsGroupImpl,
    pub data_reader_list: Mutex<Vec<RtpsShared<DataReaderImpl>>>,
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
}

impl SubscriberAttributes {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroupImpl,
        data_reader_list: Vec<RtpsShared<DataReaderImpl>>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_reader_list: Mutex::new(data_reader_list),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
        }
    }
}

impl ProcessDataSubmessage for SubscriberAttributes {
    fn process_data_submessage(
        &mut self,
        source_guid_prefix: GuidPrefix,
        data: &DataSubmessageRead,
    ) {
        {
            let data_reader_list_lock = self.data_reader_list.lock().unwrap();
            for data_reader in data_reader_list_lock.iter().cloned() {
                let mut as_mut_rtps_reader_lock = rtps_shared_write_lock(&data_reader);
                let rtps_reader = as_mut_rtps_reader_lock.as_mut();
                match rtps_reader {
                    RtpsReader::Stateless(stateless_rtps_reader) => {
                        for mut stateless_reader_behavior in stateless_rtps_reader.into_iter() {
                            stateless_reader_behavior.receive_data(source_guid_prefix, data)
                        }
                    }
                    RtpsReader::Stateful(stateful_rtps_reader) => {
                        for stateful_reader_behavior in stateful_rtps_reader.into_iter() {
                            match stateful_reader_behavior {
                                StatefulReaderBehavior::BestEffort(_) => todo!(),
                                StatefulReaderBehavior::Reliable(mut reliable_stateful_reader) => {
                                    reliable_stateful_reader.receive_data(source_guid_prefix, data)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct SubscriberProxy {
    participant: DomainParticipantProxy,
    subscriber_impl: RtpsWeak<SubscriberAttributes>,
}

impl SubscriberProxy {
    pub fn new(
        participant: DomainParticipantProxy,
        subscriber_impl: RtpsWeak<SubscriberAttributes>,
    ) -> Self {
        Self {
            participant,
            subscriber_impl,
        }
    }
}

impl AsRef<RtpsWeak<SubscriberAttributes>> for SubscriberProxy {
    fn as_ref(&self) -> &RtpsWeak<SubscriberAttributes> {
        &self.subscriber_impl
    }
}

impl<Foo> SubscriberDataReaderFactory<Foo> for SubscriberProxy
where
    Foo: DdsType + for<'a> DdsDeserialize<'a> + Send + Sync + 'static,
{
    type TopicType = TopicProxy<Foo>;
    type DataReaderType = DataReaderProxy<Foo>;

    fn datareader_factory_create_datareader(
        &self,
        a_topic: &Self::TopicType,
        qos: Option<DataReaderQos>,
        a_listener: Option<&'static dyn DataReaderListener>,
        mask: StatusMask,
    ) -> Option<Self::DataReaderType> {
        // let subscriber_shared = rtps_weak_upgrade(&self.subscriber_impl).ok()?;
        // let topic_shared = rtps_weak_upgrade(a_topic.as_ref()).ok()?;
        // let data_reader_shared =
        //     SubscriberDataReaderFactory::<Foo>::datareader_factory_create_datareader(
        //         &*rtps_shared_write_lock(&subscriber_shared),
        //         &topic_shared,
        //         qos,
        //         a_listener,
        //         mask,
        //     )?;
        // let qos = qos.unwrap_or(self.default_data_reader_qos.clone());
        // qos.is_consistent().ok()?;

        // let (entity_kind, topic_kind) = match Foo::has_key() {
        //     true => (USER_DEFINED_WRITER_WITH_KEY, TopicKind::WithKey),
        //     false => (USER_DEFINED_WRITER_NO_KEY, TopicKind::NoKey),
        // };
        // let entity_id = EntityId::new(
        //     [
        //         self.rtps_group.guid().entity_id().entity_key()[0],
        //         self.user_defined_data_reader_counter,
        //         0,
        //     ],
        //     entity_kind,
        // );
        // let guid = Guid::new(*self.rtps_group.guid().prefix(), entity_id);
        // let reliability_level = match qos.reliability.kind {
        //     ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
        //     ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        // };

        // let heartbeat_response_delay = rust_rtps_pim::behavior::types::DURATION_ZERO;
        // let heartbeat_supression_duration = rust_rtps_pim::behavior::types::DURATION_ZERO;
        // let expects_inline_qos = false;
        // let rtps_reader = RtpsReader::Stateful(RtpsStatefulReaderImpl::new(
        //     guid,
        //     topic_kind,
        //     reliability_level,
        //     &[],
        //     &[],
        //     heartbeat_response_delay,
        //     heartbeat_supression_duration,
        //     expects_inline_qos,
        // ));
        // let reader_storage = DataReaderImpl::new(qos, rtps_reader, a_topic.clone());
        // let reader_storage_shared = rtps_shared_new(reader_storage);
        // self.data_reader_list
        //     .lock()
        //     .unwrap()
        //     .push(reader_storage_shared.clone());
        // Some(reader_storage_shared)
        // let data_reader_weak = rtps_shared_downgrade(&data_reader_shared);
        // let data_reader = DataReaderProxy::new(self.clone(), a_topic.clone(), data_reader_weak);
        // Some(data_reader)
        todo!()
    }

    fn datareader_factory_delete_datareader(
        &self,
        a_datareader: &Self::DataReaderType,
    ) -> DDSResult<()> {
        if std::ptr::eq(&a_datareader.get_subscriber()?, self) {
            let datareader_shared = rtps_weak_upgrade(a_datareader.as_ref())?;
            todo!()
            // SubscriberDataReaderFactory::<Foo>::datareader_factory_delete_datareader(
            //     &*rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?),
            //     &datareader_shared,
            // )
        } else {
            Err(DDSError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher".to_string(),
            ))
        }
    }

    fn datareader_factory_lookup_datareader(
        &self,
        _topic: &Self::TopicType,
    ) -> Option<Self::DataReaderType> {
        // let data_reader_list_lock = self.data_reader_list.lock().unwrap();
        // let found_data_reader = data_reader_list_lock.iter().cloned().find(|x| {
        //     rtps_shared_read_lock(&rtps_shared_read_lock(x).topic)
        //         .type_name
        //         == Foo::type_name()
        // });

        // if let Some(found_data_reader) = found_data_reader {
        //     return Some(found_data_reader);
        // };

        // None
        todo!()
    }
}

impl Subscriber for SubscriberProxy {
    type DomainParticipant = DomainParticipantProxy;

    fn begin_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_datareaders(
        &self,
        _readers: &mut [&mut dyn AnyDataReader],
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn notify_datareaders(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> DDSResult<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datareader_qos(&self, _qos: Option<DataReaderQos>) -> DDSResult<()> {
        todo!()
    }

    fn get_default_datareader_qos(&self) -> DDSResult<DataReaderQos> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_participant(&self) -> Self::DomainParticipant {
        self.participant.clone()
    }
}

impl Entity for SubscriberProxy {
    type Qos = SubscriberQos;
    type Listener = &'static dyn SubscriberListener;

    fn set_qos(&mut self, qos: Option<Self::Qos>) -> DDSResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_qos()
        todo!()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DDSResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?)
        // .set_listener(a_listener, mask)
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_listener()
        todo!()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_statuscondition()
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_status_changes()
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).enable()
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_instance_handle()
        todo!()
    }
}
