use mockall::mock;
use rtps_pim::{
    behavior::writer::reader_locator::RtpsReaderLocatorAttributes, structure::types::Locator,
};

mock! {
    pub RtpsReaderLocator{}

    impl RtpsReaderLocatorAttributes for RtpsReaderLocator {
        fn locator(&self) -> Locator;
        fn expects_inline_qos(&self) -> bool;
    }
}
