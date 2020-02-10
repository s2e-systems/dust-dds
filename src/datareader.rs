use crate::cache::{HistoryCache};
use crate::types::{LocatorList};

pub struct Reader {
    // Endpoint 
    // topic_kind: 
    // reli
    unicast_locator_list: LocatorList,

    expects_inline_qos: bool,
    // heartbeat_response_delay: Duration,
    reader_cache: HistoryCache,
}

impl Reader
{
    pub fn read()
    {
        //unicast_locator_list[0].
    }
}