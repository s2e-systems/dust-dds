use crate::dds::domain::domain_participant::DomainParticipant;
use crate::dds::infrastructure::entity::{Entity, StatusCondition};
use crate::dds::infrastructure::qos::{DataWriterQos, PublisherQos, TopicQos};
use crate::dds::infrastructure::status::StatusMask;
use crate::dds::publication::data_writer::{AnyRtpsWriter, DataWriter, RtpsDataWriter};
use crate::dds::publication::publisher_listener::PublisherListener;
use crate::dds::rtps_implementation::rtps_object::{RtpsObject, RtpsObjectList, RtpsObjectRef};
use crate::dds::topic::topic::Topic;
use crate::rtps::structure::Group;
use crate::rtps::types::{EntityId, EntityKind, GUID};
use crate::types::{DDSType, Duration, InstanceHandle, ReturnCode, TopicKind};
use std::sync::{atomic, Mutex};

pub struct RtpsPublisher {
    pub group: Group,
    pub writer_list: RtpsObjectList<Box<dyn AnyRtpsWriter>>,
    pub writer_count: atomic::AtomicU8,
    pub default_datawriter_qos: Mutex<DataWriterQos>,
    pub qos: PublisherQos,
}

impl RtpsPublisher {
    pub fn new(guid: GUID, qos: PublisherQos) -> Self {
        Self {
            group: Group::new(guid),
            writer_list: Default::default(),
            writer_count: atomic::AtomicU8::new(0),
            default_datawriter_qos: Mutex::new(DataWriterQos::default()),
            qos,
        }
    }
}

/// The Publisher acts on the behalf of one or several DataWriter objects that belong to it. When it is informed of a change to the
/// data associated with one of its DataWriter objects, it decides when it is appropriate to actually send the data-update message.
/// In making this decision, it considers any extra information that goes with the data (timestamp, writer, etc.) as well as the QoS
/// of the Publisher and the DataWriter.
/// All operations except for the base-class operations set_qos, get_qos, set_listener, get_listener, enable, get_statuscondition,
/// create_datawriter, and delete_datawriter may return the value NOT_ENABLED.
pub struct Publisher<'a> {
    pub(crate) parent_participant: &'a DomainParticipant,
    pub(crate) rtps_publisher: RtpsObjectRef<'a, RtpsObject<Box<RtpsPublisher>>>,
}

impl<'a> Publisher<'a> {
    /// This operation creates a DataWriter. The returned DataWriter will be attached and belongs to the Publisher.
    /// The DataWriter returned by the create_datawriter operation will in fact be a derived class, specific to the data-type associated
    /// with the Topic. As described in 2.2.2.3.7, for each application-defined type “Foo” there is an implied, auto-generated class
    /// FooDataWriter that extends DataWriter and contains the operations to write data of type “Foo.”
    /// In case of failure, the operation will return a ‘nil’ value (as specified by the platform).
    /// Note that a common application pattern to construct the QoS for the DataWriter is to:
    /// • Retrieve the QoS policies on the associated Topic by means of the get_qos operation on the Topic.
    /// • Retrieve the default DataWriter qos by means of the get_default_datawriter_qos operation on the Publisher.
    /// • Combine those two QoS policies and selectively modify policies as desired.
    /// • Use the resulting QoS policies to construct the DataWriter.
    /// The special value DATAWRITER_QOS_DEFAULT can be used to indicate that the DataWriter should be created with the
    /// default DataWriter QoS set in the factory. The use of this value is equivalent to the application obtaining the default
    /// DataWriter QoS by means of the operation get_default_datawriter_qos (2.2.2.4.1.15) and using the resulting QoS to create
    /// the DataWriter.
    /// The special value DATAWRITER_QOS_USE_TOPIC_QOS can be used to indicate that the DataWriter should be created
    /// with a combination of the default DataWriter QoS and the Topic QoS. The use of this value is equivalent to the application
    /// obtaining the default DataWriter QoS and the Topic QoS (by means of the operation Topic::get_qos) and then combining these
    /// two QoS using the operation copy_from_topic_qos whereby any policy that is set on the Topic QoS “overrides” the
    /// corresponding policy on the default QoS. The resulting QoS is then applied to the creation of the DataWriter.
    /// The Topic passed to this operation must have been created from the same DomainParticipant that was used to create this
    /// Publisher. If the Topic was created from a different DomainParticipant, the operation will fail and return a nil result.
    pub fn create_datawriter<T: DDSType>(
        &'a self,
        a_topic: &'a Topic<'a, T>,
        qos: Option<DataWriterQos>,
        // _a_listener: impl DataWriterListener<T>,
        // _mask: StatusMask
    ) -> Option<DataWriter<T>> {
        let this = self.rtps_publisher.value().ok()?;
        let topic = a_topic.rtps_topic.value().ok()?.clone();
        let guid_prefix = this.group.entity.guid.prefix();
        let entity_key = [
            0,
            this.writer_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_kind = match topic.topic_kind() {
            TopicKind::WithKey => EntityKind::UserDefinedWriterWithKey,
            TopicKind::NoKey => EntityKind::UserDefinedWriterNoKey,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let new_writer_guid = GUID::new(guid_prefix, entity_id);
        let new_writer_qos = qos.unwrap_or(self.get_default_datawriter_qos().ok()?);
        let new_writer: Box<RtpsDataWriter<T>> = Box::new(RtpsDataWriter::new(new_writer_guid, topic, new_writer_qos, None));
        // discovery.insert_writer(&new_writer).ok()?;
        let rtps_datawriter = this.writer_list.add(new_writer)?;

        Some(DataWriter {
            parent_publisher: self,
            topic: a_topic,
            rtps_datawriter,
        })
    }

    /// This operation deletes a DataWriter that belongs to the Publisher.
    /// The delete_datawriter operation must be called on the same Publisher object used to create the DataWriter. If
    /// delete_datawriter is called on a different Publisher, the operation will have no effect and it will return
    /// PRECONDITION_NOT_MET.
    /// The deletion of the DataWriter will automatically unregister all instances. Depending on the settings of the
    /// WRITER_DATA_LIFECYCLE QosPolicy, the deletion of the DataWriter may also dispose all instances. Refer to 2.2.3.21 for
    /// details.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    pub fn delete_datawriter<T: DDSType>(&self, a_datawriter: &DataWriter<T>) -> ReturnCode<()> {
        a_datawriter
            .rtps_datawriter
            .value_as::<RtpsDataWriter<T>>()?
            .topic
            .lock()
            .unwrap()
            .take(); // Drop the topic
        a_datawriter.rtps_datawriter.delete();
        Ok(())
    }

    /// This operation retrieves a previously created DataWriter belonging to the Publisher that is attached to a Topic with a matching
    /// topic_name. If no such DataWriter exists, the operation will return ’nil.’
    /// If multiple DataWriter attached to the Publisher satisfy this condition, then the operation will return one of them. It is not
    /// specified which one.
    pub fn lookup_datawriter<T: DDSType>(&self, _topic: &'a Topic<'a, T>) -> Option<DataWriter<T>> {
        todo!()
    }

    /// This operation indicates to the Service that the application is about to make multiple modifications using DataWriter objects
    /// belonging to the Publisher.
    /// It is a hint to the Service so it can optimize its performance by e.g., holding the dissemination of the modifications and then
    /// batching them.
    /// It is not required that the Service use this hint in any way.
    /// The use of this operation must be matched by a corresponding call to resume_publications indicating that the set of
    /// modifications has completed. If the Publisher is deleted before resume_publications is called, any suspended updates yet to
    /// be published will be discarded.
    pub fn suspend_publications(&self) -> ReturnCode<()> {
        todo!()
    }

    /// This operation indicates to the Service that the application has completed the multiple changes initiated by the previous
    /// suspend_publications. This is a hint to the Service that can be used by a Service implementation to e.g., batch all the
    /// modifications made since the suspend_publications.
    /// The call to resume_publications must match a previous call to suspend_publications. Otherwise the operation will return the
    /// error PRECONDITION_NOT_MET.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    pub fn resume_publications(&self) -> ReturnCode<()> {
        todo!()
    }

    /// This operation requests that the application will begin a ‘coherent set’ of modifications using DataWriter objects attached to
    /// the Publisher. The ‘coherent set’ will be completed by a matching call to end_coherent_changes.
    /// A ‘coherent set’ is a set of modifications that must be propagated in such a way that they are interpreted at the receivers’ side
    /// as a consistent set of modifications; that is, the receiver will only be able to access the data after all the modifications in the set
    /// are available at the receiver end. This does not imply that the middleware has to encapsulate all the modifications in a single message; it only implies that the
    /// receiving applications will behave as if this was the case.
    /// A connectivity change may occur in the middle of a set of coherent changes; for example, the set of partitions used by the
    /// Publisher or one of its Subscribers may change, a late-joining DataReader may appear on the network, or a communication
    /// failure may occur. In the event that such a change prevents an entity from receiving the entire set of coherent changes, that
    /// entity must behave as if it had received none of the set.
    /// These calls can be nested. In that case, the coherent set terminates only with the last call to end_coherent_ changes.
    /// The support for ‘coherent changes’ enables a publishing application to change the value of several data-instances that could
    /// belong to the same or different topics and have those changes be seen ‘atomically’ by the readers. This is useful in cases where
    /// the values are inter-related (for example, if there are two data-instances representing the ‘altitude’ and ‘velocity vector’ of the
    /// same aircraft and both are changed, it may be useful to communicate those values in a way the reader can see both together;
    /// otherwise, it may e.g., erroneously interpret that the aircraft is on a collision course).
    pub fn begin_coherent_changes(&self) -> ReturnCode<()> {
        todo!()
    }

    /// This operation terminates the ‘coherent set’ initiated by the matching call to begin_coherent_ changes. If there is no matching
    /// call to begin_coherent_ changes, the operation will return the error PRECONDITION_NOT_MET.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET
    pub fn end_coherent_changes(&self) -> ReturnCode<()> {
        todo!()
    }

    /// This operation blocks the calling thread until either all data written by the reliable DataWriter entities is acknowledged by all
    /// matched reliable DataReader entities, or else the duration specified by the max_wait parameter elapses, whichever happens
    /// first. A return value of OK indicates that all the samples written have been acknowledged by all reliable matched data readers;
    /// a return value of TIMEOUT indicates that max_wait elapsed before all the data was acknowledged.
    pub fn wait_for_acknowledgments(&self, _max_wait: Duration) -> ReturnCode<()> {
        todo!()
    }

    /// This operation deletes all the entities that were created by means of the “create” operations on the Publisher. That is, it deletes
    /// all contained DataWriter objects.
    /// The operation will return PRECONDITION_NOT_MET if the any of the contained entities is in a state where it cannot be
    /// deleted.
    /// Once delete_contained_entities returns successfully, the application may delete the Publisher knowing that it has no
    /// contained DataWriter objects
    pub fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    /// This operation sets a default value of the DataWriter QoS policies which will be used for newly created DataWriter entities in
    /// the case where the QoS policies are defaulted in the create_datawriter operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    /// The special value DATAWRITER_QOS_DEFAULT may be passed to this operation to indicate that the default QoS should be
    /// reset back to the initial values the factory would use, that is the values that would be used if the set_default_datawriter_qos
    /// operation had never been called.
    pub fn set_default_datawriter_qos(&self, _qos: DataWriterQos) -> ReturnCode<()> {
        todo!()
    }

    /// This operation retrieves the default value of the DataWriter QoS, that is, the QoS policies which will be used for newly created
    /// DataWriter entities in the case where the QoS policies are defaulted in the create_datawriter operation.
    /// The values retrieved by get_default_datawriter_qos will match the set of values specified on the last successful call to
    /// set_default_datawriter_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3, Supported
    /// QoS.
    pub fn get_default_datawriter_qos(&self) -> ReturnCode<DataWriterQos> {
        Ok(self
            .rtps_publisher
            .value()?
            .default_datawriter_qos
            .lock()
            .unwrap()
            .clone())
    }

    /// This operation copies the policies in the a_topic_qos to the corresponding policies in the a_datawriter_qos (replacing values
    /// in the a_datawriter_qos, if present).
    /// This is a “convenience” operation most useful in combination with the operations get_default_datawriter_qos and
    /// Topic::get_qos. The operation copy_from_topic_qos can be used to merge the DataWriter default QoS policies with the
    /// corresponding ones on the Topic. The resulting QoS can then be used to create a new DataWriter, or set its QoS.
    /// This operation does not check the resulting a_datawriter_qos for consistency. This is because the ‘merged’ a_datawriter_qos
    /// may not be the final one, as the application can still modify some policies prior to applying the policies to the DataWriter.
    pub fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> ReturnCode<()> {
        todo!()
    }

    /// This operation returns the DomainParticipant to which the Publisher belongs.
    pub fn get_participant(&self) -> &DomainParticipant {
        self.parent_participant
    }
}

impl<'a> Entity for Publisher<'a> {
    type Qos = PublisherQos;
    type Listener = Box<dyn PublisherListener>;

    fn set_qos(&self, _qos: Self::Qos) -> ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self) -> ReturnCode<Self::Qos> {
        Ok(self.rtps_publisher.value()?.qos.clone())
    }

    fn set_listener(&self, _a_listener: Self::Listener, _mask: StatusMask) -> ReturnCode<()> {
        todo!()
    }

    fn get_listener(&self) -> &Self::Listener {
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        Ok(self.rtps_publisher.value()?.group.entity.guid.into())
    }
}
