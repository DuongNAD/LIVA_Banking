//! Statement Template DSL module.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

pub mod schema;
pub mod validator;

pub use schema::*;
pub use validator::validate_template;
