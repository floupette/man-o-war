use std::fmt::Debug;

/// Implemented to anything representing a possible `Section` of the `MainContent`.
///
/// Allows for coercing into `Vec<Box<dyn Body>>` while parsing said sections.
pub trait Body: Debug {}
