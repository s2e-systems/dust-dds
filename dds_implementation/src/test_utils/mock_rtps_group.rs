use mockall::mock;
use rtps_pim::structure::{entity::RtpsEntityAttributes, group::RtpsGroupConstructor, types::Guid};

mock! {
    pub RtpsGroup{}

    impl RtpsEntityAttributes for RtpsGroup {
        fn guid(&self) -> Guid;
    }
}

impl RtpsGroupConstructor for MockRtpsGroup {
    fn new(_guid: Guid) -> Self {
        MockRtpsGroup::new()
    }
}
