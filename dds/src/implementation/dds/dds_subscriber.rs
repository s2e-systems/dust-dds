use std::collections::HashMap;

use super::{
    dds_data_reader::{self, DdsDataReader},
    dds_domain_participant::DdsDomainParticipant,
    dds_subscriber_listener::DdsSubscriberListener,
};
use crate::{
    implementation::{
        dds::{
            dds_domain_participant_listener::DdsDomainParticipantListener,
            dds_status_condition::DdsStatusCondition,
        },
        rtps::{
            group::RtpsGroup,
            messages::overall_structure::{RtpsMessageHeader, RtpsMessageRead},
            types::Guid,
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::actor::{
            actor_mailbox_interface, spawn_actor, Actor, ActorAddress, Mail, MailHandler,
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
    status_condition: Actor<DdsStatusCondition>,
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
        let status_condition = spawn_actor(DdsStatusCondition::default());
        DdsSubscriber {
            qos,
            rtps_group,
            data_reader_list: HashMap::new(),
            enabled: false,
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: Default::default(),
            status_condition,
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

    pub fn get_statuscondition(&self) -> ActorAddress<DdsStatusCondition> {
        self.status_condition.address().clone()
    }
}}

pub struct GetQos;

impl Mail for GetQos {
    type Result = SubscriberQos;
}

#[async_trait::async_trait]
impl MailHandler<GetQos> for DdsSubscriber {
    async fn handle(&mut self, _mail: GetQos) -> <GetQos as Mail>::Result {
        self.qos.clone()
    }
}

pub struct DataReaderList;

impl Mail for DataReaderList {
    type Result = Vec<ActorAddress<DdsDataReader>>;
}

#[async_trait::async_trait]
impl MailHandler<DataReaderList> for DdsSubscriber {
    async fn handle(&mut self, _mail: DataReaderList) -> <DataReaderList as Mail>::Result {
        self.data_reader_list
            .values()
            .map(|dr| dr.address().clone())
            .collect()
    }
}

pub struct GetListener;

impl Mail for GetListener {
    type Result = Option<ActorAddress<DdsSubscriberListener>>;
}

#[async_trait::async_trait]
impl MailHandler<GetListener> for DdsSubscriber {
    async fn handle(&mut self, _mail: GetListener) -> <GetListener as Mail>::Result {
        self.listener.as_ref().map(|l| l.address().clone())
    }
}

pub struct GetStatusKind;

impl Mail for GetStatusKind {
    type Result = Vec<StatusKind>;
}

#[async_trait::async_trait]
impl MailHandler<GetStatusKind> for DdsSubscriber {
    async fn handle(&mut self, _mail: GetStatusKind) -> <GetStatusKind as Mail>::Result {
        self.status_kind.clone()
    }
}
pub struct SendMessage {
    header: RtpsMessageHeader,
    udp_transport_write: ActorAddress<UdpTransportWrite>,
}

impl SendMessage {
    pub fn new(
        header: RtpsMessageHeader,
        udp_transport_write: ActorAddress<UdpTransportWrite>,
    ) -> Self {
        Self {
            header,
            udp_transport_write,
        }
    }
}

impl Mail for SendMessage {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<SendMessage> for DdsSubscriber {
    async fn handle(&mut self, mail: SendMessage) -> <SendMessage as Mail>::Result {
        for data_reader_address in self.data_reader_list.values().map(|a| a.address()) {
            data_reader_address
                .send_only(dds_data_reader::SendMessage::new(
                    mail.header,
                    mail.udp_transport_write.clone(),
                ))
                .await
                .expect("Should not fail to send command");
        }
    }
}

pub struct ProcessRtpsMessage {
    message: RtpsMessageRead,
    reception_timestamp: Time,
    participant_address: ActorAddress<DdsDomainParticipant>,
    subscriber_address: ActorAddress<DdsSubscriber>,
    participant_mask_listener: (
        Option<ActorAddress<DdsDomainParticipantListener>>,
        Vec<StatusKind>,
    ),
}

impl ProcessRtpsMessage {
    pub fn new(
        message: RtpsMessageRead,
        reception_timestamp: Time,
        participant_address: ActorAddress<DdsDomainParticipant>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_mask_listener: (
            Option<ActorAddress<DdsDomainParticipantListener>>,
            Vec<StatusKind>,
        ),
    ) -> Self {
        Self {
            message,
            reception_timestamp,
            participant_address,
            subscriber_address,
            participant_mask_listener,
        }
    }
}

impl Mail for ProcessRtpsMessage {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<ProcessRtpsMessage> for DdsSubscriber {
    async fn handle(&mut self, mail: ProcessRtpsMessage) -> <ProcessRtpsMessage as Mail>::Result {
        let subscriber_mask_listener = (
            self.listener.as_ref().map(|a| a.address()).cloned(),
            self.status_kind.clone(),
        );

        for data_reader_address in self.data_reader_list.values().map(|a| a.address()) {
            data_reader_address
                .send_only(dds_data_reader::ProcessRtpsMessage::new(
                    mail.message.clone(),
                    mail.reception_timestamp,
                    data_reader_address.clone(),
                    mail.subscriber_address.clone(),
                    mail.participant_address.clone(),
                    self.status_condition.address().clone(),
                    subscriber_mask_listener.clone(),
                    mail.participant_mask_listener.clone(),
                ))
                .await
                .expect("Should not fail to send command");
        }
    }
}
