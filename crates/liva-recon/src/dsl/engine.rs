//! Declarative Reconciliation Engine
//!
//! Interprets `ReconRuleConfig` to execute high-performance, deterministic 3-tier matching
//! and HITL quarantine management.
//!
//! Invariants:
//! - `#![deny(clippy::float_arithmetic)]`: zero floating-point arithmetic.
//! - Strict mathematical balance conservation.
//! - Segregation of Duties (SoD) for HITL Maker-Checker.

use crate::dsl::jaro_winkler::{
    compute_semantic_similarity_bps, integer_jaro_winkler, is_acronym_pattern,
};
use crate::dsl::schema::{
    FeeScheduleConfig, QuarantineRiskTier, ReconRuleConfig,
};
use crate::dsl::subset_sum::{solve_subset_sum, SubsetCandidate, SubsetSolverConfig};
use crate::{
    normalize_vietnamese_text, HitlQuarantineItem, InternalInvoice, MatchTier,
    ReconciliationMatch, ReconError,
};
use liva_money::Money;
use liva_parse::CanonicalTx;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use uuid::Uuid;

/// Pre-computed semantic similarity inputs or integer embedding dot products.
///
/// Strictly enforces `#![deny(clippy::float_arithmetic)]`: zero f32/f64.
/// All similarity values and dot products are represented as integer basis points (0..=10,000 bps).
#[derive(Debug, Clone, Default)]
pub struct SemanticReconContext {
    /// Pre-computed semantic similarity in basis points (0..=10,000 bps).
    /// Keys are (tx_row_id, invoice_id).
    pub similarity_by_tx_inv: HashMap<(usize, String), u32>,

    /// Optional pre-computed integer quantized vector embeddings for transactions.
    /// Each vector has elements scaled by 1,000 (dot product produces basis points directly).
    pub tx_embeddings: HashMap<usize, Vec<i32>>,

    /// Optional pre-computed integer quantized vector embeddings for invoices.
    pub invoice_embeddings: HashMap<String, Vec<i32>>,
}

impl SemanticReconContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_similarity(mut self, tx_row_id: usize, invoice_id: impl Into<String>, similarity_bps: u32) -> Self {
        self.similarity_by_tx_inv.insert((tx_row_id, invoice_id.into()), similarity_bps.min(10_000));
        self
    }

    pub fn with_embeddings(
        mut self,
        tx_row_id: usize,
        tx_vec: Vec<i32>,
        invoice_id: impl Into<String>,
        inv_vec: Vec<i32>,
    ) -> Self {
        self.tx_embeddings.insert(tx_row_id, tx_vec);
        self.invoice_embeddings.insert(invoice_id.into(), inv_vec);
        self
    }

    /// Retrieves similarity in basis points if pre-computed.
    pub fn get_similarity(&self, tx_row_id: usize, invoice_id: &str) -> Option<u32> {
        if let Some(&score) = self.similarity_by_tx_inv.get(&(tx_row_id, invoice_id.to_string())) {
            return Some(score.min(10_000));
        }

        // Check if integer vector dot product is available
        if let (Some(v1), Some(v2)) = (self.tx_embeddings.get(&tx_row_id), self.invoice_embeddings.get(invoice_id)) {
            if v1.len() == v2.len() && !v1.is_empty() {
                let dot: i64 = v1.iter().zip(v2.iter()).map(|(&a, &b)| (a as i64) * (b as i64)).sum();
                let clamped_dot = dot.clamp(0, 1_000_000);
                let bps = (clamped_dot / 100) as u32;
                return Some(bps.min(10_000));
            }
        }

        None
    }
}

/// Result of a batch reconciliation run across Tier 1, Tier 2, Tier 3, and HITL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconBatchResult {
    pub matches: Vec<ReconciliationMatch>,
    pub hitl_quarantine: Vec<HitlQuarantineItem>,
    pub matched_exact_count: usize,
    pub matched_fuzzy_count: usize,
    pub matched_semantic_count: usize,
    pub matched_split_count: usize,
    pub total_matched_count: usize,
    pub unmatched_tx_count: usize,
    pub unmatched_invoice_count: usize,
}

#[derive(Debug, Error)]
pub enum ReconDslError {
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid regex '{pattern}': {source}")]
    Regex {
        pattern: String,
        source: regex::Error,
    },

    #[error("Configuration validation error: {0}")]
    Validation(String),
}

/// Strips standard punctuation characters (-, _, /, space, .) from reference codes.
pub fn strip_ref_punctuation(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '-' && *c != '_' && *c != '/' && *c != ' ' && *c != '.')
        .collect()
}

/// Compiled declarative reconciliation engine holding precompiled regexes and rule configurations.
#[derive(Debug, Clone)]
pub struct DeclarativeReconEngine {
    pub config: ReconRuleConfig,
    tier1_regexes: Vec<Regex>,
}

impl DeclarativeReconEngine {
    /// Instantiates the engine from a `ReconRuleConfig`, compiling all regex patterns.
    pub fn new(config: ReconRuleConfig) -> Result<Self, ReconDslError> {
        config.validate()?;

        let mut tier1_regexes = Vec::new();
        for pattern in &config.tier1.reference_matching.extraction_patterns {
            let re = Regex::new(pattern).map_err(|source| ReconDslError::Regex {
                pattern: pattern.clone(),
                source,
            })?;
            tier1_regexes.push(re);
        }

        Ok(Self {
            config,
            tier1_regexes,
        })
    }

    /// Loads the default Vietnamese banking reconciliation rules.
    pub fn default_rules() -> Self {
        let config = ReconRuleConfig::default();
        Self::new(config).expect("Default recon rules must be valid")
    }

    /// Loads engine from a YAML string.
    pub fn from_yaml(yaml_str: &str) -> Result<Self, ReconDslError> {
        let config: ReconRuleConfig = serde_yaml::from_str(yaml_str)?;
        Self::new(config)
    }

    /// Loads engine from a YAML configuration file path.
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ReconDslError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }

    /// Executes Tier 1 exact match using configured window and regex reference extraction.
    pub fn match_tier1(
        &self,
        txs: &[CanonicalTx],
        invoices: &[InternalInvoice],
    ) -> (Vec<ReconciliationMatch>, Vec<usize>, Vec<String>) {
        if !self.config.tier1.enabled {
            let unmatched_txs = (0..txs.len()).collect();
            let unmatched_invs = invoices.iter().map(|i| i.invoice_id.clone()).collect();
            return (Vec::new(), unmatched_txs, unmatched_invs);
        }

        let mut matched = Vec::new();
        let mut matched_tx_indices = HashSet::new();
        let mut matched_invoice_ids = HashSet::new();

        // Index invoices by (amount_vnd, clean_uppercase_doc_ref)
        // Values: list of invoices (in case of duplicate refs/amounts)
        let mut invoice_map: HashMap<(i64, String), Vec<&InternalInvoice>> = HashMap::new();
        for inv in invoices {
            let clean_doc_ref = strip_ref_punctuation(&inv.doc_ref.to_uppercase());
            invoice_map
                .entry((inv.amount.amount(), clean_doc_ref))
                .or_default()
                .push(inv);
        }

        let time_tol = self.config.tier1.timestamp_tolerance_seconds;
        let conf_score = (self.config.tier1.confidence_score_bps / 100).min(100);

        for (idx, tx) in txs.iter().enumerate() {
            let tx_amount = tx.amount.amount();
            let clean_tx_ref = strip_ref_punctuation(&tx.ref_code.to_uppercase());

            // 1. Check direct ref_code match
            let mut matched_inv: Option<&InternalInvoice> = None;
            if !clean_tx_ref.is_empty() {
                if let Some(candidates) = invoice_map.get(&(tx_amount, clean_tx_ref.clone())) {
                    for candidate in candidates {
                        if !matched_invoice_ids.contains(&candidate.invoice_id)
                            && (tx.tx_timestamp - candidate.timestamp).abs() <= time_tol
                        {
                            matched_inv = Some(candidate);
                            break;
                        }
                    }
                }
            }

            // 2. If no direct match and extraction is enabled, scan narration via regexes
            if matched_inv.is_none() && self.config.tier1.reference_matching.extract_from_narration {
                for re in &self.tier1_regexes {
                    for m in re.find_iter(&tx.narration) {
                        let extracted_clean = strip_ref_punctuation(&m.as_str().to_uppercase());
                        if let Some(candidates) = invoice_map.get(&(tx_amount, extracted_clean)) {
                            for candidate in candidates {
                                if !matched_invoice_ids.contains(&candidate.invoice_id)
                                    && (tx.tx_timestamp - candidate.timestamp).abs() <= time_tol
                                {
                                    matched_inv = Some(candidate);
                                    break;
                                }
                            }
                        }
                        if matched_inv.is_some() {
                            break;
                        }
                    }
                    if matched_inv.is_some() {
                        break;
                    }
                }
            }

            if let Some(inv) = matched_inv {
                matched.push(ReconciliationMatch {
                    match_id: Uuid::new_v4().to_string(),
                    tier: MatchTier::Tier1Exact,
                    bank_tx_id: tx.row_id,
                    invoice_id: inv.invoice_id.clone(),
                    bank_amount: tx.amount,
                    invoice_amount: inv.amount,
                    fee_deducted: None,
                    confidence_score: conf_score,
                    constituent_invoice_ids: None,
                    constituent_bank_tx_ids: None,
                    is_semantic: false,
                    semantic_similarity_bps: None,
                });
                matched_tx_indices.insert(idx);
                matched_invoice_ids.insert(inv.invoice_id.clone());
            }
        }

        let unmatched_txs: Vec<usize> = (0..txs.len())
            .filter(|i| !matched_tx_indices.contains(i))
            .collect();

        let unmatched_invoices: Vec<String> = invoices
            .iter()
            .map(|inv| inv.invoice_id.clone())
            .filter(|id| !matched_invoice_ids.contains(id))
            .collect();

        (matched, unmatched_txs, unmatched_invoices)
    }

    /// Executes Tier 2 fuzzy heuristic matching with Integer Jaro-Winkler and dynamic fee schedules.
    pub fn match_tier2(
        &self,
        unmatched_tx_indices: &[usize],
        all_txs: &[CanonicalTx],
        unmatched_inv_ids: &[String],
        all_invoices: &[InternalInvoice],
    ) -> (Vec<ReconciliationMatch>, Vec<usize>, Vec<String>) {
        self.match_tier2_with_semantic(
            unmatched_tx_indices,
            all_txs,
            unmatched_inv_ids,
            all_invoices,
            None,
        )
    }

    /// Executes Tier 2 fuzzy heuristic matching with 2-gate hybrid semantic elevation.
    ///
    /// - Gate 1 (Fast Path): `integer_jaro_winkler(&party, &narration) >= 8,500` bps (or matched_words >= 2, containment >= 80%)
    ///   matches immediately in Tier 2.
    /// - Gate 2 (Semantic Elevation): when JW is in uncertainty band (e.g. 5,000..8,499 bps) or acronym pattern detected:
    ///   evaluates semantic similarity in integer basis points (0..10,000 bps) using token cosine / dot product similarity
    ///   (supporting pre-computed vector cosine or semantic similarity inputs).
    ///   If semantic similarity >= 8,200 bps, elevates to Tier 2 match with combined score:
    ///   `(40 * jw + 60 * semantic_sim) / 100`.
    /// - Deterministic Fallback: When semantic inputs are absent or disabled (`enable_semantic_recon = false`),
    ///   falls back deterministically to pure Integer Jaro-Winkler without errors.
    pub fn match_tier2_with_semantic(
        &self,
        unmatched_tx_indices: &[usize],
        all_txs: &[CanonicalTx],
        unmatched_inv_ids: &[String],
        all_invoices: &[InternalInvoice],
        semantic_context: Option<&SemanticReconContext>,
    ) -> (Vec<ReconciliationMatch>, Vec<usize>, Vec<String>) {
        if !self.config.tier2.enabled {
            return (
                Vec::new(),
                unmatched_tx_indices.to_vec(),
                unmatched_inv_ids.to_vec(),
            );
        }

        let mut matched = Vec::new();
        let mut matched_tx_set = HashSet::new();
        let mut matched_inv_set = HashSet::new();

        let inv_lookup: HashMap<String, &InternalInvoice> = all_invoices
            .iter()
            .map(|i| (i.invoice_id.clone(), i))
            .collect();

        let time_tol = self.config.tier2.timestamp_tolerance_seconds;
        let sim_thresh = self.config.tier2.similarity_threshold_bps;
        let cont_thresh = self.config.tier2.token_containment_threshold_bps;
        let exact_conf = (self.config.tier2.confidence.exact_match_bps / 100).min(100);
        let fee_conf = (self.config.tier2.confidence.fee_deducted_bps / 100).min(100);

        for &tx_idx in unmatched_tx_indices {
            let tx = &all_txs[tx_idx];
            let mut tx_norm = normalize_vietnamese_text(&tx.narration);

            // Strip bank boilerplate phrases
            for phrase in &self.config.tier2.bank_narration_phrases {
                let p_norm = normalize_vietnamese_text(phrase);
                if tx_norm.contains(&p_norm) {
                    tx_norm = tx_norm.replace(&p_norm, " ");
                }
            }

            for inv_id in unmatched_inv_ids {
                if matched_inv_set.contains(inv_id) {
                    continue;
                }
                let inv = match inv_lookup.get(inv_id) {
                    Some(i) => *i,
                    None => continue,
                };

                // Check timestamp window
                if (tx.tx_timestamp - inv.timestamp).abs() > time_tol {
                    continue;
                }

                // Check amount delta
                let delta = inv.amount.amount() - tx.amount.amount();
                let is_exact = delta == 0;
                let fee_matched = if is_exact {
                    false
                } else {
                    self.config
                        .tier2
                        .fee_schedules
                        .iter()
                        .any(|sched| sched.matches_fee(delta))
                };

                if !is_exact && !fee_matched {
                    continue;
                }

                // Clean counterparty name
                let mut inv_party_norm = normalize_vietnamese_text(&inv.counterparty_name);

                // Strip multi-word legal phrases
                for phrase in &self.config.tier2.legal_phrases {
                    let p_norm = normalize_vietnamese_text(phrase);
                    if inv_party_norm.contains(&p_norm) {
                        inv_party_norm = inv_party_norm.replace(&p_norm, " ");
                    }
                }

                // Filter single legal stop tokens
                let words: Vec<&str> = inv_party_norm
                    .split_whitespace()
                    .filter(|w| !self.config.tier2.legal_tokens.iter().any(|t| t == *w))
                    .collect();

                if words.is_empty() {
                    continue;
                }

                // 1. Token containment check
                let matched_words = words.iter().filter(|&&w| tx_norm.contains(w)).count();
                let containment_bps = (matched_words as u32 * 10_000) / (words.len() as u32);

                // 2. Pure Integer Jaro-Winkler similarity check
                let cleaned_party = words.join(" ");
                let jw_bps = integer_jaro_winkler(&cleaned_party, &tx_norm);

                // Gate 1 (Fast Path):
                let party_matched_gate1 =
                    containment_bps >= cont_thresh || jw_bps >= sim_thresh;

                let mut is_semantic_elevation = false;
                let mut semantic_sim_score = 0u32;
                let mut combined_conf_score = 0u32;

                // Gate 2 (Semantic Elevation):
                if !party_matched_gate1 && self.config.tier2.enable_semantic_recon {
                    let has_precomputed = semantic_context
                        .and_then(|ctx| ctx.get_similarity(tx.row_id, &inv.invoice_id))
                        .is_some();
                    let in_uncertainty_band = jw_bps >= self.config.tier2.uncertainty_lower_bound_bps
                        && jw_bps < self.config.tier2.similarity_threshold_bps;
                    let acronym_detected = is_acronym_pattern(&cleaned_party, &tx_norm);

                    if in_uncertainty_band || acronym_detected || has_precomputed {
                        let semantic_sim = if let Some(ctx) = semantic_context {
                            if let Some(precomputed) = ctx.get_similarity(tx.row_id, &inv.invoice_id) {
                                precomputed
                            } else {
                                compute_semantic_similarity_bps(&cleaned_party, &tx_norm)
                            }
                        } else {
                            compute_semantic_similarity_bps(&cleaned_party, &tx_norm)
                        };

                        if semantic_sim >= self.config.tier2.semantic_elevation_threshold_bps {
                            let combined_bps = (self.config.tier2.jw_weight * jw_bps
                                + self.config.tier2.semantic_weight * semantic_sim)
                                / 100;
                            let conf = ((combined_bps + 50) / 100).min(100);

                            is_semantic_elevation = true;
                            semantic_sim_score = semantic_sim;
                            combined_conf_score = conf;
                        }
                    }
                }

                if party_matched_gate1 || is_semantic_elevation {
                    let (fee_deducted, conf, tier, is_sem, sem_sim) = if is_semantic_elevation {
                        let fee = if is_exact {
                            None
                        } else {
                            Some(Money::vnd(delta))
                        };
                        (
                            fee,
                            combined_conf_score,
                            MatchTier::Tier2Semantic,
                            true,
                            Some(semantic_sim_score),
                        )
                    } else if is_exact {
                        (None, exact_conf, MatchTier::Tier2Fuzzy, false, None)
                    } else {
                        (
                            Some(Money::vnd(delta)),
                            fee_conf,
                            MatchTier::Tier2Fuzzy,
                            false,
                            None,
                        )
                    };

                    matched.push(ReconciliationMatch {
                        match_id: Uuid::new_v4().to_string(),
                        tier,
                        bank_tx_id: tx.row_id,
                        invoice_id: inv.invoice_id.clone(),
                        bank_amount: tx.amount,
                        invoice_amount: inv.amount,
                        fee_deducted,
                        confidence_score: conf,
                        constituent_invoice_ids: None,
                        constituent_bank_tx_ids: None,
                        is_semantic: is_sem,
                        semantic_similarity_bps: sem_sim,
                    });
                    matched_tx_set.insert(tx_idx);
                    matched_inv_set.insert(inv_id.clone());
                    break;
                }
            }
        }

        let rem_txs: Vec<usize> = unmatched_tx_indices
            .iter()
            .copied()
            .filter(|i| !matched_tx_set.contains(i))
            .collect();

        let rem_invs: Vec<String> = unmatched_inv_ids
            .iter()
            .filter(|id| !matched_inv_set.contains(*id))
            .cloned()
            .collect();

        (matched, rem_txs, rem_invs)
    }

    /// Executes Tier 3 Subset-Sum composite split matching (both 1:N and N:1).
    pub fn match_tier3(
        &self,
        unmatched_tx_indices: &[usize],
        all_txs: &[CanonicalTx],
        unmatched_inv_ids: &[String],
        all_invoices: &[InternalInvoice],
    ) -> (Vec<ReconciliationMatch>, Vec<usize>, Vec<String>) {
        if !self.config.tier3.enabled {
            return (
                Vec::new(),
                unmatched_tx_indices.to_vec(),
                unmatched_inv_ids.to_vec(),
            );
        }

        let mut matched = Vec::new();
        let mut matched_tx_set = HashSet::new();
        let mut matched_inv_set = HashSet::new();

        let inv_lookup: HashMap<String, &InternalInvoice> = all_invoices
            .iter()
            .map(|i| (i.invoice_id.clone(), i))
            .collect();

        let solver_config = SubsetSolverConfig {
            max_depth: self.config.tier3.max_depth,
            max_iterations: self.config.tier3.max_iterations,
            timeout_ms: self.config.tier3.timeout_ms,
            require_zero_delta: self.config.tier3.require_zero_delta,
            base_confidence_bps: self.config.tier3.base_confidence_bps,
            item_penalty_bps: self.config.tier3.item_penalty_bps,
        };

        // ---------------------------------------------------------------------
        // Phase A: 1:N Split Resolution (1 Bank Tx paid by N Invoices)
        // ---------------------------------------------------------------------
        for &tx_idx in unmatched_tx_indices {
            let tx = &all_txs[tx_idx];
            let target = tx.amount.amount();
            if target <= 0 {
                continue;
            }

            // Build candidate pool from remaining invoices
            let candidate_invoices: Vec<&InternalInvoice> = unmatched_inv_ids
                .iter()
                .filter(|id| !matched_inv_set.contains(*id))
                .filter_map(|id| inv_lookup.get(id).copied())
                .filter(|inv| inv.amount.amount() > 0 && inv.amount.amount() < target)
                .collect();

            if candidate_invoices.len() < 2 {
                continue;
            }

            let tx_norm = normalize_vietnamese_text(&tx.narration);

            // Step 1: Invoices whose invoice_id or doc_ref is explicitly cited in narration
            let ref_candidates: Vec<SubsetCandidate> = candidate_invoices
                .iter()
                .filter(|inv| {
                    tx.narration.contains(&inv.invoice_id)
                        || tx.narration.contains(&inv.doc_ref)
                        || (!inv.doc_ref.is_empty()
                            && tx_norm.contains(&strip_ref_punctuation(&inv.doc_ref.to_uppercase())))
                })
                .enumerate()
                .map(|(idx, inv)| SubsetCandidate {
                    id: inv.invoice_id.clone(),
                    amount_vnd: inv.amount.amount(),
                    original_index: idx,
                })
                .collect();

            let solution = if ref_candidates.len() >= 2 {
                solve_subset_sum(target, &ref_candidates, &solver_config)
            } else {
                None
            };

            // Step 2: Prioritize candidates sharing counterparty tokens
            let solution = solution.or_else(|| {
                let cp_candidates: Vec<SubsetCandidate> = candidate_invoices
                    .iter()
                    .filter(|inv| {
                        let cp_norm = normalize_vietnamese_text(&inv.counterparty_name);
                        tx_norm.contains(&cp_norm)
                            || cp_norm
                                .split_whitespace()
                                .any(|w| w.len() > 3 && tx_norm.contains(w))
                    })
                    .enumerate()
                    .map(|(idx, inv)| SubsetCandidate {
                        id: inv.invoice_id.clone(),
                        amount_vnd: inv.amount.amount(),
                        original_index: idx,
                    })
                    .collect();

                if cp_candidates.len() >= 2 {
                    solve_subset_sum(target, &cp_candidates, &solver_config)
                } else {
                    None
                }
            });

            // Step 3: Fallback to all candidates
            let solution = solution.or_else(|| {
                let all_cands: Vec<SubsetCandidate> = candidate_invoices
                    .iter()
                    .enumerate()
                    .map(|(idx, inv)| SubsetCandidate {
                        id: inv.invoice_id.clone(),
                        amount_vnd: inv.amount.amount(),
                        original_index: idx,
                    })
                    .collect();
                solve_subset_sum(target, &all_cands, &solver_config)
            });

            if let Some(solution) = solution {
                let match_id = Uuid::new_v4().to_string();
                let k = solution.candidate_ids.len() as u32;
                let conf_bps = solver_config
                    .base_confidence_bps
                    .saturating_sub(k * solver_config.item_penalty_bps);
                let conf_score = (conf_bps / 100).min(100);

                for id in &solution.candidate_ids {
                    matched_inv_set.insert(id.clone());
                }
                matched_tx_set.insert(tx_idx);

                matched.push(ReconciliationMatch {
                    match_id,
                    tier: MatchTier::Tier3Split,
                    bank_tx_id: tx.row_id,
                    invoice_id: solution.candidate_ids[0].clone(),
                    bank_amount: tx.amount,
                    invoice_amount: tx.amount,
                    fee_deducted: None,
                    confidence_score: conf_score,
                    constituent_invoice_ids: Some(solution.candidate_ids),
                    constituent_bank_tx_ids: None,
                    is_semantic: false,
                    semantic_similarity_bps: None,
                });
            }
        }

        // ---------------------------------------------------------------------
        // Phase B: N:1 Split Resolution (N Bank Txs paying 1 Invoice)
        // ---------------------------------------------------------------------
        let remaining_inv_ids: Vec<String> = unmatched_inv_ids
            .iter()
            .filter(|id| !matched_inv_set.contains(*id))
            .cloned()
            .collect();

        for inv_id in &remaining_inv_ids {
            let inv = match inv_lookup.get(inv_id) {
                Some(i) => *i,
                None => continue,
            };
            let target = inv.amount.amount();
            if target <= 0 {
                continue;
            }

            let inv_cp_norm = normalize_vietnamese_text(&inv.counterparty_name);

            // Build candidate bank transactions
            let candidate_txs: Vec<(&CanonicalTx, usize)> = unmatched_tx_indices
                .iter()
                .filter(|idx| !matched_tx_set.contains(idx))
                .map(|&idx| (&all_txs[idx], idx))
                .filter(|(tx, _)| tx.amount.amount() > 0 && tx.amount.amount() < target)
                .collect();

            if candidate_txs.len() < 2 {
                continue;
            }

            // Step 1: Prioritize transactions sharing counterparty tokens
            let cp_txs: Vec<SubsetCandidate> = candidate_txs
                .iter()
                .filter(|(tx, _)| {
                    let tx_norm = normalize_vietnamese_text(&tx.narration);
                    tx_norm.contains(&inv_cp_norm)
                        || inv_cp_norm
                            .split_whitespace()
                            .any(|w| w.len() > 3 && tx_norm.contains(w))
                })
                .enumerate()
                .map(|(seq, (tx, _))| SubsetCandidate {
                    id: format!("{}", tx.row_id),
                    amount_vnd: tx.amount.amount(),
                    original_index: seq,
                })
                .collect();

            let solution = if cp_txs.len() >= 2 {
                solve_subset_sum(target, &cp_txs, &solver_config)
            } else {
                None
            };

            // Step 2: Fallback to all candidates
            let solution = solution.or_else(|| {
                let all_txs_cand: Vec<SubsetCandidate> = candidate_txs
                    .iter()
                    .enumerate()
                    .map(|(seq, (tx, _))| SubsetCandidate {
                        id: format!("{}", tx.row_id),
                        amount_vnd: tx.amount.amount(),
                        original_index: seq,
                    })
                    .collect();
                solve_subset_sum(target, &all_txs_cand, &solver_config)
            });

            if let Some(solution) = solution {
                let match_id = Uuid::new_v4().to_string();
                let k = solution.candidate_ids.len() as u32;
                let conf_bps = solver_config
                    .base_confidence_bps
                    .saturating_sub(k * solver_config.item_penalty_bps);
                let conf_score = (conf_bps / 100).min(100);

                let matched_tx_rows: Vec<usize> = solution
                    .candidate_ids
                    .iter()
                    .filter_map(|s| s.parse::<usize>().ok())
                    .collect();

                for &row_id in &matched_tx_rows {
                    if let Some(pos) = all_txs.iter().position(|t| t.row_id == row_id) {
                        matched_tx_set.insert(pos);
                    }
                }
                matched_inv_set.insert(inv_id.clone());

                matched.push(ReconciliationMatch {
                    match_id,
                    tier: MatchTier::Tier3Split,
                    bank_tx_id: matched_tx_rows.first().copied().unwrap_or(0),
                    invoice_id: inv.invoice_id.clone(),
                    bank_amount: inv.amount,
                    invoice_amount: inv.amount,
                    fee_deducted: None,
                    confidence_score: conf_score,
                    constituent_invoice_ids: None,
                    constituent_bank_tx_ids: Some(matched_tx_rows),
                    is_semantic: false,
                    semantic_similarity_bps: None,
                });
            }
        }

        let rem_txs: Vec<usize> = unmatched_tx_indices
            .iter()
            .copied()
            .filter(|i| !matched_tx_set.contains(i))
            .collect();

        let rem_invs: Vec<String> = unmatched_inv_ids
            .iter()
            .filter(|id| !matched_inv_set.contains(*id))
            .cloned()
            .collect();

        (matched, rem_txs, rem_invs)
    }

    /// Enqueues unresolved transactions into HITL quarantine with risk classification.
    pub fn enqueue_hitl(
        &self,
        unmatched_tx_indices: &[usize],
        all_txs: &[CanonicalTx],
        remaining_invoices: &[InternalInvoice],
        now_timestamp: i64,
    ) -> Vec<HitlQuarantineItem> {
        unmatched_tx_indices
            .iter()
            .map(|&idx| {
                let tx = &all_txs[idx];

                // Determine risk tier
                let mut risk_tier = QuarantineRiskTier::Low;
                let mut stat_ref = None;

                for rule in &self.config.hitl_quarantine.risk_rules {
                    if let Some(min_amt) = rule.min_amount_vnd {
                        if tx.amount.amount() >= min_amt {
                            risk_tier = risk_tier.max(rule.risk_tier);
                            stat_ref = Some(rule.statutory_reference.clone());
                        }
                    }
                }

                HitlQuarantineItem {
                    token: Uuid::new_v4().to_string(),
                    bank_tx: tx.clone(),
                    candidate_invoices: remaining_invoices.to_vec(),
                    created_at: now_timestamp,
                    ttl_seconds: self.config.hitl_quarantine.token_ttl_seconds,
                    maker: None,
                    checker: None,
                    approved: false,
                    risk_tier: Some(risk_tier),
                    statutory_reference: stat_ref,
                }
            })
            .collect()
    }

    /// Maker-Checker two-phase approval for a quarantined item.
    pub fn approve_hitl(
        &self,
        item: &mut HitlQuarantineItem,
        maker: impl Into<String>,
        checker: impl Into<String>,
        now: i64,
    ) -> Result<(), ReconError> {
        let m = maker.into();
        let c = checker.into();

        if self.config.hitl_quarantine.enforce_maker_checker_sod && m == c {
            return Err(ReconError::MakerCheckerViolation);
        }
        if now - item.created_at > item.ttl_seconds {
            return Err(ReconError::InvalidToken);
        }

        item.maker = Some(m);
        item.checker = Some(c);
        item.approved = true;
        Ok(())
    }

    /// Executes end-to-end 3-tier declarative reconciliation with optional semantic context.
    pub fn reconcile_batch_with_semantic(
        &self,
        txs: &[CanonicalTx],
        invoices: &[InternalInvoice],
        semantic_context: Option<&SemanticReconContext>,
    ) -> ReconBatchResult {
        // Tier 1 Exact Match
        let (t1_matches, rem_tx_1, rem_inv_1) = self.match_tier1(txs, invoices);

        // Tier 2 Fuzzy & Semantic Match
        let (t2_matches, rem_tx_2, rem_inv_2) =
            self.match_tier2_with_semantic(&rem_tx_1, txs, &rem_inv_1, invoices, semantic_context);

        // Tier 3 Subset-Sum Split Match
        let (t3_matches, rem_tx_3, rem_inv_3) =
            self.match_tier3(&rem_tx_2, txs, &rem_inv_2, invoices);

        // HITL Quarantine Queue
        let hitl_items = self.enqueue_hitl(
            &rem_tx_3,
            txs,
            invoices,
            chrono::Utc::now().timestamp(),
        );

        let mut all_matches = Vec::with_capacity(t1_matches.len() + t2_matches.len() + t3_matches.len());
        let exact_count = t1_matches.len();
        let split_count = t3_matches.len();

        let mut fuzzy_count = 0;
        let mut semantic_count = 0;
        for m in &t2_matches {
            if m.is_semantic {
                semantic_count += 1;
            } else {
                fuzzy_count += 1;
            }
        }

        all_matches.extend(t1_matches);
        all_matches.extend(t2_matches);
        all_matches.extend(t3_matches);

        let total_matched = all_matches.len();

        ReconBatchResult {
            matches: all_matches,
            hitl_quarantine: hitl_items,
            matched_exact_count: exact_count,
            matched_fuzzy_count: fuzzy_count,
            matched_semantic_count: semantic_count,
            matched_split_count: split_count,
            total_matched_count: total_matched,
            unmatched_tx_count: rem_tx_3.len(),
            unmatched_invoice_count: rem_inv_3.len(),
        }
    }

    /// Executes end-to-end 3-tier declarative reconciliation.
    pub fn reconcile_batch(
        &self,
        txs: &[CanonicalTx],
        invoices: &[InternalInvoice],
    ) -> ReconBatchResult {
        self.reconcile_batch_with_semantic(txs, invoices, None)
    }
}

impl ReconRuleConfig {
    /// Validates internal consistency of reconciliation rules.
    pub fn validate(&self) -> Result<(), ReconDslError> {
        if self.tier1.timestamp_tolerance_seconds < 0 {
            return Err(ReconDslError::Validation(
                "tier1.timestamp_tolerance_seconds cannot be negative".to_string(),
            ));
        }

        for pat in &self.tier1.reference_matching.extraction_patterns {
            Regex::new(pat).map_err(|source| ReconDslError::Regex {
                pattern: pat.clone(),
                source,
            })?;
        }

        if self.tier2.similarity_threshold_bps > 10_000 {
            return Err(ReconDslError::Validation(
                "tier2.similarity_threshold_bps must be <= 10,000".to_string(),
            ));
        }

        if self.tier2.uncertainty_lower_bound_bps > self.tier2.similarity_threshold_bps {
            return Err(ReconDslError::Validation(
                "tier2.uncertainty_lower_bound_bps must be <= similarity_threshold_bps".to_string(),
            ));
        }

        if self.tier2.semantic_elevation_threshold_bps > 10_000 {
            return Err(ReconDslError::Validation(
                "tier2.semantic_elevation_threshold_bps must be <= 10,000".to_string(),
            ));
        }

        if self.tier2.jw_weight + self.tier2.semantic_weight != 100 {
            return Err(ReconDslError::Validation(
                "tier2.jw_weight + tier2.semantic_weight must equal 100".to_string(),
            ));
        }

        for sched in &self.tier2.fee_schedules {
            if let FeeScheduleConfig::Range { min_vnd, max_vnd, name } = sched {
                if min_vnd > max_vnd {
                    return Err(ReconDslError::Validation(format!(
                        "Fee schedule range '{}' has min_vnd ({}) > max_vnd ({})",
                        name, min_vnd, max_vnd
                    )));
                }
            }
        }

        if self.tier3.max_depth < 2 || self.tier3.max_depth > 8 {
            return Err(ReconDslError::Validation(
                "tier3.max_depth must be between 2 and 8".to_string(),
            ));
        }

        if self.hitl_quarantine.token_ttl_seconds <= 0 {
            return Err(ReconDslError::Validation(
                "hitl_quarantine.token_ttl_seconds must be positive".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use liva_ledger::PostingType;

    #[test]
    fn test_engine_default_rules_valid() {
        let engine = DeclarativeReconEngine::default_rules();
        assert!(engine.config.validate().is_ok());
        assert_eq!(engine.config.metadata.name, "Standard Vietnamese Banking Reconciliation");
    }

    #[test]
    fn test_tier1_regex_extraction_from_narration() {
        let engine = DeclarativeReconEngine::default_rules();

        // Bank transaction has blank ref_code but narration contains invoice ref
        let txs = vec![CanonicalTx {
            row_id: 101,
            bank_code: "VCB".to_string(),
            ref_code: "".to_string(),
            posting_type: PostingType::Credit,
            amount: Money::vnd(25_000_000),
            balance_after: None,
            counterparty_name: None,
            counterparty_account: None,
            narration: "CTY ABC THANH TOAN TIEN THEO HD-2026-8889 XIN CAM ON".to_string(),
            tx_timestamp: 1726400000,
        }];

        let invoices = vec![InternalInvoice {
            invoice_id: "INV-8889".to_string(),
            doc_ref: "HD-2026-8889".to_string(),
            counterparty_name: "CONG TY TNHH ABC".to_string(),
            amount: Money::vnd(25_000_000),
            timestamp: 1726400000,
        }];

        let (matched, rem_tx, rem_inv) = engine.match_tier1(&txs, &invoices);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].tier, MatchTier::Tier1Exact);
        assert_eq!(matched[0].invoice_id, "INV-8889");
        assert!(rem_tx.is_empty());
        assert!(rem_inv.is_empty());
    }

    #[test]
    fn test_tier2_dynamic_fee_range() {
        let engine = DeclarativeReconEngine::default_rules();

        // Transaction with 7.700 fee deduction
        let txs = vec![CanonicalTx {
            row_id: 202,
            bank_code: "TCB".to_string(),
            ref_code: "".to_string(),
            posting_type: PostingType::Credit,
            amount: Money::vnd(49_992_300), // 50M - 7.700
            balance_after: None,
            counterparty_name: None,
            counterparty_account: None,
            narration: "CHUYEN TIEN MUA HANG CONG TY AN PHAT".to_string(),
            tx_timestamp: 1726400000,
        }];

        let invoices = vec![InternalInvoice {
            invoice_id: "INV-AP-01".to_string(),
            doc_ref: "".to_string(),
            counterparty_name: "CÔNG TY CỔ PHẦN AN PHÁT".to_string(),
            amount: Money::vnd(50_000_000),
            timestamp: 1726400000,
        }];

        let (matched, rem_tx, rem_inv) = engine.match_tier2(&[0], &txs, &["INV-AP-01".to_string()], &invoices);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].fee_deducted, Some(Money::vnd(7_700)));
        assert!(rem_tx.is_empty());
        assert!(rem_inv.is_empty());
    }

    #[test]
    fn test_hitl_critical_risk_classification() {
        let engine = DeclarativeReconEngine::default_rules();

        let txs = vec![CanonicalTx {
            row_id: 303,
            bank_code: "BIDV".to_string(),
            ref_code: "TX-LARGE".to_string(),
            posting_type: PostingType::Credit,
            amount: Money::vnd(500_000_000), // >= 400M VND
            balance_after: None,
            counterparty_name: None,
            counterparty_account: None,
            narration: "Giao dich gia tri lon chua co hoa don".to_string(),
            tx_timestamp: 1726400000,
        }];

        let queue = engine.enqueue_hitl(&[0], &txs, &[], 1726400000);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].risk_tier, Some(QuarantineRiskTier::Critical));
        assert!(queue[0].statutory_reference.as_ref().unwrap().contains("11/2023"));
    }
}
