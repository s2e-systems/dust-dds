use crate::{
    dds_type::{DdsDeserialize, DdsType},
    domain::domain_participant_factory::THE_PARTICIPANT_FACTORY,
    implementation::{
        dds_impl::data_reader_impl::{AnyDataReaderListener, DataReaderImpl},
        utils::{shared_object::DdsWeak, timer::ThreadTimer},
    },
    infrastructure::{instance::InstanceHandle, status::StatusKind},
};
use crate::{
    subscription::data_reader_listener::DataReaderListener,
    {
        builtin_topics::PublicationBuiltinTopicData,
        infrastructure::{
            entity::{Entity, StatusCondition},
            error::DdsResult,
            qos::DataReaderQos,
            status::{
                LivelinessChangedStatus, RequestedDeadlineMissedStatus,
                RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
                SubscriptionMatchedStatus,
            },
        },
    },
};

use std::marker::PhantomData;

use crate::topic_definition::topic::Topic;

use super::{
    sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
    subscriber::Subscriber,
};

pub struct Sample<Foo> {
    pub data: Option<Foo>,
    pub sample_info: SampleInfo,
}

/// A DataReader allows the application (1) to declare the data it wishes to receive (i.e., make a subscription) and (2) to access the
/// data received by the attached Subscriber.
///
/// A DataReader refers to exactly one TopicDescription (either a Topic, a ContentFilteredTopic, or a MultiTopic) that identifies
/// the data to be read. The subscription has a unique resulting type. The data-reader may give access to several instances of the
/// resulting type, which can be distinguished from each other by their key (as described in 2.2.1.2.2, Overall Conceptual Model).
///
/// All operations except for the base-class operations set_qos, get_qos, set_listener, get_listener, enable, and
/// get_statuscondition may return the error NOT_ENABLED.
/// All sample-accessing operations, namely all variants of read, take may return the error PRECONDITION_NOT_MET. The
/// circumstances that result on this are described in 2.2.2.5.2.8.
pub struct DataReader<Foo> {
    data_reader_attributes: DdsWeak<DataReaderImpl<ThreadTimer>>,
    phantom: PhantomData<Foo>,
}

// Not automatically derived because in that case it is only available if Foo: Clone
impl<Foo> Clone for DataReader<Foo> {
    fn clone(&self) -> Self {
        Self {
            data_reader_attributes: self.data_reader_attributes.clone(),
            phantom: self.phantom,
        }
    }
}

impl<Foo> DataReader<Foo> {
    pub fn new(data_reader_attributes: DdsWeak<DataReaderImpl<ThreadTimer>>) -> Self {
        Self {
            data_reader_attributes,
            phantom: PhantomData,
        }
    }
}

impl<Foo> AsRef<DdsWeak<DataReaderImpl<ThreadTimer>>> for DataReader<Foo> {
    fn as_ref(&self) -> &DdsWeak<DataReaderImpl<ThreadTimer>> {
        &self.data_reader_attributes
    }
}

impl<Foo> DataReader<Foo>
where
    Foo: DdsType + for<'de> DdsDeserialize<'de>,
{
    /// This operation accesses a collection of Data values from the DataReader. The size of the returned collection will be limited to
    /// the specified max_samples. The properties of the data_values collection and the setting of the PRESENTATION QoS policy
    /// (see 2.2.3.6) may impose further limits on the size of the returned ‘list.’
    /// 1. If PRESENTATION access_scope is INSTANCE, then the returned collection is a ‘list’ where samples belonging to
    /// the same data-instance are consecutive.
    /// 2. If PRESENTATION access_scope is TOPIC and ordered_access is set to FALSE, then the returned collection is a
    /// ‘list’ where samples belonging to the same data-instance are consecutive.
    /// 3. If PRESENTATION access_scope is TOPIC and ordered_access is set to TRUE, then the returned collection is a
    /// ‘list’ were samples belonging to the same instance may or may not be consecutive. This is because to preserve order
    /// it may be necessary to mix samples from different instances.
    /// 4. If PRESENTATION access_scope is GROUP and ordered_access is set to FALSE, then the returned collection is a
    /// ‘list’ where samples belonging to the same data instance are consecutive.
    /// 5. If PRESENTATION access_scope is GROUP and ordered_access is set to TRUE, then the returned collection
    /// contains at most one sample. The difference in this case is due to the fact that it is required that the application is able
    /// to read samples belonging to different DataReader objects in a specific order.
    /// In any case, the relative order between the samples of one instance is consistent with the DESTINATION_ORDER QosPolicy:
    /// • If DESTINATION_ORDER is BY_RECEPTION_TIMESTAMP, samples belonging to the same instances will appear
    /// in the relative order in which there were received (FIFO, earlier samples ahead of the later samples).
    /// • If DESTINATION_ORDER is BY_SOURCE_TIMESTAMP, samples belonging to the same instances will appear in
    /// the relative order implied by the source_timestamp (FIFO, smaller values of source_timestamp ahead of the larger
    /// values).
    /// In addition to the collection of samples, the read operation also uses a collection of SampleInfo structures (sample_infos), see
    /// 2.2.2.5.5, SampleInfo Class.
    /// The initial (input) properties of the data_values and sample_infos collections will determine the precise behavior of read
    /// operation. For the purposes of this description the collections are modeled as having three properties: the current-length (len),
    /// the maximum length (max_len), and whether the collection container owns the memory of the elements within (owns). PSM
    /// mappings that do not provide these facilities may need to change the signature of the read operation slightly to compensate for
    /// it.
    /// The initial (input) values of the len, max_len, and owns properties for the data_values and sample_infos collections govern the
    /// behavior of the read operation as specified by the following rules:
    /// values of len, max_len, and owns for the two collections must be identical. Otherwise read will and return
    /// PRECONDITION_NOT_MET.
    /// 2. On successful output, the values of len, max_len, and owns will be the same for both collections.
    /// 3. If the input max_len= =0, then the data_values and sample_infos collections will be filled with elements that are
    /// ‘loaned’ by the DataReader. On output, owns will be FALSE, len will be set to the number of values returned, and
    /// max_len will be set to a value verifying max_len >= len. The use of this variant allows for zero-copy22 access to the
    /// data and the application will need to “return the loan” to the DataWriter using the return_loan operation (see
    /// 2.2.2.5.3.20).
    /// 4. If the input max_len>0 and the input owns= =FALSE, then the read operation will fail and return
    /// PRECONDITION_NOT_MET. This avoids the potential hard-to-detect memory leaks caused by an application
    /// forgetting to “return the loan.”
    /// 5. If input max_len>0 and the input owns= =TRUE, then the read operation will copy the Data values and SampleInfo
    /// values into the elements already inside the collections. On output, owns will be TRUE, len will be set to the
    /// number of values copied, and max_len will remain unchanged. The use of this variant forces a copy but the
    /// application can control where the copy is placed and the application will not need to “return the loan.” The number
    /// of samples copied depends on the relative values of max_len and max_samples:
    /// • If max_samples = LENGTH_UNLIMITED, then at most max_len values will be copied. The use of this variant lets
    /// the application limit the number of samples returned to what the sequence can accommodate.
    /// • If max_samples <= max_len, then at most max_samples values will be copied. The use of this variant lets the
    /// application limit the number of samples returned to fewer that what the sequence can accommodate.
    /// • If max_samples > max_len, then the read operation will fail and return PRECONDITION_NOT_MET. This
    /// avoids the potential confusion where the application expects to be able to access up to max_samples, but that
    /// number can never be returned, even if they are available in the DataReader, because the output sequence cannot
    /// accommodate them.
    /// As described above, upon return the data_values and sample_infos collections may contain elements “loaned” from the
    /// DataReader. If this is the case, the application will need to use the return_loan operation (see 2.2.2.5.3.20) to return the
    /// “loan” once it is no longer using the Data in the collection. Upon return from return_loan, the collection will have max_len=0
    /// and owns=FALSE.
    /// The application can determine whether it is necessary to “return the loan” or not based on how the state of the collections when
    /// the read operation was called, or by accessing the ‘owns’ property. However, in many cases it may be simpler to always call
    /// return_loan, as this operation is harmless (i.e., leaves all elements unchanged) if the collection does not have a loan.
    /// To avoid potential memory leaks, the implementation of the Data and SampleInfo collections should disallow changing the
    /// length of a collection for which owns= =FALSE. Furthermore, deleting a collection for which owns= =FALSE should be
    /// considered an error.
    /// On output, the collection of Data values and the collection of SampleInfo structures are of the same length and are in a one-toone
    /// correspondence. Each SampleInfo provides information, such as the source_timestamp, the sample_state, view_state, and
    /// instance_state, etc., about the corresponding sample.
    /// Some elements in the returned collection may not have valid data. If the instance_state in the SampleInfo is
    /// NOT_ALIVE_DISPOSED or NOT_ALIVE_NO_WRITERS, then the last sample for that instance in the collection, that is,
    /// the one whose SampleInfo has sample_rank==0 does not contain valid data. Samples that contain no data do not count
    /// towards the limits imposed by the RESOURCE_LIMITS QoS policy.
    /// The act of reading a sample sets its sample_state to READ. If the sample belongs to the most recent generation of the instance,
    /// it will also set the view_state of the instance to NOT_NEW. It will not affect the instance_state of the instance.
    /// This operation must be provided on the specialized class that is generated for the particular application data-type that is being
    /// read.
    /// If the DataReader has no samples that meet the constraints, the return value will be NO_DATA.
    pub fn read(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        self.data_reader_attributes.upgrade()?.read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
        )
    }

    /// This operation accesses a collection of data-samples from the DataReader and a corresponding collection of SampleInfo
    /// structures. The operation will return either a ‘list’ of samples or else a single sample. This is controlled by the
    /// PRESENTATION QosPolicy using the same logic as for the read operation (see 2.2.2.5.3.8).
    /// The act of taking a sample removes it from the DataReader so it cannot be ‘read’ or ‘taken’ again. If the sample belongs to the
    /// most recent generation of the instance, it will also set the view_state of the instance to NOT_NEW. It will not affect the
    /// instance_state of the instance.
    /// The behavior of the take operation follows the same rules than the read operation regarding the pre-conditions and postconditions
    /// for the data_values and sample_infos collections. Similar to read, the take operation may ‘loan’ elements to the
    /// output collections which must then be returned by means of return_loan. The only difference with read is that, as stated, the
    /// sampled returned by take will no longer be accessible to successive calls to read or take.
    /// Similar to read, this operation must be provided on the specialized class that is generated for the particular application datatype
    /// that is being taken.
    /// If the DataReader has no samples that meet the constraints, the return value will be NO_DATA.
    pub fn take(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        self.data_reader_attributes.upgrade()?.take(
            max_samples,
            sample_states,
            view_states,
            instance_states,
        )
    }

    /// This operation copies the next, non-previously accessed Data value from the DataReader; the operation also copies the
    /// corresponding SampleInfo. The implied order among the samples stored in the DataReader is the same as for the read
    /// operation (2.2.2.5.3.8).
    /// The read_next_sample operation is semantically equivalent to the read operation where the input Data sequence has
    /// max_len=1, the sample_states=NOT_READ, the view_states=ANY_VIEW_STATE, and the
    /// instance_states=ANY_INSTANCE_STATE.
    /// The read_next_sample operation provides a simplified API to ‘read’ samples avoiding the need for the application to manage
    /// sequences and specify states.
    /// If there is no unread data in the DataReader, the operation will return NO_DATA and nothing is copied.
    pub fn read_next_sample(
        &self,
        data_value: &mut [Foo],
        sample_info: &mut [SampleInfo],
    ) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .read_next_sample(data_value, sample_info)
    }

    /// This operation copies the next, non-previously accessed Data value from the DataReader and ‘removes’ it from the
    /// DataReader so it is no longer accessible. The operation also copies the corresponding SampleInfo. This operation is
    /// analogous to the read_next_sample except for the fact that the sample is ‘removed’ from the DataReader.
    /// The take_next_sample operation is semantically equivalent to the take operation where the input sequence has max_len=1, the
    /// sample_states=NOT_READ, the view_states=ANY_VIEW_STATE, and the instance_states=ANY_INSTANCE_STATE.
    /// This operation provides a simplified API to ’take’ samples avoiding the need for the application to manage sequences and
    /// specify states.
    /// If there is no unread data in the DataReader, the operation will return NO_DATA and nothing is copied.
    pub fn take_next_sample(
        &self,
        data_value: &mut [Foo],
        sample_info: &mut [SampleInfo],
    ) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .take_next_sample(data_value, sample_info)
    }

    /// This operation accesses a collection of Data values from the DataReader. The behavior is identical to read except that all
    /// samples returned belong to the single specified instance whose handle is a_handle.
    /// Upon successful return, the Data collection will contain samples all belonging to the same instance. The corresponding
    /// SampleInfo verifies instance_handle == a_handle.
    /// The semantics are the same as for the read operation, except in building the collection the DataReader will check that the
    /// sample belongs to the specified instance and otherwise it will not place the sample in the returned collection.
    /// The behavior of the read_instance operation follows the same rules as the read operation regarding the pre-conditions and
    /// post-conditions for the data_values and sample_infos collections. Similar to read, the read_instance operation may ‘loan’
    /// elements to the output collections which must then be returned by means of return_loan.
    /// Similar to read, this operation must be provided on the specialized class that is generated for the particular application datatype
    /// that is being taken.
    /// If the DataReader has no samples that meet the constraints, the return value will be NO_DATA.
    /// This operation may return BAD_PARAMETER if the InstanceHandle_t a_handle does not correspond to an existing dataobject
    /// known to the DataReader. If the implementation is not able to check invalid handles, then the result in this situation is
    /// unspecified.
    #[allow(clippy::too_many_arguments)]
    pub fn read_instance(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
        max_samples: i32,
        a_handle: InstanceHandle,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.read_instance(
            data_values,
            sample_infos,
            max_samples,
            a_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    /// This operation accesses a collection of Data values from the DataReader. The behavior is identical to take except for that all
    /// samples returned belong to the single specified instance whose handle is a_handle.
    /// The semantics are the same as for the take operation, except in building the collection the DataReader will check that the
    /// sample belongs to the specified instance and otherwise it will not place the sample in the returned collection.
    /// The behavior of the take_instance operation follows the same rules as the read operation regarding the pre-conditions and
    /// post-conditions for the data_values and sample_infos collections. Similar to read, the take_instance operation may ‘loan’
    /// elements to the output collections which must then be returned by means of return_loan.
    /// Similar to read, this operation must be provided on the specialized class that is generated for the particular application datatype
    /// that is being taken.
    /// If the DataReader has no samples that meet the constraints, the return value will be NO_DATA.
    /// This operation may return BAD_PARAMETER if the InstanceHandle_t a_handle does not correspond to an existing dataobject
    /// known to the DataReader. If the implementation is not able to check invalid handles, then the result in this situation is
    /// unspecified.
    #[allow(clippy::too_many_arguments)]
    pub fn take_instance(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
        max_samples: i32,
        a_handle: InstanceHandle,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.take_instance(
            data_values,
            sample_infos,
            max_samples,
            a_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    /// This operation accesses a collection of Data values from the DataReader where all the samples belong to a single instance.
    /// The behavior is similar to read_instance except that the actual instance is not directly specified. Rather the samples will all
    /// belong to the ‘next’ instance with instance_handle ‘greater23’ than the specified previous_handle that has available samples.
    /// This operation implies the existence of a total order ‘greater-than’ relationship between the instance handles. The specifics of
    /// this relationship are not all important and are implementation specific. The important thing is that, according to the
    /// middleware, all instances are ordered relative to each other. This ordering is between the instance handles: It should not
    /// depend on the state of the instance (e.g., whether it has data or not) and must be defined even for instance handles that do not
    /// correspond to instances currently managed by the DataReader. For the purposes of the ordering it should be ‘as if’ each
    /// instance handle was represented as a unique integer.
    /// The behavior of read_next_instance is ‘as if’ the DataReader invoked read_instance passing the smallest instance_handle
    /// among all the ones that (a) are greater than previous_handle and (b) have available samples (i.e., samples that meet the
    /// constraints imposed by the specified states).
    /// The special value HANDLE_NIL is guaranteed to be ‘less than’ any valid instance_handle. So the use of the parameter value
    /// previous_handle==HANDLE_NIL will return the samples for the instance which has the smallest instance_handle among all
    /// the instances that contain available samples.
    /// The operation read_next_instance is intended to be used in an application-driven iteration where the application starts by
    /// passing previous_handle= =HANDLE_NIL, examines the samples returned, and then uses the instance_handle returned in
    /// the SampleInfo as the value of the previous_handle argument to the next call to read_next_instance. The iteration continues
    /// until read_next_instance returns the value NO_DATA.
    /// Note that it is possible to call the ‘read_next_instance’ operation with a previous_handle that does not correspond to an
    /// instance currently managed by the DataReader. This is because as stated earlier the ‘greater-than’ relationship is defined even
    /// for handles not managed by the DataReader. One practical situation where this may occur is when an application is iterating
    /// though all the instances, takes all the samples of a NOT_ALIVE_NO_WRITERS instance, returns the loan (at which point the
    /// instance information may be removed, and thus the handle becomes invalid), and tries to read the next instance.
    /// The behavior of the read_next_instance operation follows the same rules than the read operation regarding the pre-conditions
    /// and post-conditions for the data_values and sample_infos collections. Similar to read, the read_next_instance operation may
    /// ‘loan’ elements to the output collections which must then be returned by means of return_loan.
    /// Similar to read, this operation must be provided on the specialized class that is generated for the particular application datatype
    /// that is being taken.
    /// If the DataReader has no samples that meet the constraints, the return value will be NO_DATA.
    #[allow(clippy::too_many_arguments)]
    pub fn read_next_instance(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
        max_samples: i32,
        previous_handle: InstanceHandle,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.read_next_instance(
            data_values,
            sample_infos,
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    /// This operation accesses a collection of Data values from the DataReader and ‘removes’ them from the DataReader.
    /// This operation has the same behavior as read_next_instance except that the samples are ‘taken’ from the DataReader such
    /// that they are no longer accessible via subsequent ‘read’ or ‘take’ operations.
    /// Similar to the operation read_next_instance (see 2.2.2.5.3.16) it is possible to call take_next_instance with a
    /// previous_handle that does not correspond to an instance currently managed by the DataReader.
    /// The behavior of the take_next_instance operation follows the same rules as the read operation regarding the pre-conditions
    /// and post-conditions for the data_values and sample_infos collections. Similar to read, the take_next_instance operation may
    /// ‘loan’ elements to the output collections which must then be returned by means of return_loan.
    /// Similar to read, this operation must be provided on the specialized class that is generated for the particular application datatype
    /// that is being taken.
    /// If the DataReader has no samples that meet the constraints, the return value will be NO_DATA.
    #[allow(clippy::too_many_arguments)]
    pub fn take_next_instance(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
        max_samples: i32,
        previous_handle: InstanceHandle,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.take_next_instance(
            data_values,
            sample_infos,
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    /// This operation indicates to the DataReader that the application is done accessing the collection of data_values and
    /// sample_infos obtained by some earlier invocation of read or take on the DataReader.
    /// The data_values and sample_infos must belong to a single related ‘pair;’ that is, they should correspond to a pair returned from
    /// a single call to read or take. The data_values and sample_infos must also have been obtained from the same DataReader to
    /// which they are returned. If either of these conditions is not met, the operation will fail and return
    /// PRECONDITION_NOT_MET.
    /// The operation return_loan allows implementations of the read and take operations to “loan” buffers from the DataReader to
    /// the application and in this manner provide “zero-copy” access to the data. During the loan, the DataReader will guarantee that
    /// the data and sample-information are not modified.
    /// It is not necessary for an application to return the loans immediately after the read or take calls. However, as these buffers
    /// correspond to internal resources inside the DataReader, the application should not retain them indefinitely.
    /// The use of the return_loan operation is only necessary if the read or take calls “loaned” buffers to the application. As
    /// described in 2.2.2.5.3.8 this only occurs if the data_values and sample_infos collections had max_len=0 at the time read or
    /// take was called. The application may also examine the ‘owns’ property of the collection to determine where there is an
    /// outstanding loan. However, calling return_loan on a collection that does not have a loan is safe and has no side effects.
    /// If the collections had a loan, upon return from return_loan the collections will have max_len=0.
    /// Similar to read, this operation must be provided on the specialized class that is generated for the particular application datatype
    /// that is being taken.
    pub fn return_loan(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
    ) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .return_loan(data_values, sample_infos)
    }

    /// This operation can be used to retrieve the instance key that corresponds to an instance_handle. The operation will only fill the
    /// fields that form the key inside the key_holder instance.
    /// This operation may return BAD_PARAMETER if the InstanceHandle_t a_handle does not correspond to an existing dataobject
    /// known to the DataReader. If the implementation is not able to check invalid handles then the result in this situation is
    /// unspecified.
    pub fn get_key_value(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .get_key_value(key_holder, handle)
    }

    /// This operation takes as a parameter an instance and returns a handle that can be used in subsequent operations that accept an
    /// instance handle as an argument. The instance parameter is only used for the purpose of examining the fields that define the
    /// key.
    /// This operation does not register the instance in question. If the instance has not been previously registered, or if for any other
    /// reason the Service is unable to provide an instance handle, the Service will return the special value HANDLE_NIL.
    pub fn lookup_instance(&self, instance: &Foo) -> DdsResult<InstanceHandle> {
        self.data_reader_attributes
            .upgrade()?
            .lookup_instance(instance)
    }

    /// This operation allows access to the LIVELINESS_CHANGED communication status. Communication statuses are described
    /// in 2.2.4.1.
    pub fn get_liveliness_changed_status(&self) -> DdsResult<LivelinessChangedStatus> {
        self.data_reader_attributes
            .upgrade()?
            .get_liveliness_changed_status()
    }

    /// This operation allows access to the REQUESTED_DEADLINE_MISSED communication status. Communication statuses are
    /// described in 2.2.4.1.
    pub fn get_requested_deadline_missed_status(&self) -> DdsResult<RequestedDeadlineMissedStatus> {
        self.data_reader_attributes
            .upgrade()?
            .get_requested_deadline_missed_status()
    }

    /// This operation allows access to the REQUESTED_INCOMPATIBLE_QOS communication status. Communication statuses
    /// are described in 2.2.4.1.
    pub fn get_requested_incompatible_qos_status(
        &self,
    ) -> DdsResult<RequestedIncompatibleQosStatus> {
        self.data_reader_attributes
            .upgrade()?
            .get_requested_incompatible_qos_status()
    }

    /// This operation allows access to the SAMPLE_LOST communication status. Communication statuses are described in 2.2.4.1.
    pub fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        self.data_reader_attributes
            .upgrade()?
            .get_sample_lost_status()
    }

    /// This operation allows access to the SAMPLE_REJECTED communication status. Communication statuses are described in
    /// 2.2.4.1.
    pub fn get_sample_rejected_status(&self) -> DdsResult<SampleRejectedStatus> {
        self.data_reader_attributes
            .upgrade()?
            .get_sample_rejected_status()
    }

    /// This operation allows access to the SUBSCRIPTION_MATCHED communication status. Communication statuses are
    /// described in 2.2.4.1.
    pub fn get_subscription_matched_status(&self) -> DdsResult<SubscriptionMatchedStatus> {
        self.data_reader_attributes
            .upgrade()?
            .get_subscription_matched_status()
    }

    /// This operation returns the TopicDescription associated with the DataReader. This is the same TopicDescription that was used
    /// to create the DataReader.
    pub fn get_topicdescription(&self) -> DdsResult<Topic<Foo>> {
        Ok(Topic::new(
            self.data_reader_attributes
                .upgrade()?
                .get_topicdescription()
                .downgrade(),
        ))
    }

    /// This operation returns the Subscriber to which the DataReader belongs.
    pub fn get_subscriber(&self) -> DdsResult<Subscriber> {
        Ok(Subscriber::new(
            self.data_reader_attributes
                .upgrade()?
                .get_subscriber()
                .downgrade(),
        ))
    }

    /// This operation deletes all the entities that were created by means of the “create” operations on the DataReader. That is, it
    /// deletes all contained ReadCondition and QueryCondition objects.
    /// The operation will return PRECONDITION_NOT_MET if the any of the contained entities is in a state where it cannot be
    /// deleted.
    /// Once delete_contained_entities returns successfully, the application may delete the DataReader knowing that it has no
    /// contained ReadCondition and QueryCondition objects.
    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .delete_contained_entities()
    }

    /// This operation is intended only for DataReader entities that have a non-VOLATILE PERSISTENCE QoS kind.
    /// As soon as an application enables a non-VOLATILE DataReader it will start receiving both “historical” data, i.e., the data that
    /// was written prior to the time the DataReader joined the domain, as well as any new data written by the DataWriter entities.
    /// There are situations where the application logic may require the application to wait until all “historical” data is received. This
    /// is the purpose of the wait_for_historical_data operation.
    /// The operation wait_for_historical_data blocks the calling thread until either all “historical” data is received, or else the
    /// duration specified by the max_wait parameter elapses, whichever happens first. A return value of OK indicates that all the
    /// “historical” data was received; a return value of TIMEOUT indicates that max_wait elapsed before all the data was received.
    pub fn wait_for_historical_data(&self) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .wait_for_historical_data()
    }

    /// This operation retrieves information on a publication that is currently “associated” with the DataReader; that is, a publication
    /// with a matching Topic and compatible QoS that the application has not indicated should be “ignored” by means of the
    /// DomainParticipant ignore_publication operation.
    /// The publication_handle must correspond to a publication currently associated with the DataReader otherwise the operation
    /// will fail and return BAD_PARAMETER. The operation get_matched_publications can be used to find the publications that
    /// are currently matched with the DataReader.
    /// The operation may also fail if the infrastructure does not hold the information necessary to fill in the publication_data. In this
    /// case the operation will return UNSUPPORTED
    pub fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        self.data_reader_attributes
            .upgrade()?
            .get_matched_publication_data(publication_handle)
    }

    /// This operation retrieves the list of publications currently “associated” with the DataReader; that is, publications that have a
    /// matching Topic and compatible QoS that the application has not indicated should be “ignored” by means of the
    /// DomainParticipant ignore_ publication operation.
    /// The handles returned in the ‘publication_handles’ list are the ones that are used by the DDS implementation to locally identify
    /// the corresponding matched DataWriter entities. These handles match the ones that appear in the ‘instance_handle’ field of the
    /// SampleInfo when reading the “DCPSPublications” builtin topic
    /// The operation may fail if the infrastructure does not locally maintain the connectivity information.
    pub fn get_matched_publications(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.data_reader_attributes
            .upgrade()?
            .get_matched_publications()
    }
}

impl<Foo> Entity for DataReader<Foo>
where
    Foo: DdsType + for<'de> DdsDeserialize<'de> + 'static,
{
    type Qos = DataReaderQos;
    type Listener = Box<dyn DataReaderListener<Foo = Foo> + Send + Sync>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.set_qos(qos)
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        self.data_reader_attributes.upgrade()?.get_qos()
    }

    fn set_listener(
        &self,
        a_listener: Option<Self::Listener>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        #[allow(clippy::redundant_closure)]
        self.data_reader_attributes.upgrade()?.set_listener(
            a_listener.map::<Box<dyn AnyDataReaderListener + Send + Sync>, _>(|l| Box::new(l)),
            mask,
        )
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        Ok(StatusCondition::new(
            self.data_reader_attributes.upgrade()?.get_statuscondition(),
        ))
    }

    fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        Ok(self.data_reader_attributes.upgrade()?.get_status_changes())
    }

    fn enable(&self) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.enable(
            &THE_PARTICIPANT_FACTORY
                .lookup_participant_by_entity_handle(self.get_instance_handle()?),
        )
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.data_reader_attributes.upgrade()?.get_instance_handle()
    }
}

pub trait AnyDataReader {}
