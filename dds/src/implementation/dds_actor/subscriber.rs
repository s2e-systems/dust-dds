use std::marker::PhantomData;

use crate::{
    implementation::{
        dds::{dds_data_reader::DdsDataReader, dds_subscriber::DdsSubscriber},
        rtps::{stateful_reader::RtpsStatefulReader, types::Locator},
        utils::actor::{ActorAddress, Mail, MailHandler},
    },
    infrastructure::{
        error::DdsResult,
        qos::{DataReaderQos, QosKind},
    },
    topic_definition::type_support::DdsDeserialize,
    DdsType,
};

impl ActorAddress<DdsSubscriber> {
    pub fn create_datareader<Foo>(
        &self,
        topic_name: String,
        qos: QosKind<DataReaderQos>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> DdsResult<ActorAddress<DdsDataReader<RtpsStatefulReader>>>
    where
        Foo: DdsType + for<'de> DdsDeserialize<'de> + Send + 'static,
    {
        struct CreateDataReader<Foo> {
            phantom: PhantomData<Foo>,
            topic_name: String,
            qos: QosKind<DataReaderQos>,
            default_unicast_locator_list: Vec<Locator>,
            default_multicast_locator_list: Vec<Locator>,
        }

        impl<Foo> Mail for CreateDataReader<Foo> {
            type Result = DdsResult<ActorAddress<DdsDataReader<RtpsStatefulReader>>>;
        }

        impl<Foo> MailHandler<CreateDataReader<Foo>> for DdsSubscriber
        where
            Foo: DdsType + for<'de> DdsDeserialize<'de>,
        {
            fn handle(
                &mut self,
                mail: CreateDataReader<Foo>,
            ) -> <CreateDataReader<Foo> as Mail>::Result {
                self.create_datareader::<Foo>(
                    mail.topic_name,
                    mail.qos,
                    mail.default_unicast_locator_list,
                    mail.default_multicast_locator_list,
                )
            }
        }

        self.send_blocking(CreateDataReader {
            phantom: PhantomData::<Foo>,
            topic_name,
            qos,
            default_unicast_locator_list,
            default_multicast_locator_list,
        })?
    }

    pub fn enable(&self) -> DdsResult<()> {
        struct Enable;

        impl Mail for Enable {
            type Result = ();
        }

        impl MailHandler<Enable> for DdsSubscriber {
            fn handle(&mut self, _mail: Enable) -> <Enable as Mail>::Result {
                self.enable().ok();
            }
        }

        self.send_blocking(Enable)
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        struct DeleteContainedEntities;

        impl Mail for DeleteContainedEntities {
            type Result = ();
        }

        impl MailHandler<DeleteContainedEntities> for DdsSubscriber {
            fn handle(
                &mut self,
                _mail: DeleteContainedEntities,
            ) -> <DeleteContainedEntities as Mail>::Result {
                // todo!()

                // for data_reader in user_defined_subscriber.stateful_data_reader_drain() {
                //     if data_reader.is_enabled() {
                //         self.announce_sender
                //             .try_send(AnnounceKind::DeletedDataReader(
                //                 data_reader.get_instance_handle(),
                //             ))
                //             .ok();
                //     }
                // }
            }
        }
        self.send_blocking(DeleteContainedEntities)
    }
}
