use crate::{
    behavior::{
        reader::stateless_reader::RtpsStatelessReaderOperations,
        types::DURATION_ZERO,
        writer::{
            reader_locator::RtpsReaderLocatorOperations,
            stateless_writer::{RtpsStatelessWriter, RtpsStatelessWriterOperations},
        },
    },
    structure::types::{EntityId, Guid, GuidPrefix, Locator, ReliabilityKind, TopicKind},
};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId = EntityId::new(
    [0x00, 0x01, 0x00],
    crate::structure::types::EntityKind::BuiltInWriterWithKey,
);

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER: EntityId = EntityId::new(
    [0x00, 0x01, 0x00],
    crate::structure::types::EntityKind::BuiltInReaderWithKey,
);

pub struct SpdpBuiltinParticipantWriter;

impl SpdpBuiltinParticipantWriter {
    pub fn create<T>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        reader_locators: &[Locator],
    ) -> T
    where
        T: RtpsStatelessWriterOperations + RtpsStatelessWriter,
        T::ReaderLocatorType: RtpsReaderLocatorOperations,
    {
        let spdp_builtin_participant_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER);

        let mut spdp_builtin_participant_writer = T::new(
            spdp_builtin_participant_writer_guid,
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            unicast_locator_list,
            multicast_locator_list,
            true,
            DURATION_ZERO,
            DURATION_ZERO,
            DURATION_ZERO,
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
    pub fn create<T: RtpsStatelessReaderOperations>(guid_prefix: GuidPrefix) -> T {
        let spdp_builtin_participant_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER);
        T::new(
            spdp_builtin_participant_reader_guid,
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            &[],
            &[],
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        )
    }
}
