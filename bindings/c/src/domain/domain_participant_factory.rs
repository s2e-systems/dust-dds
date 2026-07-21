use std::ptr::NonNull;

use crate::domain::domain_participant::DustDdsDomainParticipant;
use crate::infrastructure::error::{RETCODE_OK, ReturnCode};
use crate::infrastructure::qos::DustDdsDomainParticipantQos;
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
pub unsafe extern "C" fn dust_dds_domain_participant_factory_get_instance()
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
/// Returns a raw pointer to DustDdsDomainParticipant on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_factory_create_participant(
    factory: Option<NonNull<DustDdsDomainParticipantFactory>>,
    domain_id: i32,
    qos: Option<NonNull<DustDdsDomainParticipantQos>>,
) -> Option<NonNull<DustDdsDomainParticipant>> {
    let Some(factory) = factory else {
        return None;
    };

    let qos = match qos {
        Some(q) => QosKind::Specific(unsafe { q.as_ref() }.inner().clone()),
        None => QosKind::Default,
    };

    match unsafe { factory.as_ref() }.0.create_participant(
        domain_id,
        qos,
        None::<NoListener>,
        &[],
    ) {
        Ok(participant) => NonNull::new(Box::into_raw(Box::new(DustDdsDomainParticipant::new(
            participant,
        )))),
        Err(_) => None,
    }
}

/// Deletes an existing DomainParticipant.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_factory_delete_participant(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::qos::dust_dds_domain_participant_qos_default;

    #[test]
    fn create_participant_null_factory() {
        let participant =
            unsafe { dust_dds_domain_participant_factory_create_participant(None, 0, None) };
        assert!(participant.is_none());
    }

    #[test]
    fn create_participant_valid_factory() {
        let factory = unsafe { dust_dds_domain_participant_factory_get_instance() };
        let participant = unsafe {
            dust_dds_domain_participant_factory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                None,
            )
        };
        assert!(participant.is_some());
        let result = unsafe {
            dust_dds_domain_participant_factory_delete_participant(
                NonNull::new(factory as *mut _),
                participant,
            )
        };
        assert_eq!(result, RETCODE_OK);
    }

    #[test]
    fn create_participant_with_qos() {
        let factory = unsafe { dust_dds_domain_participant_factory_get_instance() };
        let qos = unsafe { dust_dds_domain_participant_qos_default() };
        let participant = unsafe {
            dust_dds_domain_participant_factory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                qos,
            )
        };
        assert!(participant.is_some());
        let result = unsafe {
            dust_dds_domain_participant_factory_delete_participant(
                NonNull::new(factory as *mut _),
                participant,
            )
        };
        assert_eq!(result, RETCODE_OK);
        unsafe {
            crate::infrastructure::qos::dust_dds_domain_participant_qos_free(qos);
        }
    }
}
