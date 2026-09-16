use liva_ledger::PostingType;
use liva_money::Money;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchType {
    Exact1To1,
    FuzzyHeuristic,
    FeeSplit,
    CompositeSplit,
    ManualHitl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankTransaction {
    pub id: String,
    pub tx_date: i64,
    pub value_date: Option<i64>,
    pub doc_ref: Option<String>,
    pub direction: PostingType,
    pub amount: Money,
    pub narration: String,
    pub counterparty_name: Option<String>,
}

impl BankTransaction {
    pub fn effective_date(&self) -> i64 {
        self.value_date.filter(|&v| v > 0).unwrap_or(self.tx_date)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: String,
    pub doc_no: String,
    pub entry_date: i64,
    pub direction: PostingType,
    pub amount: Money,
    pub partner_name: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateMatch {
    pub ledger_entry_id: String,
    pub score: f64,
    pub amount_diff: i64,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationMatch {
    pub match_id: String,
    pub match_type: MatchType,
    pub bank_tx_ids: Vec<String>,
    pub ledger_entry_ids: Vec<String>,
    pub matched_amount: Money,
    pub fee_amount: Money,
    pub discrepancy_amount: Money,
    pub confidence_score: f64,
    pub explanation: String,
    pub timestamp: i64,
    pub status: String,
    pub hitl_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchConfig {
    pub tier1_window_secs: i64, // Default: 86_400 (24h)
    pub tier2_window_secs: i64, // Default: 259_200 (72h)
    pub tier2_min_party_similarity: f64, // Default: 0.85
    pub tier2_min_combined_score: f64, // Default: 0.85
    pub fee_tolerance_min_vnd: i64, // Default: 1_100
    pub fee_tolerance_max_vnd: i64, // Default: 22_000
    pub account_bank_cash: String, // Default: "1121"
    pub account_bank_fee: String, // Default: "6425"
    pub account_receivable: String, // Default: "131"
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            tier1_window_secs: 86_400,
            tier2_window_secs: 259_200,
            tier2_min_party_similarity: 0.85,
            tier2_min_combined_score: 0.85,
            fee_tolerance_min_vnd: 1_100,
            fee_tolerance_max_vnd: 22_000,
            account_bank_cash: "1121".to_string(),
            account_bank_fee: "6425".to_string(),
            account_receivable: "131".to_string(),
        }
    }
}
