use std::marker::PhantomData;

use crate::{
    implementation::{
        dds::{dds_data_reader::DdsDataReader, dds_subscriber::DdsSubscriber},
        rtps::{stateful_reader::RtpsStatefulReader, types::Locator},
        utils::actor::{ActorAddress, Handler, Message},
    },
    infrastructure::{
        error::DdsResult,
        qos::{DataReaderQos, QosKind},
    },
    topic_definition::type_support::DdsDeserialize,
    DdsType,
};

pub struct CreateDataReader<Foo> {
    phantom: PhantomData<Foo>,
    topic_name: String,
    qos: QosKind<DataReaderQos>,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
}

impl<Foo> CreateDataReader<Foo> {
    pub fn new(
        topic_name: String,
        qos: QosKind<DataReaderQos>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> Self {
        Self {
            phantom: PhantomData,
            topic_name,
            qos,
            default_unicast_locator_list,
            default_multicast_locator_list,
        }
    }
}

impl<Foo> Message for CreateDataReader<Foo> {
    type Result = DdsResult<ActorAddress<DdsDataReader<RtpsStatefulReader>>>;
}

impl<Foo> Handler<CreateDataReader<Foo>> for DdsSubscriber
where
    Foo: DdsType + for<'de> DdsDeserialize<'de>,
{
    fn handle(
        &mut self,
        mail: CreateDataReader<Foo>,
    ) -> <CreateDataReader<Foo> as Message>::Result {
        self.create_datareader::<Foo>(
            mail.topic_name,
            mail.qos,
            mail.default_unicast_locator_list,
            mail.default_multicast_locator_list,
        )
    }
}

pub struct Enable;

impl Message for Enable {
    type Result = ();
}

impl Handler<Enable> for DdsSubscriber {
    fn handle(&mut self, _mail: Enable) -> <Enable as Message>::Result {
        self.enable().ok();
    }
}

pub struct DeleteContainedEntities;

impl Message for DeleteContainedEntities {
    type Result = ();
}

impl Handler<DeleteContainedEntities> for DdsSubscriber {
    fn handle(
        &mut self,
        _mail: DeleteContainedEntities,
    ) -> <DeleteContainedEntities as Message>::Result {
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
