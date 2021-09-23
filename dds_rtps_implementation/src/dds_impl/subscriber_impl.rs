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
        reader::stateful_reader::RtpsStatefulReaderOperations,
        stateless_reader_behavior::StatelessReaderBehavior,
    },
    messages::{submessage_elements::Parameter, submessages::DataSubmessage},
    structure::{
        types::{
            EntityId, Guid, GuidPrefix, ReliabilityKind, TopicKind, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
        RtpsEntity,
    },
};

use crate::{
    dds_type::DdsType,
    rtps_impl::{rtps_group_impl::RtpsGroupImpl, rtps_reader_impl::RtpsReaderImpl},
    utils::{
        message_receiver::ProcessDataSubmessage,
        shared_object::{RtpsShared, RtpsWeak},
    },
};

use super::data_reader_impl::DataReaderImpl;

pub struct SubscriberImpl {
    qos: SubscriberQos,
    rtps_group: RtpsGroupImpl,
    data_reader_storage_list: Vec<RtpsShared<DataReaderImpl>>,
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
}

impl SubscriberImpl {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroupImpl,
        data_reader_storage_list: Vec<RtpsShared<DataReaderImpl>>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_reader_storage_list,
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
        }
    }

    /// Get a reference to the subscriber storage's readers.
    pub fn readers(&self) -> &[RtpsShared<DataReaderImpl>] {
        self.data_reader_storage_list.as_slice()
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
                self.rtps_group.guid().entity_id().entity_key()[0],
                self.user_defined_data_reader_counter,
                0,
            ],
            entity_kind,
        );
        let guid = Guid::new(*self.rtps_group.guid().prefix(), entity_id);
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };

        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let heartbeat_response_delay = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let heartbeat_supression_duration = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let expects_inline_qos = false;
        let rtps_reader = RtpsReaderImpl::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_response_delay,
            heartbeat_supression_duration,
            expects_inline_qos,
        );
        let reader_storage = DataReaderImpl::new(qos, rtps_reader);
        let reader_storage_shared = RtpsShared::new(reader_storage);
        let reader_storage_weak = reader_storage_shared.downgrade();
        self.data_reader_storage_list.push(reader_storage_shared);
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
    fn process_data_submessage(&self, data: &DataSubmessage<Vec<Parameter<'_>>>) {
        for reader in &self.data_reader_storage_list {
            reader
                .write()
                .rtps_reader_mut()
                .receive_data(GuidPrefix([7; 12]), data);
        }
    }
}
