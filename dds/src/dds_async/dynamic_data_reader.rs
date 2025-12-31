use super::{condition::StatusConditionAsync, subscriber::SubscriberAsync};
use crate::{
    dcps::{
        actor::ActorAddress,
        channels::{mpsc::MpscSender, oneshot::oneshot},
        domain_participant_mail::{DcpsDomainParticipantMail, ReaderServiceMail},
        status_condition::DcpsStatusCondition,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::DataReaderQos,
        sample_info::{
            InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind, ANY_INSTANCE_STATE,
            ANY_VIEW_STATE,
        },
        status::StatusKind,
    },
    xtypes::dynamic_type::{DynamicData, DynamicType},
};
use alloc::{string::String, vec, vec::Vec};

/// A sample containing dynamic data and its associated sample info.
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicSample {
    /// The dynamic data, if valid.
    pub data: Option<DynamicData>,
    /// Information about the sample.
    pub sample_info: SampleInfo,
}

impl DynamicSample {
    /// Creates a new DynamicSample.
    pub fn new(data: Option<DynamicData>, sample_info: SampleInfo) -> Self {
        Self { data, sample_info }
    }
}

/// A [`DynamicDataReaderAsync`] allows the application to subscribe to a topic and access received data using runtime type information.
///
/// Unlike [`DataReaderAsync`](crate::dds_async::data_reader::DataReaderAsync), this reader works with
/// [`DynamicData`] and does not require compile-time type knowledge.
/// This is useful for generic monitoring tools, data recorders, or dynamic language bindings.
pub struct DynamicDataReaderAsync {
    handle: InstanceHandle,
    status_condition_address: ActorAddress<DcpsStatusCondition>,
    subscriber: SubscriberAsync,
    topic_name: String,
    dynamic_type: DynamicType,
}

impl Clone for DynamicDataReaderAsync {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            status_condition_address: self.status_condition_address.clone(),
            subscriber: self.subscriber.clone(),
            topic_name: self.topic_name.clone(),
            dynamic_type: self.dynamic_type.clone(),
        }
    }
}

impl DynamicDataReaderAsync {
    pub(crate) fn new(
        handle: InstanceHandle,
        status_condition_address: ActorAddress<DcpsStatusCondition>,
        subscriber: SubscriberAsync,
        topic_name: String,
        dynamic_type: DynamicType,
    ) -> Self {
        Self {
            handle,
            status_condition_address,
            subscriber,
            topic_name,
            dynamic_type,
        }
    }

    fn participant_address(&self) -> &MpscSender<DcpsDomainParticipantMail> {
        self.subscriber.participant_address()
    }

    /// Returns the dynamic type associated with this reader.
    pub fn get_type(&self) -> &DynamicType {
        &self.dynamic_type
    }

    /// Returns the topic name this reader is subscribed to.
    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
    }

    /// Read samples with full filtering options.
    #[tracing::instrument(skip(self))]
    pub async fn read(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<DynamicSample>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Reader(ReaderServiceMail::Read {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                max_samples,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
                specific_instance_handle: None,
                reply_sender,
            }))
            .await?;
        let samples = reply_receiver.await??;

        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| DynamicSample::new(data, sample_info))
            .collect())
    }

    /// Take samples with full filtering options.
    #[tracing::instrument(skip(self))]
    pub async fn take(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<DynamicSample>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Reader(ReaderServiceMail::Take {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                max_samples,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
                specific_instance_handle: None,
                reply_sender,
            }))
            .await?;
        let samples = reply_receiver.await??;

        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| DynamicSample::new(data, sample_info))
            .collect())
    }

    /// Read the next unread sample.
    #[tracing::instrument(skip(self))]
    pub async fn read_next_sample(&self) -> DdsResult<DynamicSample> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Reader(ReaderServiceMail::Read {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                max_samples: 1,
                sample_states: vec![SampleStateKind::NotRead],
                view_states: ANY_VIEW_STATE.to_vec(),
                instance_states: ANY_INSTANCE_STATE.to_vec(),
                specific_instance_handle: None,
                reply_sender,
            }))
            .await?;
        let mut samples = reply_receiver.await??;
        let (data, sample_info) = samples.pop().expect("Would return NoData if empty");
        Ok(DynamicSample::new(data, sample_info))
    }

    /// Take the next unread sample.
    #[tracing::instrument(skip(self))]
    pub async fn take_next_sample(&self) -> DdsResult<DynamicSample> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Reader(ReaderServiceMail::Take {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                max_samples: 1,
                sample_states: vec![SampleStateKind::NotRead],
                view_states: ANY_VIEW_STATE.to_vec(),
                instance_states: ANY_INSTANCE_STATE.to_vec(),
                specific_instance_handle: None,
                reply_sender,
            }))
            .await?;
        let mut samples = reply_receiver.await??;
        let (data, sample_info) = samples.pop().expect("Would return NoData if empty");
        Ok(DynamicSample::new(data, sample_info))
    }

    /// Get the status condition for this reader.
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(self.status_condition_address.clone())
    }

    /// Get the subscriber that created this reader.
    pub fn get_subscriber(&self) -> SubscriberAsync {
        self.subscriber.clone()
    }

    /// Get the instance handle of this reader.
    pub async fn get_instance_handle(&self) -> InstanceHandle {
        self.handle
    }

    /// Get the QoS settings for this reader.
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DataReaderQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Reader(
                ReaderServiceMail::GetQos {
                    subscriber_handle: self.subscriber.get_instance_handle().await,
                    data_reader_handle: self.handle,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Enable this reader.
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Reader(
                ReaderServiceMail::Enable {
                    subscriber_handle: self.subscriber.get_instance_handle().await,
                    data_reader_handle: self.handle,
                    participant_address: self.participant_address().clone(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Get the status changes for this reader.
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }
}

#[cfg(all(test, feature = "type_lookup"))]
mod tests {
    use super::*;
    use crate::dds_async::domain_participant_factory::DomainParticipantFactoryAsync;
    use crate::infrastructure::{
        qos::QosKind,
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::NO_STATUS,
        time::{Duration, DurationKind},
        type_support::TypeSupport,
    };
    use crate::xtypes::{
        binding::XTypesBinding,
        data_storage::DataStorageMapping,
        dynamic_type::{
            DynamicTypeBuilderFactory, ExtensibilityKind, MemberDescriptor, TryConstructKind,
            TypeDescriptor, TypeKind,
        },
    };
    use alloc::string::String;

    /// A simple test type for integration testing.
    #[derive(Debug, Clone, PartialEq)]
    struct TestMessage {
        id: i32,
        message: String,
    }

    impl TypeSupport for TestMessage {
        fn get_type_name() -> &'static str {
            "TestMessage"
        }

        fn get_type() -> DynamicType {
            let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
                kind: TypeKind::STRUCTURE,
                name: String::from("TestMessage"),
                base_type: None,
                discriminator_type: None,
                bound: Vec::new(),
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            });
            builder
                .add_member(MemberDescriptor {
                    name: String::from("id"),
                    id: 0,
                    r#type: <i32 as XTypesBinding>::get_dynamic_type(),
                    default_value: None,
                    index: 0,
                    label: Vec::new(),
                    try_construct_kind: TryConstructKind::UseDefault,
                    is_key: true,
                    is_optional: false,
                    is_must_understand: false,
                    is_shared: false,
                    is_default_label: false,
                })
                .unwrap();
            builder
                .add_member(MemberDescriptor {
                    name: String::from("message"),
                    id: 1,
                    r#type: DynamicTypeBuilderFactory::create_string_type(0).build(),
                    default_value: None,
                    index: 1,
                    label: Vec::new(),
                    try_construct_kind: TryConstructKind::UseDefault,
                    is_key: false,
                    is_optional: false,
                    is_must_understand: false,
                    is_shared: false,
                    is_default_label: false,
                })
                .unwrap();
            builder.build()
        }

        fn create_sample(src: DynamicData) -> Self {
            let id = src
                .get_value(0)
                .ok()
                .and_then(|s| i32::try_from_storage(s.clone()).ok())
                .unwrap_or(0);
            let message = src
                .get_value(1)
                .ok()
                .and_then(|s| String::try_from_storage(s.clone()).ok())
                .unwrap_or_default();
            TestMessage { id, message }
        }

        fn create_dynamic_sample(self) -> DynamicData {
            let mut data = crate::xtypes::dynamic_type::DynamicDataFactory::create_data(
                Self::get_type(),
            );
            data.set_value(0, self.id.into_storage());
            data.set_value(1, self.message.into_storage());
            data
        }
    }

    /// Integration test: Typed DataWriter writes, DynamicDataReader reads.
    ///
    /// This test verifies AC #7: "participant discovers remote type and reads data
    /// without compile-time type". While this test uses manually-constructed
    /// DynamicType (simulating what TypeLookup would provide), it proves that
    /// DynamicDataReader can successfully read data written by a typed DataWriter.
    #[tokio::test]
    async fn test_dynamic_data_reader_reads_typed_writer_data() {
        // Create participant
        let factory = DomainParticipantFactoryAsync::get_instance();
        let participant = factory
            .create_participant(200, QosKind::Default, None::<()>, NO_STATUS)
            .await
            .expect("Failed to create participant");

        // Create topic with typed DataWriter's type
        let topic = participant
            .create_topic::<TestMessage>("TestTopic", "TestMessage", QosKind::Default, None::<()>, NO_STATUS)
            .await
            .expect("Failed to create topic");

        // Create publisher and typed DataWriter
        let publisher = participant
            .create_publisher(QosKind::Default, None::<()>, NO_STATUS)
            .await
            .expect("Failed to create publisher");

        let mut writer_qos = crate::infrastructure::qos::DataWriterQos::default();
        writer_qos.reliability = ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        };
        let writer = publisher
            .create_datawriter::<TestMessage>(&topic, QosKind::Specific(writer_qos), None::<()>, NO_STATUS)
            .await
            .expect("Failed to create datawriter");

        // Create subscriber and DynamicDataReader with the same type (simulating TypeLookup discovery)
        let subscriber = participant
            .create_subscriber(QosKind::Default, None::<()>, NO_STATUS)
            .await
            .expect("Failed to create subscriber");

        // This is the key part: create DynamicDataReader with DynamicType, not compile-time type
        let dynamic_type = TestMessage::get_type();
        let mut reader_qos = crate::infrastructure::qos::DataReaderQos::default();
        reader_qos.reliability = ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        };
        let dynamic_reader = subscriber
            .create_dynamic_datareader("TestTopic", dynamic_type, QosKind::Specific(reader_qos))
            .await
            .expect("Failed to create dynamic datareader");

        // Wait for discovery
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Write data with typed DataWriter
        let test_data = TestMessage {
            id: 42,
            message: String::from("Hello, Dynamic World!"),
        };
        writer.write(test_data, None).await.expect("Failed to write");

        // Wait for data to propagate
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Read with DynamicDataReader
        let samples = dynamic_reader
            .take(10, &ANY_SAMPLE_STATE, &ANY_VIEW_STATE, &ANY_INSTANCE_STATE)
            .await
            .expect("Failed to take samples");

        assert!(!samples.is_empty(), "Expected at least one sample");

        let sample = &samples[0];
        assert!(sample.data.is_some(), "Expected valid data");

        let data = sample.data.as_ref().unwrap();

        // Verify data using by-name accessors
        let id = data.get_int32_value_by_name("id").expect("Failed to get id");
        assert_eq!(*id, 42);

        let message = data
            .get_string_value_by_name("message")
            .expect("Failed to get message");
        assert_eq!(message.as_str(), "Hello, Dynamic World!");

        // Cleanup
        participant
            .delete_contained_entities()
            .await
            .expect("Failed to delete entities");
        factory
            .delete_participant(&participant)
            .await
            .expect("Failed to delete participant");
    }
}
