/// Contains the classes needed to handle the data, key and other information for being
/// able to publish and subscribe structs using DustDDS.
pub mod dds_data;

/// Contains the traits needed to serialize and deserialize
pub mod serde;

// Convinience re-export of the most commonly use DdsType trait
pub use dds_data::DdsType;
