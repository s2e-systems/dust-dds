use std::ptr::NonNull;

use crate::{
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        condition::StatusMask,
        error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode},
        listeners::{CDomainParticipantListenerWrapper, DomainParticipantListener},
        qos::{DomainParticipantFactoryQos, DomainParticipantQos},
    },
};
use dust_dds::infrastructure::qos::QosKind;

/// cbindgen:opaque
pub struct DomainParticipantFactory(
    pub(crate) &'static dust_dds::domain::domain_participant_factory::DomainParticipantFactory,
);

/// Returns the DomainParticipantFactory singleton instance.
///
/// # Safety
///
/// There are no special safety invariants for calling this function caller must observe the following safety invariants:
/// - The caller must observe the standard FFI safety constraints when calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DomainParticipantFactory_get_instance()
-> *const DomainParticipantFactory {
    static INSTANCE: std::sync::OnceLock<DomainParticipantFactory> =
        std::sync::OnceLock::new();
    INSTANCE.get_or_init(|| {
        DomainParticipantFactory(
            dust_dds::domain::domain_participant_factory::DomainParticipantFactory::get_instance(),
        )
    })
}

/// Creates a new DomainParticipant object.
/// Passing NULL (`DUST_DDS_PARTICIPANT_QOS_DEFAULT`) for `qos` represents the default QoS.
/// Returns a raw pointer to DomainParticipant on success, or NULL on failure.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `factory` must point to a valid, initialized `DomainParticipantFactory` instance.
/// - `qos` must be a valid pointer to a `DomainParticipantQos` instance (or null).
/// - `listener` must be a valid pointer to a `DomainParticipantListener` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DomainParticipantFactory_create_participant(
    factory: Option<NonNull<DomainParticipantFactory>>,
    domain_id: i32,
    qos: *const DomainParticipantQos,
    listener: *const DomainParticipantListener,
    mask: StatusMask,
) -> Option<NonNull<DomainParticipant>> {
    let factory = factory?;

    let qos = if qos.is_null() {
        QosKind::Default
    } else {
        QosKind::Specific((*unsafe { &*qos }).into())
    };

    let status_kinds = crate::infrastructure::condition::mask_to_status_kinds(mask);

    let result = if listener.is_null() {
        unsafe { factory.as_ref() }.0.create_participant(
            domain_id,
            qos,
            None::<CDomainParticipantListenerWrapper>,
            &status_kinds,
        )
    } else {
        let wrapper = CDomainParticipantListenerWrapper {
            listener: unsafe { *listener },
        };
        unsafe { factory.as_ref() }.0.create_participant(
            domain_id,
            qos,
            Some(wrapper),
            &status_kinds,
        )
    };

    match result {
        Ok(participant) => NonNull::new(Box::into_raw(Box::new(DomainParticipant::new(
            participant,
        )))),
        Err(_) => None,
    }
}

/// Deletes an existing DomainParticipant.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `factory` must point to a valid, initialized `DomainParticipantFactory` instance.
/// - `participant` must point to a valid, initialized `DomainParticipant` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DomainParticipantFactory_delete_participant(
    factory: Option<NonNull<DomainParticipantFactory>>,
    participant: Option<NonNull<DomainParticipant>>,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_OK;
    };

    let factory_ref = factory.map_or_else(
        dust_dds::domain::domain_participant_factory::DomainParticipantFactory::get_instance,
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
/// Returns a raw pointer to DomainParticipant on success, or NULL if not found or on error.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `factory` must point to a valid, initialized `DomainParticipantFactory` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DomainParticipantFactory_lookup_participant(
    factory: Option<NonNull<DomainParticipantFactory>>,
    domain_id: i32,
) -> Option<NonNull<DomainParticipant>> {
    let factory = factory?;

    match unsafe { factory.as_ref() }.0.lookup_participant(domain_id) {
        Ok(Some(participant)) => NonNull::new(Box::into_raw(Box::new(
            DomainParticipant::new(participant),
        ))),
        _ => None,
    }
}

/// Sets the default DomainParticipantQos.
/// Passing NULL represents the default QoS.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `factory` must point to a valid, initialized `DomainParticipantFactory` instance.
/// - `qos` must be a valid pointer to a `DomainParticipantQos` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DomainParticipantFactory_set_default_participant_qos(
    factory: Option<NonNull<DomainParticipantFactory>>,
    qos: *const DomainParticipantQos,
) -> ReturnCode {
    let Some(factory) = factory else {
        return RETCODE_BAD_PARAMETER;
    };

    let qos = if qos.is_null() {
        QosKind::Default
    } else {
        QosKind::Specific((*unsafe { &*qos }).into())
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
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `factory` must point to a valid, initialized `DomainParticipantFactory` instance.
/// - `qos` must be a valid pointer to a `DomainParticipantQos` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DomainParticipantFactory_get_default_participant_qos(
    factory: Option<NonNull<DomainParticipantFactory>>,
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
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `factory` must point to a valid, initialized `DomainParticipantFactory` instance.
/// - `qos` must be a valid pointer to a `DomainParticipantFactoryQos` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DomainParticipantFactory_set_qos(
    factory: Option<NonNull<DomainParticipantFactory>>,
    qos: *const DomainParticipantFactoryQos,
) -> ReturnCode {
    let Some(factory) = factory else {
        return RETCODE_BAD_PARAMETER;
    };

    let qos = if qos.is_null() {
        QosKind::Default
    } else {
        QosKind::Specific((*unsafe { &*qos }).into())
    };

    match unsafe { factory.as_ref() }.0.set_qos(qos) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the DomainParticipantFactoryQos policies.
/// Writes the QoS into the user-supplied pointer.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `factory` must point to a valid, initialized `DomainParticipantFactory` instance.
/// - `qos` must be a valid pointer to a `DomainParticipantFactoryQos` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DomainParticipantFactory_get_qos(
    factory: Option<NonNull<DomainParticipantFactory>>,
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
    use crate::infrastructure::qos::DDS_DomainParticipant_qos_default;

    #[test]
    fn create_participant_null_factory() {
        let participant = unsafe {
            DDS_DomainParticipantFactory_create_participant(
                None,
                0,
                std::ptr::null(),
                std::ptr::null(),
                0,
            )
        };
        assert!(participant.is_none());
    }

    #[test]
    fn create_participant_valid_factory() {
        let factory = unsafe { DDS_DomainParticipantFactory_get_instance() };
        let participant = unsafe {
            DDS_DomainParticipantFactory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                std::ptr::null(),
                std::ptr::null(),
                0,
            )
        };
        assert!(participant.is_some());
        let result = unsafe {
            DDS_DomainParticipantFactory_delete_participant(
                NonNull::new(factory as *mut _),
                participant,
            )
        };
        assert_eq!(result, RETCODE_OK);
    }

    #[test]
    fn create_participant_with_qos() {
        let factory = unsafe { DDS_DomainParticipantFactory_get_instance() };
        let qos = unsafe { DDS_DomainParticipant_qos_default() };
        let participant = unsafe {
            DDS_DomainParticipantFactory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                &qos,
                std::ptr::null(),
                0,
            )
        };
        assert!(participant.is_some());
        let result = unsafe {
            DDS_DomainParticipantFactory_delete_participant(
                NonNull::new(factory as *mut _),
                participant,
            )
        };
        assert_eq!(result, RETCODE_OK);
    }

    #[test]
    fn lookup_participant() {
        let factory = unsafe { DDS_DomainParticipantFactory_get_instance() };
        let participant = unsafe {
            DDS_DomainParticipantFactory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                std::ptr::null(),
                std::ptr::null(),
                0,
            )
        };
        assert!(participant.is_some());

        let looked_up = unsafe {
            DDS_DomainParticipantFactory_lookup_participant(NonNull::new(factory as *mut _), 0)
        };
        assert!(looked_up.is_some());

        unsafe {
            DDS_DomainParticipantFactory_delete_participant(
                NonNull::new(factory as *mut _),
                looked_up,
            );
            drop(Box::from_raw(participant.unwrap().as_ptr()));
        }
    }

    #[test]
    fn default_participant_qos() {
        let factory = unsafe { DDS_DomainParticipantFactory_get_instance() };
        let mut qos = DomainParticipantQos::default();
        let result = unsafe {
            DDS_DomainParticipantFactory_get_default_participant_qos(
                NonNull::new(factory as *mut _),
                &mut qos,
            )
        };
        assert_eq!(result, RETCODE_OK);

        let result = unsafe {
            DDS_DomainParticipantFactory_set_default_participant_qos(
                NonNull::new(factory as *mut _),
                &qos,
            )
        };
        assert_eq!(result, RETCODE_OK);
    }

    #[test]
    fn factory_qos() {
        use crate::infrastructure::qos::DDS_DomainParticipantFactory_qos_default;

        let factory = unsafe { DDS_DomainParticipantFactory_get_instance() };
        let mut qos = DomainParticipantFactoryQos::default();
        let result = unsafe {
            DDS_DomainParticipantFactory_get_qos(NonNull::new(factory as *mut _), &mut qos)
        };
        assert_eq!(result, RETCODE_OK);

        let result = unsafe {
            DDS_DomainParticipantFactory_set_qos(NonNull::new(factory as *mut _), &qos)
        };
        assert_eq!(result, RETCODE_OK);

        let default_qos = unsafe { DDS_DomainParticipantFactory_qos_default() };
        assert!(default_qos.entity_factory.autoenable_created_entities);
    }
}
