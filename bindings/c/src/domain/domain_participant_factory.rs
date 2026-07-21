use std::ptr::NonNull;

use crate::domain::domain_participant::DustDdsDomainParticipant;
use crate::infrastructure::error::{RETCODE_OK, ReturnCode};
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
) -> Option<NonNull<DustDdsDomainParticipant>> {
    let Some(factory) = factory else {
        return None;
    };

    match unsafe { factory.as_ref() }.0.create_participant(
        domain_id,
        QosKind::Default,
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
    factory: *const DustDdsDomainParticipantFactory,
    participant: *mut DustDdsDomainParticipant,
) -> ReturnCode {
    if participant.is_null() {
        return RETCODE_OK;
    }

    let factory_ref = if factory.is_null() {
        dust_dds::domain::domain_participant_factory::DomainParticipantFactory::get_instance()
    } else {
        unsafe { (*factory).0 }
    };

    let dp_box = unsafe { Box::from_raw(participant) };
    match factory_ref.delete_participant(dp_box.inner()) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::domain_participant::dust_dds_domain_participant_free;

    #[test]
    fn create_participant_null_factory() {
        let participant =
            unsafe { dust_dds_domain_participant_factory_create_participant(None, 0) };
        assert!(participant.is_none());
    }

    #[test]
    fn create_participant_valid_factory() {
        let factory = unsafe { dust_dds_domain_participant_factory_get_instance() };
        let participant = unsafe {
            dust_dds_domain_participant_factory_create_participant(
                NonNull::new(factory as *mut _),
                0,
            )
        };
        assert!(participant.is_some());
        unsafe {
            dust_dds_domain_participant_free(participant.unwrap().as_ptr());
        }
    }
}
