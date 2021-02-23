use std::sync::{atomic, Arc, Mutex, Weak};

use rust_dds_api::{
    dcps_psm::StatusMask,
    dds_type::DDSType,
    infrastructure::{
        qos::{DataReaderQos, SubscriberQos},
        qos_policy::ReliabilityQosPolicyKind,
    },
    return_type::{DDSError, DDSResult},
    subscription::{
        data_reader_listener::DataReaderListener, subscriber_listener::SubscriberListener,
    },
};
use rust_rtps::{
    behavior::StatefulReader,
    structure::Group,
    types::{
        constants::{
            ENTITY_KIND_USER_DEFINED_READER_NO_KEY, ENTITY_KIND_USER_DEFINED_READER_WITH_KEY,
        },
        EntityId, ReliabilityKind, TopicKind, GUID,
    },
};

use super::rtps_datareader_impl::{RtpsDataReaderImpl, RtpsReaderFlavor};

pub struct RtpsSubscriberImpl {
    group: Group,
    reader_list: Mutex<Vec<Arc<RtpsDataReaderImpl>>>,
    reader_count: atomic::AtomicU8,
    default_datareader_qos: Mutex<DataReaderQos>,
    qos: Mutex<SubscriberQos>,
    listener: Option<Box<dyn SubscriberListener>>,
    status_mask: StatusMask,
}

impl RtpsSubscriberImpl {
    pub fn new(
        group: Group,
        qos: SubscriberQos,
        listener: Option<Box<dyn SubscriberListener>>,
        status_mask: StatusMask,
    ) -> Self {
        Self {
            group,
            reader_list: Default::default(),
            reader_count: atomic::AtomicU8::new(0),
            default_datareader_qos: Mutex::new(DataReaderQos::default()),
            qos: Mutex::new(qos),
            listener,
            status_mask,
        }
    }

    pub fn create_datareader<'a, T: DDSType>(
        &'a self,
        qos: Option<DataReaderQos>,
        a_listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        mask: StatusMask,
    ) -> Option<Weak<RtpsDataReaderImpl>> {
        let qos = qos.unwrap_or(self.default_datareader_qos.lock().unwrap().clone());
        qos.is_consistent().ok()?;

        let entity_key = [
            0,
            self.reader_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let guid_prefix = self.group.entity.guid.prefix();
        let entity_kind = match T::has_key() {
            true => ENTITY_KIND_USER_DEFINED_READER_WITH_KEY,
            false => ENTITY_KIND_USER_DEFINED_READER_NO_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let topic_kind = match T::has_key() {
            true => TopicKind::WithKey,
            false => TopicKind::NoKey,
        };
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let expects_inline_qos = false;
        let heartbeat_response_delay = rust_rtps::behavior::types::Duration::from_millis(200);
        let heartbeat_supression_duration = rust_rtps::behavior::types::constants::DURATION_ZERO;
        let stateful_reader = StatefulReader::new(
            guid,
            unicast_locator_list,
            multicast_locator_list,
            topic_kind,
            reliability_level,
            expects_inline_qos,
            heartbeat_response_delay,
            heartbeat_supression_duration,
        );

        let data_reader = Arc::new(RtpsDataReaderImpl::new(
            RtpsReaderFlavor::Stateful(stateful_reader),
            qos,
            a_listener,
            mask,
        ));

        self.reader_list.lock().unwrap().push(data_reader.clone());

        Some(Arc::downgrade(&data_reader))
    }

    pub fn delete_datareader(&self, a_datareader: &Weak<RtpsDataReaderImpl>) -> DDSResult<()> {
        let datareader_impl = a_datareader.upgrade().ok_or(DDSError::AlreadyDeleted)?;
        let mut reader_list = self.reader_list.lock().unwrap();
        // If there are two references, i.e. the one that is going to be deleted and the one in the
        // vector then we are sure no other is left for the user and they can both be dropped
        if Arc::strong_count(&datareader_impl) == 2 {
            reader_list.retain(|x| !std::ptr::eq(x.as_ref(), datareader_impl.as_ref()));
            Ok(())
        } else {
            Err(DDSError::PreconditionNotMet(
                "More instances of this writer are available. Resources will not be freed.",
            ))
        }
    }

    pub fn get_qos(&self) -> SubscriberQos {
        self.qos.lock().unwrap().clone()
    }

    pub fn set_qos(&self, qos: Option<SubscriberQos>) {
        let qos = qos.unwrap_or_default();
        *self.qos.lock().unwrap() = qos;
    }
}

// enum EntityType {
//     BuiltIn,
//     UserDefined,
// }
// pub struct RtpsSubscriberInner {
//     group: Group,
//     entity_type: EntityType,
//     reader_list: MaybeValidList<Mutex<RtpsDataReaderFlavor>>,
//     reader_count: atomic::AtomicU8,
//     default_datareader_qos: Mutex<DataReaderQos>,
//     qos: Mutex<SubscriberQos>,
//     listener: Option<Box<dyn SubscriberListener>>,
//     status_mask: StatusMask,
// }

// impl RtpsSubscriberInner {
//     pub fn new_builtin(
//         guid_prefix: GuidPrefix,
//         entity_key: [u8; 3],
//         qos: SubscriberQos,
//         listener: Option<Box<dyn SubscriberListener>>,
//         status_mask: StatusMask,
//     ) -> Self {
//         Self::new(
//             guid_prefix,
//             entity_key,
//             qos,
//             listener,
//             status_mask,
//             EntityType::BuiltIn,
//         )
//     }

//     pub fn new_user_defined(
//         guid_prefix: GuidPrefix,
//         entity_key: [u8; 3],
//         qos: SubscriberQos,
//         listener: Option<Box<dyn SubscriberListener>>,
//         status_mask: StatusMask,
//     ) -> Self {
//         Self::new(
//             guid_prefix,
//             entity_key,
//             qos,
//             listener,
//             status_mask,
//             EntityType::UserDefined,
//         )
//     }

//     fn new(
//         guid_prefix: GuidPrefix,
//         entity_key: [u8; 3],
//         qos: SubscriberQos,
//         listener: Option<Box<dyn SubscriberListener>>,
//         status_mask: StatusMask,
//         entity_type: EntityType,
//     ) -> Self {
//         let entity_id = match entity_type {
//             EntityType::BuiltIn => EntityId::new(entity_key, ENTITY_KIND_BUILT_IN_READER_GROUP),
//             EntityType::UserDefined => {
//                 EntityId::new(entity_key, ENTITY_KIND_USER_DEFINED_READER_GROUP)
//             }
//         };
//         let guid = GUID::new(guid_prefix, entity_id);

//         Self {
//             group: Group::new(guid),
//             entity_type,
//             reader_list: Default::default(),
//             reader_count: atomic::AtomicU8::new(0),
//             default_datareader_qos: Mutex::new(DataReaderQos::default()),
//             qos: Mutex::new(qos),
//             listener,
//             status_mask,
//         }
//     }
// }

// pub type RtpsSubscriberInnerRef<'a> = MaybeValidRef<'a, Box<RtpsSubscriberInner>>;

// impl<'a> RtpsSubscriberInnerRef<'a> {
//     pub fn get(&self) -> DDSResult<&Box<RtpsSubscriberInner>> {
//         MaybeValid::get(self).ok_or(DDSError::AlreadyDeleted)
//     }

//     pub fn delete(&self) -> DDSResult<()> {
//         if self.get()?.reader_list.is_empty() {
//             MaybeValid::delete(self);
//             Ok(())
//         } else {
//             Err(DDSError::PreconditionNotMet(
//                 "Subscriber still contains data readers",
//             ))
//         }
//     }

//     pub fn create_datareader<T: DDSType>(
//         &self,
//         a_topic: &RtpsTopicInnerRef,
//         qos: Option<DataReaderQos>,
//         a_listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
//         status_mask: StatusMask,
//     ) -> Option<RtpsAnyDataReaderInnerRef> {
//         let this = self.get().ok()?;
//         let topic = a_topic.get().ok()?;
//         let entity_key = [
//             0,
//             self.get()
//                 .ok()?
//                 .reader_count
//                 .fetch_add(1, atomic::Ordering::Relaxed),
//             0,
//         ];
//         let guid_prefix = this.group.entity.guid.prefix();
//         let qos = qos.unwrap_or(this.default_datareader_qos.lock().unwrap().clone());

//         let data_reader_inner = RtpsStatefulDataReaderInner::new_user_defined(
//             guid_prefix,
//             entity_key,
//             vec![],
//             vec![],
//             topic,
//             qos,
//             a_listener,
//             status_mask,
//         );

//         self.get()
//             .ok()?
//             .reader_list
//             .add(Mutex::new(RtpsDataReaderFlavor::Stateful(
//                 data_reader_inner,
//             )))
//     }

//     pub fn get_qos(&self) -> DDSResult<SubscriberQos> {
//         Ok(self.get()?.qos.lock().unwrap().clone())
//     }

//     pub fn get_default_datareader_qos(&self) -> DDSResult<DataReaderQos> {
//         Ok(self.get()?.default_datareader_qos.lock().unwrap().clone())
//     }

//     pub fn set_default_datareader_qos(&self, qos: Option<DataReaderQos>) -> DDSResult<()> {
//         let datareader_qos = qos.unwrap_or_default();
//         datareader_qos.is_consistent()?;
//         *self.get()?.default_datareader_qos.lock().unwrap() = datareader_qos;
//         Ok(())
//     }
// }

// pub fn create_builtin_datareader<T: DDSType>(
//     &self,
//     a_topic: Arc<dyn RtpsAnyTopicInner>,
//     qos: Option<DataReaderQos>,
//     // _a_listener: impl DataReaderListener<T>,
//     // _mask: StatusMask
// ) -> Option<RtpsAnyDataReaderRef> {
//     self.create_datareader::<T>(a_topic, qos, EntityType::BuiltIn)
// }

// pub fn create_user_defined_datareader<T: DDSType>(
//     &self,
//     a_topic: Arc<dyn RtpsAnyTopicInner>,
//     qos: Option<DataReaderQos>,
//     // _a_listener: impl DataReaderListener<T>,
//     // _mask: StatusMask
// ) -> Option<RtpsAnyDataReaderRef> {
//     self.create_datareader::<T>(a_topic, qos, EntityType::BuiltIn)
// }

// fn create_datareader<T: DDSType>(
//     &self,
//     a_topic: Arc<dyn RtpsAnyTopicInner>,
//     qos: Option<DataReaderQos>,
//     entity_type: EntityType,
//     // _a_listener: impl DataReaderListener<T>,
//     // _mask: StatusMask
// ) -> Option<RtpsAnyDataReaderRef> {
//     let guid_prefix = self.group.entity.guid.prefix();
//     let entity_key = [
//         0,
//         self.reader_count.fetch_add(1, atomic::Ordering::Relaxed),
//         0,
//     ];
//     let entity_kind = match (a_topic.topic_kind(), entity_type) {
//         (TopicKind::WithKey, EntityType::UserDefined) => {
//             ENTITY_KIND_USER_DEFINED_READER_WITH_KEY
//         }
//         (TopicKind::NoKey, EntityType::UserDefined) => ENTITY_KIND_USER_DEFINED_READER_NO_KEY,
//         (TopicKind::WithKey, EntityType::BuiltIn) => ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
//         (TopicKind::NoKey, EntityType::BuiltIn) => ENTITY_KIND_BUILT_IN_READER_NO_KEY,
//     };
//     let entity_id = EntityId::new(entity_key, entity_kind);
//     let new_reader_guid = GUID::new(guid_prefix, entity_id);
//     let new_reader_qos = qos.unwrap_or(self.get_default_datareader_qos());
//     let new_reader: Box<RtpsDataReaderInner<T>> = Box::new(RtpsDataReaderInner::new(
//         new_reader_guid,
//         a_topic,
//         new_reader_qos,
//         None,
//         0,
//     ));
//     self.reader_list.add(new_reader)
// }
