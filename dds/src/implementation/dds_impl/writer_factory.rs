use crate::{
    implementation::rtps::{
        endpoint::RtpsEndpoint,
        group::RtpsGroup,
        stateful_writer::RtpsStatefulWriter,
        types::{
            EntityId, EntityKey, Guid, Locator, TopicKind, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
        writer::RtpsWriter,
    },
    infrastructure::{
        error::DdsResult,
        qos::{DataWriterQos, QosKind},
        time::{Duration, DURATION_ZERO},
    },
};

pub struct WriterFactory {
    user_defined_data_writer_counter: u8,
    default_datawriter_qos: DataWriterQos,
}

impl WriterFactory {
    pub fn new() -> Self {
        Self {
            user_defined_data_writer_counter: 0,
            default_datawriter_qos: Default::default(),
        }
    }

    pub fn get_default_datawriter_qos(&self) -> &DataWriterQos {
        &self.default_datawriter_qos
    }

    pub fn set_default_datawriter_qos(&mut self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_datawriter_qos = DataWriterQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                self.default_datawriter_qos = q;
            }
        }

        Ok(())
    }

    pub fn create_writer(
        &mut self,
        rtps_group: &RtpsGroup,
        has_key: bool,
        qos: QosKind<DataWriterQos>,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
        data_max_size_serialized: usize,
    ) -> DdsResult<RtpsStatefulWriter> {
        let qos = match qos {
            QosKind::Default => self.default_datawriter_qos.clone(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;

        let guid = self.create_unique_writer_guid(rtps_group, has_key);

        let topic_kind = match has_key {
            true => TopicKind::WithKey,
            false => TopicKind::NoKey,
        };

        Ok(RtpsStatefulWriter::new(RtpsWriter::new(
            RtpsEndpoint::new(
                guid,
                topic_kind,
                default_unicast_locator_list,
                default_multicast_locator_list,
            ),
            true,
            Duration::new(0, 200_000_000),
            DURATION_ZERO,
            DURATION_ZERO,
            data_max_size_serialized,
            qos,
        )))
    }

    fn create_unique_writer_guid(&mut self, rtps_group: &RtpsGroup, has_key: bool) -> Guid {
        let entity_kind = match has_key {
            true => USER_DEFINED_WRITER_WITH_KEY,
            false => USER_DEFINED_WRITER_NO_KEY,
        };

        let entity_key = EntityKey::new([
            <[u8; 3]>::from(rtps_group.guid().entity_id().entity_key())[0],
            self.user_defined_data_writer_counter,
            0,
        ]);

        self.user_defined_data_writer_counter += 1;

        let entity_id = EntityId::new(entity_key, entity_kind);

        Guid::new(rtps_group.guid().prefix(), entity_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps::types::{GuidPrefix, USER_DEFINED_WRITER_GROUP};

    use super::*;

    #[test]
    fn each_created_writer_has_different_guid() {
        let mut factory = WriterFactory::new();
        let rtps_group = RtpsGroup::new(Guid::new(
            GuidPrefix::new([1; 12]),
            EntityId::new(EntityKey::new([1; 3]), USER_DEFINED_WRITER_GROUP),
        ));
        let has_key = true;
        let default_unicast_locator_list = &[];
        let default_multicast_locator_list = &[];
        let data_max_size_serialized = 5000;

        let dw1 = factory
            .create_writer(
                &rtps_group,
                has_key,
                QosKind::Default,
                default_unicast_locator_list,
                default_multicast_locator_list,
                data_max_size_serialized,
            )
            .unwrap();

        let dw2 = factory
            .create_writer(
                &rtps_group,
                has_key,
                QosKind::Default,
                default_unicast_locator_list,
                default_multicast_locator_list,
                data_max_size_serialized,
            )
            .unwrap();

        assert_ne!(dw1.guid(), dw2.guid());
    }
}
