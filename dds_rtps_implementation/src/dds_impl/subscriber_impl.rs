use rust_dds_api::{
    dcps_psm::StatusMask,
    infrastructure::{
        qos::{DataReaderQos, SubscriberQos},
        qos_policy::ReliabilityQosPolicyKind,
    },
    return_type::DDSResult,
    subscription::data_reader_listener::DataReaderListener,
};
use rust_rtps_pim::{
    behavior::{
        reader::{reader::RtpsReader, stateless_reader::RtpsStatelessReader},
        stateless_reader_behavior::StatelessReaderBehavior,
    },
    messages::{submessage_elements::Parameter, submessages::DataSubmessage},
    structure::{
        types::{
            EntityId, Guid, GuidPrefix, ReliabilityKind, TopicKind, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
        RtpsEndpoint, RtpsEntity, RtpsGroup, RtpsHistoryCache,
    },
};

use crate::{
    dds_type::DdsType,
    rtps_impl::rtps_reader_history_cache_impl::ReaderHistoryCache,
    utils::{
        message_receiver::ProcessDataSubmessage,
        shared_object::{
            rtps_shared_downgrade, rtps_shared_new, rtps_shared_write_lock, RtpsShared, RtpsWeak,
        },
    },
};

use super::data_reader_impl::{DataReaderImpl, RtpsReaderFlavor};

pub trait DataReaderObject: Send + Sync + ProcessDataSubmessage {}

impl<T> DataReaderObject for T where T: Send + Sync + ProcessDataSubmessage {}

pub struct SubscriberImpl {
    qos: SubscriberQos,
    rtps_group: RtpsGroup,
    data_reader_list: Vec<RtpsShared<dyn DataReaderObject>>,
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
}

impl SubscriberImpl {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroup,
        data_reader_list: Vec<RtpsShared<dyn DataReaderObject>>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_reader_list,
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
        }
    }

    pub fn create_datareader<T: DdsType + 'static>(
        &mut self,
        _a_topic: (),
        qos: Option<DataReaderQos>,
        _a_listener: Option<&'static dyn DataReaderListener<DataPIM = T>>,
        _mask: StatusMask,
    ) -> Option<RtpsWeak<DataReaderImpl>> {
        let qos = qos.unwrap_or(self.default_data_reader_qos.clone());
        qos.is_consistent().ok()?;

        let (entity_kind, topic_kind) = match T::has_key() {
            true => (USER_DEFINED_WRITER_WITH_KEY, TopicKind::WithKey),
            false => (USER_DEFINED_WRITER_NO_KEY, TopicKind::NoKey),
        };
        let entity_id = EntityId::new(
            [
                self.rtps_group.entity.guid.entity_id().entity_key()[0],
                self.user_defined_data_reader_counter,
                0,
            ],
            entity_kind,
        );
        let guid = Guid::new(*self.rtps_group.entity.guid.prefix(), entity_id);
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };

        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let heartbeat_response_delay = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let heartbeat_supression_duration = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let expects_inline_qos = false;
        let rtps_reader = RtpsReaderFlavor::Stateless(RtpsStatelessReader(RtpsReader {
            endpoint: RtpsEndpoint {
                entity: RtpsEntity { guid },
                topic_kind,
                reliability_level,
                unicast_locator_list,
                multicast_locator_list,
            },
            heartbeat_response_delay,
            heartbeat_supression_duration,
            reader_cache: ReaderHistoryCache::new(),
            expects_inline_qos,
        }));
        let reader_storage = DataReaderImpl::new(qos, rtps_reader);
        let reader_storage_shared = rtps_shared_new(reader_storage);
        let reader_storage_weak = rtps_shared_downgrade(&reader_storage_shared);
        self.data_reader_list.push(reader_storage_shared);
        Some(reader_storage_weak)
    }

    pub fn set_qos(&mut self, qos: Option<SubscriberQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        self.qos = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> &SubscriberQos {
        &self.qos
    }
}

impl ProcessDataSubmessage for SubscriberImpl {
    fn process_data_submessage(
        &mut self,
        source_guid_prefix: GuidPrefix,
        data: &DataSubmessage<Vec<Parameter<'_>>>,
    ) {
        for reader in &self.data_reader_list {
            rtps_shared_write_lock(reader).process_data_submessage(source_guid_prefix, data);
            //  rtps_reader_mut() {
        }
    }
}
