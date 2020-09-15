use std::sync::Weak;

use crate::dds::types::ReturnCode;
use crate::dds::infrastructure::status::InconsistentTopicStatus;
use crate::dds::domain::domain_participant::DomainParticipant;
use crate::dds::domain::domain_participant_impl::DomainParticipantImpl;

pub struct TopicImpl{
    parent_participant: Weak<DomainParticipantImpl>,
    name: String,
    type_name: String,
}

impl TopicImpl {
    pub(crate) fn get_inconsistent_topic_status(
        _this: &Weak<TopicImpl>,
        _status: &mut InconsistentTopicStatus,
    ) -> ReturnCode {
        todo!()
    }

    ///////////////// Topic description trait methods
    pub(crate) fn get_participant(this: &Weak<TopicImpl>) -> DomainParticipant {
        DomainParticipant(this.upgrade().unwrap().parent_participant.upgrade().unwrap())
    }

    pub(crate) fn get_type_name(this: &Weak<TopicImpl>) -> String {
        this.upgrade().unwrap().name.clone()
    }

    pub(crate) fn get_name(this: &Weak<TopicImpl>) -> String {
        this.upgrade().unwrap().type_name.clone()
    }

    ///////////////// Entity trait methods
    
    //TODO
    

    //////////////// From here on are the functions that do not belong to the standard API
    pub(crate) fn new(
        parent_participant: Weak<DomainParticipantImpl>,
        name: String,
        type_name: String,
    ) -> Self {
        Self{
            parent_participant,
            name,
            type_name,
        }
    }
}