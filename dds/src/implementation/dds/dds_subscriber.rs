use std::collections::HashMap;

use super::{
    dds_data_reader::DdsDataReader, dds_domain_participant::DdsDomainParticipant,
    dds_subscriber_listener::DdsSubscriberListener, status_condition_impl::StatusConditionImpl,
};
use crate::{
    implementation::{
        dds::dds_domain_participant_listener::DdsDomainParticipantListener,
        rtps::{
            group::RtpsGroup,
            messages::overall_structure::{RtpsMessageHeader, RtpsMessageRead},
            types::Guid,
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::{
            actor::{actor_command_interface, actor_mailbox_interface, Actor, ActorAddress},
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        status::StatusKind,
        time::Time,
    },
};

pub struct DdsSubscriber {
    qos: SubscriberQos,
    rtps_group: RtpsGroup,
    data_reader_list: HashMap<InstanceHandle, Actor<DdsDataReader>>,
    enabled: bool,
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    listener: Option<Actor<DdsSubscriberListener>>,
    status_kind: Vec<StatusKind>,
}

impl DdsSubscriber {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroup,
        listener: Option<Actor<DdsSubscriberListener>>,
        status_kind: Vec<StatusKind>,
    ) -> Self {
        DdsSubscriber {
            qos,
            rtps_group,
            data_reader_list: HashMap::new(),
            enabled: false,
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: Default::default(),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            listener,
            status_kind,
        }
    }
}

actor_mailbox_interface! {
impl DdsSubscriber {
    pub fn delete_contained_entities(&mut self) {

    }

    pub fn guid(&self) -> Guid {
        self.rtps_group.guid()
    }

    pub fn is_empty(&self) -> bool {
        self.data_reader_list.is_empty()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_qos(&self) -> SubscriberQos {
        self.qos.clone()
    }

    pub fn get_unique_reader_id(&mut self) -> u8 {
        let counter = self.user_defined_data_reader_counter;
        self.user_defined_data_reader_counter += 1;
        counter
    }

    pub fn data_reader_add(
        &mut self,
        instance_handle: InstanceHandle,
        data_reader: Actor<DdsDataReader>,
    ) {
        self.data_reader_list.insert(instance_handle, data_reader);
    }

    pub fn data_reader_list(&self) -> Vec<ActorAddress<DdsDataReader>> {
        self.data_reader_list.values().map(|dr| dr.address().clone()).collect()
    }

    pub fn data_reader_delete(&mut self, handle: InstanceHandle) {
        self.data_reader_list.remove(&handle);
    }

    pub fn set_default_datareader_qos(&mut self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_data_reader_qos = DataReaderQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                self.default_data_reader_qos = q;
            }
        }
        Ok(())
    }

    pub fn get_default_datareader_qos(&self) -> DataReaderQos {
        self.default_data_reader_qos.clone()
    }

    pub fn set_qos(&mut self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        if self.enabled {
            self.qos.check_immutability(&qos)?;
        }

        self.qos = qos;

        Ok(())
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_listener(&self) -> Option<ActorAddress<DdsSubscriberListener>> {
        self.listener.as_ref().map(|l| l.address().clone())
    }

    pub fn status_kind(&self) -> Vec<StatusKind> {
        self.status_kind.clone()
    }
}}

actor_command_interface! {
impl DdsSubscriber {
    pub fn process_rtps_message(
        &self,
        message: RtpsMessageRead,
        reception_timestamp: Time,
        participant_address: ActorAddress<DdsDomainParticipant>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_mask_listener: (
            Option<ActorAddress<DdsDomainParticipantListener>>,
            Vec<StatusKind>,
        ),
    ) {
        let subscriber_mask_listener = (self.listener.as_ref().map(|a| a.address()).cloned(),self.status_kind.clone());

        for data_reader_address in self.data_reader_list.values().map(|a| a.address()) {
            data_reader_address
                .process_rtps_message(
                    message.clone(),
                    reception_timestamp,
                    data_reader_address.clone(),
                    subscriber_address.clone(),
                    participant_address.clone(),
                    self.status_condition.clone(),
                    subscriber_mask_listener.clone(),
                    participant_mask_listener.clone(),
                )
                .expect("Should not fail to send command");
        }
    }

    pub fn send_message(
        &self,
        header: RtpsMessageHeader,
        udp_transport_write: ActorAddress<UdpTransportWrite>,
    ) {
        for data_reader_address in self.data_reader_list.values().map(|a| a.address()) {
            data_reader_address
                .send_message(header, udp_transport_write.clone())
                .expect("Should not fail to send command");
        }
    }
}
}
