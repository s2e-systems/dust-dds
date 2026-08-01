use std::ptr::NonNull;

use crate::domain::domain_participant::DustDdsDomainParticipant;
use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};
use crate::infrastructure::qos::{DomainParticipantFactoryQos, DomainParticipantQos};
use dust_dds::domain::domain_participant_listener::DomainParticipantListener;
use dust_dds::infrastructure::qos::QosKind;

struct NoListener;
impl DomainParticipantListener for NoListener {}

/// cbindgen:opaque
pub struct DustDdsDomainParticipantFactory(
    pub(crate) &'static dust_dds::domain::domain_participant_factory::DomainParticipantFactory,
);

/// Returns the DomainParticipantFactory singleton instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_factory_get_instance()
-> *const DustDdsDomainParticipantFactory {
    static INSTANCE: std::sync::OnceLock<DustDdsDomainParticipantFactory> =
        std::sync::OnceLock::new();
    INSTANCE.get_or_init(|| {
        DustDdsDomainParticipantFactory(
            dust_dds::domain::domain_participant_factory::DomainParticipantFactory::get_instance(),
        )
    })
}

/// Creates a new DomainParticipant object.
/// Passing NULL (`DUST_DDS_PARTICIPANT_QOS_DEFAULT`) for `qos` represents the default QoS.
/// Returns a raw pointer to DustDdsDomainParticipant on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_factory_create_participant(
    factory: Option<NonNull<DustDdsDomainParticipantFactory>>,
    domain_id: i32,
    qos: *const DomainParticipantQos,
) -> Option<NonNull<DustDdsDomainParticipant>> {
    let Some(factory) = factory else {
        return None;
    };

    let qos = if qos.is_null() {
        QosKind::Default
    } else {
        QosKind::Specific(unsafe { &*qos }.clone().into())
    };

    match unsafe { factory.as_ref() }
        .0
        .create_participant(domain_id, qos, None::<NoListener>, &[])
    {
        Ok(participant) => NonNull::new(Box::into_raw(Box::new(DustDdsDomainParticipant::new(
            participant,
        )))),
        Err(_) => None,
    }
}

/// Deletes an existing DomainParticipant.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_factory_delete_participant(
    factory: Option<NonNull<DustDdsDomainParticipantFactory>>,
    participant: Option<NonNull<DustDdsDomainParticipant>>,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_OK;
    };

    let factory_ref = factory.map_or_else(
        || dust_dds::domain::domain_participant_factory::DomainParticipantFactory::get_instance(),
        |f| unsafe { f.as_ref() }.0,
    );

    let participant_ref = unsafe { participant.as_ref() };
    match factory_ref.delete_participant(participant_ref.inner()) {
        Ok(()) => {
            unsafe {
                drop(Box::from_raw(participant.as_ptr()));
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Retrieves a previously created DomainParticipant belonging to the specified domain_id.
/// Returns a raw pointer to DustDdsDomainParticipant on success, or NULL if not found or on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_factory_lookup_participant(
    factory: Option<NonNull<DustDdsDomainParticipantFactory>>,
    domain_id: i32,
) -> Option<NonNull<DustDdsDomainParticipant>> {
    let Some(factory) = factory else {
        return None;
    };

    match unsafe { factory.as_ref() }.0.lookup_participant(domain_id) {
        Ok(Some(participant)) => NonNull::new(Box::into_raw(Box::new(
            DustDdsDomainParticipant::new(participant),
        ))),
        _ => None,
    }
}

/// Sets the default DomainParticipantQos.
/// Passing NULL represents the default QoS.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_factory_set_default_participant_qos(
    factory: Option<NonNull<DustDdsDomainParticipantFactory>>,
    qos: *const DomainParticipantQos,
) -> ReturnCode {
    let Some(factory) = factory else {
        return RETCODE_BAD_PARAMETER;
    };

    let qos = if qos.is_null() {
        QosKind::Default
    } else {
        QosKind::Specific(unsafe { &*qos }.clone().into())
    };

    match unsafe { factory.as_ref() }
        .0
        .set_default_participant_qos(qos)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the default DomainParticipantQos.
/// Writes the QoS into the user-supplied pointer.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_factory_get_default_participant_qos(
    factory: Option<NonNull<DustDdsDomainParticipantFactory>>,
    qos: *mut DomainParticipantQos,
) -> ReturnCode {
    let Some(factory) = factory else {
        return RETCODE_BAD_PARAMETER;
    };
    if qos.is_null() {
        return RETCODE_BAD_PARAMETER;
    }

    match unsafe { factory.as_ref() }.0.get_default_participant_qos() {
        Ok(q) => {
            unsafe { *qos = q.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Sets the DomainParticipantFactoryQos policies.
/// Passing NULL represents the default QoS.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_factory_set_qos(
    factory: Option<NonNull<DustDdsDomainParticipantFactory>>,
    qos: *const DomainParticipantFactoryQos,
) -> ReturnCode {
    let Some(factory) = factory else {
        return RETCODE_BAD_PARAMETER;
    };

    let qos = if qos.is_null() {
        QosKind::Default
    } else {
        QosKind::Specific(unsafe { &*qos }.clone().into())
    };

    match unsafe { factory.as_ref() }.0.set_qos(qos) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the DomainParticipantFactoryQos policies.
/// Writes the QoS into the user-supplied pointer.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_factory_get_qos(
    factory: Option<NonNull<DustDdsDomainParticipantFactory>>,
    qos: *mut DomainParticipantFactoryQos,
) -> ReturnCode {
    let Some(factory) = factory else {
        return RETCODE_BAD_PARAMETER;
    };
    if qos.is_null() {
        return RETCODE_BAD_PARAMETER;
    }

    match unsafe { factory.as_ref() }.0.get_qos() {
        Ok(q) => {
            unsafe { *qos = q.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::qos::dds_domain_participant_qos_default;

    #[test]
    fn create_participant_null_factory() {
        let participant =
            unsafe { dds_domain_participant_factory_create_participant(None, 0, std::ptr::null()) };
        assert!(participant.is_none());
    }

    #[test]
    fn create_participant_valid_factory() {
        let factory = unsafe { dds_domain_participant_factory_get_instance() };
        let participant = unsafe {
            dds_domain_participant_factory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                std::ptr::null(),
            )
        };
        assert!(participant.is_some());
        let result = unsafe {
            dds_domain_participant_factory_delete_participant(
                NonNull::new(factory as *mut _),
                participant,
            )
        };
        assert_eq!(result, RETCODE_OK);
    }

    #[test]
    fn create_participant_with_qos() {
        let factory = unsafe { dds_domain_participant_factory_get_instance() };
        let qos = unsafe { dds_domain_participant_qos_default() };
        let participant = unsafe {
            dds_domain_participant_factory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                &qos,
            )
        };
        assert!(participant.is_some());
        let result = unsafe {
            dds_domain_participant_factory_delete_participant(
                NonNull::new(factory as *mut _),
                participant,
            )
        };
        assert_eq!(result, RETCODE_OK);
    }

    #[test]
    fn lookup_participant() {
        let factory = unsafe { dds_domain_participant_factory_get_instance() };
        let participant = unsafe {
            dds_domain_participant_factory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                std::ptr::null(),
            )
        };
        assert!(participant.is_some());

        let looked_up = unsafe {
            dds_domain_participant_factory_lookup_participant(NonNull::new(factory as *mut _), 0)
        };
        assert!(looked_up.is_some());

        unsafe {
            dds_domain_participant_factory_delete_participant(
                NonNull::new(factory as *mut _),
                looked_up,
            );
            drop(Box::from_raw(participant.unwrap().as_ptr()));
        }
    }

    #[test]
    fn default_participant_qos() {
        let factory = unsafe { dds_domain_participant_factory_get_instance() };
        let mut qos = DomainParticipantQos::default();
        let result = unsafe {
            dds_domain_participant_factory_get_default_participant_qos(
                NonNull::new(factory as *mut _),
                &mut qos,
            )
        };
        assert_eq!(result, RETCODE_OK);

        let result = unsafe {
            dds_domain_participant_factory_set_default_participant_qos(
                NonNull::new(factory as *mut _),
                &qos,
            )
        };
        assert_eq!(result, RETCODE_OK);
    }

    #[test]
    fn factory_qos() {
        use crate::infrastructure::qos::dds_domain_participant_factory_qos_default;

        let factory = unsafe { dds_domain_participant_factory_get_instance() };
        let mut qos = DomainParticipantFactoryQos::default();
        let result = unsafe {
            dds_domain_participant_factory_get_qos(NonNull::new(factory as *mut _), &mut qos)
        };
        assert_eq!(result, RETCODE_OK);

        let result = unsafe {
            dds_domain_participant_factory_set_qos(NonNull::new(factory as *mut _), &qos)
        };
        assert_eq!(result, RETCODE_OK);

        let default_qos = unsafe { dds_domain_participant_factory_qos_default() };
        assert_eq!(default_qos.entity_factory.autoenable_created_entities, true);
    }
}
