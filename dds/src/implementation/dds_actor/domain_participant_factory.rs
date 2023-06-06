use crate::{
    domain::{
        domain_participant_factory::DomainId,
        domain_participant_listener::DomainParticipantListener,
    },
    implementation::{
        dds::{
            dds_domain_participant::DdsDomainParticipant,
            dds_domain_participant_factory::DdsDomainParticipantFactory,
        },
        utils::actor::{ActorAddress, Mail, MailHandler},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
};

impl ActorAddress<DdsDomainParticipantFactory> {
    pub fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<ActorAddress<DdsDomainParticipant>> {
        pub struct CreateParticipant {
            domain_id: DomainId,
            qos: QosKind<DomainParticipantQos>,
            a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
            mask: Vec<StatusKind>,
        }

        impl Mail for CreateParticipant {
            type Result = DdsResult<ActorAddress<DdsDomainParticipant>>;
        }

        impl MailHandler<CreateParticipant> for DdsDomainParticipantFactory {
            fn handle(&mut self, mail: CreateParticipant) -> <CreateParticipant as Mail>::Result {
                self.create_participant(mail.domain_id, mail.qos, mail.a_listener, &mail.mask)
            }
        }

        self.send_blocking(CreateParticipant {
            domain_id,
            qos,
            a_listener,
            mask,
        })?
    }

    pub fn delete_participant(&self, handle: InstanceHandle) -> DdsResult<()> {
        struct DeleteParticipant {
            handle: InstanceHandle,
        }

        impl Mail for DeleteParticipant {
            type Result = DdsResult<()>;
        }

        impl MailHandler<DeleteParticipant> for DdsDomainParticipantFactory {
            fn handle(&mut self, mail: DeleteParticipant) -> <DeleteParticipant as Mail>::Result {
                self.delete_participant(mail.handle)
            }
        }

        self.send_blocking(DeleteParticipant { handle })?
    }

    pub fn lookup_participant(
        &self,
        domain_id: DomainId,
    ) -> DdsResult<Option<ActorAddress<DdsDomainParticipant>>> {
        struct LookupParticipant {
            domain_id: DomainId,
        }

        impl Mail for LookupParticipant {
            type Result = Option<ActorAddress<DdsDomainParticipant>>;
        }

        impl MailHandler<LookupParticipant> for DdsDomainParticipantFactory {
            fn handle(&mut self, mail: LookupParticipant) -> <LookupParticipant as Mail>::Result {
                self.lookup_participant(mail.domain_id)
            }
        }

        self.send_blocking(LookupParticipant { domain_id })
    }

    pub fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        struct GetQos;

        impl Mail for GetQos {
            type Result = DomainParticipantFactoryQos;
        }

        impl MailHandler<GetQos> for DdsDomainParticipantFactory {
            fn handle(&mut self, _mail: GetQos) -> <GetQos as Mail>::Result {
                self.get_qos().clone()
            }
        }

        self.send_blocking(GetQos)
    }

    pub fn set_qos(&self, qos_kind: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        struct SetQos {
            qos_kind: QosKind<DomainParticipantFactoryQos>,
        }

        impl Mail for SetQos {
            type Result = ();
        }

        impl MailHandler<SetQos> for DdsDomainParticipantFactory {
            fn handle(&mut self, mail: SetQos) -> <SetQos as Mail>::Result {
                self.set_qos(mail.qos_kind)
            }
        }
        self.send_blocking(SetQos { qos_kind })
    }

    pub fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        pub struct GetDefaultParticipantQos;

        impl Mail for GetDefaultParticipantQos {
            type Result = DomainParticipantQos;
        }

        impl MailHandler<GetDefaultParticipantQos> for DdsDomainParticipantFactory {
            fn handle(
                &mut self,
                _mail: GetDefaultParticipantQos,
            ) -> <GetDefaultParticipantQos as Mail>::Result {
                self.get_default_participant_qos().clone()
            }
        }

        self.send_blocking(GetDefaultParticipantQos)
    }

    pub fn set_default_participant_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        struct SetDefaultParticipantQos {
            qos: QosKind<DomainParticipantQos>,
        }

        impl Mail for SetDefaultParticipantQos {
            type Result = ();
        }

        impl MailHandler<SetDefaultParticipantQos> for DdsDomainParticipantFactory {
            fn handle(
                &mut self,
                mail: SetDefaultParticipantQos,
            ) -> <SetDefaultParticipantQos as Mail>::Result {
                self.set_default_participant_qos(mail.qos)
            }
        }
        self.send_blocking(SetDefaultParticipantQos { qos })
    }
}
