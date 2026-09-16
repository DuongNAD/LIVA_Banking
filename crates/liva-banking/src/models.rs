//! Core data models for LIVA Banking Engine.
//!
//! Enforces zero-floating-point calculations, strict balance invariants,
//! and regulatory compliant data types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    Debit,
    Credit,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Debit => write!(f, "DEBIT"),
            TransactionType::Credit => write!(f, "CREDIT"),
        }
    }
}

impl std::str::FromStr for TransactionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "DEBIT" | "DB" | "NO" | "GHI_NO" | "OUT" | "EXPENSE" => Ok(TransactionType::Debit),
            "CREDIT" | "CR" | "CO" | "GHI_CO" | "IN" | "INCOME" => Ok(TransactionType::Credit),
            _ => Err(format!("Unknown transaction type: '{s}'")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BankType {
    Vietcombank,
    Techcombank,
    Bidv,
    VietinBank,
    MbBank,
    Agribank,
    Iso20022Generic,
    Erp,
    Unknown,
}

impl BankType {
    pub fn as_code(&self) -> &'static str {
        match self {
            BankType::Vietcombank => "VCB",
            BankType::Techcombank => "TCB",
            BankType::Bidv => "BIDV",
            BankType::VietinBank => "CTG",
            BankType::MbBank => "MBB",
            BankType::Agribank => "VBA",
            BankType::Iso20022Generic => "ISO20022",
            BankType::Erp => "ERP",
            BankType::Unknown => "UNKNOWN",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code.trim().to_uppercase().as_str() {
            "VCB" | "VIETCOMBANK" => BankType::Vietcombank,
            "TCB" | "TECHCOMBANK" => BankType::Techcombank,
            "BIDV" => BankType::Bidv,
            "CTG" | "VIETINBANK" => BankType::VietinBank,
            "MB" | "MBB" | "MBBANK" => BankType::MbBank,
            "AGRI" | "AGRIBANK" | "VBA" => BankType::Agribank,
            "ISO20022" | "CAMT" => BankType::Iso20022Generic,
            "ERP" => BankType::Erp,
            _ => BankType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatementFormat {
    Excel,
    Csv,
    Pdf,
    Xml,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatementStatus {
    Uploaded,
    Validating,
    VerifiedBalanced,
    QuarantinedUnbalanced,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionRecord {
    pub row_id: usize,
    pub tx_date: i64,
    pub value_date: i64,
    pub doc_ref: Option<String>,
    pub tx_type: TransactionType,
    pub amount: u64,
    pub balance_after: Option<u64>,
    pub counterparty_account: Option<String>,
    pub counterparty_name: Option<String>,
    pub counterparty_bank: Option<String>,
    pub narration: String,
    pub raw_ref: Option<String>,
}

impl TransactionRecord {
    pub fn new(
        row_id: usize,
        tx_date: i64,
        value_date: i64,
        doc_ref: Option<String>,
        tx_type: TransactionType,
        amount: u64,
        balance_after: Option<u64>,
        counterparty_account: Option<String>,
        counterparty_name: Option<String>,
        counterparty_bank: Option<String>,
        narration: String,
    ) -> Self {
        Self {
            row_id,
            tx_date,
            value_date,
            doc_ref,
            tx_type,
            amount,
            balance_after,
            counterparty_account,
            counterparty_name,
            counterparty_bank,
            narration,
            raw_ref: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankStatement {
    pub bank_code: String,
    pub bank_type: BankType,
    pub format: StatementFormat,
    pub account_number: Option<String>,
    pub account_name: Option<String>,
    pub opening_balance: Option<u64>,
    pub closing_balance: Option<u64>,
    pub total_credit: u64,
    pub total_debit: u64,
    pub balance_checksum_passed: bool,
    pub status: StatementStatus,
    pub file_hash_sha256: Option<String>,
    pub transactions: Vec<TransactionRecord>,
}

impl BankStatement {
    /// Verifies the balance invariant:
    /// Ending Balance = Opening Balance + Total Credit (Inflow) - Total Debit (Outflow)
    pub fn verify_balance_invariant(&mut self) -> bool {
        let mut total_cr: u128 = 0;
        let mut total_db: u128 = 0;

        for tx in &self.transactions {
            match tx.tx_type {
                TransactionType::Credit => total_cr += tx.amount as u128,
                TransactionType::Debit => total_db += tx.amount as u128,
            }
        }

        self.total_credit = total_cr as u64;
        self.total_debit = total_db as u64;

        if let (Some(open), Some(close)) = (self.opening_balance, self.closing_balance) {
            let expected_close = (open as u128) + total_cr - total_db;
            let passed = expected_close == (close as u128);
            self.balance_checksum_passed = passed;
            self.status = if passed {
                StatementStatus::VerifiedBalanced
            } else {
                StatementStatus::QuarantinedUnbalanced
            };
            passed
        } else {
            // Cannot mathematically verify if opening/closing balances are missing
            self.balance_checksum_passed = false;
            self.status = StatementStatus::QuarantinedUnbalanced;
            false
        }
    }
}

/// Standardized open ERP Document / Invoice for reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErpDocument {
    pub id: String,
    pub voucher_no: String,
    pub invoice_no: Option<String>,
    pub partner_code: String,
    pub partner_name: String,
    pub doc_date: i64,
    pub due_date: Option<i64>,
    pub doc_type: TransactionType,
    pub total_amount: u64,
    pub open_amount: u64,
    pub currency: String,
    pub description: String,
    pub version: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReconciliationStatus {
    Unmatched,
    Matched,
    Discrepancy,
    PendingHitl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchType {
    Exact1To1,
    FuzzyHeuristic,
    CompositeSplit,
    ManualHitl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchProposal {
    pub proposal_id: String,
    pub bank_tx_id: usize,
    pub erp_doc_ids: Vec<String>,
    pub match_type: MatchType,
    pub confidence_score: u32, // Scaled by 100, e.g. 100 = 1.00, 85 = 0.85
    pub matched_amount: u64,
    pub fee_amount: u64,
    pub fee_account: Option<String>, // TK 6425
    pub notes: String,
    pub status: String, // "DRAFT", "SUBMITTED", "APPROVED", "REJECTED", "STALE"
    pub version: u64,
    pub created_at: i64,
}

/// Helper function to parse Vietnamese formatted amounts (e.g. "15.000.000,00" or "15,000,000.00")
pub fn parse_vietnamese_amount(s: &str) -> Option<u64> {
    let clean = s.trim().replace("VND", "").replace("đ", "").replace(" ", "");
    if clean.is_empty() {
        return None;
    }

    // Determine decimal separator
    let has_dot = clean.contains('.');
    let has_comma = clean.contains(',');

    let normalized = if has_dot && has_comma {
        let last_dot = clean.rfind('.').unwrap_or(0);
        let last_comma = clean.rfind(',').unwrap_or(0);
        if last_comma > last_dot {
            // "15.000.000,00" -> drop decimal part
            clean[..last_comma].replace('.', "")
        } else {
            // "15,000,000.00" -> drop decimal part
            clean[..last_dot].replace(',', "")
        }
    } else if has_dot {
        let dot_count = clean.chars().filter(|&c| c == '.').count();
        if dot_count > 1 {
            // Thousand separator
            clean.replace('.', "")
        } else {
            let parts: Vec<&str> = clean.split('.').collect();
            if parts.len() == 2 && parts[1].len() == 3 {
                // E.g. "150.000" VND
                format!("{}{}", parts[0], parts[1])
            } else {
                parts[0].to_string()
            }
        }
    } else if has_comma {
        let comma_count = clean.chars().filter(|&c| c == ',').count();
        if comma_count > 1 {
            clean.replace(',', "")
        } else {
            let parts: Vec<&str> = clean.split(',').collect();
            if parts.len() == 2 && parts[1].len() == 3 {
                format!("{}{}", parts[0], parts[1])
            } else {
                parts[0].to_string()
            }
        }
    } else {
        clean
    };

    normalized.parse::<u64>().ok()
}

/// Parses date string into Unix epoch seconds
pub fn parse_banking_date(s: &str) -> Option<i64> {
    let clean = s.trim();
    if clean.is_empty() {
        return None;
    }

    // Common VN formats: DD/MM/YYYY, YYYY-MM-DD, DD-MM-YYYY
    let parts: Vec<&str> = if clean.contains('/') {
        clean.split('/').collect()
    } else if clean.contains('-') {
        clean.split('-').collect()
    } else {
        return None;
    };

    if parts.len() < 3 {
        return None;
    }

    let (day, month, year) = if parts[0].len() == 4 {
        // YYYY-MM-DD
        (parts[2].parse::<i64>().ok()?, parts[1].parse::<i64>().ok()?, parts[0].parse::<i64>().ok()?)
    } else {
        // DD/MM/YYYY
        (parts[0].parse::<i64>().ok()?, parts[1].parse::<i64>().ok()?, parts[2].parse::<i64>().ok()?)
    };

    // Approximate Unix timestamp without heavy chrono dep
    // Days since Unix epoch (1970-01-01)
    let y = year - 1970;
    let leap_years = (year - 1969) / 4;
    let days_in_months = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let m_idx = ((month - 1).clamp(0, 11)) as usize;
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let leap_adj = if is_leap && month > 2 { 1 } else { 0 };

    let total_days = y * 365 + leap_years + days_in_months[m_idx] + leap_adj + (day - 1);
    Some(total_days * 86400)
}
