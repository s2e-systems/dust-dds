use std::ptr::NonNull;

use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};
use crate::infrastructure::qos::{DustDdsPublisherQos, DustDdsSubscriberQos};
use crate::publication::publisher::DustDdsPublisher;
use crate::subscription::subscriber::DustDdsSubscriber;
use dust_dds::infrastructure::qos::QosKind;
use dust_dds::publication::publisher_listener::PublisherListener;
use dust_dds::subscription::subscriber_listener::SubscriberListener;

struct NoPublisherListener;
impl PublisherListener for NoPublisherListener {}

struct NoSubscriberListener;
impl SubscriberListener for NoSubscriberListener {}

/// cbindgen:opaque
pub struct DustDdsDomainParticipant(
    pub(crate) dust_dds::domain::domain_participant::DomainParticipant,
);

impl DustDdsDomainParticipant {
    pub fn new(dp: dust_dds::domain::domain_participant::DomainParticipant) -> Self {
        Self(dp)
    }

    pub fn inner(&self) -> &dust_dds::domain::domain_participant::DomainParticipant {
        &self.0
    }
}

/// Creates a new Publisher object.
/// Returns a raw pointer to DustDdsPublisher on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_create_publisher(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    qos: Option<NonNull<DustDdsPublisherQos>>,
) -> Option<NonNull<DustDdsPublisher>> {
    let Some(participant) = participant else {
        return None;
    };

    let qos = match qos {
        Some(q) => QosKind::Specific(unsafe { q.as_ref() }.inner().clone()),
        None => QosKind::Default,
    };

    match unsafe { participant.as_ref() }.inner().create_publisher(
        qos,
        None::<NoPublisherListener>,
        &[],
    ) {
        Ok(publisher) => NonNull::new(Box::into_raw(Box::new(DustDdsPublisher::new(publisher)))),
        Err(_) => None,
    }
}

/// Deletes an existing Publisher object.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_delete_publisher(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    publisher: Option<NonNull<DustDdsPublisher>>,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_OK;
    };

    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };

    let participant_ref = unsafe { participant.as_ref() };
    let publisher_ref = unsafe { publisher.as_ref() };

    match participant_ref
        .inner()
        .delete_publisher(publisher_ref.inner())
    {
        Ok(()) => {
            unsafe {
                drop(Box::from_raw(publisher.as_ptr()));
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Creates a new Subscriber object.
/// Returns a raw pointer to DustDdsSubscriber on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_create_subscriber(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    qos: Option<NonNull<DustDdsSubscriberQos>>,
) -> Option<NonNull<DustDdsSubscriber>> {
    let Some(participant) = participant else {
        return None;
    };

    let qos = match qos {
        Some(q) => QosKind::Specific(unsafe { q.as_ref() }.inner().clone()),
        None => QosKind::Default,
    };

    match unsafe { participant.as_ref() }.inner().create_subscriber(
        qos,
        None::<NoSubscriberListener>,
        &[],
    ) {
        Ok(subscriber) => NonNull::new(Box::into_raw(Box::new(DustDdsSubscriber::new(subscriber)))),
        Err(_) => None,
    }
}

/// Deletes an existing Subscriber object.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_delete_subscriber(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    subscriber: Option<NonNull<DustDdsSubscriber>>,
) -> ReturnCode {
    let Some(subscriber) = subscriber else {
        return RETCODE_OK;
    };

    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };

    let participant_ref = unsafe { participant.as_ref() };
    let subscriber_ref = unsafe { subscriber.as_ref() };

    match participant_ref
        .inner()
        .delete_subscriber(subscriber_ref.inner())
    {
        Ok(()) => {
            unsafe {
                drop(Box::from_raw(subscriber.as_ptr()));
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::domain_participant_factory::{
        dust_dds_domain_participant_factory_create_participant,
        dust_dds_domain_participant_factory_delete_participant,
        dust_dds_domain_participant_factory_get_instance,
    };
    use crate::infrastructure::qos::{
        dust_dds_publisher_qos_default, dust_dds_publisher_qos_free,
        dust_dds_subscriber_qos_default, dust_dds_subscriber_qos_free,
    };

    #[test]
    fn create_delete_publisher() {
        let factory = unsafe { dust_dds_domain_participant_factory_get_instance() };
        let participant = unsafe {
            dust_dds_domain_participant_factory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                None,
            )
        };
        assert!(participant.is_some());

        let publisher = unsafe { dust_dds_domain_participant_create_publisher(participant, None) };
        assert!(publisher.is_some());

        let res = unsafe { dust_dds_domain_participant_delete_publisher(participant, publisher) };
        assert_eq!(res, RETCODE_OK);

        let qos = unsafe { dust_dds_publisher_qos_default() };
        let publisher = unsafe { dust_dds_domain_participant_create_publisher(participant, qos) };
        assert!(publisher.is_some());
        unsafe { dust_dds_publisher_qos_free(qos) };

        let res = unsafe { dust_dds_domain_participant_delete_publisher(participant, publisher) };
        assert_eq!(res, RETCODE_OK);

        unsafe {
            dust_dds_domain_participant_factory_delete_participant(
                NonNull::new(factory as *mut _),
                participant,
            );
        }
    }

    #[test]
    fn create_delete_subscriber() {
        let factory = unsafe { dust_dds_domain_participant_factory_get_instance() };
        let participant = unsafe {
            dust_dds_domain_participant_factory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                None,
            )
        };
        assert!(participant.is_some());

        let subscriber =
            unsafe { dust_dds_domain_participant_create_subscriber(participant, None) };
        assert!(subscriber.is_some());

        let res = unsafe { dust_dds_domain_participant_delete_subscriber(participant, subscriber) };
        assert_eq!(res, RETCODE_OK);

        let qos = unsafe { dust_dds_subscriber_qos_default() };
        let subscriber = unsafe { dust_dds_domain_participant_create_subscriber(participant, qos) };
        assert!(subscriber.is_some());
        unsafe { dust_dds_subscriber_qos_free(qos) };

        let res = unsafe { dust_dds_domain_participant_delete_subscriber(participant, subscriber) };
        assert_eq!(res, RETCODE_OK);

        unsafe {
            dust_dds_domain_participant_factory_delete_participant(
                NonNull::new(factory as *mut _),
                participant,
            );
        }
    }
}
