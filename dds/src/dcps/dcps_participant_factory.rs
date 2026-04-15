use crate::{
    dcps::{
        dcps_domain_participant::DcpsDomainParticipant,
        dcps_mail::{DcpsMail, DiscoveryServiceMail, MessageServiceMail},
        listeners::domain_participant_listener::DcpsDomainParticipantListener,
    },
    dds_async::domain_participant_factory::DcpsSender,
    infrastructure::{
        domain::DomainId,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
    runtime::{DdsRuntime, Spawner, Timer},
    transport::{interface::RtpsTransportParticipant, types::GuidPrefix},
};
use alloc::{string::String, vec::Vec};

pub struct DcpsParticipantFactory<R: DdsRuntime> {
    pub domain_participant_list: Vec<DcpsDomainParticipant>,
    pub qos: DomainParticipantFactoryQos,
    pub default_participant_qos: DomainParticipantQos,
    pub runtime: R,
    pub dcps_sender: DcpsSender,
}

impl<R: DdsRuntime> DcpsParticipantFactory<R> {
    pub fn new(runtime: R, dcps_sender: DcpsSender) -> Self {
        Self {
            domain_participant_list: Default::default(),
            qos: Default::default(),
            default_participant_qos: Default::default(),
            runtime,
            dcps_sender,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_participant(
        &mut self,
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        status_kind: Vec<StatusKind>,
        transport_participant: RtpsTransportParticipant,
        domain_tag: String,
        participant_announcement_interval: core::time::Duration,
    ) -> DdsResult<InstanceHandle> {
        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let spawner_handle = self.runtime.spawner();

        let listener_sender = dcps_listener.map(|l| l.spawn(&self.runtime.spawner()));

        let mut dcps_participant = DcpsDomainParticipant::new(
            domain_id,
            domain_tag,
            guid_prefix,
            domain_participant_qos,
            listener_sender,
            status_kind,
            transport_participant,
            self.dcps_sender,
        );
        let participant_handle = dcps_participant.get_instance_handle();

        //****** Spawn the participant actor and tasks **********//

        // Start the regular participant announcement task
        let dcps_sender_clone = self.dcps_sender;
        let mut timer_handle = self.runtime.timer().clone();

        spawner_handle.spawn(async move {
            loop {
                dcps_sender_clone
                    .send(DcpsMail::Discovery(
                        DiscoveryServiceMail::AnnounceParticipant { participant_handle },
                    ))
                    .await;

                timer_handle.delay(participant_announcement_interval).await;
            }
        });

        // Start regular message writing
        let dcps_sender_clone = self.dcps_sender;
        let mut timer_handle = self.runtime.timer();
        spawner_handle.spawn(async move {
            loop {
                dcps_sender_clone
                    .send(DcpsMail::Message(MessageServiceMail::Poke {
                        participant_handle,
                    }))
                    .await;

                timer_handle
                    .delay(core::time::Duration::from_millis(50))
                    .await;
            }
        });

        if self.qos.entity_factory.autoenable_created_entities {
            dcps_participant.enable_domain_participant(&self.runtime)?;
        }

        self.domain_participant_list.push(dcps_participant);

        Ok(participant_handle)
    }

    pub fn delete_participant(&mut self, participant_handle: InstanceHandle) -> DdsResult<()> {
        let index = self
            .domain_participant_list
            .iter()
            .position(|h| h.get_instance_handle() == participant_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        if !self.domain_participant_list[index].is_participant_empty() {
            return Err(DdsError::PreconditionNotMet(String::from(
                "Domain participant still contains other entities",
            )));
        }
        let mut participant = self.domain_participant_list.remove(index);
        participant.announce_deleted_participant(&self.runtime);
        Ok(())
    }

    pub fn find_participant(
        &mut self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<&mut DcpsDomainParticipant> {
        self.domain_participant_list
            .iter_mut()
            .find(|x| x.get_instance_handle() == participant_handle)
            .ok_or(DdsError::AlreadyDeleted)
    }

    pub fn set_default_participant_qos(
        &mut self,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_participant_qos = qos;

        Ok(())
    }

    pub fn get_default_participant_qos(&mut self) -> DomainParticipantQos {
        self.default_participant_qos.clone()
    }

    pub fn set_qos(&mut self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;
        Ok(())
    }

    pub fn get_qos(&mut self) -> DomainParticipantFactoryQos {
        self.qos.clone()
    }
}
