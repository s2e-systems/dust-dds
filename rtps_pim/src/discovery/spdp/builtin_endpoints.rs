use crate::{
    behavior::{
        reader::stateless_reader::RTPSStatelessReaderOperations,
        types::Duration,
        writer::{
            reader_locator::RTPSReaderLocatorOperations,
            stateless_writer::{RTPSStatelessWriter, RTPSStatelessWriterOperations},
        },
    },
    structure::types::{EntityId, GuidPrefix, Locator, ReliabilityKind, TopicKind, GUID},
};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId = EntityId {
    entity_key: [0x00, 0x01, 0x00],
    entity_kind: crate::structure::types::EntityKind::BuiltInWriterWithKey,
};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER: EntityId = EntityId {
    entity_key: [0x00, 0x01, 0x00],
    entity_kind: crate::structure::types::EntityKind::BuiltInReaderWithKey,
};

pub struct SpdpBuiltinParticipantWriter;

impl SpdpBuiltinParticipantWriter {
    pub fn create<T>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        reader_locators: &[Locator],
    ) -> T
    where
        T: RTPSStatelessWriterOperations + RTPSStatelessWriter,
        T::ReaderLocatorType: RTPSReaderLocatorOperations,
    {
        let spdp_builtin_participant_writer_guid =
            GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER);

        let mut spdp_builtin_participant_writer = T::new(
            spdp_builtin_participant_writer_guid,
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            unicast_locator_list,
            multicast_locator_list,
            true,
            Duration(0),
            Duration(0),
            Duration(0),
            None,
        );

        for reader_locator in reader_locators {
            spdp_builtin_participant_writer
                .reader_locator_add(T::ReaderLocatorType::new(*reader_locator, false))
        }

        spdp_builtin_participant_writer
    }
}

pub struct SpdpBuiltinParticipantReader;

impl SpdpBuiltinParticipantReader {
    pub fn create<T: RTPSStatelessReaderOperations>(guid_prefix: GuidPrefix) -> T {
        let spdp_builtin_participant_reader_guid =
            GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER);
        T::new(
            spdp_builtin_participant_reader_guid,
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            &[],
            &[],
            Duration(0),
            Duration(0),
            false,
        )
    }
}
