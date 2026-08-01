#![allow(improper_ctypes_definitions)]

use std::ptr::NonNull;

use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_ERROR, RETCODE_OK, ReturnCode};
use crate::infrastructure::qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos};
use crate::publication::publisher::DustDdsPublisher;
use crate::subscription::subscriber::DustDdsSubscriber;
use crate::topic_definition::dynamic_type::DustDdsDynamicType;
use crate::topic_definition::topic::DustDdsTopic;
use dust_dds::infrastructure::qos::QosKind;

use crate::infrastructure::condition::DustDdsStatusMask;
use crate::infrastructure::listeners::{
    CDomainParticipantListenerWrapper, CPublisherListenerWrapper, CSubscriberListenerWrapper,
    CTopicListenerWrapper, DustDdsDomainParticipantListener, DustDdsPublisherListener,
    DustDdsSubscriberListener, DustDdsTopicListener,
};
use crate::infrastructure::qos_policy::{Duration_t, StringSeq, Time_t};
use crate::infrastructure::status::{
    BuiltInTopicKey, InstanceHandle_t, InstanceHandleSeq, ParticipantBuiltinTopicData,
    TopicBuiltinTopicData,
};
use crate::topic_definition::content_filtered_topic::DustDdsContentFilteredTopic;
use crate::topic_definition::topic_description::DustDdsTopicDescription;

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
pub unsafe extern "C" fn dds_domain_participant_create_publisher(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    qos: *const PublisherQos,
    listener: *const DustDdsPublisherListener,
    mask: DustDdsStatusMask,
) -> Option<NonNull<DustDdsPublisher>> {
    let Some(participant) = participant else {
        return None;
    };

    let qos = if qos.is_null() {
        QosKind::Default
    } else {
        QosKind::Specific(unsafe { &*qos }.clone().into())
    };

    let status_kinds = crate::infrastructure::condition::mask_to_status_kinds(mask);

    let result = if listener.is_null() {
        unsafe { participant.as_ref() }.inner().create_publisher(
            qos,
            None::<CPublisherListenerWrapper>,
            &status_kinds,
        )
    } else {
        let wrapper = CPublisherListenerWrapper {
            listener: unsafe { *listener },
        };
        unsafe { participant.as_ref() }
            .inner()
            .create_publisher(qos, Some(wrapper), &status_kinds)
    };

    match result {
        Ok(publisher) => NonNull::new(Box::into_raw(Box::new(DustDdsPublisher::new(publisher)))),
        Err(_) => None,
    }
}

/// Deletes an existing Publisher object.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_delete_publisher(
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
pub unsafe extern "C" fn dds_domain_participant_create_subscriber(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    qos: *const SubscriberQos,
    listener: *const DustDdsSubscriberListener,
    mask: DustDdsStatusMask,
) -> Option<NonNull<DustDdsSubscriber>> {
    let Some(participant) = participant else {
        return None;
    };

    let qos = if qos.is_null() {
        QosKind::Default
    } else {
        QosKind::Specific(unsafe { &*qos }.clone().into())
    };

    let status_kinds = crate::infrastructure::condition::mask_to_status_kinds(mask);

    let result = if listener.is_null() {
        unsafe { participant.as_ref() }.inner().create_subscriber(
            qos,
            None::<CSubscriberListenerWrapper>,
            &status_kinds,
        )
    } else {
        let wrapper = CSubscriberListenerWrapper {
            listener: unsafe { *listener },
        };
        unsafe { participant.as_ref() }
            .inner()
            .create_subscriber(qos, Some(wrapper), &status_kinds)
    };

    match result {
        Ok(subscriber) => NonNull::new(Box::into_raw(Box::new(DustDdsSubscriber::new(subscriber)))),
        Err(_) => None,
    }
}

/// Deletes an existing Subscriber object.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_delete_subscriber(
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

/// Returns the builtin Subscriber.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_get_builtin_subscriber(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
) -> Option<NonNull<DustDdsSubscriber>> {
    let Some(participant) = participant else {
        return None;
    };
    let sub = unsafe { participant.as_ref() }
        .inner()
        .get_builtin_subscriber();
    NonNull::new(Box::into_raw(Box::new(DustDdsSubscriber::new(sub))))
}

/// Creates a new Topic object.
/// Passing NULL (`DUST_DDS_TOPIC_QOS_DEFAULT`) for `qos` represents the default QoS.
/// Returns a raw pointer to DustDdsTopic on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_create_topic(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    topic_name: *const std::os::raw::c_char,
    type_name: *const std::os::raw::c_char,
    qos: *const TopicQos,
    listener: *const DustDdsTopicListener,
    mask: DustDdsStatusMask,
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

    let qos = if qos.is_null() {
        QosKind::Default
    } else {
        QosKind::Specific(unsafe { &*qos }.clone().into())
    };

    let status_kinds = crate::infrastructure::condition::mask_to_status_kinds(mask);
    let participant_ref = unsafe { participant.as_ref() };
    let dynamic_type_ref = unsafe { dynamic_type.as_ref() };

    let result = if listener.is_null() {
        participant_ref.inner().create_dynamic_topic(
            topic_name_str,
            type_name_str,
            qos,
            None::<CTopicListenerWrapper>,
            &status_kinds,
            dynamic_type_ref.inner().clone(),
        )
    } else {
        let wrapper = CTopicListenerWrapper {
            listener: unsafe { *listener },
        };
        participant_ref.inner().create_dynamic_topic(
            topic_name_str,
            type_name_str,
            qos,
            Some(wrapper),
            &status_kinds,
            dynamic_type_ref.inner().clone(),
        )
    };

    match result {
        Ok(topic) => NonNull::new(Box::into_raw(Box::new(DustDdsTopic::new(topic)))),
        Err(_) => None,
    }
}

/// Deletes an existing Topic object.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_delete_topic(
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

/// Finds a topic.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_find_topic(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    topic_name: *const std::os::raw::c_char,
    timeout: Duration_t,
) -> Option<NonNull<DustDdsTopic>> {
    let Some(participant) = participant else {
        return None;
    };
    if topic_name.is_null() {
        return None;
    }
    let topic_name_str = unsafe { std::ffi::CStr::from_ptr(topic_name) }
        .to_str()
        .ok()?;

    let rust_timeout = timeout.into();
    match unsafe { participant.as_ref() }
        .inner()
        .find_topic::<dust_dds::xtypes::dynamic_type::DynamicData<'static>>(
            topic_name_str,
            rust_timeout,
        ) {
        Ok(topic) => NonNull::new(Box::into_raw(Box::new(DustDdsTopic::new(topic)))),
        Err(_) => None,
    }
}

/// Looks up a topic description.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_lookup_topicdescription(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    name: *const std::os::raw::c_char,
) -> Option<NonNull<DustDdsTopicDescription>> {
    let Some(participant) = participant else {
        return None;
    };
    if name.is_null() {
        return None;
    }
    let name_str = unsafe { std::ffi::CStr::from_ptr(name) }.to_str().ok()?;

    match unsafe { participant.as_ref() }
        .inner()
        .lookup_topicdescription(name_str)
    {
        Ok(Some(td)) => NonNull::new(Box::into_raw(Box::new(DustDdsTopicDescription::new(
            Box::new(td),
        )))),
        _ => None,
    }
}

/// Creates a new ContentFilteredTopic.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_create_contentfilteredtopic(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    name: *const std::os::raw::c_char,
    related_topic: Option<NonNull<DustDdsTopic>>,
    filter_expression: *const std::os::raw::c_char,
    expression_parameters: *const StringSeq,
) -> Option<NonNull<DustDdsContentFilteredTopic>> {
    let Some(participant) = participant else {
        return None;
    };
    if name.is_null() || filter_expression.is_null() {
        return None;
    }
    let Some(related_topic) = related_topic else {
        return None;
    };

    let name_str = unsafe { std::ffi::CStr::from_ptr(name) }.to_str().ok()?;
    let filter_expression_str = unsafe { std::ffi::CStr::from_ptr(filter_expression) }
        .to_str()
        .ok()?;

    let expression_parameters_vec = if expression_parameters.is_null() {
        Vec::new()
    } else {
        unsafe { (*expression_parameters).to_vec() }
    };

    let participant_ref = unsafe { participant.as_ref() };
    let related_topic_ref = unsafe { related_topic.as_ref() };

    match participant_ref.inner().create_contentfilteredtopic(
        name_str,
        related_topic_ref.inner(),
        filter_expression_str.to_string(),
        expression_parameters_vec,
    ) {
        Ok(cft) => NonNull::new(Box::into_raw(Box::new(DustDdsContentFilteredTopic::new(
            cft,
        )))),
        Err(_) => None,
    }
}

/// Deletes a ContentFilteredTopic.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_delete_contentfilteredtopic(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    contentfilteredtopic: Option<NonNull<DustDdsContentFilteredTopic>>,
) -> ReturnCode {
    let Some(contentfilteredtopic) = contentfilteredtopic else {
        return RETCODE_OK;
    };
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };

    let participant_ref = unsafe { participant.as_ref() };
    let contentfilteredtopic_ref = unsafe { contentfilteredtopic.as_ref() };

    match participant_ref
        .inner()
        .delete_contentfilteredtopic(contentfilteredtopic_ref.inner())
    {
        Ok(()) => {
            unsafe {
                drop(Box::from_raw(contentfilteredtopic.as_ptr()));
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Deletes all contained entities.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_delete_contained_entities(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { participant.as_ref() }
        .inner()
        .delete_contained_entities()
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Sets the QoS policies of the DomainParticipant.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_set_qos(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    qos: *const DomainParticipantQos,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    let qos = if qos.is_null() {
        QosKind::Default
    } else {
        QosKind::Specific(unsafe { &*qos }.clone().into())
    };
    match unsafe { participant.as_ref() }.inner().set_qos(qos) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the QoS policies of the DomainParticipant.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_get_qos(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    qos: *mut DomainParticipantQos,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if qos.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { participant.as_ref() }.inner().get_qos() {
        Ok(q) => {
            unsafe { *qos = q.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Sets the DomainParticipantListener and StatusMask.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_set_listener(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    listener: *const DustDdsDomainParticipantListener,
    mask: DustDdsStatusMask,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    let status_kinds = crate::infrastructure::condition::mask_to_status_kinds(mask);
    let result = if listener.is_null() {
        unsafe { participant.as_ref() }
            .inner()
            .set_listener(None::<CDomainParticipantListenerWrapper>, &status_kinds)
    } else {
        let wrapper = CDomainParticipantListenerWrapper {
            listener: unsafe { *listener },
        };
        unsafe { participant.as_ref() }
            .inner()
            .set_listener(Some(wrapper), &status_kinds)
    };
    match result {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the DomainParticipantListener.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_get_listener(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
) -> DustDdsDomainParticipantListener {
    let _ = participant;
    DustDdsDomainParticipantListener {
        listener_data: std::ptr::null_mut(),
        on_inconsistent_topic: None,
        on_liveliness_lost: None,
        on_offered_deadline_missed: None,
        on_offered_incompatible_qos: None,
        on_sample_lost: None,
        on_data_available: None,
        on_sample_rejected: None,
        on_liveliness_changed: None,
        on_requested_deadline_missed: None,
        on_requested_incompatible_qos: None,
        on_publication_matched: None,
        on_subscription_matched: None,
    }
}

/// Ignores a participant.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_ignore_participant(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    handle: *const InstanceHandle_t,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let rust_handle = dust_dds::infrastructure::instance::InstanceHandle::new(unsafe { *handle });
    match unsafe { participant.as_ref() }
        .inner()
        .ignore_participant(rust_handle)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Ignores a topic.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_ignore_topic(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    handle: *const InstanceHandle_t,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let rust_handle = dust_dds::infrastructure::instance::InstanceHandle::new(unsafe { *handle });
    match unsafe { participant.as_ref() }
        .inner()
        .ignore_topic(rust_handle)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Ignores a publication.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_ignore_publication(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    handle: *const InstanceHandle_t,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let rust_handle = dust_dds::infrastructure::instance::InstanceHandle::new(unsafe { *handle });
    match unsafe { participant.as_ref() }
        .inner()
        .ignore_publication(rust_handle)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Ignores a subscription.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_ignore_subscription(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    handle: *const InstanceHandle_t,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let rust_handle = dust_dds::infrastructure::instance::InstanceHandle::new(unsafe { *handle });
    match unsafe { participant.as_ref() }
        .inner()
        .ignore_subscription(rust_handle)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the domain ID.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_get_domain_id(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
) -> i32 {
    let Some(participant) = participant else {
        return -1;
    };
    unsafe { participant.as_ref() }.inner().get_domain_id()
}

/// Gets the instance handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_get_instance_handle(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    handle: *mut InstanceHandle_t,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    unsafe {
        *handle = participant.as_ref().inner().get_instance_handle().into();
    }
    RETCODE_OK
}

/// Asserts liveliness.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_assert_liveliness(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { participant.as_ref() }.inner().assert_liveliness() {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Sets the default Publisher QoS policies.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_set_default_publisher_qos(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    qos: *const PublisherQos,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    let qos = if qos.is_null() {
        QosKind::Default
    } else {
        QosKind::Specific(unsafe { &*qos }.clone().into())
    };
    match unsafe { participant.as_ref() }
        .inner()
        .set_default_publisher_qos(qos)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the default Publisher QoS policies.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_get_default_publisher_qos(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    qos: *mut PublisherQos,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if qos.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { participant.as_ref() }
        .inner()
        .get_default_publisher_qos()
    {
        Ok(q) => {
            unsafe { *qos = q.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Sets the default Subscriber QoS policies.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_set_default_subscriber_qos(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    qos: *const SubscriberQos,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    let qos = if qos.is_null() {
        QosKind::Default
    } else {
        QosKind::Specific(unsafe { &*qos }.clone().into())
    };
    match unsafe { participant.as_ref() }
        .inner()
        .set_default_subscriber_qos(qos)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the default Subscriber QoS policies.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_get_default_subscriber_qos(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    qos: *mut SubscriberQos,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if qos.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { participant.as_ref() }
        .inner()
        .get_default_subscriber_qos()
    {
        Ok(q) => {
            unsafe { *qos = q.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Sets the default Topic QoS policies.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_set_default_topic_qos(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    qos: *const TopicQos,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    let qos = if qos.is_null() {
        QosKind::Default
    } else {
        QosKind::Specific(unsafe { &*qos }.clone().into())
    };
    match unsafe { participant.as_ref() }
        .inner()
        .set_default_topic_qos(qos)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the default Topic QoS policies.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_get_default_topic_qos(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    qos: *mut TopicQos,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if qos.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { participant.as_ref() }
        .inner()
        .get_default_topic_qos()
    {
        Ok(q) => {
            unsafe { *qos = q.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Gets the discovered participants.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_get_discovered_participants(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    participant_handles: *mut InstanceHandleSeq,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if participant_handles.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { participant.as_ref() }
        .inner()
        .get_discovered_participants()
    {
        Ok(handles) => {
            unsafe { *participant_handles = InstanceHandleSeq::from_vec(&handles) };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Gets the data of a discovered participant.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_get_discovered_participant_data(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    participant_data: *mut ParticipantBuiltinTopicData,
    participant_handle: *const InstanceHandle_t,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if participant_data.is_null() || participant_handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let handle =
        dust_dds::infrastructure::instance::InstanceHandle::new(unsafe { *participant_handle });
    match unsafe { participant.as_ref() }
        .inner()
        .get_discovered_participant_data(handle)
    {
        Ok(data) => {
            unsafe {
                (*participant_data).key = BuiltInTopicKey {
                    value: data.key().value,
                };
                (*participant_data).user_data = data.user_data().clone().into();
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Gets the discovered topics.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_get_discovered_topics(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    topic_handles: *mut InstanceHandleSeq,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if topic_handles.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { participant.as_ref() }
        .inner()
        .get_discovered_topics()
    {
        Ok(handles) => {
            unsafe { *topic_handles = InstanceHandleSeq::from_vec(&handles) };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Gets the data of a discovered topic.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_get_discovered_topic_data(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    topic_data: *mut TopicBuiltinTopicData,
    topic_handle: *const InstanceHandle_t,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if topic_data.is_null() || topic_handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let handle = dust_dds::infrastructure::instance::InstanceHandle::new(unsafe { *topic_handle });
    match unsafe { participant.as_ref() }
        .inner()
        .get_discovered_topic_data(handle)
    {
        Ok(data) => {
            let name_c = match std::ffi::CString::new(data.name()) {
                Ok(c) => c.into_raw(),
                Err(_) => return RETCODE_ERROR,
            };
            let type_name_c = match std::ffi::CString::new(data.get_type_name()) {
                Ok(c) => c.into_raw(),
                Err(_) => {
                    unsafe {
                        let _ = std::ffi::CString::from_raw(name_c);
                    }
                    return RETCODE_ERROR;
                }
            };
            unsafe {
                (*topic_data).key = BuiltInTopicKey {
                    value: data.key().value,
                };
                (*topic_data).name = name_c;
                (*topic_data).type_name = type_name_c;
                (*topic_data).durability = data.durability().clone().into();
                (*topic_data).deadline = data.deadline().clone().into();
                (*topic_data).latency_budget = data.latency_budget().clone().into();
                (*topic_data).liveliness = data.liveliness().clone().into();
                (*topic_data).reliability = data.reliability().clone().into();
                (*topic_data).transport_priority = data.transport_priority().clone().into();
                (*topic_data).lifespan = data.lifespan().clone().into();
                (*topic_data).destination_order = data.destination_order().clone().into();
                (*topic_data).history = data.history().clone().into();
                (*topic_data).resource_limits = data.resource_limits().clone().into();
                (*topic_data).ownership = data.ownership().clone().into();
                (*topic_data).topic_data = data.topic_data().clone().into();
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Checks whether the participant contains the entity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_contains_entity(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    a_handle: *const InstanceHandle_t,
) -> bool {
    let Some(participant) = participant else {
        return false;
    };
    if a_handle.is_null() {
        return false;
    }
    let handle = dust_dds::infrastructure::instance::InstanceHandle::new(unsafe { *a_handle });
    match unsafe { participant.as_ref() }
        .inner()
        .contains_entity(handle)
    {
        Ok(res) => res,
        Err(_) => false,
    }
}

/// Gets the current time.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_domain_participant_get_current_time(
    participant: Option<NonNull<DustDdsDomainParticipant>>,
    current_time: *mut Time_t,
) -> ReturnCode {
    let Some(participant) = participant else {
        return RETCODE_BAD_PARAMETER;
    };
    if current_time.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { participant.as_ref() }.inner().get_current_time() {
        Ok(t) => {
            unsafe { *current_time = t.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::domain_participant_factory::{
        dds_domain_participant_factory_create_participant,
        dds_domain_participant_factory_delete_participant,
        dds_domain_participant_factory_get_instance,
    };
    use crate::infrastructure::qos::{dds_publisher_qos_default, dds_subscriber_qos_default};

    #[test]
    fn create_delete_publisher() {
        let factory = unsafe { dds_domain_participant_factory_get_instance() };
        let participant = unsafe {
            dds_domain_participant_factory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                std::ptr::null(),
                std::ptr::null(),
                0,
            )
        };
        assert!(participant.is_some());

        let publisher = unsafe {
            dds_domain_participant_create_publisher(
                participant,
                std::ptr::null(),
                std::ptr::null(),
                0,
            )
        };
        assert!(publisher.is_some());

        let res = unsafe { dds_domain_participant_delete_publisher(participant, publisher) };
        assert_eq!(res, RETCODE_OK);

        let qos = unsafe { dds_publisher_qos_default() };
        let publisher = unsafe {
            dds_domain_participant_create_publisher(participant, &qos, std::ptr::null(), 0)
        };
        assert!(publisher.is_some());

        let res = unsafe { dds_domain_participant_delete_publisher(participant, publisher) };
        assert_eq!(res, RETCODE_OK);

        unsafe {
            dds_domain_participant_factory_delete_participant(
                NonNull::new(factory as *mut _),
                participant,
            );
        }
    }

    #[test]
    fn create_delete_subscriber() {
        let factory = unsafe { dds_domain_participant_factory_get_instance() };
        let participant = unsafe {
            dds_domain_participant_factory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                std::ptr::null(),
                std::ptr::null(),
                0,
            )
        };
        assert!(participant.is_some());

        let subscriber = unsafe {
            dds_domain_participant_create_subscriber(
                participant,
                std::ptr::null(),
                std::ptr::null(),
                0,
            )
        };
        assert!(subscriber.is_some());

        let res = unsafe { dds_domain_participant_delete_subscriber(participant, subscriber) };
        assert_eq!(res, RETCODE_OK);

        let qos = unsafe { dds_subscriber_qos_default() };
        let subscriber = unsafe {
            dds_domain_participant_create_subscriber(participant, &qos, std::ptr::null(), 0)
        };
        assert!(subscriber.is_some());

        let res = unsafe { dds_domain_participant_delete_subscriber(participant, subscriber) };
        assert_eq!(res, RETCODE_OK);

        unsafe {
            dds_domain_participant_factory_delete_participant(
                NonNull::new(factory as *mut _),
                participant,
            );
        }
    }

    #[test]
    fn create_delete_topic() {
        use crate::topic_definition::dynamic_type::{
            DustDdsMemberDescriptor, TYPE_KIND_INT32, dds_dynamic_type_builder_add_member,
            dds_dynamic_type_builder_build, dds_dynamic_type_builder_create_struct,
            dds_dynamic_type_free, dds_dynamic_type_get_primitive_type,
        };

        let factory = unsafe { dds_domain_participant_factory_get_instance() };
        let participant = unsafe {
            dds_domain_participant_factory_create_participant(
                NonNull::new(factory as *mut _),
                0,
                std::ptr::null(),
                std::ptr::null(),
                0,
            )
        };
        assert!(participant.is_some());

        let struct_name = std::ffi::CString::new("MyStruct").unwrap();
        let builder = unsafe { dds_dynamic_type_builder_create_struct(struct_name.as_ptr()) };
        assert!(builder.is_some());

        let field_name = std::ffi::CString::new("a").unwrap();
        let int32_type = unsafe { dds_dynamic_type_get_primitive_type(TYPE_KIND_INT32) };
        assert!(int32_type.is_some());

        let member_descriptor = DustDdsMemberDescriptor {
            name: field_name.as_ptr(),
            id: 0,
            r#type: int32_type.unwrap().as_ptr(),
            is_key: false,
            is_optional: false,
            is_must_understand: true,
        };

        let res = unsafe { dds_dynamic_type_builder_add_member(builder, &member_descriptor) };
        assert_eq!(res, RETCODE_OK);

        let dynamic_type = unsafe { dds_dynamic_type_builder_build(builder) };
        assert!(dynamic_type.is_some());

        let topic_name = std::ffi::CString::new("MyTopic").unwrap();
        let type_name = std::ffi::CString::new("MyStruct").unwrap();

        let topic = unsafe {
            dds_domain_participant_create_topic(
                participant,
                topic_name.as_ptr(),
                type_name.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                dynamic_type,
            )
        };
        assert!(topic.is_some());

        let res = unsafe { dds_domain_participant_delete_topic(participant, topic) };
        assert_eq!(res, RETCODE_OK);

        unsafe {
            dds_dynamic_type_free(dynamic_type);
            dds_domain_participant_factory_delete_participant(
                NonNull::new(factory as *mut _),
                participant,
            );
        }
    }
}
