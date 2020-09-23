use rust_dds_interface::types::ReturnCode;
use crate::domain::DomainParticipant;

/// The TypeSupport interface is an abstract interface that has to be specialized for each concrete type that will be used by the application.
/// It is required that each implementation of the Service provides an automatic means to generate this type-specific class from a
/// description of the type (using IDL for example in the OMG IDL mapping). A TypeSupport must be registered using the
/// register_type operation on this type-specific class before it can be used to create Topic objects
struct TypeSupport {}

impl TypeSupport {
    /// This operation allows an application to communicate to the Service the existence of a data type. The generated implementation
    /// of that operation embeds all the knowledge that has to be communicated to the middleware in order to make it able to manage
    /// the contents of data of that data type. This includes in particular the key definition that will allow the Service to distinguish
    /// different instances of the same type.
    /// It is a pre-condition error to use the same type_name to register two different TypeSupport with the same DomainParticipant.
    /// If an application attempts this, the operation will fail and return PRECONDITION_NOT_MET. However, it is allowed to
    /// register the same TypeSupport multiple times with a DomainParticipant using the same or different values for the type_name.
    /// If register_type is called multiple times on the same TypeSupport with the same DomainParticipant and type_name the
    /// second (and subsequent) registrations are ignored but the operation returns OK.
    /// The application may pass nil as the value for the type_name. In this case the default type-name as defined by the TypeSupport
    /// (i.e., the value returned by the get_type_name operation) will be used.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET and OUT_OF_RESOURCES
    pub fn register_type(
        _participant: DomainParticipant,
        _type_name: String,
    ) -> ReturnCode<()> {
        todo!()
    }

    /// This operation returns the default name for the data-type represented by the TypeSupport.
    pub fn get_type_name() -> String {
        todo!()
    }
}

