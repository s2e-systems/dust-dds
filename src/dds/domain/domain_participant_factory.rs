use std::sync::{Arc, Weak, Mutex};
use crate::dds::types::{DomainId, StatusMask, ReturnCode};
use crate::dds::domain::domain_participant::DomainParticipant;
use crate::dds::domain::domain_participant_impl::DomainParticipantImpl;
use crate::dds::domain::domain_participant_listener::DomainParticipantListener;
use crate::dds::domain::domain_participant::qos::DomainParticipantQos;
use crate::dds::infrastructure::entity::Entity;
use qos::DomainParticipantFactoryQos;

pub mod qos {
    use crate::dds::infrastructure::qos_policy::EntityFactoryQosPolicy;

    #[derive(Default, Debug, PartialEq, Clone)]
    pub struct DomainParticipantFactoryQos {
        pub entity_factory: EntityFactoryQosPolicy,
    }
}

pub struct DomainParticipantFactory{
    participant_list: Mutex<Vec<Weak<DomainParticipantImpl>>>,
    domain_participant_default_qos: Mutex<DomainParticipantQos>,
    domain_participant_factory_qos: Mutex<DomainParticipantFactoryQos>,
}

lazy_static! {
    pub static ref DOMAIN_PARTICIPANT_FACTORY: DomainParticipantFactory = DomainParticipantFactory{
        participant_list: Mutex::new(Vec::new()),
        domain_participant_default_qos: Mutex::new(DomainParticipantQos::default()),
        domain_participant_factory_qos: Mutex::new(DomainParticipantFactoryQos::default()),
    };
}

// pub const PARTICIPANT_QOS_DEFAULT;

impl DomainParticipantFactory {
    /// This operation creates a new DomainParticipant object. The DomainParticipant signifies that the calling application intends
    /// to join the Domain identified by the domain_id argument.
    /// If the specified QoS policies are not consistent, the operation will fail and no DomainParticipant will be created.
    /// The special value PARTICIPANT_QOS_DEFAULT can be used to indicate that the DomainParticipant should be created
    /// with the default DomainParticipant QoS set in the factory. The use of this value is equivalent to the application obtaining the
    /// default DomainParticipant QoS by means of the operation get_default_participant_qos (2.2.2.2.2.6) and using the resulting
    /// QoS to create the DomainParticipant.
    /// In case of failure, the operation will return a ‘nil’ value (as specified by the platform).
    pub fn create_participant (
        &self,
        domain_id: DomainId,
        qos_list: DomainParticipantQos,
        a_listener: impl DomainParticipantListener,
        mask: StatusMask,
    ) ->  Option<DomainParticipant> {
        let new_participant = DomainParticipant(Arc::new(DomainParticipantImpl::new(domain_id, qos_list, a_listener, mask)));

        let domain_participant_factory_qos = self.domain_participant_factory_qos.lock().unwrap();
        if domain_participant_factory_qos.entity_factory.autoenable_created_entities {
            new_participant.enable();
        }

        self.participant_list.lock().unwrap().push(Arc::downgrade(&new_participant.0));

        Some(new_participant)
    }

    /// This operation deletes an existing DomainParticipant. This operation can only be invoked if all domain entities belonging to
    /// the participant have already been deleted. Otherwise the error PRECONDITION_NOT_MET is returned.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    pub fn delete_participant(
        &self,
        a_participant: DomainParticipant,
    ) -> ReturnCode {
        // We rely on the Drop of the DomainParticipant to do the deletion so nothing needs to be done here.
        // If the user calls this function, DomainParticipant has been moved into here and then just gets dropped
        // The deletion only happens if this is the last reference to the DomainParticipantImpl, otherwise the 
        // entity will still be alive. Even if there are alive entities inside the DomainParticipant these will be
        // deleted since the DomainParticipantFactory doesn't own the object and as such can't decide to keep the
        // reference alive
        if Arc::strong_count(&a_participant.0) == 1 {
            ReturnCode::Ok
        } else {
            ReturnCode::PreconditionNotMet
        }
    }

    /// This operation returns the DomainParticipantFactory singleton. The operation is idempotent, that is, it can be called multiple
    /// times without side-effects and it will return the same DomainParticipantFactory instance.
    /// The get_instance operation is a static operation implemented using the syntax of the native language and can therefore not be
    /// expressed in the IDL PSM.
    /// The pre-defined value TheParticipantFactory can also be used as an alias for the singleton factory returned by the operation
    /// get_instance.
    pub fn get_instance() -> &'static DOMAIN_PARTICIPANT_FACTORY {
        &DOMAIN_PARTICIPANT_FACTORY
    }

    /// This operation retrieves a previously created DomainParticipant belonging to specified domain_id. If no such
    /// DomainParticipant exists, the operation will return a ‘nil’ value.
    /// If multiple DomainParticipant entities belonging to that domain_id exist, then the operation will return one of them. It is not
    /// specified which one.
    pub fn lookup_participant(
        &self,
        domain_id: DomainId,
    ) -> Option<DomainParticipant> {
        let participant_list_lock = self.participant_list.lock().unwrap();
        let domain_participant_impl = participant_list_lock
            .iter()
            .find(|&x| {
                if let Some(dp) = &x.upgrade() {
                    DomainParticipantImpl::get_domain_id(dp) == domain_id
                } else {
                    false
                }
            }
        )?;

        Some(DomainParticipant(domain_participant_impl.upgrade()?))
    }

    /// This operation sets a default value of the DomainParticipant QoS policies which will be used for newly created
    /// DomainParticipant entities in the case where the QoS policies are defaulted in the create_participant operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    pub fn set_default_participant_qos(
        &self,
        qos: DomainParticipantQos,
    ) -> ReturnCode {
        *self.domain_participant_default_qos.lock().unwrap() = qos;
        ReturnCode::Ok
    }

    /// This operation retrieves the default value of the DomainParticipant QoS, that is, the QoS policies which will be used for
    /// newly created DomainParticipant entities in the case where the QoS policies are defaulted in the create_participant
    /// operation.
    /// The values retrieved get_default_participant_qos will match the set of values specified on the last successful call to
    /// set_default_participant_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3,
    /// Supported QoS.
    pub fn get_default_participant_qos(
        &self,
        qos: &mut DomainParticipantQos,
    ) -> ReturnCode {
        qos.clone_from(&self.domain_participant_default_qos.lock().unwrap());
        ReturnCode::Ok
    }

    /// This operation sets the value of the DomainParticipantFactory QoS policies. These policies control the behavior of the object
    /// a factory for entities.
    /// Note that despite having QoS, the DomainParticipantFactory is not an Entity.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    pub fn set_qos(
        &self,
        qos: DomainParticipantFactoryQos,
    ) -> ReturnCode {
        *self.domain_participant_factory_qos.lock().unwrap() = qos;
        ReturnCode::Ok
    }

    /// This operation returns the value of the DomainParticipantFactory QoS policies.
    pub fn get_qos(
        &self,
        qos: &mut DomainParticipantFactoryQos,
    ) -> ReturnCode {
        qos.clone_from(&self.domain_participant_factory_qos.lock().unwrap());
        ReturnCode::Ok
    }


    //////////////// From here on are the functions that do not belong to the standard API
    pub(crate) fn remove_participant_reference(&self, weak_participant_impl: &Weak<DomainParticipantImpl>) {
        let mut participant_list_lock = self.participant_list.lock().unwrap();
        if let Some(index) = participant_list_lock.iter().position(|x| x.ptr_eq(weak_participant_impl)) {
            participant_list_lock.swap_remove(index);
        }
    } 
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dds::infrastructure::listener::NoListener;

    #[test]
    fn create_participants() {
        let domain_participant_factory = DomainParticipantFactory::get_instance();

        assert_eq!(domain_participant_factory.participant_list.lock().unwrap().len(), 0);
        let participant = domain_participant_factory.create_participant(1, DomainParticipantQos::default(),NoListener, 0).unwrap();

        assert_eq!(domain_participant_factory.participant_list.lock().unwrap().len(), 1);
        assert!(std::ptr::eq(
            participant.0.as_ref(),
            domain_participant_factory.participant_list.lock().unwrap()[0].upgrade().unwrap().as_ref())
        );

        let participant2 = domain_participant_factory.create_participant(2, DomainParticipantQos::default(),NoListener, 0).unwrap();
        assert_eq!(domain_participant_factory.participant_list.lock().unwrap().len(), 2);
        assert!(std::ptr::eq(
            participant.0.as_ref(),
            domain_participant_factory.participant_list.lock().unwrap()[0].upgrade().unwrap().as_ref())
        );

        assert!(std::ptr::eq(
            participant2.0.as_ref(),
            domain_participant_factory.participant_list.lock().unwrap()[1].upgrade().unwrap().as_ref())
        );
    }

    #[test]
    fn create_and_lookup_participants() {
        let domain_participant_factory = DomainParticipantFactory::get_instance();
        let participant1 = domain_participant_factory.create_participant(1, DomainParticipantQos::default(),NoListener, 0).unwrap();

        // Lookup an existing participant
        let found_participant1 = domain_participant_factory.lookup_participant(1).unwrap();
        assert!(std::ptr::eq(participant1.0.as_ref(), found_participant1.0.as_ref()));

        // Lookup an inexisting participant
        assert!(domain_participant_factory.lookup_participant(2).is_none());

        // Lookup a dropped participant
        {
            let _participant5 = domain_participant_factory.create_participant(5, DomainParticipantQos::default(),NoListener, 0).unwrap();
            assert!(domain_participant_factory.lookup_participant(5).is_some());
        }
        assert!(domain_participant_factory.lookup_participant(5).is_none());

        domain_participant_factory.delete_participant(participant1);
        domain_participant_factory.delete_participant(found_participant1);
    }

    #[test]
    fn delete_participant() {
        let domain_participant_factory = DomainParticipantFactory::get_instance();
        let participant1 = domain_participant_factory.create_participant(1, DomainParticipantQos::default(),NoListener, 0).unwrap();

        // Lookup an existing participant (total reference count is 2)
        let found_participant1 = domain_participant_factory.lookup_participant(1).unwrap();

        // First explicit deletion will fail because there is still one more reference
        assert_eq!(domain_participant_factory.delete_participant(participant1),ReturnCode::PreconditionNotMet);
        // Second deletion should work correctly
        assert_eq!(domain_participant_factory.delete_participant(found_participant1),ReturnCode::Ok);
    }

    #[test]
    fn set_and_get_qos() {
        let domain_participant_factory = DomainParticipantFactory::get_instance();
        // Create an object to retrieve the qos, modify it and check that the returned value is the default one
        let mut qos = DomainParticipantFactoryQos::default();
        qos.entity_factory.autoenable_created_entities = false;
        domain_participant_factory.get_qos(&mut qos);

        assert_eq!(qos, DomainParticipantFactoryQos::default());

        // Modify the qos and verify that the new qos is retrieved
        qos.entity_factory.autoenable_created_entities = false;
        domain_participant_factory.set_qos(qos.clone());    
        let mut new_qos = DomainParticipantFactoryQos::default();
        domain_participant_factory.get_qos(&mut new_qos);
        assert_eq!(qos, new_qos);

        // Set back the default
        domain_participant_factory.set_qos(DomainParticipantFactoryQos::default());
    }
}