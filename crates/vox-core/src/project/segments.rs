//! Persistent speech alternatives, explicit selection and exact local assembly.
mod assemble;
mod model;
mod plan;
mod render;
pub(crate) use model::validate_records;
pub use model::*;
#[cfg(test)]
mod tests;
