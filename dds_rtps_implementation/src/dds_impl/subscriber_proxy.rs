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
    behavior::{
        reader::{
            reader::RtpsReaderAttributes,
            stateful_reader::RtpsStatefulReaderAttributes,
            writer_proxy::{RtpsWriterProxyAttributes, RtpsWriterProxyOperations},
        },
        stateful_reader_behavior::StatefulReaderBehavior,
        stateless_reader_behavior::BestEffortStatelessReaderBehavior,
    },
    structure::{
        cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
        types::GuidPrefix,
    },
};
use rust_rtps_psm::messages::{submessage_elements::Parameter, submessages::DataSubmessageRead};

use crate::{
    dds_type::{DdsDeserialize, DdsType},
    rtps_impl::rtps_group_impl::RtpsGroupImpl,
    utils::{
        message_receiver::ProcessDataSubmessage,
        rtps_structure::RtpsStructure,
        shared_object::{RtpsShared, RtpsWeak},
    },
};

use super::{
    data_reader_proxy::{DataReaderAttributes, DataReaderProxy, RtpsReader},
    domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
    topic_proxy::TopicProxy,
};

pub struct SubscriberAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub qos: SubscriberQos,
    pub rtps_group: RtpsGroupImpl,
    pub data_reader_list: Vec<RtpsShared<DataReaderAttributes<Rtps>>>,
    pub user_defined_data_reader_counter: u8,
    pub default_data_reader_qos: DataReaderQos,
    pub parent_domain_participant: RtpsWeak<DomainParticipantAttributes<Rtps>>,
}

impl<Rtps> SubscriberAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroupImpl,
        parent_domain_participant: RtpsWeak<DomainParticipantAttributes<Rtps>>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_reader_list: Vec::new(),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant,
        }
    }
}

impl<Rtps> ProcessDataSubmessage for SubscriberAttributes<Rtps>
where
    Rtps: RtpsStructure,
    Rtps::StatelessReader: RtpsReaderAttributes,
    <Rtps::StatelessReader as RtpsReaderAttributes>::ReaderHistoryCacheType:
        RtpsHistoryCacheOperations,
    for<'a> < <Rtps::StatelessReader as RtpsReaderAttributes>::ReaderHistoryCacheType as RtpsHistoryCacheOperations>::CacheChangeType: RtpsCacheChangeConstructor<'a, DataType = [u8], ParameterListType = [Parameter<'a>]>,
    for<'a> &'a mut Rtps::StatelessReader: IntoIterator<
        Item = BestEffortStatelessReaderBehavior<
            'a,
            <Rtps::StatelessReader as RtpsReaderAttributes>::ReaderHistoryCacheType,
        >,
    >,
    Rtps::StatefulReader: RtpsReaderAttributes + RtpsStatefulReaderAttributes,
    <Rtps::StatefulReader as RtpsReaderAttributes>::ReaderHistoryCacheType: RtpsHistoryCacheOperations,
    for<'a> <<Rtps::StatefulReader as RtpsReaderAttributes>::ReaderHistoryCacheType as RtpsHistoryCacheOperations>::CacheChangeType: RtpsCacheChangeConstructor<'a, DataType = [u8], ParameterListType = [Parameter<'a>]> + RtpsCacheChangeAttributes,
    <Rtps::StatefulReader as RtpsStatefulReaderAttributes>::WriterProxyType: RtpsWriterProxyOperations + RtpsWriterProxyAttributes,
    for<'a> &'a mut Rtps::StatefulReader: IntoIterator<
        Item = StatefulReaderBehavior<
            'a,
            <Rtps::StatefulReader as RtpsStatefulReaderAttributes>::WriterProxyType,
            <Rtps::StatefulReader as RtpsReaderAttributes>::ReaderHistoryCacheType,
        >,
    >,
{
    fn process_data_submessage(
        &mut self,
        source_guid_prefix: GuidPrefix,
        data: &DataSubmessageRead,
    ) {
        for data_reader in self.data_reader_list.iter().cloned() {
            let mut as_mut_rtps_reader_lock = data_reader.write_lock();
            let rtps_reader = &mut as_mut_rtps_reader_lock.rtps_reader;
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

#[derive(Clone)]
pub struct SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    participant: DomainParticipantProxy<Rtps>,
    subscriber_impl: RtpsWeak<SubscriberAttributes<Rtps>>,
}

impl<Rtps> SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(
        participant: DomainParticipantProxy<Rtps>,
        subscriber_impl: RtpsWeak<SubscriberAttributes<Rtps>>,
    ) -> Self {
        Self {
            participant,
            subscriber_impl,
        }
    }
}

impl<Rtps> AsRef<RtpsWeak<SubscriberAttributes<Rtps>>> for SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    fn as_ref(&self) -> &RtpsWeak<SubscriberAttributes<Rtps>> {
        &self.subscriber_impl
    }
}

impl<Foo, Rtps> SubscriberDataReaderFactory<Foo> for SubscriberProxy<Rtps>
where
    Foo: DdsType + for<'a> DdsDeserialize<'a> + Send + Sync + 'static,
    Rtps: RtpsStructure,
    Rtps::StatelessReader: RtpsReaderAttributes,
    Rtps::StatefulReader: RtpsReaderAttributes,
    <Rtps::StatelessReader as RtpsReaderAttributes>::ReaderHistoryCacheType:
        RtpsHistoryCacheAttributes,
    <Rtps::StatefulReader as RtpsReaderAttributes>::ReaderHistoryCacheType:
        RtpsHistoryCacheAttributes,
    <<Rtps::StatelessReader as RtpsReaderAttributes>::ReaderHistoryCacheType as RtpsHistoryCacheAttributes>::CacheChangeType: RtpsCacheChangeAttributes<DataType = [u8]>,
    <<Rtps::StatefulReader as RtpsReaderAttributes>::ReaderHistoryCacheType as RtpsHistoryCacheAttributes>::CacheChangeType: RtpsCacheChangeAttributes<DataType = [u8]>,
{
    type TopicType = TopicProxy<Foo, Rtps>;
    type DataReaderType = DataReaderProxy<Foo, Rtps>;

    fn datareader_factory_create_datareader(
        &self,
        _a_topic: &Self::TopicType,
        _qos: Option<DataReaderQos>,
        _a_listener: Option<&'static dyn DataReaderListener>,
        _mask: StatusMask,
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
            let _datareader_shared = a_datareader.as_ref().upgrade()?;
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

impl<Rtps> Subscriber for SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    type DomainParticipant = DomainParticipantProxy<Rtps>;

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

impl<Rtps> Entity for SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    type Qos = SubscriberQos;
    type Listener = &'static dyn SubscriberListener;

    fn set_qos(&mut self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.subscriber_impl)?).get_qos()
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
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
