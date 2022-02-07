use rust_dds_api::{
    dcps_psm::{
        InstanceHandle, InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask,
        ViewStateKind,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos}, qos_policy::ReliabilityQosPolicyKind,
    },
    return_type::{DDSResult, DDSError},
    subscription::{
        data_reader::AnyDataReader,
        data_reader_listener::DataReaderListener,
        subscriber::{Subscriber, SubscriberDataReaderFactory},
        subscriber_listener::SubscriberListener,
    },
};
use rust_rtps_pim::{structure::{types::{USER_DEFINED_WRITER_WITH_KEY, USER_DEFINED_WRITER_NO_KEY, TopicKind, EntityId, Guid, ReliabilityKind}, entity::RtpsEntityAttributes}, behavior::reader::{stateful_reader::RtpsStatefulReaderConstructor}};

use crate::{
    dds_type::{DdsDeserialize, DdsType},
    rtps_impl::{rtps_group_impl::RtpsGroupImpl},
    utils::{
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
    Rtps::StatefulReader: RtpsStatefulReaderConstructor,
{
    type TopicType = TopicProxy<Foo, Rtps>;
    type DataReaderType = DataReaderProxy<Foo, Rtps>;

    fn datareader_factory_create_datareader(
        &self,
        a_topic: &Self::TopicType,
        qos: Option<DataReaderQos>,
        _a_listener: Option<&'static dyn DataReaderListener>,
        _mask: StatusMask,
    ) -> Option<Self::DataReaderType> {
        let subscriber_shared = self.as_ref().upgrade().ok()?; // rtps_weak_upgrade(&self.subscriber_impl).ok()?;
        let mut subscriber_shared_lock = subscriber_shared.write().ok()?;
        
        // let topic_shared = a_topic.as_ref().upgrade().ok()?;

        let qos = qos.unwrap_or(subscriber_shared_lock.default_data_reader_qos.clone());
        qos.is_consistent().ok()?;

        let (entity_kind, topic_kind) = match Foo::has_key() {
            true => (USER_DEFINED_WRITER_WITH_KEY, TopicKind::WithKey),
            false => (USER_DEFINED_WRITER_NO_KEY, TopicKind::NoKey),
        };
        let entity_id = EntityId::new(
            [
                subscriber_shared_lock.rtps_group.guid().entity_id().entity_key()[0],
                subscriber_shared_lock.user_defined_data_reader_counter,
                0,
            ],
            entity_kind,
        );
        let guid = Guid::new(*subscriber_shared_lock.rtps_group.guid().prefix(), entity_id);
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };

        let heartbeat_response_delay = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let heartbeat_supression_duration = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let expects_inline_qos = false;
        let rtps_reader = RtpsReader::Stateful(Rtps::StatefulReader::new(
            guid,
            topic_kind,
            reliability_level,
            &[],
            &[],
            heartbeat_response_delay,
            heartbeat_supression_duration,
            expects_inline_qos,
        ));
        let reader_storage = DataReaderAttributes {
            rtps_reader,
            _qos: qos,
            topic: a_topic.as_ref().upgrade().ok()?,
            _listener: None,
            parent_subscriber: self.as_ref().clone(),
        };

        let reader_storage_shared = RtpsShared::new(reader_storage);

        subscriber_shared_lock.data_reader_list
            .push(reader_storage_shared.clone());

        Some(DataReaderProxy::new(reader_storage_shared.downgrade()))
    }

    fn datareader_factory_delete_datareader(
        &self,
        datareader: &Self::DataReaderType,
    ) -> DDSResult<()> {
        let subscriber_shared = self.as_ref().upgrade()?;
        let datareader_shared = datareader.as_ref().upgrade()?;

        let data_reader_list = &mut subscriber_shared
            .write().map_err(|_| DDSError::Error)?
            .data_reader_list;

        data_reader_list.remove(
            data_reader_list.iter().position(|x| x == &datareader_shared)
            .ok_or(DDSError::PreconditionNotMet(
                "Data reader can only be deleted from its parent subscriber".to_string(),
            ))?
        );

        Ok(())
    }

    fn datareader_factory_lookup_datareader(
        &self,
        _topic: &Self::TopicType,
    ) -> Option<Self::DataReaderType> {
        let subscriber_shared = self.as_ref().upgrade().ok()?;
        let data_reader_list = &subscriber_shared.write().ok()?.data_reader_list;

        data_reader_list.iter()
        .find(|x| {
            x.read_lock().topic.read_lock().type_name
            ==
            Foo::type_name()
        })
        .map(
            |found_data_reader| DataReaderProxy::new(found_data_reader.downgrade())
        )
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
