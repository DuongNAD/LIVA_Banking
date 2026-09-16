//! # liva-match
//!
//! Deterministic zero-hallucination reconciliation matching engine and bank wire fee splitter for LIVA Banking.
//!
//! - **Tier 1 Exact O(1) Matcher**: Scaled integer exact amount, canonicalized reference tokens,
//!   strict direction invariance, ±24h time window.
//! - **Tier 2 Amount-Bucketed Fuzzy Matcher**: 11,000 VND bucketed search space, Vietnamese diacritics
//!   normalization, corporate noise elimination, Jaro-Winkler scoring ≥ 0.85, ±72h clearing window.
//! - **Fee Splitter Engine**: Detects standard wire transfer fees (1,100 to 22,000 VND + VAT) and
//!   constructs balanced double-entry `JournalEntry` instances with debit to Account 6425 per Circular 200/2014/TT-BTC.

pub mod fee_splitter;
pub mod jaro_winkler;
pub mod models;
pub mod text_cleaner;
pub mod tier1_exact;
pub mod tier2_fuzzy;

pub use fee_splitter::{
    create_fee_split_journal, create_pure_fee_journal, FeeSplitter, WireFeeDisentanglement,
    KNOWN_VIETNAMESE_WIRE_FEES,
};
pub use jaro_winkler::{compare_party_names, jaro_similarity, jaro_winkler};
pub use models::{
    BankTransaction, CandidateMatch, LedgerEntry, MatchConfig, MatchType, ReconciliationMatch,
};
pub use text_cleaner::{
    normalize_vietnamese_text, strip_bank_narration_noise, strip_corporate_legal_noise,
};
pub use tier1_exact::{normalize_doc_ref, HashMatcher};
pub use tier2_fuzzy::FuzzyMatcher;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationSummary {
    pub total_bank_txs: usize,
    pub total_ledger_entries: usize,
    pub total_matched: usize,
    pub tier1_matches: usize,
    pub tier2_matches: usize,
    pub fee_split_matches: usize,
    pub unallocated_bank_count: usize,
    pub unallocated_ledger_count: usize,
    pub matches: Vec<ReconciliationMatch>,
}

/// Facade for the reconciliation engine orchestrating Tier 1 and Tier 2 matching.
pub struct ReconciliationEngine {
    config: MatchConfig,
}

impl ReconciliationEngine {
    pub fn new(config: MatchConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(MatchConfig::default())
    }

    pub fn config(&self) -> &MatchConfig {
        &self.config
    }

    /// Executes multi-tier reconciliation matching across provided bank transactions and internal ledger entries.
    pub fn reconcile(
        &self,
        bank_txs: &[BankTransaction],
        ledger_entries: &[LedgerEntry],
    ) -> ReconciliationSummary {
        // Step 1: Tier 1 Exact Hash Match (O(1), ±24h)
        let (tier1_matches, unalloc_b, unalloc_l) =
            HashMatcher::match_tier1(bank_txs, ledger_entries, &self.config);

        // Step 2: Tier 2 Amount-Bucketed Fuzzy Match + Fee Splitter (±72h)
        let (tier2_matches, final_unalloc_b, final_unalloc_l) =
            FuzzyMatcher::match_tier2(bank_txs, ledger_entries, &unalloc_b, &unalloc_l, &self.config);

        let t1_count = tier1_matches.len();
        let mut t2_count = 0;
        let mut fee_count = 0;

        for m in &tier2_matches {
            if m.match_type == MatchType::FeeSplit {
                fee_count += 1;
            } else {
                t2_count += 1;
            }
        }

        let mut all_matches = tier1_matches;
        all_matches.extend(tier2_matches);

        ReconciliationSummary {
            total_bank_txs: bank_txs.len(),
            total_ledger_entries: ledger_entries.len(),
            total_matched: all_matches.len(),
            tier1_matches: t1_count,
            tier2_matches: t2_count,
            fee_split_matches: fee_count,
            unallocated_bank_count: final_unalloc_b.len(),
            unallocated_ledger_count: final_unalloc_l.len(),
            matches: all_matches,
        }
    }
}
