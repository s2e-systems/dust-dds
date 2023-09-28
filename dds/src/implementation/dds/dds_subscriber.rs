use std::collections::HashMap;

use dust_dds_derive::actor_interface;

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
        utils::actor::{spawn_actor, Actor, ActorAddress},
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

#[actor_interface]
impl DdsSubscriber {
    async fn delete_contained_entities(&mut self) {}

    async fn guid(&self) -> Guid {
        self.rtps_group.guid()
    }

    async fn is_empty(&self) -> bool {
        self.data_reader_list.is_empty()
    }

    async fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn get_unique_reader_id(&mut self) -> u8 {
        let counter = self.user_defined_data_reader_counter;
        self.user_defined_data_reader_counter += 1;
        counter
    }

    async fn data_reader_add(
        &mut self,
        instance_handle: InstanceHandle,
        data_reader: Actor<DdsDataReader>,
    ) {
        self.data_reader_list.insert(instance_handle, data_reader);
    }

    async fn data_reader_delete(&mut self, handle: InstanceHandle) {
        self.data_reader_list.remove(&handle);
    }

    async fn set_default_datareader_qos(&mut self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_data_reader_qos = DataReaderQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                self.default_data_reader_qos = q;
            }
        }
        Ok(())
    }

    async fn get_default_datareader_qos(&self) -> DataReaderQos {
        self.default_data_reader_qos.clone()
    }

    async fn set_qos(&mut self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
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

    async fn enable(&mut self) {
        self.enabled = true;
    }

    async fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }

    async fn get_statuscondition(&self) -> ActorAddress<DdsStatusCondition> {
        self.status_condition.address().clone()
    }

    async fn get_qos(&self) -> SubscriberQos {
        self.qos.clone()
    }

    async fn data_reader_list(&self) -> Vec<ActorAddress<DdsDataReader>> {
        self.data_reader_list
            .values()
            .map(|dr| dr.address().clone())
            .collect()
    }

    async fn get_listener(&self) -> Option<ActorAddress<DdsSubscriberListener>> {
        self.listener.as_ref().map(|l| l.address().clone())
    }

    async fn get_status_kind(&self) -> Vec<StatusKind> {
        self.status_kind.clone()
    }

    async fn send_message(
        &self,
        header: RtpsMessageHeader,
        udp_transport_write: ActorAddress<UdpTransportWrite>,
    ) {
        for data_reader_address in self.data_reader_list.values().map(|a| a.address()) {
            data_reader_address
                .send_mail(dds_data_reader::send_message::new(
                    header,
                    udp_transport_write.clone(),
                ))
                .await
                .expect("Should not fail to send command");
        }
    }

    async fn process_rtps_message(
        &self,
        message: RtpsMessageRead,
        reception_timestamp: Time,
        participant_address: ActorAddress<DdsDomainParticipant>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_mask_listener: (
            Option<ActorAddress<DdsDomainParticipantListener>>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        let subscriber_mask_listener = (
            self.listener.as_ref().map(|a| a.address()).cloned(),
            self.status_kind.clone(),
        );

        for data_reader_address in self.data_reader_list.values().map(|a| a.address()) {
            data_reader_address
                .send_mail_and_await_reply(dds_data_reader::process_rtps_message::new(
                    message.clone(),
                    reception_timestamp,
                    data_reader_address.clone(),
                    subscriber_address.clone(),
                    participant_address.clone(),
                    self.status_condition.address().clone(),
                    subscriber_mask_listener.clone(),
                    participant_mask_listener.clone(),
                ))
                .await??;
        }
        Ok(())
    }
}
