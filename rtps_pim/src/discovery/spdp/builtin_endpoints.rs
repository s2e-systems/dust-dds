use crate::{
    behavior::{
        reader::stateless_reader::RtpsStatelessReaderOperations, types::DURATION_ZERO,
        writer::stateless_writer::RtpsStatelessWriterOperations,
    },
    structure::types::{
        EntityId, Guid, GuidPrefix, Locator, ReliabilityKind, TopicKind, BUILT_IN_READER_WITH_KEY,
        BUILT_IN_WRITER_WITH_KEY,
    },
};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_READER_WITH_KEY);

pub struct SpdpBuiltinParticipantWriter;

impl SpdpBuiltinParticipantWriter {
    pub fn create<T>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        reader_locators: &[Locator],
    ) -> T
    where
        T: RtpsStatelessWriterOperations,
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
            spdp_builtin_participant_writer.reader_locator_add(*reader_locator)
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
