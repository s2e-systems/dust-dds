use std::collections::HashMap;

use crate::cache::{ReaderHistoryCache};
use crate::types::{LocatorList, GUID, TopicKind, ReliabilityKind, EntityId, Duration};
use crate::proxy::{WriterProxy};
use crate::endpoint::{Endpoint};

pub struct StatefulReader<'a> {
    reader : Reader,
    matched_writers : HashMap<GUID, WriterProxy<'a>>,
}

impl<'a> StatefulReader<'a>
{
    pub fn new(endpoint : Endpoint,
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration : Duration,
        expects_inline_qos: bool) -> Self
    {
        StatefulReader
        {
            reader : Reader::new(endpoint, heartbeat_response_delay, heartbeat_suppression_duration, expects_inline_qos),
            matched_writers : HashMap::new(),
        }
    }

    pub fn matched_writer_add(&mut self, a_writer_proxy : WriterProxy<'a>)
    {
        self.matched_writers.insert(a_writer_proxy.remote_writer_guid(), a_writer_proxy);
    }
    
    pub fn matched_writer_remove(&mut self, a_writer_proxy : WriterProxy)
    {
        self.matched_writers.remove(&a_writer_proxy.remote_writer_guid());
    }

    pub fn matched_writer_lookup(&self, a_writer_guid : GUID) -> Option<&WriterProxy>
    {
        self.matched_writers.get(&a_writer_guid)
    }
}

pub struct Reader {
    endpoint : Endpoint,
    pub heartbeat_response_delay: Duration,
    pub heartbeat_suppression_duration : Duration,
    pub reader_cache: ReaderHistoryCache,
    expects_inline_qos: bool,
}

impl Reader
{
    pub fn new(endpoint : Endpoint,
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration : Duration,
        expects_inline_qos: bool) -> Self {
            Reader{
                endpoint, heartbeat_response_delay, heartbeat_suppression_duration, reader_cache : ReaderHistoryCache::new(), expects_inline_qos
            }
    }
}