use std::ptr::NonNull;

use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};
use crate::infrastructure::qos::{DustDdsPublisherQos, DustDdsSubscriberQos, DustDdsTopicQos};
use crate::publication::publisher::DustDdsPublisher;
use crate::subscription::subscriber::DustDdsSubscriber;
use crate::topic_definition::dynamic_type::DustDdsDynamicType;
use crate::topic_definition::topic::DustDdsTopic;
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
/// Passing NULL (`DUST_DDS_PUBLISHER_QOS_DEFAULT`) for `qos` represents the default QoS.
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
/// Passing NULL (`DUST_DDS_SUBSCRIBER_QOS_DEFAULT`) for `qos` represents the default QoS.
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

/// Creates a new Topic object.
/// Passing NULL (`DUST_DDS_TOPIC_QOS_DEFAULT`) for `qos` represents the default QoS.
/// Returns a raw pointer to DustDdsTopic on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_create_topic(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    topic_name: *const std::os::raw::c_char,
    type_name: *const std::os::raw::c_char,
    qos: Option<NonNull<DustDdsTopicQos>>,
    dynamic_type: Option<NonNull<DustDdsDynamicType>>,
) -> Option<NonNull<DustDdsTopic>> {
    let Some(participant) = participant else {
        return None;
    };
    if topic_name.is_null() || type_name.is_null() {
        return None;
    }
    let Some(dynamic_type) = dynamic_type else {
        return None;
    };

    let topic_name_str = unsafe { std::ffi::CStr::from_ptr(topic_name) }
        .to_str()
        .ok()?;
    let type_name_str = unsafe { std::ffi::CStr::from_ptr(type_name) }
        .to_str()
        .ok()?;

    let qos = match qos {
        Some(q) => QosKind::Specific(unsafe { q.as_ref() }.inner().clone()),
        None => QosKind::Default,
    };

    struct NoTopicListener;
    impl dust_dds::topic_definition::topic_listener::TopicListener for NoTopicListener {}

    let participant_ref = unsafe { participant.as_ref() };
    let dynamic_type_ref = unsafe { dynamic_type.as_ref() };

    match participant_ref.inner().create_dynamic_topic(
        topic_name_str,
        type_name_str,
        qos,
        None::<NoTopicListener>,
        &[],
        dynamic_type_ref.inner().clone(),
    ) {
        Ok(topic) => NonNull::new(Box::into_raw(Box::new(DustDdsTopic::new(topic)))),
        Err(_) => None,
    }
}

/// Deletes an existing Topic object.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_delete_topic(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    topic: Option<NonNull<DustDdsTopic>>,
) -> ReturnCode {
    let Some(topic) = topic else {
        return RETCODE_OK;
    };

    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };

    let participant_ref = unsafe { participant.as_ref() };
    let topic_ref = unsafe { topic.as_ref() };

    match participant_ref.inner().delete_topic(topic_ref.inner()) {
        Ok(()) => {
            unsafe {
                drop(Box::from_raw(topic.as_ptr()));
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

    #[test]
    fn create_delete_topic() {
        use crate::topic_definition::dynamic_type::{
            TYPE_KIND_INT32, DustDdsMemberDescriptor, dust_dds_dynamic_type_builder_add_member,
            dust_dds_dynamic_type_builder_build, dust_dds_dynamic_type_builder_create_struct,
            dust_dds_dynamic_type_free, dust_dds_dynamic_type_get_primitive_type,
        };

        let factory = unsafe { dust_dds_domain_participant_factory_get_instance() };
        let participant = unsafe {
            dust_dds_domain_participant_factory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                None,
            )
        };
        assert!(participant.is_some());

        let struct_name = std::ffi::CString::new("MyStruct").unwrap();
        let builder = unsafe { dust_dds_dynamic_type_builder_create_struct(struct_name.as_ptr()) };
        assert!(builder.is_some());

        let field_name = std::ffi::CString::new("a").unwrap();
        let int32_type = unsafe { dust_dds_dynamic_type_get_primitive_type(TYPE_KIND_INT32) };
        assert!(int32_type.is_some());

        let member_descriptor = DustDdsMemberDescriptor {
            name: field_name.as_ptr(),
            id: 0,
            r#type: int32_type.unwrap().as_ptr(),
            is_key: false,
            is_optional: false,
            is_must_understand: true,
        };

        let res = unsafe {
            dust_dds_dynamic_type_builder_add_member(builder, &member_descriptor)
        };
        assert_eq!(res, RETCODE_OK);

        let dynamic_type = unsafe { dust_dds_dynamic_type_builder_build(builder) };
        assert!(dynamic_type.is_some());

        let topic_name = std::ffi::CString::new("MyTopic").unwrap();
        let type_name = std::ffi::CString::new("MyStruct").unwrap();

        let topic = unsafe {
            dust_dds_domain_participant_create_topic(
                participant,
                topic_name.as_ptr(),
                type_name.as_ptr(),
                None,
                dynamic_type,
            )
        };
        assert!(topic.is_some());

        let res = unsafe { dust_dds_domain_participant_delete_topic(participant, topic) };
        assert_eq!(res, RETCODE_OK);

        unsafe {
            dust_dds_dynamic_type_free(dynamic_type);
            dust_dds_domain_participant_factory_delete_participant(
                NonNull::new(factory as *mut _),
                participant,
            );
        }
    }
}
