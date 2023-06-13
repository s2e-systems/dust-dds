use std::marker::PhantomData;

use crate::{
    implementation::{
        data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
        dds::dds_data_reader::DdsDataReader,
        rtps::{
            messages::overall_structure::RtpsMessageRead, stateful_reader::RtpsStatefulReader,
            stateless_reader::RtpsStatelessReader, types::Locator, writer_proxy::RtpsWriterProxy,
        },
        utils::actor::{ActorAddress, Mail, MailHandler},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{SubscriberQos, TopicQos},
        time::Time,
    },
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::DdsDeserialize,
};

impl<T> ActorAddress<DdsDataReader<T>> {
    pub fn enable(&self) -> DdsResult<()> {
        struct Enable;

        impl Mail for Enable {
            type Result = ();
        }

        impl<T> MailHandler<Enable> for DdsDataReader<T> {
            fn handle(&mut self, _mail: Enable) -> <Enable as Mail>::Result {
                self.enable()
            }
        }
        self.send_blocking(Enable)
    }

    pub fn is_enabled(&self) -> DdsResult<bool> {
        struct IsEnabled;

        impl Mail for IsEnabled {
            type Result = bool;
        }

        impl<T> MailHandler<IsEnabled> for DdsDataReader<T> {
            fn handle(&mut self, _mail: IsEnabled) -> <IsEnabled as Mail>::Result {
                self.is_enabled()
            }
        }

        self.send_blocking(IsEnabled)
    }

    pub fn get_type_name(&self) -> DdsResult<&'static str> {
        struct GetTypeName;

        impl Mail for GetTypeName {
            type Result = &'static str;
        }

        impl<T> MailHandler<GetTypeName> for DdsDataReader<T> {
            fn handle(&mut self, _mail: GetTypeName) -> <GetTypeName as Mail>::Result {
                self.get_type_name()
            }
        }
        self.send_blocking(GetTypeName)
    }

    pub fn get_topic_name(&self) -> DdsResult<String> {
        struct GetTopicName;

        impl Mail for GetTopicName {
            type Result = String;
        }

        impl<T> MailHandler<GetTopicName> for DdsDataReader<T> {
            fn handle(&mut self, _mail: GetTopicName) -> <GetTopicName as Mail>::Result {
                self.get_topic_name()
            }
        }

        self.send_blocking(GetTopicName)
    }
}

impl ActorAddress<DdsDataReader<RtpsStatefulReader>> {
    pub fn process_rtps_message(
        &self,
        message: RtpsMessageRead,
        reception_timestamp: Time,
    ) -> DdsResult<()> {
        struct ProcessRtpsMessage {
            message: RtpsMessageRead,
            reception_timestamp: Time,
        }

        impl Mail for ProcessRtpsMessage {
            type Result = ();
        }

        impl MailHandler<ProcessRtpsMessage> for DdsDataReader<RtpsStatefulReader> {
            fn handle(&mut self, mail: ProcessRtpsMessage) -> <ProcessRtpsMessage as Mail>::Result {
                self.process_rtps_message(mail.message, mail.reception_timestamp)
            }
        }

        self.send_blocking(ProcessRtpsMessage {
            message,
            reception_timestamp,
        })
    }

    pub fn matched_writer_add(&self, a_writer_proxy: RtpsWriterProxy) -> DdsResult<()> {
        struct MatchedWriterAdd {
            a_writer_proxy: RtpsWriterProxy,
        }

        impl Mail for MatchedWriterAdd {
            type Result = ();
        }

        impl MailHandler<MatchedWriterAdd> for DdsDataReader<RtpsStatefulReader> {
            fn handle(&mut self, mail: MatchedWriterAdd) -> <MatchedWriterAdd as Mail>::Result {
                self.matched_writer_add(mail.a_writer_proxy)
            }
        }

        self.send_blocking(MatchedWriterAdd { a_writer_proxy })
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        struct GetInstanceHandle;

        impl Mail for GetInstanceHandle {
            type Result = InstanceHandle;
        }

        impl MailHandler<GetInstanceHandle> for DdsDataReader<RtpsStatefulReader> {
            fn handle(&mut self, _mail: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
                self.guid().into()
            }
        }

        self.send_blocking(GetInstanceHandle)
    }

    pub fn read<Foo>(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de> + Send + 'static,
    {
        struct Read<Foo> {
            phantom: PhantomData<Foo>,
            max_samples: i32,
            sample_states: Vec<SampleStateKind>,
            view_states: Vec<ViewStateKind>,
            instance_states: Vec<InstanceStateKind>,
            specific_instance_handle: Option<InstanceHandle>,
        }

        impl<Foo> Mail for Read<Foo> {
            type Result = DdsResult<Vec<Sample<Foo>>>;
        }

        impl<Foo> MailHandler<Read<Foo>> for DdsDataReader<RtpsStatefulReader>
        where
            Foo: for<'de> DdsDeserialize<'de>,
        {
            fn handle(&mut self, mail: Read<Foo>) -> <Read<Foo> as Mail>::Result {
                self.read(
                    mail.max_samples,
                    &mail.sample_states,
                    &mail.view_states,
                    &mail.instance_states,
                    mail.specific_instance_handle,
                )
            }
        }

        self.send_blocking(Read {
            phantom: PhantomData,
            max_samples,
            sample_states: sample_states.to_vec(),
            view_states: view_states.to_vec(),
            instance_states: instance_states.to_vec(),
            specific_instance_handle,
        })?
    }

    pub fn as_discovered_reader_data(
        &self,
        topic_qos: TopicQos,
        subscriber_qos: SubscriberQos,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> DdsResult<DiscoveredReaderData> {
        struct AsDiscoveredReaderData {
            topic_qos: TopicQos,
            subscriber_qos: SubscriberQos,
            default_unicast_locator_list: Vec<Locator>,
            default_multicast_locator_list: Vec<Locator>,
        }

        impl Mail for AsDiscoveredReaderData {
            type Result = DiscoveredReaderData;
        }

        impl MailHandler<AsDiscoveredReaderData> for DdsDataReader<RtpsStatefulReader> {
            fn handle(
                &mut self,
                mail: AsDiscoveredReaderData,
            ) -> <AsDiscoveredReaderData as Mail>::Result {
                self.as_discovered_reader_data(
                    mail.topic_qos,
                    mail.subscriber_qos,
                    mail.default_unicast_locator_list,
                    mail.default_multicast_locator_list,
                )
            }
        }

        self.send_blocking(AsDiscoveredReaderData {
            topic_qos,
            subscriber_qos,
            default_unicast_locator_list,
            default_multicast_locator_list,
        })
    }
}

impl ActorAddress<DdsDataReader<RtpsStatelessReader>> {
    pub fn read<Foo>(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de> + Send + 'static,
    {
        struct Read<Foo> {
            phantom: PhantomData<Foo>,
            max_samples: i32,
            sample_states: Vec<SampleStateKind>,
            view_states: Vec<ViewStateKind>,
            instance_states: Vec<InstanceStateKind>,
            specific_instance_handle: Option<InstanceHandle>,
        }

        impl<Foo> Mail for Read<Foo> {
            type Result = DdsResult<Vec<Sample<Foo>>>;
        }

        impl<Foo> MailHandler<Read<Foo>> for DdsDataReader<RtpsStatelessReader>
        where
            Foo: for<'de> DdsDeserialize<'de>,
        {
            fn handle(&mut self, mail: Read<Foo>) -> <Read<Foo> as Mail>::Result {
                self.read(
                    mail.max_samples,
                    &mail.sample_states,
                    &mail.view_states,
                    &mail.instance_states,
                    mail.specific_instance_handle,
                )
            }
        }

        self.send_blocking(Read {
            phantom: PhantomData,
            max_samples,
            sample_states: sample_states.to_vec(),
            view_states: view_states.to_vec(),
            instance_states: instance_states.to_vec(),
            specific_instance_handle,
        })?
    }

    pub fn process_rtps_message(
        &self,
        message: RtpsMessageRead,
        reception_timestamp: Time,
    ) -> DdsResult<()> {
        struct ProcessRtpsMessage {
            message: RtpsMessageRead,
            reception_timestamp: Time,
        }

        impl Mail for ProcessRtpsMessage {
            type Result = ();
        }

        impl MailHandler<ProcessRtpsMessage> for DdsDataReader<RtpsStatelessReader> {
            fn handle(&mut self, mail: ProcessRtpsMessage) -> <ProcessRtpsMessage as Mail>::Result {
                self.process_rtps_message(mail.message, mail.reception_timestamp)
            }
        }

        self.send_blocking(ProcessRtpsMessage {
            message,
            reception_timestamp,
        })
    }
}
