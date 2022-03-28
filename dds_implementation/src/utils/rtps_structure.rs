use rtps_pim::{
    behavior::{
        reader::{
            reader::RtpsReaderAttributes, stateful_reader::RtpsStatefulReaderConstructor,
            stateless_reader::RtpsStatelessReaderConstructor,
        },
        writer::{
            stateful_writer::RtpsStatefulWriterConstructor,
            stateless_writer::RtpsStatelessWriterConstructor,
            writer::{RtpsWriterAttributes, RtpsWriterOperations},
        },
    },
    structure::{
        cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
        entity::RtpsEntityAttributes,
        group::RtpsGroupConstructor,
        history_cache::{
            RtpsHistoryCacheAttributes, RtpsHistoryCacheConstructor, RtpsHistoryCacheOperations,
        },
        participant::{RtpsParticipantAttributes, RtpsParticipantConstructor},
    },
};

pub trait RtpsStructure {
    type Group: RtpsGroupConstructor + RtpsEntityAttributes;
    type Participant: RtpsParticipantConstructor + RtpsParticipantAttributes + RtpsEntityAttributes;

    type StatelessWriter: RtpsStatelessWriterConstructor
        + RtpsWriterOperations<
            DataType = Vec<u8>,
            ParameterListType = Vec<u8>,
            CacheChangeType = Self::CacheChange,
        > + RtpsWriterAttributes<HistoryCacheType = Self::HistoryCache>;
    type StatefulWriter: RtpsStatefulWriterConstructor
        + RtpsWriterOperations<
            DataType = Vec<u8>,
            ParameterListType = Vec<u8>,
            CacheChangeType = Self::CacheChange,
        > + RtpsWriterAttributes<HistoryCacheType = Self::HistoryCache>
        + RtpsHistoryCacheOperations<CacheChangeType = Self::CacheChange>;

    type StatelessReader: RtpsStatelessReaderConstructor
        + RtpsReaderAttributes<HistoryCacheType = Self::HistoryCache>;
    type StatefulReader: RtpsStatefulReaderConstructor
        + RtpsReaderAttributes<HistoryCacheType = Self::HistoryCache>;
    type HistoryCache: RtpsHistoryCacheConstructor
        + RtpsHistoryCacheAttributes<CacheChangeType = Self::CacheChange>
        + RtpsHistoryCacheOperations<CacheChangeType = Self::CacheChange>;
    type CacheChange: RtpsCacheChangeConstructor + RtpsCacheChangeAttributes<DataType = [u8]>;
}
