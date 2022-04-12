use mockall::mock;
use rtps_pim::{
    behavior::writer::reader_proxy::RtpsReaderProxyAttributes,
    structure::types::{EntityId, Guid, Locator},
};

mock! {
    pub RtpsReaderProxy {}

    impl RtpsReaderProxyAttributes for RtpsReaderProxy {
        type ChangeForReaderType = ();
        fn remote_reader_guid(&self) -> Guid;
        fn remote_group_entity_id(&self) -> EntityId;
        fn unicast_locator_list(&self) -> &[Locator];
        fn multicast_locator_list(&self) -> &[Locator];
        fn changes_for_reader(&self) -> &[()];
        fn expects_inline_qos(&self) -> bool;
        fn is_active(&self) -> bool;
    }
}
