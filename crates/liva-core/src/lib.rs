//! LIVA Banking — `liva-core` Crate
//!
//! Core financial primitives, zero-float scaled integer money arithmetic,
//! currency definitions, and inter-period balance continuity invariant verification.

#![deny(clippy::float_arithmetic)]

pub use liva_money::{Currency, Money, MoneyError};

pub mod continuity;
pub use continuity::{ContinuityError, PeriodBalance, verify_period_sequence, verify_period_transition};
