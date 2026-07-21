use std::ptr::NonNull;

use crate::infrastructure::qos_policy::DustDdsUserDataQosPolicy;

/// cbindgen:opaque
pub struct DustDdsDomainParticipantQos(
    pub(crate) dust_dds::infrastructure::qos::DomainParticipantQos,
);

pub type DomainParticipantQos = DustDdsDomainParticipantQos;

impl DustDdsDomainParticipantQos {
    pub fn new(qos: dust_dds::infrastructure::qos::DomainParticipantQos) -> Self {
        Self(qos)
    }

    pub fn inner(&self) -> &dust_dds::infrastructure::qos::DomainParticipantQos {
        &self.0
    }
}

/// Creates a default DomainParticipantQos object.
/// Returns a raw pointer to DustDdsDomainParticipantQos on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_qos_default()
-> Option<NonNull<DustDdsDomainParticipantQos>> {
    NonNull::new(Box::into_raw(Box::new(DustDdsDomainParticipantQos(
        dust_dds::infrastructure::qos::DomainParticipantQos::default(),
    ))))
}

/// Frees a DomainParticipantQos object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_qos_free(
    qos: Option<NonNull<DustDdsDomainParticipantQos>>,
) {
    if let Some(qos) = qos {
        unsafe {
            drop(Box::from_raw(qos.as_ptr()));
        }
    }
}

/// Sets the UserDataQosPolicy on a DomainParticipantQos object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_qos_set_user_data(
    qos: Option<NonNull<DustDdsDomainParticipantQos>>,
    user_data: Option<NonNull<DustDdsUserDataQosPolicy>>,
) {
    if let (Some(mut qos), Some(user_data)) = (qos, user_data) {
        unsafe { qos.as_mut() }.0.user_data = unsafe { user_data.as_ref() }.inner().clone();
    }
}

/// Gets the UserDataQosPolicy from a DomainParticipantQos object.
/// Returns a new DustDdsUserDataQosPolicy instance on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_qos_get_user_data(
    qos: Option<NonNull<DustDdsDomainParticipantQos>>,
) -> Option<NonNull<DustDdsUserDataQosPolicy>> {
    let qos = qos?;
    let user_data = unsafe { qos.as_ref() }.0.user_data.clone();
    NonNull::new(Box::into_raw(Box::new(DustDdsUserDataQosPolicy::new(
        user_data,
    ))))
}

/// cbindgen:opaque
pub struct DustDdsPublisherQos(pub(crate) dust_dds::infrastructure::qos::PublisherQos);

pub type PublisherQos = DustDdsPublisherQos;

impl DustDdsPublisherQos {
    pub fn new(qos: dust_dds::infrastructure::qos::PublisherQos) -> Self {
        Self(qos)
    }

    pub fn inner(&self) -> &dust_dds::infrastructure::qos::PublisherQos {
        &self.0
    }
}

/// Creates a default PublisherQos object.
/// Returns a raw pointer to DustDdsPublisherQos on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_publisher_qos_default() -> Option<NonNull<DustDdsPublisherQos>> {
    NonNull::new(Box::into_raw(Box::new(DustDdsPublisherQos(
        dust_dds::infrastructure::qos::PublisherQos::default(),
    ))))
}

/// Frees a PublisherQos object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_publisher_qos_free(qos: Option<NonNull<DustDdsPublisherQos>>) {
    if let Some(qos) = qos {
        unsafe {
            drop(Box::from_raw(qos.as_ptr()));
        }
    }
}

/// cbindgen:opaque
pub struct DustDdsSubscriberQos(pub(crate) dust_dds::infrastructure::qos::SubscriberQos);

pub type SubscriberQos = DustDdsSubscriberQos;

impl DustDdsSubscriberQos {
    pub fn new(qos: dust_dds::infrastructure::qos::SubscriberQos) -> Self {
        Self(qos)
    }

    pub fn inner(&self) -> &dust_dds::infrastructure::qos::SubscriberQos {
        &self.0
    }
}

/// Creates a default SubscriberQos object.
/// Returns a raw pointer to DustDdsSubscriberQos on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_subscriber_qos_default() -> Option<NonNull<DustDdsSubscriberQos>>
{
    NonNull::new(Box::into_raw(Box::new(DustDdsSubscriberQos(
        dust_dds::infrastructure::qos::SubscriberQos::default(),
    ))))
}

/// Frees a SubscriberQos object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_subscriber_qos_free(qos: Option<NonNull<DustDdsSubscriberQos>>) {
    if let Some(qos) = qos {
        unsafe {
            drop(Box::from_raw(qos.as_ptr()));
        }
    }
}
