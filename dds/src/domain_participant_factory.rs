use rust_dds_api::{
    dcps_psm::{DomainId, StatusMask},
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::qos::DomainParticipantQos,
};
use rust_dds_rtps_implementation::domain_participant::DomainParticipant;
use rust_dds_rtps_implementation::transport::memory::MemoryTransport;
use rust_rtps::types::Locator;

/// The DomainParticipant object plays several roles:
/// - It acts as a container for all other Entity objects.
/// - It acts as factory for the Publisher, Subscriber, Topic, and MultiTopic Entity objects.
/// - It represents the participation of the application on a communication plane that isolates applications running on the
/// same set of physical computers from each other. A domain establishes a “virtual network” linking all applications that
/// share the same domainId and isolating them from applications running on different domains. In this way, several
/// independent distributed applications can coexist in the same physical network without interfering, or even being aware
/// of each other.
/// - It provides administration services in the domain, offering operations that allow the application to ‘ignore’ locally any
/// information about a given participant (ignore_participant), publication (ignore_publication), subscription
/// (ignore_subscription), or topic (ignore_topic).
///
/// The following sub clauses explain all the operations in detail.
/// The following operations may be called even if the DomainParticipant is not enabled. Other operations will have the value
/// NOT_ENABLED if called on a disabled DomainParticipant:
/// - Operations defined at the base-class level namely, set_qos, get_qos, set_listener, get_listener, and enable.
/// - Factory methods: create_topic, create_publisher, create_subscriber, delete_topic, delete_publisher,
/// delete_subscriber
/// - Operations that access the status: get_statuscondition
pub struct DomainParticipantFactory;

impl DomainParticipantFactory {
    /// This operation creates a new DomainParticipant object. The DomainParticipant signifies that the calling application intends
    /// to join the Domain identified by the domain_id argument.
    /// If the specified QoS policies are not consistent, the operation will fail and no DomainParticipant will be created.
    /// The special value PARTICIPANT_QOS_DEFAULT can be used to indicate that the DomainParticipant should be created
    /// with the default DomainParticipant QoS set in the factory. The use of this value is equivalent to the application obtaining the
    /// default DomainParticipant QoS by means of the operation get_default_participant_qos (2.2.2.2.2.6) and using the resulting
    /// QoS to create the DomainParticipant.
    /// In case of failure, the operation will return a ‘nil’ value (as specified by the platform).
    pub fn create_participant(
        domain_id: DomainId,
        qos: Option<DomainParticipantQos>,
        a_listener: Option<Box<dyn DomainParticipantListener>>,
        mask: StatusMask,
        //     enabled: bool,
    ) -> Option<DomainParticipant> {
        // let interface = "Wi-Fi";
        let unicast_locator = Locator::new(0, 7400, [1; 16]);
        let multicast_locator = Locator::new(0, 7400, [2; 16]);
        let userdata_transport = Box::new(MemoryTransport::new(unicast_locator, vec![multicast_locator]).unwrap());            
        let metatraffic_transport = Box::new(MemoryTransport::new(unicast_locator, vec![multicast_locator]).unwrap());            
        let qos = qos.unwrap_or_default();

        let rtps_participant = DomainParticipant::new(
            domain_id,
            qos.clone(),
            userdata_transport,
            metatraffic_transport,
            a_listener,
            mask,
        );

        // if enabled {
        //     new_participant.enable().ok()?;
        // }

        Some(rtps_participant)
    }

    // pub fn delete_participant(_a_participant: impl DomainParticipant) {}
}
