use crate::{
    implementation::rtps::{
        endpoint::RtpsEndpoint,
        group::RtpsGroup,
        reader::RtpsReader,
        stateful_reader::RtpsStatefulReader,
        types::{
            EntityId, EntityKey, Guid, Locator, TopicKind, USER_DEFINED_READER_NO_KEY,
            USER_DEFINED_READER_WITH_KEY,
        },
    },
    infrastructure::{
        error::DdsResult,
        qos::{DataReaderQos, QosKind},
        time::DURATION_ZERO,
    },
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

pub struct ReaderFactory {
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
}

impl ReaderFactory {
    pub fn new() -> Self {
        Self {
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: Default::default(),
        }
    }

    pub fn get_default_datareader_qos(&self) -> &DataReaderQos {
        &self.default_data_reader_qos
    }

    pub fn set_default_datareader_qos(&mut self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_data_reader_qos = DataReaderQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                self.default_data_reader_qos = q;
            }
        }
        Ok(())
    }

    pub fn create_reader<Foo>(
        &mut self,
        rtps_group: &RtpsGroup,
        has_key: bool,
        qos: QosKind<DataReaderQos>,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
    ) -> DdsResult<RtpsStatefulReader>
    where
        Foo: DdsType + for<'de> DdsDeserialize<'de>,
    {
        let qos = match qos {
            QosKind::Default => self.default_data_reader_qos.clone(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;

        let guid = self.create_unique_reader_guid(rtps_group, has_key);

        let topic_kind = match has_key {
            true => TopicKind::WithKey,
            false => TopicKind::NoKey,
        };

        Ok(RtpsStatefulReader::new(RtpsReader::new::<Foo>(
            RtpsEndpoint::new(
                guid,
                topic_kind,
                default_unicast_locator_list,
                default_multicast_locator_list,
            ),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
            qos,
        )))
    }

    fn create_unique_reader_guid(&mut self, rtps_group: &RtpsGroup, has_key: bool) -> Guid {
        let entity_kind = match has_key {
            true => USER_DEFINED_READER_WITH_KEY,
            false => USER_DEFINED_READER_NO_KEY,
        };

        let entity_key = EntityKey::new([
            <[u8; 3]>::from(rtps_group.guid().entity_id().entity_key())[0],
            self.user_defined_data_reader_counter,
            0,
        ]);

        self.user_defined_data_reader_counter += 1;

        let entity_id = EntityId::new(entity_key, entity_kind);

        Guid::new(rtps_group.guid().prefix(), entity_id)
    }
}
