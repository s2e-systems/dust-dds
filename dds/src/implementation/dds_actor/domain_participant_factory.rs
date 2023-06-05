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
        utils::actor::{ActorAddress, MailHandler, Mail},
    },
    infrastructure::{
        error::DdsResult,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
};

pub struct CreateParticipant {
    domain_id: DomainId,
    qos: QosKind<DomainParticipantQos>,
    a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
    mask: Vec<StatusKind>,
}

impl CreateParticipant {
    pub fn new(
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        mask: Vec<StatusKind>,
    ) -> Self {
        Self {
            domain_id,
            qos,
            a_listener,
            mask,
        }
    }
}

impl Mail for CreateParticipant {
    type Result = DdsResult<ActorAddress<DdsDomainParticipant>>;
}

impl MailHandler<CreateParticipant> for DdsDomainParticipantFactory {
    fn handle(&mut self, message: CreateParticipant) -> <CreateParticipant as Mail>::Result {
        self.create_participant(
            message.domain_id,
            message.qos,
            message.a_listener,
            &message.mask,
        )
    }
}

pub struct DeleteParticipant {
    address: ActorAddress<DdsDomainParticipant>,
}

impl DeleteParticipant {
    pub fn new(address: ActorAddress<DdsDomainParticipant>) -> Self {
        Self { address }
    }
}

impl Mail for DeleteParticipant {
    type Result = DdsResult<()>;
}

impl MailHandler<DeleteParticipant> for DdsDomainParticipantFactory {
    fn handle(&mut self, message: DeleteParticipant) -> <DeleteParticipant as Mail>::Result {
        self.delete_participant(&message.address)
    }
}

pub struct LookupParticipant {
    domain_id: DomainId,
}

impl LookupParticipant {
    pub fn new(domain_id: DomainId) -> Self {
        Self { domain_id }
    }
}

impl Mail for LookupParticipant {
    type Result = Option<ActorAddress<DdsDomainParticipant>>;
}

impl MailHandler<LookupParticipant> for DdsDomainParticipantFactory {
    fn handle(&mut self, message: LookupParticipant) -> <LookupParticipant as Mail>::Result {
        self.lookup_participant(message.domain_id)
    }
}

pub struct GetQos;

impl Mail for GetQos {
    type Result = DomainParticipantFactoryQos;
}

impl MailHandler<GetQos> for DdsDomainParticipantFactory {
    fn handle(&mut self, _message: GetQos) -> <GetQos as Mail>::Result {
        self.get_qos().clone()
    }
}

pub struct SetQos {
    qos_kind: QosKind<DomainParticipantFactoryQos>,
}

impl SetQos {
    pub fn new(qos_kind: QosKind<DomainParticipantFactoryQos>) -> Self {
        Self { qos_kind }
    }
}

impl Mail for SetQos {
    type Result = ();
}

impl MailHandler<SetQos> for DdsDomainParticipantFactory {
    fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        self.set_qos(message.qos_kind)
    }
}

pub struct GetDefaultParticipantQos;

impl Mail for GetDefaultParticipantQos {
    type Result = DomainParticipantQos;
}

impl MailHandler<GetDefaultParticipantQos> for DdsDomainParticipantFactory {
    fn handle(
        &mut self,
        _message: GetDefaultParticipantQos,
    ) -> <GetDefaultParticipantQos as Mail>::Result {
        self.get_default_participant_qos().clone()
    }
}

pub struct SetDefaultParticipantQos {
    qos_kind: QosKind<DomainParticipantQos>,
}

impl SetDefaultParticipantQos {
    pub fn new(qos_kind: QosKind<DomainParticipantQos>) -> Self {
        Self { qos_kind }
    }
}

impl Mail for SetDefaultParticipantQos {
    type Result = ();
}

impl MailHandler<SetDefaultParticipantQos> for DdsDomainParticipantFactory {
    fn handle(
        &mut self,
        message: SetDefaultParticipantQos,
    ) -> <SetDefaultParticipantQos as Mail>::Result {
        self.set_default_participant_qos(message.qos_kind)
    }
}
