//! LIVA Banking Harness Core Crate.
//!
//! Production-grade core banking, multi-bank statement parsers (VCB, TCB, ISO20022),
//! ERP invoice ingestion, 3-Tier deterministic reconciliation matching engine,
//! Circular 09/2020 Maker-Checker 4-eyes dual control, RFC 6962 binary Merkle tree
//! audit verification, and zero-egress security enforcement.

#![deny(clippy::float_arithmetic)]

pub mod compliance;
pub mod models;
pub mod parser;
pub mod reconciliation;
pub mod service;

pub use compliance::{
    BinaryMerkleTree, ChallengeToken, MakerCheckerEngine, MerkleInclusionProof, ProposalLifecycle,
    UserIdentity, UserRole, ZeroEgressNetfilter,
};
pub use models::{
    BankStatement, BankType, ErpDocument, MatchProposal, MatchType, ReconciliationStatus,
    StatementFormat, StatementStatus, TransactionRecord, TransactionType, parse_banking_date,
    parse_vietnamese_amount,
};
pub use parser::{
    BankStatementParser, ErpInvoiceParser, Iso20022XmlParser, ParserError, TcbCsvParser,
    VcbExcelParser, compute_sha256, sniff_and_parse,
};
pub use reconciliation::{FullReconciliationResult, ReconciliationEngine, SplitSolver, Tier1Matcher, Tier2Matcher};
pub use service::{BankingAction, IngestionService, PeriodCloseGate, PeriodCloseReport, RbacGate};
