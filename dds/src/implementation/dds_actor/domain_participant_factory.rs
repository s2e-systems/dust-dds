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
        utils::actor::{ActorAddress, Handler, Message},
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

impl Message for CreateParticipant {
    type Result = DdsResult<ActorAddress<DdsDomainParticipant>>;
}

impl Handler<CreateParticipant> for DdsDomainParticipantFactory {
    fn handle(&mut self, message: CreateParticipant) -> <CreateParticipant as Message>::Result {
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

impl Message for DeleteParticipant {
    type Result = DdsResult<()>;
}

impl Handler<DeleteParticipant> for DdsDomainParticipantFactory {
    fn handle(&mut self, message: DeleteParticipant) -> <DeleteParticipant as Message>::Result {
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

impl Message for LookupParticipant {
    type Result = Option<ActorAddress<DdsDomainParticipant>>;
}

impl Handler<LookupParticipant> for DdsDomainParticipantFactory {
    fn handle(&mut self, message: LookupParticipant) -> <LookupParticipant as Message>::Result {
        self.lookup_participant(message.domain_id)
    }
}

pub struct GetQos;

impl Message for GetQos {
    type Result = DomainParticipantFactoryQos;
}

impl Handler<GetQos> for DdsDomainParticipantFactory {
    fn handle(&mut self, _message: GetQos) -> <GetQos as Message>::Result {
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

impl Message for SetQos {
    type Result = ();
}

impl Handler<SetQos> for DdsDomainParticipantFactory {
    fn handle(&mut self, message: SetQos) -> <SetQos as Message>::Result {
        self.set_qos(message.qos_kind)
    }
}

pub struct GetDefaultParticipantQos;

impl Message for GetDefaultParticipantQos {
    type Result = DomainParticipantQos;
}

impl Handler<GetDefaultParticipantQos> for DdsDomainParticipantFactory {
    fn handle(
        &mut self,
        _message: GetDefaultParticipantQos,
    ) -> <GetDefaultParticipantQos as Message>::Result {
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

impl Message for SetDefaultParticipantQos {
    type Result = ();
}

impl Handler<SetDefaultParticipantQos> for DdsDomainParticipantFactory {
    fn handle(
        &mut self,
        message: SetDefaultParticipantQos,
    ) -> <SetDefaultParticipantQos as Message>::Result {
        self.set_default_participant_qos(message.qos_kind)
    }
}
