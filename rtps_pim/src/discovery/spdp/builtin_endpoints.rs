use crate::{
    behavior::{
        reader::stateless_reader::RTPSStatelessReaderOperations, types::Duration,
        writer::stateless_writer::RTPSStatelessWriterOperations,
    },
    structure::types::{EntityId, GuidPrefix, ReliabilityKind, TopicKind, GUID},
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
    pub fn create<T: RTPSStatelessWriterOperations>(guid_prefix: GuidPrefix) -> T {
        let spdp_builtin_participant_writer_guid =
            GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER);

        T::new(
            spdp_builtin_participant_writer_guid,
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            &[],
            &[],
            true,
            Duration(0),
            Duration(0),
            Duration(0),
            None,
        )
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
