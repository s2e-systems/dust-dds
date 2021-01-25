use std::sync::{atomic, Arc, Mutex};

use crate::utils::maybe_valid::{MaybeValid, MaybeValidList, MaybeValidNode, MaybeValidRef};
use rust_dds_api::{domain::domain_participant::DomainParticipant, infrastructure::{entity::{Entity, StatusCondition}, qos::{DataReaderQos, SubscriberQos, TopicQos}, status::{InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask, ViewStateKind}}, publication::publisher::Publisher, subscription::{data_reader::{AnyDataReader, DataReader}, data_reader_listener::DataReaderListener, subscriber::Subscriber, subscriber_listener::SubscriberListener}, topic::topic::Topic};
use rust_dds_types::{DDSType, InstanceHandle, ReturnCode, ReturnCodes, TopicKind};
use rust_rtps::{
    structure::Group,
    types::{
        constants::{
            ENTITY_KIND_BUILT_IN_READER_WITH_KEY, ENTITY_KIND_USER_DEFINED_READER_NO_KEY,
            ENTITY_KIND_USER_DEFINED_READER_WITH_KEY,
        },
        EntityId, GUID,
    },
};

use super::{
    rtps_datareader::{AnyRtpsReader, RtpsAnyDataReaderRef, RtpsDataReader},
    rtps_participant::RtpsParticipant,
    rtps_topic::AnyRtpsTopic,
};

enum EntityType {
    BuiltIn,
    UserDefined,
}
pub struct RtpsSubscriber {
    pub group: Group,
    pub reader_list: MaybeValidList<Box<dyn AnyRtpsReader>>,
    pub reader_count: atomic::AtomicU8,
    pub default_datareader_qos: Mutex<DataReaderQos>,
    pub qos: SubscriberQos,
    pub listener: Option<Box<dyn SubscriberListener>>,
    pub status_mask: StatusMask,
}

impl RtpsSubscriber {
    pub fn new(
        guid: GUID,
        qos: SubscriberQos,
        listener: Option<Box<dyn SubscriberListener>>,
        status_mask: StatusMask,
    ) -> Self {
        Self {
            group: Group::new(guid),
            reader_list: Default::default(),
            reader_count: atomic::AtomicU8::new(0),
            default_datareader_qos: Mutex::new(DataReaderQos::default()),
            qos,
            listener,
            status_mask,
        }
    }

    pub fn create_builtin_datareader<T: DDSType>(
        &self,
        a_topic: Arc<dyn AnyRtpsTopic>,
        qos: Option<DataReaderQos>,
        // _a_listener: impl DataReaderListener<T>,
        // _mask: StatusMask
    ) -> Option<RtpsAnyDataReaderRef> {
        self.create_datareader::<T>(a_topic, qos, EntityType::BuiltIn)
    }

    pub fn create_user_defined_datareader<T: DDSType>(
        &self,
        a_topic: Arc<dyn AnyRtpsTopic>,
        qos: Option<DataReaderQos>,
        // _a_listener: impl DataReaderListener<T>,
        // _mask: StatusMask
    ) -> Option<RtpsAnyDataReaderRef> {
        self.create_datareader::<T>(a_topic, qos, EntityType::BuiltIn)
    }

    fn create_datareader<T: DDSType>(
        &self,
        a_topic: Arc<dyn AnyRtpsTopic>,
        qos: Option<DataReaderQos>,
        entity_type: EntityType,
        // _a_listener: impl DataReaderListener<T>,
        // _mask: StatusMask
    ) -> Option<RtpsAnyDataReaderRef> {
        let guid_prefix = self.group.entity.guid.prefix();
        let entity_key = [
            0,
            self.reader_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_kind = match (a_topic.topic_kind(), entity_type) {
            (TopicKind::WithKey, EntityType::UserDefined) => {
                ENTITY_KIND_USER_DEFINED_READER_WITH_KEY
            }
            (TopicKind::NoKey, EntityType::UserDefined) => ENTITY_KIND_USER_DEFINED_READER_NO_KEY,
            (TopicKind::WithKey, EntityType::BuiltIn) => ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
            (TopicKind::NoKey, EntityType::BuiltIn) => ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let new_reader_guid = GUID::new(guid_prefix, entity_id);
        let new_reader_qos = qos.unwrap_or(self.get_default_datareader_qos());
        let new_reader: Box<RtpsDataReader<T>> = Box::new(RtpsDataReader::new(
            new_reader_guid,
            a_topic,
            new_reader_qos,
            None,
            0,
        ));
        self.reader_list.add(new_reader)
    }

    pub fn get_default_datareader_qos(&self) -> DataReaderQos {
        self.default_datareader_qos.lock().unwrap().clone()
    }

    pub fn set_default_datawriter_qos(&self, qos: Option<DataReaderQos>) -> ReturnCode<()> {
        let datareader_qos = qos.unwrap_or_default();
        datareader_qos.is_consistent()?;
        *self.default_datareader_qos.lock().unwrap() = datareader_qos;
        Ok(())
    }
}

pub type RtpsSubscriberNode<'a> = MaybeValidNode<'a, RtpsParticipant<'a>, Box<RtpsSubscriber>>;

impl<'a> Subscriber for RtpsSubscriberNode<'a> {
    fn create_datareader<T: DDSType>(
        &self,
        a_topic: &Box<dyn Topic<T>>,
        qos: Option<DataReaderQos>,
        _a_listener: Option<Box<dyn DataReaderListener<T>>>,
        _mask: StatusMask
    ) -> Option<Box<dyn DataReader<T>>> {
        todo!()
    }

    fn delete_datareader<T: DDSType>(
        &self,
        _a_datareader: &Box<dyn DataReader<T>>,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn lookup_datareader<T: DDSType>(
        &self,
        _topic: &Box<dyn Topic<T>>,
    ) -> Option<Box<dyn DataReader<T>>> {
        todo!()
    }

    fn begin_access(&self) -> ReturnCode<()> {
        todo!()
    }

    fn end_access(&self) -> ReturnCode<()> {
        todo!()
    }

    fn notify_datareaders(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> ReturnCode<()> {
        todo!()
    }

    fn get_participant(
        &self,
    ) -> &dyn DomainParticipant<SubscriberType = dyn Subscriber, PublisherType = dyn Publisher>
    {
        todo!()
    }

    fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    fn set_default_datareader_qos(&self, qos: Option<DataReaderQos>) -> ReturnCode<()> {
        todo!()
    }

    fn get_default_datareader_qos(&self) -> ReturnCode<DataReaderQos> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_datareaders(
        &self,
        readers: &mut [&mut dyn AnyDataReader],
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> ReturnCode<()> {
        todo!()
    }
}

impl<'a> Entity for RtpsSubscriberNode<'a> {
    type Qos = SubscriberQos;
    type Listener = Box<dyn SubscriberListener>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self) -> ReturnCode<Self::Qos> {
        todo!()
    }

    fn set_listener(&self, a_listener: Self::Listener, mask: StatusMask) -> ReturnCode<()> {
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
