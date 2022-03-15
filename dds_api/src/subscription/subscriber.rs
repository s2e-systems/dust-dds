use crate::{
    dcps_psm::{InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask, ViewStateKind},
    infrastructure::{
        entity::Entity,
        qos::{DataReaderQos, TopicQos},
    },
    return_type::DDSResult,
};

use super::data_reader::AnyDataReader;

pub trait SubscriberDataReaderFactory<Foo> {
    type TopicType;
    type DataReaderType;

    fn datareader_factory_create_datareader(
        &self,
        a_topic: &Self::TopicType,
        qos: Option<<Self::DataReaderType as Entity>::Qos>,
        a_listener: Option<<Self::DataReaderType as Entity>::Listener>,
        mask: StatusMask,
    ) -> DDSResult<Self::DataReaderType>
    where
        Self::DataReaderType: Entity;

    fn datareader_factory_delete_datareader(
        &self,
        a_datareader: &Self::DataReaderType,
    ) -> DDSResult<()>;

    fn datareader_factory_lookup_datareader(
        &self,
        topic: &Self::TopicType,
    ) -> DDSResult<Self::DataReaderType>;
}

/// A Subscriber is the object responsible for the actual reception of the data resulting from its subscriptions
///
/// A Subscriber acts on the behalf of one or several DataReader objects that are related to it. When it receives data (from the
/// other parts of the system), it builds the list of concerned DataReader objects, and then indicates to the application that data is
/// available, through its listener or by enabling related conditions. The application can access the list of concerned DataReader
/// objects through the operation get_datareaders and then access the data available though operations on the DataReader.
/// All operations except for the base-class operations set_qos, get_qos, set_listener, get_listener, enable, get_statuscondition,
/// and create_datareader may return the value NOT_ENABLED.
pub trait Subscriber {
    type DomainParticipant;

    /// This operation creates a DataReader. The returned DataReader will be attached and belong to the Subscriber.
    ///
    /// The DataReader returned by the create_datareader operation will in fact be a derived class, specific to the data-type
    /// associated with the Topic. As described in 2.2.2.3.7, for each application-defined type “Foo” there is an implied auto-generated
    /// class FooDataReader that extends DataReader and contains the operations to read data of type “Foo.”
    /// In case of failure, the operation will return a ‘nil’ value (as specified by the platform).
    /// Note that a common application pattern to construct the QoS for the DataReader is to:
    /// - Retrieve the QoS policies on the associated Topic by means of the get_qos operation on the Topic.
    /// - Retrieve the default DataReader qos by means of the get_default_datareader_qos operation on the Subscriber.
    /// - Combine those two QoS policies and selectively modify policies as desired.
    /// - Use the resulting QoS policies to construct the DataReader.
    /// The special value DATAREADER_QOS_DEFAULT can be used to indicate that the DataReader should be created with the
    /// default DataReader QoS set in the factory. The use of this value is equivalent to the application obtaining the default
    /// DataReader QoS by means of the operation get_default_datareader_qos (2.2.2.4.1.15) and using the resulting QoS to create
    /// the DataReader.
    /// Provided that the TopicDescription passed to this method is a Topic or a ContentFilteredTopic, the special value
    /// DATAREADER_QOS_USE_TOPIC_QOS can be used to indicate that the DataReader should be created with a combination
    /// of the default DataReader QoS and the Topic QoS. (In the case of a ContentFilteredTopic, the Topic in question is the
    /// ContentFilteredTopic’s “related Topic.”) The use of this value is equivalent to the application obtaining the default
    /// DataReader QoS and the Topic QoS (by means of the operation Topic::get_qos) and then combining these two QoS using the
    /// operation copy_from_topic_qos whereby any policy that is set on the Topic QoS “overrides” the corresponding policy on the
    /// default QoS. The resulting QoS is then applied to the creation of the DataReader. It is an error to use
    /// DATAREADER_QOS_USE_TOPIC_QOS when creating a DataReader with a MultiTopic; this method will return a ‘nil’
    /// value in that case.
    /// The TopicDescription passed to this operation must have been created from the same DomainParticipant that was used to
    /// create this Subscriber. If the TopicDescription was created from a different DomainParticipant, the operation will fail and
    /// return a nil result.
    fn create_datareader<Foo>(
        &self,
        a_topic: &Self::TopicType,
        qos: Option<<Self::DataReaderType as Entity>::Qos>,
        a_listener: Option<<Self::DataReaderType as Entity>::Listener>,
        mask: StatusMask,
    ) -> DDSResult<Self::DataReaderType>
    where
        Self: SubscriberDataReaderFactory<Foo> + Sized,
        Self::DataReaderType: Entity,
    {
        self.datareader_factory_create_datareader(a_topic, qos, a_listener, mask)
    }

    /// This operation deletes a DataReader that belongs to the Subscriber. If the DataReader does not belong to the Subscriber, the
    /// operation returns the error PRECONDITION_NOT_MET.
    /// The deletion of a DataReader is not allowed if there are any existing ReadCondition or QueryCondition objects that are
    /// attached to the DataReader. If the delete_datareader operation is called on a DataReader with any of these existing objects
    /// attached to it, it will return PRECONDITION_NOT_MET.
    /// The deletion of a DataReader is not allowed if it has any outstanding loans as a result of a call to read, take, or one of the
    /// variants thereof. If the delete_datareader operation is called on a DataReader with one or more outstanding loans, it will
    /// return PRECONDITION_NOT_MET.
    /// The delete_datareader operation must be called on the same Subscriber object used to create the DataReader. If
    /// delete_datareader is called on a different Subscriber, the operation will have no effect and it will return
    /// PRECONDITION_NOT_MET.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    fn delete_datareader<Foo>(&self, a_datareader: &Self::DataReaderType) -> DDSResult<()>
    where
        Self: SubscriberDataReaderFactory<Foo> + Sized,
    {
        self.datareader_factory_delete_datareader(a_datareader)
    }

    /// This operation retrieves a previously-created DataReader belonging to the Subscriber that is attached to a Topic with a
    /// matching topic_name. If no such DataReader exists, the operation will return ’nil.’
    /// If multiple DataReaders attached to the Subscriber satisfy this condition, then the operation will return one of them. It is not
    /// specified which one.
    /// The use of this operation on the built-in Subscriber allows access to the built-in DataReader entities for the built-in topics
    fn lookup_datareader<Foo>(&self, topic: &Self::TopicType) -> DDSResult<Self::DataReaderType>
    where
        Self: SubscriberDataReaderFactory<Foo> + Sized,
    {
        self.datareader_factory_lookup_datareader(topic)
    }

    /// This operation indicates that the application is about to access the data samples in any of the DataReader objects attached to
    /// the Subscriber.
    /// The application is required to use this operation only if PRESENTATION QosPolicy of the Subscriber to which the
    /// DataReader belongs has the access_scope set to ‘GROUP.’
    /// In the aforementioned case, the operation begin_access must be called prior to calling any of the sample-accessing operations,
    /// namely: get_datareaders on the Subscriber and read, take, read_w_condition, take_w_condition on any DataReader.
    /// Otherwise the sample-accessing operations will return the error PRECONDITION_NOT_MET. Once the application has
    /// finished accessing the data samples it must call end_access.
    /// It is not required for the application to call begin_access/end_access if the PRESENTATION QosPolicy has the access_scope
    /// set to something other than ‘GROUP.’ Calling begin_access/end_access in this case is not considered an error and has no
    /// effect.
    /// The calls to begin_access/end_access may be nested. In that case, the application must call end_access as many times as it
    /// called begin_access.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    fn begin_access(&self) -> DDSResult<()>;

    /// Indicates that the application has finished accessing the data samples in DataReader objects managed by the Subscriber.
    /// This operation must be used to ‘close’ a corresponding begin_access.
    /// After calling end_access the application should no longer access any of the Data or SampleInfo elements returned from the
    /// sample-accessing operations. This call must close a previous call to begin_access otherwise the operation will return the error
    /// PRECONDITION_NOT_MET.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    fn end_access(&self) -> DDSResult<()>;

    /// This operation allows the application to access the DataReader objects that contain samples with the specified sample_states,
    /// view_states, and instance_states.
    ///
    /// If the PRESENTATION QosPolicy of the Subscriber to which the DataReader belongs has the access_scope set to ‘GROUP.’
    /// This operation should only be invoked inside a begin_access/end_access block. Otherwise it will return the error
    /// PRECONDITION_NOT_MET.
    /// Depending on the setting of the PRESENTATION QoS policy (see 2.2.3.6), the returned collection of DataReader objects may
    /// be a ‘set’ containing each DataReader at most once in no specified order, or a ‘list’ containing each DataReader one or more
    /// times in a specific order.
    /// 1. If PRESENTATION access_scope is INSTANCE or TOPIC the returned collection is a ‘set.’
    /// 2. If PRESENTATION access_scope is GROUP and ordered_access is set to TRUE, then the returned collection is a
    /// ‘list.’
    /// This difference is due to the fact that, in the second situation it is required to access samples belonging to different DataReader
    /// objects in a particular order. In this case, the application should process each DataReader in the same order it appears in the
    /// ‘list’ and read or take exactly one sample from each DataReader. The patterns that an application should use to access data is
    /// fully described in 2.2.2.5.1, Access to the data.
    fn get_datareaders(
        &self,
        readers: &mut [&mut dyn AnyDataReader],
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DDSResult<()>;

    /// This operation invokes the operation on_data_available on the DataReaderListener objects attached to contained DataReader
    /// entities with a DATA_AVAILABLE status that is considered changed as described in 2.2.4.2.2, Changes in Read
    /// Communication Statuses.
    /// This operation is typically invoked from the on_data_on_readers operation in the SubscriberListener. That way the
    /// SubscriberListener can delegate to the DataReaderListener objects the handling of the data.
    fn notify_datareaders(&self) -> DDSResult<()>;

    /// This operation returns the DomainParticipant to which the Subscriber belongs.
    fn get_participant(&self) -> DDSResult<Self::DomainParticipant>;

    /// This operation allows access to the SAMPLE_LOST communication status. Communication statuses are described in 2.2.4.1
    fn get_sample_lost_status(&self, status: &mut SampleLostStatus) -> DDSResult<()>;

    /// This operation deletes all the entities that were created by means of the “create” operations on the Subscriber. That is, it
    /// deletes all contained DataReader objects. This pattern is applied recursively. In this manner the operation
    /// delete_contained_entities on the Subscriber will end up deleting all the entities recursively contained in the Subscriber, that
    /// is also the QueryCondition and ReadCondition objects belonging to the contained DataReaders.
    /// The operation will return PRECONDITION_NOT_MET if any of the contained entities is in a state where it cannot be deleted.
    /// This will occur, for example, if a contained DataReader cannot be deleted because the application has called a read or take
    /// operation and has not called the corresponding return_loan operation to return the loaned samples.
    /// Once delete_contained_entities returns successfully, the application may delete the Subscriber knowing that it has no
    /// contained DataReader objects.
    fn delete_contained_entities(&self) -> DDSResult<()>;

    /// This operation sets a default value of the DataReader QoS policies which will be used for newly created DataReader entities
    /// in the case where the QoS policies are defaulted in the create_datareader operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    /// The special value DATAREADER_QOS_DEFAULT may be passed to this operation to indicate that the default QoS should
    /// be reset back to the initial values the factory would use, that is the values that would be used if the
    /// set_default_datareader_qos operation had never been called.
    fn set_default_datareader_qos(&self, qos: Option<DataReaderQos>) -> DDSResult<()>;

    /// This operation retrieves the default value of the DataReader QoS, that is, the QoS policies which will be used for newly
    /// created DataReader entities in the case where the QoS policies are defaulted in the create_datareader operation.
    /// The values retrieved get_default_datareader_qos will match the set of values specified on the last successful call to
    /// get_default_datareader_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3,
    /// Supported QoS.
    fn get_default_datareader_qos(&self) -> DDSResult<DataReaderQos>;

    /// This operation copies the policies in the a_topic_qos to the corresponding policies in the a_datareader_qos (replacing values
    /// in the a_datareader_qos, if present).
    /// This is a “convenience” operation most useful in combination with the operations get_default_datareader_qos and
    /// Topic::get_qos. The operation copy_from_topic_qos can be used to merge the DataReader default QoS policies with the
    /// corresponding ones on the Topic. The resulting QoS can then be used to create a new DataReader, or set its QoS.
    /// This operation does not check the resulting a_datareader_qos for consistency. This is because the ‘merged’ a_datareader_qos
    /// may not be the final one, as the application can still modify some policies prior to applying the policies to the DataReader.
    fn copy_from_topic_qos(
        &self,
        a_datareader_qos: &mut DataReaderQos,
        a_topic_qos: &TopicQos,
    ) -> DDSResult<()>;
}
