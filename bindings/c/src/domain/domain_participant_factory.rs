use crate::domain::domain_participant::DustDdsDomainParticipant;
use crate::infrastructure::error::{DustDdsReturnCode, DUST_DDS_ERROR, DUST_DDS_OK};
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
pub unsafe extern "C" fn dust_dds_domain_participant_factory_get_instance(
) -> *const DustDdsDomainParticipantFactory {
    static INSTANCE: std::sync::OnceLock<DustDdsDomainParticipantFactory> =
        std::sync::OnceLock::new();
    INSTANCE.get_or_init(|| {
        DustDdsDomainParticipantFactory(
            dust_dds::domain::domain_participant_factory::DomainParticipantFactory::get_instance(
            ),
        )
    })
}

/// Creates a new DomainParticipant object.
/// Returns a raw pointer to DustDdsDomainParticipant on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_factory_create_participant(
    factory: *const DustDdsDomainParticipantFactory,
    domain_id: i32,
) -> *mut DustDdsDomainParticipant {
    let factory_ref = if factory.is_null() {
        dust_dds::domain::domain_participant_factory::DomainParticipantFactory::get_instance()
    } else {
        unsafe { (*factory).0 }
    };

    match factory_ref.create_participant(
        domain_id,
        QosKind::Default,
        None::<NoListener>,
        &[],
    ) {
        Ok(participant) => Box::into_raw(Box::new(DustDdsDomainParticipant::new(participant))),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Deletes an existing DomainParticipant.
/// Returns DUST_DDS_OK on success, or DUST_DDS_ERROR on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_factory_delete_participant(
    factory: *const DustDdsDomainParticipantFactory,
    participant: *mut DustDdsDomainParticipant,
) -> DustDdsReturnCode {
    if participant.is_null() {
        return DUST_DDS_OK;
    }

    let factory_ref = if factory.is_null() {
        dust_dds::domain::domain_participant_factory::DomainParticipantFactory::get_instance()
    } else {
        unsafe { (*factory).0 }
    };

    let dp_box = unsafe { Box::from_raw(participant) };
    match factory_ref.delete_participant(dp_box.inner()) {
        Ok(()) => DUST_DDS_OK,
        Err(_) => DUST_DDS_ERROR,
    }
}
