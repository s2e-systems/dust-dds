/// Enumeration of the different representations defined by the RTPS standard and supported by DustDDS.
pub enum CdrRepresentationKind {
    CdrLe,
    CdrBe,
    PlCdrBe,
    PlCdrLe,
}

/// This trait defines the representation to be used by the type when serializing and deserializing.
///
/// When used in combination with [`serde::Serialize`] and [`serde::Deserialize`] a blanket implementation
/// for the [`DdsSerializeData`] and [`DdsDeserialize`] traits is provided that uses the Cdr serializer and
/// is conformant with the CDR format as specified in the RTPS standard.
pub trait CdrRepresentation {
    const REPRESENTATION: CdrRepresentationKind;
}
