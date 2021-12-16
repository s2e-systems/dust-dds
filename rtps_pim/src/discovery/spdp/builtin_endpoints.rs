use crate::{
    behavior::{
        reader::stateless_reader::RtpsStatelessReaderConstructor, types::DURATION_ZERO,
        writer::stateless_writer::RtpsStatelessWriterConstructor,
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
    pub fn create<W>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
    ) -> W
    where
        W: RtpsStatelessWriterConstructor,
    {
        let spdp_builtin_participant_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER);

        RtpsStatelessWriterConstructor::new(
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
        )
    }
}

pub struct SpdpBuiltinParticipantReader;

impl SpdpBuiltinParticipantReader {
    pub fn create<R>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
    ) -> R
    where
        R: RtpsStatelessReaderConstructor,
    {
        let spdp_builtin_participant_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER);

        RtpsStatelessReaderConstructor::new(
            spdp_builtin_participant_reader_guid,
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            unicast_locator_list,
            multicast_locator_list,
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        )
    }
}
