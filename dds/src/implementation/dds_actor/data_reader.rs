use std::marker::PhantomData;

use crate::{
    implementation::{
        dds::dds_data_reader::DdsDataReader,
        rtps::{
            messages::overall_structure::RtpsMessageRead, stateful_reader::RtpsStatefulReader,
            stateless_reader::RtpsStatelessReader, writer_proxy::RtpsWriterProxy,
        },
        utils::actor::{ActorAddress, Mail, MailHandler},
    },
    infrastructure::{error::DdsResult, instance::InstanceHandle},
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
    pub fn process_rtps_message(&self, message: RtpsMessageRead) -> DdsResult<()> {
        struct ProcessRtpsMessage {
            message: RtpsMessageRead,
        }

        impl Mail for ProcessRtpsMessage {
            type Result = ();
        }

        impl MailHandler<ProcessRtpsMessage> for DdsDataReader<RtpsStatefulReader> {
            fn handle(&mut self, mail: ProcessRtpsMessage) -> <ProcessRtpsMessage as Mail>::Result {
                self.process_rtps_message(mail.message)
            }
        }

        self.send_blocking(ProcessRtpsMessage { message })
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

    pub fn process_rtps_message(&self, message: RtpsMessageRead) -> DdsResult<()> {
        struct ProcessRtpsMessage {
            message: RtpsMessageRead,
        }

        impl Mail for ProcessRtpsMessage {
            type Result = ();
        }

        impl MailHandler<ProcessRtpsMessage> for DdsDataReader<RtpsStatelessReader> {
            fn handle(&mut self, mail: ProcessRtpsMessage) -> <ProcessRtpsMessage as Mail>::Result {
                self.process_rtps_message(mail.message)
            }
        }

        self.send_blocking(ProcessRtpsMessage { message })
    }
}
