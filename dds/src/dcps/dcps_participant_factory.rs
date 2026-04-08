use crate::{
    dcps::{
        dcps_domain_participant::DcpsDomainParticipant,
        dcps_mail::{DcpsMail, DiscoveryServiceMail, MessageServiceMail},
        listeners::domain_participant_listener::DcpsDomainParticipantListener,
    },
    dds_async::{configuration::DustDdsConfiguration, domain_participant_factory::DcpsSender},
    infrastructure::{
        domain::DomainId,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
    runtime::{DdsRuntime, Spawner, Timer},
    transport::{
        interface::{TransportDataReceiver, TransportParticipantFactory},
        types::{ENTITYID_PARTICIPANT, Guid, GuidPrefix},
    },
};
use alloc::{borrow::ToOwned, string::String, vec::Vec};

pub struct DcpsParticipantFactory<R: DdsRuntime, T> {
    domain_participant_list: Vec<DcpsDomainParticipant<R>>,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
    configuration: DustDdsConfiguration,
    runtime: R,
    transport: T,
    entity_counter: u32,
    app_id: [u8; 4],
    host_id: [u8; 4],
    dcps_sender: DcpsSender,
}

impl<R: DdsRuntime, T: TransportParticipantFactory> DcpsParticipantFactory<R, T> {
    pub fn new(
        app_id: [u8; 4],
        host_id: [u8; 4],
        runtime: R,
        transport: T,
        dcps_sender: DcpsSender,
    ) -> Self {
        Self {
            domain_participant_list: Default::default(),
            qos: Default::default(),
            default_participant_qos: Default::default(),
            configuration: Default::default(),
            runtime,
            transport,
            entity_counter: 0,
            app_id,
            host_id,
            dcps_sender,
        }
    }

    fn get_unique_participant_id(&mut self) -> u32 {
        let id = self.entity_counter;
        self.entity_counter += 1;
        id
    }

    fn create_new_guid_prefix(&mut self) -> GuidPrefix {
        let instance_id = self.get_unique_participant_id().to_ne_bytes();

        [
            self.host_id[0],
            self.host_id[1],
            self.host_id[2],
            self.host_id[3], // Host ID
            self.app_id[0],
            self.app_id[1],
            self.app_id[2],
            self.app_id[3], // App ID
            instance_id[0],
            instance_id[1],
            instance_id[2],
            instance_id[3], // Instance ID
        ]
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_participant(
        &mut self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        status_kind: Vec<StatusKind>,
    ) -> DdsResult<InstanceHandle> {
        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let guid_prefix = self.create_new_guid_prefix();
        let participant_handle = InstanceHandle::from(Guid::new(guid_prefix, ENTITYID_PARTICIPANT));
        let transport = self.transport.create_participant(
            domain_id,
            TransportDataReceiver::new(participant_handle, self.dcps_sender),
        );

        let clock_handle = self.runtime.clock();
        let mut timer_handle = self.runtime.timer();
        let spawner_handle = self.runtime.spawner();

        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&spawner_handle));

        let mut dcps_participant: DcpsDomainParticipant<R> = DcpsDomainParticipant::new(
            domain_id,
            self.configuration.domain_tag().to_owned(),
            guid_prefix,
            domain_participant_qos,
            listener_sender,
            status_kind,
            transport,
            self.dcps_sender,
            clock_handle,
            timer_handle.clone(),
            spawner_handle.clone(),
        );
        let participant_handle = dcps_participant.get_instance_handle();

        //****** Spawn the participant actor and tasks **********//

        // Start the regular participant announcement task
        let dcps_sender_clone = self.dcps_sender;
        let mut timer_handle_clone = timer_handle.clone();
        let participant_announcement_interval =
            self.configuration.participant_announcement_interval();

        spawner_handle.spawn(async move {
            loop {
                dcps_sender_clone
                    .send(DcpsMail::Discovery(
                        DiscoveryServiceMail::AnnounceParticipant { participant_handle },
                    ))
                    .await;

                timer_handle_clone
                    .delay(participant_announcement_interval)
                    .await;
            }
        });

        // Start regular message writing
        let dcps_sender_clone = self.dcps_sender;
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
            dcps_participant.enable_domain_participant()?;
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
        participant.announce_deleted_participant();
        Ok(())
    }

    pub fn find_participant(
        &mut self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<&mut DcpsDomainParticipant<R>> {
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

    pub fn set_configuration(&mut self, configuration: DustDdsConfiguration) {
        self.configuration = configuration;
    }

    pub fn get_configuration(&mut self) -> DustDdsConfiguration {
        self.configuration.clone()
    }
}
