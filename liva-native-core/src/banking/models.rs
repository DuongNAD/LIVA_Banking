//! Core data types and models for LIVA Banking Reconciliation Engine.

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReconciliationStatus {
    Unmatched,
    Matched,
    Discrepancy,
    PendingHitl,
}

impl std::fmt::Display for ReconciliationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReconciliationStatus::Unmatched => write!(f, "UNMATCHED"),
            ReconciliationStatus::Matched => write!(f, "MATCHED"),
            ReconciliationStatus::Discrepancy => write!(f, "DISCREPANCY"),
            ReconciliationStatus::PendingHitl => write!(f, "PENDING_HITL"),
        }
    }
}

impl std::str::FromStr for ReconciliationStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "UNMATCHED" => Ok(ReconciliationStatus::Unmatched),
            "MATCHED" => Ok(ReconciliationStatus::Matched),
            "DISCREPANCY" => Ok(ReconciliationStatus::Discrepancy),
            "PENDING_HITL" | "PENDING" => Ok(ReconciliationStatus::PendingHitl),
            _ => Err(format!("Unknown reconciliation status: '{s}'")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchType {
    Exact1To1,
    FuzzyHeuristic,
    CompositeSplit,
    ManualHitl,
}

impl std::fmt::Display for MatchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchType::Exact1To1 => write!(f, "EXACT_1_TO_1"),
            MatchType::FuzzyHeuristic => write!(f, "FUZZY_HEURISTIC"),
            MatchType::CompositeSplit => write!(f, "COMPOSITE_SPLIT"),
            MatchType::ManualHitl => write!(f, "MANUAL_HITL"),
        }
    }
}

impl std::str::FromStr for MatchType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "EXACT_1_TO_1" | "EXACT" => Ok(MatchType::Exact1To1),
            "FUZZY_HEURISTIC" | "FUZZY" => Ok(MatchType::FuzzyHeuristic),
            "COMPOSITE_SPLIT" | "SPLIT" => Ok(MatchType::CompositeSplit),
            "MANUAL_HITL" | "HITL" => Ok(MatchType::ManualHitl),
            _ => Err(format!("Unknown match type: '{s}'")),
        }
    }
}

/// Identification of officially supported Vietnamese banks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BankType {
    Vietcombank,
    Techcombank,
    Bidv,
    VietinBank,
    MbBank,
    Agribank,
    Citi,
    Hsbc,
    StandardChartered,
    Shinhan,
    Iso20022Generic,
    Unknown,
}

impl BankType {
    pub fn as_code(&self) -> &'static str {
        match self {
            BankType::Vietcombank => "VCB",
            BankType::Techcombank => "TCB",
            BankType::Bidv => "BIDV",
            BankType::VietinBank => "CTG",
            BankType::MbBank => "MB",
            BankType::Agribank => "VBA",
            BankType::Citi => "CITI",
            BankType::Hsbc => "HSBC",
            BankType::StandardChartered => "SCB_INTL",
            BankType::Shinhan => "SHINHAN",
            BankType::Iso20022Generic => "ISO20022",
            BankType::Unknown => "UNKNOWN",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code.trim().to_uppercase().as_str() {
            "VCB" | "VIETCOMBANK" => BankType::Vietcombank,
            "TCB" | "TECHCOMBANK" => BankType::Techcombank,
            "BIDV" => BankType::Bidv,
            "CTG" | "VIETINBANK" | "ICB" => BankType::VietinBank,
            "MB" | "MBB" | "MBBANK" => BankType::MbBank,
            "AGRI" | "AGRIBANK" | "VBA" => BankType::Agribank,
            "CITI" | "CITIBANK" => BankType::Citi,
            "HSBC" => BankType::Hsbc,
            "SCB_INTL" | "SCB" | "STANDARD_CHARTERED" => BankType::StandardChartered,
            "SHINHAN" => BankType::Shinhan,
            "ISO20022" | "ISO_20022" | "CAMT" => BankType::Iso20022Generic,
            _ => BankType::Unknown,
        }
    }
}

impl std::fmt::Display for BankType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_code())
    }
}

impl std::str::FromStr for BankType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let b = Self::from_code(s);
        if b == BankType::Unknown && !s.trim().is_empty() && s.trim().to_uppercase() != "UNKNOWN" {
            Err(format!("Unknown bank type: '{s}'"))
        } else {
            Ok(b)
        }
    }
}

/// Bank statement input file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatementFormat {
    Excel,
    Csv,
    Pdf,
    Html,
    Xml,
    SwiftMt,
    Unknown,
}

impl StatementFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatementFormat::Excel => "EXCEL",
            StatementFormat::Csv => "CSV",
            StatementFormat::Pdf => "PDF",
            StatementFormat::Html => "HTML",
            StatementFormat::Xml => "XML",
            StatementFormat::SwiftMt => "SWIFT_MT",
            StatementFormat::Unknown => "UNKNOWN",
        }
    }
}

impl std::fmt::Display for StatementFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for StatementFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "EXCEL" | "XLSX" | "XLS" => Ok(StatementFormat::Excel),
            "CSV" | "TSV" | "TXT" => Ok(StatementFormat::Csv),
            "PDF" => Ok(StatementFormat::Pdf),
            "HTML" | "HTM" => Ok(StatementFormat::Html),
            "XML" | "ISO20022" | "CAMT" => Ok(StatementFormat::Xml),
            "SWIFT_MT" | "MT940" | "MT942" | "STA" => Ok(StatementFormat::SwiftMt),
            "UNKNOWN" => Ok(StatementFormat::Unknown),
            _ => Err(format!("Unknown statement format: '{s}'")),
        }
    }
}

/// Standardized atomic transaction record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    #[serde(default)]
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
    #[serde(default)]
    pub ft_number: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default)]
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
            ft_number: None,
            trace_id: None,
            raw_ref: None,
        }
    }

    pub fn party_name(&self) -> Option<&str> {
        self.counterparty_name.as_deref()
    }
}

/// Standardized parsed bank statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankStatement {
    pub bank_code: String,
    #[serde(default = "default_bank_type")]
    pub bank_type: BankType,
    #[serde(default = "default_bank_type")]
    pub bank: BankType,
    #[serde(default = "default_statement_format")]
    pub format: StatementFormat,
    pub account_number: Option<String>,
    pub account_name: Option<String>,
    pub opening_balance: Option<u64>,
    pub closing_balance: Option<u64>,
    pub statement_from: Option<i64>,
    pub statement_to: Option<i64>,
    #[serde(default)]
    pub total_credit: u64,
    #[serde(default)]
    pub total_debit: u64,
    #[serde(default)]
    pub total_credit_turnover: u64,
    #[serde(default)]
    pub total_debit_turnover: u64,
    #[serde(default)]
    pub balance_checksum_passed: bool,
    pub transactions: Vec<TransactionRecord>,
    pub parse_duration_ms: u64,
}

impl BankStatement {
    pub fn new(
        bank_code: String,
        bank_type: BankType,
        format: StatementFormat,
        account_number: Option<String>,
        account_name: Option<String>,
        opening_balance: Option<u64>,
        closing_balance: Option<u64>,
        statement_from: Option<i64>,
        statement_to: Option<i64>,
        transactions: Vec<TransactionRecord>,
        parse_duration_ms: u64,
    ) -> Self {
        let mut total_credit = 0u64;
        let mut total_debit = 0u64;
        for tx in &transactions {
            if tx.tx_type == TransactionType::Credit {
                total_credit = total_credit.saturating_add(tx.amount);
            } else {
                total_debit = total_debit.saturating_add(tx.amount);
            }
        }
        let checksum_passed = match (opening_balance, closing_balance) {
            (Some(open), Some(close)) => {
                let calc = (open as i128) + (total_credit as i128) - (total_debit as i128);
                calc == (close as i128)
            }
            _ => true,
        };

        Self {
            bank_code,
            bank_type,
            bank: bank_type,
            format,
            account_number,
            account_name,
            opening_balance,
            closing_balance,
            statement_from,
            statement_to,
            total_credit,
            total_debit,
            total_credit_turnover: total_credit,
            total_debit_turnover: total_debit,
            balance_checksum_passed: checksum_passed,
            transactions,
            parse_duration_ms,
        }
    }
}

fn default_bank_type() -> BankType {
    BankType::Unknown
}

fn default_statement_format() -> StatementFormat {
    StatementFormat::Unknown
}

/// Backwards compatibility aliases for existing codebase callers.
pub type ParsedStatement = BankStatement;
pub type ParsedTransaction = TransactionRecord;

/// Invariant report for mathematical balance verification:
/// Closing = Opening + Credits - Debits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInvariantReport {
    pub is_valid: bool,
    pub is_balanced: bool,
    pub opening_balance: u64,
    pub total_credit: u64,
    pub total_debit: u64,
    pub closing_balance: u64,
    pub calculated_closing: i64,
    pub computed_closing: i64,
    pub discrepancy: i64,
}

impl BankStatement {
    pub fn verify_balance_invariants(&self) -> BalanceInvariantReport {
        let opening = self.opening_balance.unwrap_or(0);
        let closing = self.closing_balance.unwrap_or(0);
        let calculated =
            (opening as i128) + (self.total_credit as i128) - (self.total_debit as i128);
        let discrepancy = (closing as i128) - calculated;
        let is_ok = discrepancy == 0;
        BalanceInvariantReport {
            is_valid: is_ok,
            is_balanced: is_ok,
            opening_balance: opening,
            total_credit: self.total_credit,
            total_debit: self.total_debit,
            closing_balance: closing,
            calculated_closing: calculated as i64,
            computed_closing: calculated as i64,
            discrepancy: discrepancy as i64,
        }
    }
}

impl BalanceInvariantReport {
    pub fn verify(
        opening_balance: Option<u64>,
        closing_balance: Option<u64>,
        transactions: &[TransactionRecord],
    ) -> Self {
        let open = opening_balance.unwrap_or(0);
        let close = closing_balance.unwrap_or(0);
        let mut total_credit: u64 = 0;
        let mut total_debit: u64 = 0;
        for tx in transactions {
            match tx.tx_type {
                TransactionType::Credit => total_credit = total_credit.saturating_add(tx.amount),
                TransactionType::Debit => total_debit = total_debit.saturating_add(tx.amount),
            }
        }
        let calculated = (open as i128) + (total_credit as i128) - (total_debit as i128);
        let discrepancy = (close as i128) - calculated;
        let is_ok = discrepancy == 0 && opening_balance.is_some() && closing_balance.is_some();
        Self {
            is_valid: is_ok,
            is_balanced: is_ok,
            opening_balance: open,
            total_credit,
            total_debit,
            closing_balance: close,
            calculated_closing: calculated as i64,
            computed_closing: calculated as i64,
            discrepancy: discrepancy as i64,
        }
    }
}

/// Standalone function for verifying balance invariants of a statement.
pub fn verify_balance_invariants(stmt: &BankStatement) -> BalanceInvariantReport {
    stmt.verify_balance_invariants()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankTransactionRow {
    pub id: String,
    pub statement_id: String,
    pub account_id: String,
    pub bank_code: String,
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
    pub reconciled_status: ReconciliationStatus,
    pub reconciled_match_id: Option<String>,
    pub created_at: i64,
}

impl BankTransactionRow {
    /// Returns value_date if positive, falling back to tx_date.
    pub fn effective_date(&self) -> i64 {
        if self.value_date > 0 {
            self.value_date
        } else {
            self.tx_date
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalLedgerEntry {
    pub id: String,
    pub account_id: String,
    pub doc_no: String,
    pub entry_date: i64,
    pub entry_type: TransactionType,
    pub amount: u64,
    pub partner_code: Option<String>,
    pub partner_name: Option<String>,
    pub description: String,
    pub reconciled_status: ReconciliationStatus,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationMatch {
    pub id: String,
    pub bank_tx_id: String,
    #[serde(default)]
    pub bank_tx_ids: Vec<String>,
    pub ledger_entry_ids: Vec<String>,
    pub match_type: MatchType,
    pub confidence_score: f64,
    pub matched_amount: u64,
    pub discrepancy_amount: i64,
    pub status: String,
    pub matched_by: String,
    pub matched_at: i64,
    pub notes: Option<String>,
    pub hitl_token: Option<String>,
}

impl ReconciliationMatch {
    pub fn is_split_n_to_one(&self) -> bool {
        self.bank_tx_ids.len() > 1 && self.ledger_entry_ids.len() == 1
    }

    pub fn is_split_one_to_n(&self) -> bool {
        self.ledger_entry_ids.len() > 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCashflowForecast {
    pub date: String,
    pub expected_inflow: u64,
    pub expected_outflow: u64,
    pub projected_balance: i64,
    pub is_deficit_risk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankingOverviewDto {
    pub total_balance: u64,
    pub vcb_balance: u64,
    pub tcb_balance: u64,
    pub bidv_balance: u64,
    pub matched_ratio: f64,
    pub matched_count: usize,
    pub total_count: usize,
    pub automatic_count: usize,
    pub hitl_count: usize,
    pub discrepancy_count: usize,
    pub last_sync_time: i64,
    pub recent_transactions: Vec<BankTransactionRow>,
    pub rolling_forecast: Vec<DailyCashflowForecast>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementIngestResultDto {
    pub statement_id: String,
    pub bank_code: String,
    pub filename: String,
    pub total_transactions: usize,
    pub parse_duration_ms: u64,
    pub opening_balance: Option<u64>,
    pub closing_balance: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationSummaryDto {
    pub total_bank_transactions: usize,
    pub matched_exact_count: usize,
    pub matched_fuzzy_count: usize,
    pub matched_split_count: usize,
    pub total_matched_count: usize,
    pub pending_hitl_count: usize,
    pub discrepancy_count: usize,
    pub match_rate: f64,
    pub execution_duration_ms: u64,
    #[serde(default)]
    pub balance_invariant_checked: bool,
    #[serde(default = "default_summary_balance_passed")]
    pub balance_invariant_passed: bool,
    #[serde(default)]
    pub balance_discrepancy_amount: i64,
}

fn default_summary_balance_passed() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationMatrixItemDto {
    pub bank_tx: BankTransactionRow,
    pub matched_entries: Vec<InternalLedgerEntry>,
    pub match_type: Option<MatchType>,
    pub confidence_score: Option<f64>,
    pub discrepancy_amount: Option<i64>,
    pub hitl_token: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationMatrixDto {
    pub items: Vec<ReconciliationMatrixItemDto>,
    pub total_count: usize,
    pub matched_count: usize,
    pub pending_hitl_count: usize,
    pub discrepancy_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlResolutionDto {
    pub match_id: String,
    pub hitl_token: String,
    pub decision: String, // "APPROVE", "REJECT", "MANUAL_SPLIT"
    pub selected_ledger_entry_ids: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlConfirmResultDto {
    pub success: bool,
    pub match_id: String,
    pub new_status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatusDto {
    pub zero_egress_verified: bool,
    pub network_listeners: Vec<String>,
    pub db_encryption_algorithm: String,
    pub key_protection: String,
    pub audit_ledger_records_count: usize,
    pub audit_chain_intact: bool,
    pub audit_genesis_hash: String,
    pub audit_latest_hash: String,
    pub pii_redaction_active: bool,
}

/// Helper function to parse Vietnamese number formatting into scaled integer units.
/// Examples:
/// "15.000.000,00" -> 15000000
/// "15,000,000.00" -> 15000000
/// "15.000.000"    -> 15000000
/// "15000000"      -> 15000000
/// "-500.000,00"   -> 500000 (absolute scaled value, signedness determined by column or caller)
pub fn parse_vietnamese_amount(raw: &str) -> Option<u64> {
    let clean = raw.trim().trim_matches(|c| c == '"' || c == '\'').trim();
    if clean.is_empty() {
        return None;
    }

    // Must contain at least one digit
    if !clean.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    // Reject tokens containing slashes, colons, or backslashes (dates, timestamps, paths)
    if clean.contains('/') || clean.contains(':') || clean.contains('\\') {
        return None;
    }

    // Hyphen validation: an amount may only have a leading '-' or '(-'
    let trimmed_sign = clean.strip_prefix('(').unwrap_or(clean).trim_start();
    let minus_count = clean.chars().filter(|&c| c == '-').count();
    if minus_count > 1 || (minus_count == 1 && !trimmed_sign.starts_with('-')) {
        return None;
    }

    // Parentheses validation: negative amounts like "(50.000.000)"
    if clean.contains('(') || clean.contains(')') {
        let open_count = clean.chars().filter(|&c| c == '(').count();
        let close_count = clean.chars().filter(|&c| c == ')').count();
        if open_count != close_count || open_count > 1 || !clean.starts_with('(') {
            return None;
        }
    }

    // Check all characters: must be digit, punctuation/spacing allowed in numbers, or alphabetic
    // Reject invalid symbols like %, #, $, @, !, ?, ;, &, *, etc.
    for c in clean.chars() {
        if !c.is_ascii_digit()
            && c != '.'
            && c != ','
            && c != '-'
            && c != '+'
            && c != '('
            && c != ')'
            && !c.is_whitespace()
            && !c.is_alphabetic()
        {
            return None;
        }
    }

    // Extract contiguous alphabetic words and ensure each is a recognized currency/indicator token
    let mut current_word = String::new();
    for c in clean.chars() {
        if c.is_alphabetic() {
            current_word.push(c);
        } else if !current_word.is_empty() {
            if !is_recognized_currency_indicator(&current_word) {
                return None;
            }
            current_word.clear();
        }
    }
    if !current_word.is_empty() && !is_recognized_currency_indicator(&current_word) {
        return None;
    }

    let is_negative = clean.starts_with('-') || (clean.starts_with('(') && clean.ends_with(')'));
    let digits_only: String = clean
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .collect();

    if digits_only.is_empty() {
        return None;
    }

    let has_dot = digits_only.contains('.');
    let has_comma = digits_only.contains(',');

    let integer_part = if has_dot && has_comma {
        let last_dot = digits_only.rfind('.').unwrap();
        let last_comma = digits_only.rfind(',').unwrap();
        if last_comma > last_dot {
            // Vietnamese style: "15.000.000,00" -> dots are thousand sep, comma is decimal
            digits_only[..last_comma].replace('.', "")
        } else {
            // US style: "15,000,000.00" -> commas are thousand sep, dot is decimal
            digits_only[..last_dot].replace(',', "")
        }
    } else if has_comma && !has_dot {
        let parts: Vec<&str> = digits_only.split(',').collect();
        if parts.len() == 2 && parts[1].len() <= 2 {
            // Single decimal separator with 1 or 2 places: "15000,00" or "15000,5"
            parts[0].to_string()
        } else if parts.len() > 1 && parts.iter().skip(1).all(|p| p.len() == 3) {
            // Standard thousands separators: "15,000,000" or "15,000"
            digits_only.replace(',', "")
        } else if parts.len() == 2 && parts[1].len() == 3 {
            digits_only.replace(',', "")
        } else {
            digits_only.replace(',', "")
        }
    } else if has_dot && !has_comma {
        let parts: Vec<&str> = digits_only.split('.').collect();
        if parts.len() == 2 && parts[1].len() <= 2 {
            // Single decimal separator with 1 or 2 places: "15000.00" or "15000.5"
            parts[0].to_string()
        } else if parts.len() > 1 && parts.iter().skip(1).all(|p| p.len() == 3) {
            // Standard Vietnamese thousand separators: "15.000.000" or "380.000"
            digits_only.replace('.', "")
        } else if parts.len() == 2 && parts[1].len() == 3 {
            digits_only.replace('.', "")
        } else {
            digits_only.replace('.', "")
        }
    } else {
        digits_only
    };

    let amount_u64: u64 = integer_part.parse().ok()?;
    let _ = is_negative;
    Some(amount_u64)
}

fn is_recognized_currency_indicator(word: &str) -> bool {
    let upper = word.to_uppercase();
    matches!(
        upper.as_str(),
        "VND" | "CR" | "DB" | "NO" | "CO" | "Đ" | "NỢ" | "CÓ"
    )
}

/// Parses date strings commonly found in Vietnamese banking statements:
/// - "dd/MM/yyyy HH:mm:ss"
/// - "dd/MM/yyyy"
/// - "yyyy-MM-dd HH:mm:ss"
/// - "yyyy-MM-dd"
/// - "dd-MM-yyyy"
pub fn parse_banking_date(raw: &str) -> Option<i64> {
    let clean = raw.trim();
    if clean.is_empty() {
        return None;
    }

    // dd/MM/yyyy or dd/MM/yyyy HH:mm:ss
    if let Some((d, m, y, h, min, s)) = parse_dmy_or_ymd(clean) {
        return date_to_unix(y, m, d, h, min, s);
    }

    None
}

fn parse_dmy_or_ymd(clean: &str) -> Option<(i32, u32, i32, u32, u32, u32)> {
    let parts: Vec<&str> = clean.split_whitespace().collect();
    let date_str = parts.first()?;
    let time_str = parts.get(1).copied().unwrap_or("00:00:00");

    let (h, min, s) = {
        let tparts: Vec<u32> = time_str.split(':').filter_map(|p| p.parse().ok()).collect();
        (
            tparts.first().copied().unwrap_or(0),
            tparts.get(1).copied().unwrap_or(0),
            tparts.get(2).copied().unwrap_or(0),
        )
    };

    if date_str.contains('/') {
        let dparts: Vec<&str> = date_str.split('/').collect();
        if dparts.len() == 3 {
            let p0: i32 = dparts[0].parse().ok()?;
            let p1: u32 = dparts[1].parse().ok()?;
            let p2: i32 = dparts[2].parse().ok()?;
            if p0 > 1000 {
                // yyyy/MM/dd
                return Some((p2, p1, p0, h, min, s));
            } else {
                // dd/MM/yyyy
                return Some((p0, p1, p2, h, min, s));
            }
        }
    } else if date_str.contains('-') {
        let dparts: Vec<&str> = date_str.split('-').collect();
        if dparts.len() == 3 {
            let p0: i32 = dparts[0].parse().ok()?;
            let p1: u32 = dparts[1].parse().ok()?;
            let p2: i32 = dparts[2].parse().ok()?;
            if p0 > 1000 {
                // yyyy-MM-dd
                return Some((p2, p1, p0, h, min, s));
            } else {
                // dd-MM-yyyy
                return Some((p0, p1, p2, h, min, s));
            }
        }
    }

    None
}

/// Convert year, month, day, hour, minute, second to Unix timestamp.
pub fn date_to_unix(year: i32, month: u32, day: i32, hour: u32, min: u32, sec: u32) -> Option<i64> {
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour >= 24
        || min >= 60
        || sec >= 60
    {
        return None;
    }

    // Days before each month (normal year)
    let days_before_month = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);

    let y = year - 1970;
    let mut leap_days = 0;
    if year >= 1970 {
        for y_cur in 1970..year {
            if (y_cur % 4 == 0 && y_cur % 100 != 0) || (y_cur % 400 == 0) {
                leap_days += 1;
            }
        }
    } else {
        for y_cur in year..1970 {
            if (y_cur % 4 == 0 && y_cur % 100 != 0) || (y_cur % 400 == 0) {
                leap_days -= 1;
            }
        }
    }

    let mut total_days = (y as i64) * 365 + (leap_days as i64);
    total_days += days_before_month[(month - 1) as usize] as i64;
    if month > 2 && is_leap {
        total_days += 1;
    }
    total_days += (day - 1) as i64;

    let total_secs = total_days * 86400 + (hour as i64) * 3600 + (min as i64) * 60 + (sec as i64);
    Some(total_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vietnamese_amount() {
        assert_eq!(parse_vietnamese_amount("15.000.000,00"), Some(15_000_000));
        assert_eq!(parse_vietnamese_amount("15.500.000,50"), Some(15_500_000));
        assert_eq!(parse_vietnamese_amount("15.000.000"), Some(15_000_000));
        assert_eq!(parse_vietnamese_amount("15,000,000.00"), Some(15_000_000));
        assert_eq!(
            parse_vietnamese_amount("1,450,230,000"),
            Some(1_450_230_000)
        );
        assert_eq!(parse_vietnamese_amount("785.600.000"), Some(785_600_000));
        assert_eq!(parse_vietnamese_amount("380.000"), Some(380_000));
        assert_eq!(parse_vietnamese_amount("  -50.000.000  "), Some(50_000_000));
        assert_eq!(parse_vietnamese_amount("0"), Some(0));

        // Required validation rejections:
        assert_eq!(parse_vietnamese_amount("HD101"), None);
        assert_eq!(parse_vietnamese_amount("15/08/2026"), None);
        assert_eq!(parse_vietnamese_amount("09:30:00"), None);
        assert_eq!(parse_vietnamese_amount("Số dư ngày 31/08/2026:"), None);
        assert_eq!(parse_vietnamese_amount("INV-2026-001"), None);
        assert_eq!(parse_vietnamese_amount("Hop dong 102/2026"), None);
    }

    #[test]
    fn test_parse_banking_date() {
        let ts = parse_banking_date("15/08/2026 14:30:00").unwrap();
        assert!(ts > 1_700_000_000);
        let ts2 = parse_banking_date("2026-08-15 14:30:00").unwrap();
        assert_eq!(ts, ts2);
        let ts3 = parse_banking_date("15/08/2026").unwrap();
        assert_eq!(ts3, date_to_unix(2026, 8, 15, 0, 0, 0).unwrap());
    }
}
