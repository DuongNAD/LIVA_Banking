//! Service module: RBAC, Ingestion Pipeline, and Period Close Gate.

pub mod ingestion;
pub mod period_close;
pub mod rbac;

pub use ingestion::IngestionService;
pub use period_close::{PeriodCloseGate, PeriodCloseReport, PeriodStatus};
pub use rbac::{BankingAction, RbacGate, UserPermissionGrant};
