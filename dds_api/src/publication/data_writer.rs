use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps_psm::{
        Duration, InstanceHandle, LivelinessLostStatus, OfferedDeadlineMissedStatus,
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, Time,
    },
    return_type::DDSResult,
    topic::topic::Topic,
};

use super::publisher::Publisher;

pub trait DataWriter<T> {
    /// This operation informs the Service that the application will be modifying a particular instance. It gives an opportunity to the
    /// Service to pre-configure itself to improve performance.
    /// It takes as a parameter an instance (to get the key value) and returns a handle that can be used in successive write or dispose
    /// operations.
    /// This operation should be invoked prior to calling any operation that modifies the instance, such as write, write_w_timestamp,
    /// dispose and dispose_w_timestamp.
    /// The special value HANDLE_NIL may be returned by the Service if it does not want to allocate any handle for that instance.
    /// This operation may block and return TIMEOUT under the same circumstances described for the write operation (2.2.2.4.2.11).
    /// This operation may return OUT_OF_RESOURCES under the same circumstances described for the write operation
    /// (2.2.2.4.2.11).
    /// The operation register_instance is idempotent. If it is called for an already registered instance, it just returns the already
    /// allocated handle. This may be used to lookup and retrieve the handle allocated to a given instance. The explicit use of this
    /// operation is optional as the application may call directly the write operation and specify a HANDLE_NIL to indicate that the
    /// ‘key’ should be examined to identify the instance.
    fn register_instance(&mut self, instance: T) -> DDSResult<Option<InstanceHandle>>;

    /// This operation performs the same function as register_instance and can be used instead of register_instance in the cases
    /// where the application desires to specify the value for the source_timestamp. The source_timestamp potentially affects the
    /// relative order in which readers observe events from multiple writers. For details see 2.2.3.17 for the QoS policy
    /// DESTINATION_ORDER).
    /// This operation may block and return TIMEOUT under the same circumstances described for the write operation (2.2.2.4.2.11).
    /// This operation may return OUT_OF_RESOURCES under the same circumstances described for the write operation
    /// (2.2.2.4.2.11).
    fn register_instance_w_timestamp(
        &mut self,
        instance: T,
        timestamp: Time,
    ) -> DDSResult<Option<InstanceHandle>>;

    /// This operation reverses the action of register_instance. It should only be called on an instance that is currently registered.
    /// The operation unregister_instance should be called just once per instance, regardless of how many times register_instance
    /// was called for that instance.
    /// This operation informs the Service that the DataWriter is not intending to modify any more of that data instance. This
    /// operation also indicates that the Service can locally remove all information regarding that instance. The application should not
    /// attempt to use the handle previously allocated to that instance after calling unregister_instance.
    /// The special value HANDLE_NIL can be used for the parameter handle. This indicates that the identity of the instance should
    /// be automatically deduced from the instance (by means of the key).
    /// If handle is any value other than HANDLE_NIL, then it must correspond to the value returned by register_instance when the
    /// instance (identified by its key) was registered. Otherwise the behavior is as follows:
    /// • If the handle corresponds to an existing instance but does not correspond to the same instance referred by the 'instance'
    /// parameter, the behavior is in general unspecified, but if detectable by the Service implementation, the operation shall
    /// fail and return the error-code PRECONDITION_NOT_MET.'
    /// • If the handle does not correspond to an existing instance the behavior is in general unspecified, but if detectable by the
    /// Service implementation, the operation shall fail and return the error-code 'BAD_PARAMETER.'
    /// If after that, the application wants to modify (write or dispose) the instance, it has to register it again, or else use the special
    /// handle value HANDLE_NIL.
    /// This operation does not indicate that the instance is deleted (that is the purpose of dispose). The operation unregister_instance
    /// just indicates that the DataWriter no longer has ‘anything to say’ about the instance. DataReader entities that are reading the
    /// instance will eventually receive a sample with a NOT_ALIVE_NO_WRITERS instance state if no other DataWriter entities
    /// are writing the instance.
    /// This operation can affect the ownership of the data instance (as described in 2.2.3.9, OWNERSHIP and 2.2.3.23.1, Ownership
    /// resolution on redundant systems). If the DataWriter was the exclusive owner of the instance, then calling unregister_instance
    /// will relinquish that ownership.
    /// This operation may block and return TIMEOUT under the same circumstances described for the write operation (2.2.2.4.2.11,
    /// write).
    /// Possible error codes returned in addition to the standard ones: TIMEOUT, PRECONDITION_NOT_MET.
    fn unregister_instance(&mut self, instance: T, handle: Option<InstanceHandle>)
        -> DDSResult<()>;

    /// This operation performs the same function as unregister_instance and can be used instead of unregister_instance in the cases
    /// where the application desires to specify the value for the source_timestamp. The source_timestamp potentially affects the
    /// relative order in which readers observe events from multiple writers. For details see 2.2.3.17 for the QoS policy
    /// DESTINATION_ORDER).
    /// The constraints on the values of the handle parameter and the corresponding error behavior are the same specified for the
    /// unregister_instance operation (2.2.2.4.2.7).
    /// This operation may block and return TIMEOUT under the same circumstances described for the write operation (2.2.2.4.2.11).
    fn unregister_instance_w_timestamp(
        &mut self,
        instance: T,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DDSResult<()>;

    /// This operation can be used to retrieve the instance key that corresponds to an instance_handle. The operation will only fill the
    /// fields that form the key inside the key_holder instance.
    /// This operation may return BAD_PARAMETER if the InstanceHandle_t a_handle does not correspond to an existing dataobject
    /// known to the DataWriter. If the implementation is not able to check invalid handles, then the result in this situation is
    /// unspecified.
    fn get_key_value(&self, key_holder: &mut T, handle: InstanceHandle) -> DDSResult<()>;

    /// This operation takes as a parameter an instance and returns a handle that can be used in subsequent operations that accept an
    /// instance handle as an argument. The instance parameter is only used for the purpose of examining the fields that define the
    /// key.
    /// This operation does not register the instance in question. If the instance has not been previously registered, or if for any other
    /// reason the Service is unable to provide an instance handle, the Service will return the special value HANDLE_NIL.
    fn lookup_instance(&self, instance: &T) -> DDSResult<Option<InstanceHandle>>;

    /// This operation modifies the value of a data instance. When this operation is used, the Service will automatically supply the
    /// value of the source_timestamp that is made available to DataReader objects by means of the source_timestamp attribute
    /// inside the SampleInfo. See 2.2.2.5, Subscription Module for more details on data timestamps at reader side and 2.2.3.17 for
    /// the QoS policy DESTINATION_ORDER.
    /// This operation must be provided on the specialized class that is generated for the particular application data-type that is being
    /// written. That way the data argument holding the data has the proper application-defined type (e.g., ‘Foo’).
    /// As a side effect, this operation asserts liveliness on the DataWriter itself, the Publisher and the DomainParticipant.
    /// The special value HANDLE_NIL can be used for the parameter handle. This indicates that the identity of the instance should
    /// be automatically deduced from the instance_data (by means of the key).
    /// If handle is any value other than HANDLE_NIL, then it must correspond to the value returned by register_instance when the
    /// instance (identified by its key) was registered. Otherwise the behavior is as follows:
    /// • If the handle corresponds to an existing instance but does not correspond to the same instance referred by the ‘data’
    /// parameter, the behavior is in general unspecified, but if detectable by the Service implementation, the operation shall
    /// fail and return the error-code PRECONDITION_NOT_MET.
    /// • If the handle does not correspond to an existing instance the behavior is in general unspecified, but if detectable by the
    /// Service implementation, the operation shall fail and return the error-code BAD_PARAMETER.
    /// If the RELIABILITY kind is set to RELIABLE, the write operation may block if the modification would cause data to be lost
    /// or else cause one of the limits specified in the RESOURCE_LIMITS to be exceeded. Under these circumstances, the
    /// RELIABILITY max_blocking_time configures the maximum time the write operation may block waiting for space to become
    /// available. If max_blocking_time elapses before the DataWriter is able to store the modification without exceeding the limits,
    /// the write operation will fail and return TIMEOUT.
    /// Specifically, the DataWriter write operation may block in the following situations (note that the list may not be exhaustive),
    /// even if its HISTORY kind is KEEP_LAST.
    /// • If (RESOURCE_LIMITS max_samples < RESOURCE_LIMITS max_instances * HISTORY depth), then in the
    /// situation where the max_samples resource limit is exhausted the Service is allowed to discard samples of some other
    /// instance as long as at least one sample remains for such an instance. If it is still not possible to make space available to
    /// store the modification, the writer is allowed to block.
    /// • If (RESOURCE_LIMITS max_samples < RESOURCE_LIMITS max_instances), then the DataWriter may block
    /// regardless of the HISTORY depth.
    /// Instead of blocking, the write operation is allowed to return immediately with the error code OUT_OF_RESOURCES
    /// provided the following two conditions are met:
    /// 1. The reason for blocking would be that the RESOURCE_LIMITS are exceeded.
    /// 2. The service determines that waiting the max_waiting_time has no chance of freeing the necessary resources. For
    /// example, if the only way to gain the necessary resources would be for the user to unregister an instance.
    /// In case the provided handle is valid, i.e., corresponds to an existing instance, but does not correspond to same instance referred
    /// by the ‘data’ parameter, the behavior is in general unspecified, but if detectable by the Service implementation, the return
    /// error-code will be PRECONDITION_NOT_MET. In case the handle is invalid, the behavior is in general unspecified, but if
    /// detectable the returned error-code will be BAD_PARAMETER.
    fn write(&mut self, data: &T, handle: Option<InstanceHandle>) -> DDSResult<()>;

    /// This operation performs the same function as write except that it also provides the value for the source_timestamp that is made
    /// available to DataReader objects by means of the source_timestamp attribute inside the SampleInfo. See 2.2.2.5, Subscription
    /// Module for more details on data timestamps at reader side and 2.2.3.17 for the QoS policy DESTINATION_ORDER.
    /// The constraints on the values of the handle parameter and the corresponding error behavior are the same specified for the write
    /// operation (2.2.2.4.2.11).
    /// This operation may block and return TIMEOUT under the same circumstances described for the write operation (2.2.2.4.2.11).
    /// This operation may return OUT_OF_RESOURCES under the same circumstances described for the write operation
    /// (2.2.2.4.2.11).
    /// This operation may return PRECONDITION_NOT_MET under the same circumstances described for the write operation
    /// (2.2.2.4.2.11).
    /// This operation may return BAD_PARAMETER under the same circumstances described for the write operation (2.2.2.4.2.11).
    /// Similar to write, this operation must also be provided on the specialized class that is generated for the particular application
    /// data-type that is being written.
    fn write_w_timestamp(
        &mut self,
        data: &T,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DDSResult<()>;

    /// This operation requests the middleware to delete the data (the actual deletion is postponed until there is no more use for that
    /// data in the whole system). In general, applications are made aware of the deletion by means of operations on the DataReader
    /// objects that already knew that instance13 (see 2.2.2.5, Subscription Module for more details).
    /// This operation does not modify the value of the instance. The instance parameter is passed just for the purposes of identifying
    /// the instance.
    /// When this operation is used, the Service will automatically supply the value of the source_timestamp that is made available to
    /// DataReader objects by means of the source_timestamp attribute inside the SampleInfo.
    /// The constraints on the values of the handle parameter and the corresponding error behavior are the same specified for the
    /// unregister_instance operation (2.2.2.4.2.7).
    /// This operation may block and return TIMEOUT under the same circumstances described for the write operation (2.2.2.4.2.11).
    /// This operation may return OUT_OF_RESOURCES under the same circumstances described for the write operation
    /// (2.2.2.4.2.11).
    fn dispose(&mut self, data: T, handle: Option<InstanceHandle>) -> DDSResult<()>;

    /// This operation performs the same functions as dispose except that the application provides the value for the source_timestamp
    /// that is made available to DataReader objects by means of the source_timestamp attribute inside the SampleInfo (see 2.2.2.5,
    /// Subscription Module).
    /// The constraints on the values of the handle parameter and the corresponding error behavior are the same specified for the
    /// dispose operation (2.2.2.4.2.13).
    /// This operation may return PRECONDITION_NOT_MET under the same circumstances described for the dispose operation
    /// (2.2.2.4.2.13).
    /// This operation may return BAD_PARAMETER under the same circumstances described for the dispose operation
    /// (2.2.2.4.2.13).
    /// This operation may block and return TIMEOUT under the same circumstances described for the write operation (2.2.2.4.2.11).
    /// This operation may return OUT_OF_RESOURCES under the same circumstances described for the write operation
    /// (2.2.2.4.2.11).
    /// Possible error codes returned in addition to the standard ones: TIMEOUT, PRECONDITION_NOT_MET.
    fn dispose_w_timestamp(
        &mut self,
        data: T,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DDSResult<()>;

    /// This operation is intended to be used only if the DataWriter has RELIABILITY QoS kind set to RELIABLE. Otherwise the
    /// operation will return immediately with RETCODE_OK.
    /// The operation wait_for_acknowledgments blocks the calling thread until either all data written by the DataWriter is
    /// acknowledged by all matched DataReader entities that have RELIABILITY QoS kind RELIABLE, or else the duration
    /// specified by the max_wait parameter elapses, whichever happens first. A return value of OK indicates that all the samples
    /// written have been acknowledged by all reliable matched data readers; a return value of TIMEOUT indicates that max_wait
    /// elapsed before all the data was acknowledged.
    fn wait_for_acknowledgments(&self, max_wait: Duration) -> DDSResult<()>;

    /// This operation allows access to the LIVELINESS_LOST communication status. Communication statuses are described in
    /// 2.2.4.1, Communication Status.
    fn get_liveliness_lost_status(&self, status: &mut LivelinessLostStatus) -> DDSResult<()>;

    /// This operation allows access to the OFFERED_DEADLINE_MISSED communication status. Communication statuses are
    /// described in 2.2.4.1, Communication Status.
    fn get_offered_deadline_missed_status(
        &self,
        status: &mut OfferedDeadlineMissedStatus,
    ) -> DDSResult<()>;

    /// This operation allows access to the OFFERED_INCOMPATIBLE_QOS communication status. Communication statuses are
    /// described in 2.2.4.1, Communication Status.
    fn get_offered_incompatible_qos_status(
        &self,
        status: &mut OfferedIncompatibleQosStatus,
    ) -> DDSResult<()>;

    /// This operation allows access to the PUBLICATION_MATCHED communication status. Communication statuses are
    /// described in 2.2.4.1, Communication Status.
    fn get_publication_matched_status(
        &self,
        status: &mut PublicationMatchedStatus,
    ) -> DDSResult<()>;

    /// This operation returns the Topic associated with the DataWriter. This is the same Topic that was used to create the DataWriter.
    fn get_topic(&self) -> &dyn Topic<T>;

    /// This operation returns the Publisher to which the data writer object belongs.
    fn get_publisher(&self) -> &dyn Publisher;

    /// This operation manually asserts the liveliness of the DataWriter. This is used in combination with the LIVELINESS QoS
    /// policy (see 2.2.3, Supported QoS) to indicate to the Service that the entity remains active.
    /// This operation need only be used if the LIVELINESS setting is either MANUAL_BY_PARTICIPANT or
    /// MANUAL_BY_TOPIC. Otherwise, it has no effect.
    /// NOTE: Writing data via the write operation on a DataWriter asserts liveliness on the DataWriter itself and its
    /// DomainParticipant. Consequently the use of assert_liveliness is only needed if the application is not writing data regularly.
    /// Complete details are provided in 2.2.3.11, LIVELINESS.
    fn assert_liveliness(&self) -> DDSResult<()>;

    /// This operation retrieves information on a subscription that is currently “associated” with the DataWriter; that is, a subscription
    /// with a matching Topic and compatible QoS that the application has not indicated should be “ignored” by means of the
    /// DomainParticipant ignore_subscription operation.
    /// The subscription_handle must correspond to a subscription currently associated with the DataWriter, otherwise the operation
    /// will fail and return BAD_PARAMETER. The operation get_matched_subscriptions can be used to find the subscriptions that
    /// are currently matched with the DataWriter.
    /// The operation may also fail if the infrastructure does not hold the information necessary to fill in the subscription_data. In this
    /// case the operation will return UNSUPPORTED.
    fn get_matched_subscription_data(
        &self,
        subscription_data: SubscriptionBuiltinTopicData,
        subscription_handle: InstanceHandle,
    ) -> DDSResult<()>;

    /// This operation retrieves the list of subscriptions currently “associated” with the DataWriter; that is, subscriptions that have a
    /// matching Topic and compatible QoS that the application has not indicated should be “ignored” by means of the
    /// DomainParticipant ignore_subscription operation.
    /// The handles returned in the ‘subscription_handles’ list are the ones that are used by the DDS implementation to locally
    /// identify the corresponding matched DataReader entities. These handles match the ones that appear in the ‘instance_handle’
    /// field of the SampleInfo when reading the “DCPSSubscriptions” builtin topic.
    /// The operation may fail if the infrastructure does not locally maintain the connectivity information.
    fn get_matched_subscriptions(
        &self,
        _subscription_handles: &mut [InstanceHandle],
    ) -> DDSResult<()>;
}

pub trait AnyDataWriter {}
