
use std::sync::Weak;

use crate::types::ReturnCode;
use crate::infrastructure::status::InconsistentTopicStatus;
use crate::domain::DomainParticipant;

use crate::implementation::domain_participant_impl::DomainParticipantImpl;

pub struct TopicImpl{
    parent_participant: Weak<DomainParticipantImpl>,
    name: String,
    type_name: String,
}

impl TopicImpl {
    pub(crate) fn get_inconsistent_topic_status(
        _this: &Weak<TopicImpl>,
        _status: &mut InconsistentTopicStatus,
    ) -> ReturnCode<()> {
        todo!()
    }

    ///////////////// Topic description trait methods
    pub(crate) fn get_participant(this: &Weak<TopicImpl>) -> Option<DomainParticipant> {
        Some(DomainParticipant(this.upgrade()?.parent_participant.upgrade()?))
    }

    pub(crate) fn get_type_name(this: &Weak<TopicImpl>) -> Option<String> {
        Some(this.upgrade()?.name.clone())
    }

    pub(crate) fn get_name(this: &Weak<TopicImpl>) -> Option<String> {
        Some(this.upgrade()?.type_name.clone())
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