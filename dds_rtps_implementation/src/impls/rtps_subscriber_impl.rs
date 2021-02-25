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
    reader_list: Vec<Arc<Mutex<RtpsDataReaderImpl>>>,
    reader_count: atomic::AtomicU8,
    default_datareader_qos: DataReaderQos,
    qos: SubscriberQos,
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
            default_datareader_qos: DataReaderQos::default(),
            qos,
            listener,
            status_mask,
        }
    }

    pub fn reader_list(&self) -> &Vec<Arc<Mutex<RtpsDataReaderImpl>>> {
        &self.reader_list
    }

    pub fn create_datareader<'a, T: DDSType>(
        &'a mut self,
        qos: Option<DataReaderQos>,
        a_listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        mask: StatusMask,
    ) -> Option<Weak<Mutex<RtpsDataReaderImpl>>> {
        let qos = qos.unwrap_or(self.default_datareader_qos.clone());
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

        let data_reader = Arc::new(Mutex::new(RtpsDataReaderImpl::new(
            RtpsReaderFlavor::Stateful(stateful_reader),
            qos,
            a_listener,
            mask,
        )));

        self.reader_list.push(data_reader.clone());

        Some(Arc::downgrade(&data_reader))
    }

    pub fn delete_datareader(
        &mut self,
        a_datareader: &Weak<Mutex<RtpsDataReaderImpl>>,
    ) -> DDSResult<()> {
        let datareader_impl = a_datareader.upgrade().ok_or(DDSError::AlreadyDeleted)?;
        self.reader_list.retain(|x| !std::ptr::eq(x.as_ref(), datareader_impl.as_ref()));
        Ok(())
    }

    pub fn get_qos(&self) -> SubscriberQos {
        self.qos.clone()
    }

    pub fn set_qos(&mut self, qos: Option<SubscriberQos>) {
        let qos = qos.unwrap_or_default();
        self.qos = qos;
    }
}
