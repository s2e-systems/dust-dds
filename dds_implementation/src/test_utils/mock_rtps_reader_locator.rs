use mockall::mock;
use rtps_pim::{
    behavior::writer::reader_locator::RtpsReaderLocatorAttributes, structure::types::{Locator, SequenceNumber},
};

mock! {
    pub RtpsReaderLocator{}

    impl RtpsReaderLocatorAttributes for RtpsReaderLocator {
        type CacheChangeListType = Vec<SequenceNumber>;

        fn unsent_changes_mut(&mut self) -> &mut Vec<SequenceNumber>;
        fn requested_changes_mut(&mut self) -> &mut Vec<SequenceNumber>;
        fn locator(&self) -> Locator;
        fn expects_inline_qos(&self) -> bool;
    }
}
