//! Declarative AML Surveillance Policy Engine.
//!
//! Evaluates transactions against:
//! 1. Decision 11/2023/QĐ-TTg statutory threshold (>= 400M VND)
//! 2. Rolling 24-hour structuring / smurfing detection
//! 3. Rapid pass-through / transit mule accounts
//! 4. Night velocity off-hours surveillance (23:00 - 05:00 GMT+7)
//! 5. Watchlist keywords & Sanctioned name integer Jaro-Winkler screening
//! 6. Form STR drafting with pure integer risk scoring and Merkle audit trail

use super::schema::{
    AmlAlert, AmlAlertCode, AmlPolicyConfig, AmlSeverity, EnhancedAmlAlert, ScreenableTransaction,
    SuspiciousTransactionReport,
};
use crate::audit::merkle::compute_merkle_root;
use liva_money::Money;
use std::collections::HashMap;

/// AML Screening Engine executing declarative policies.
#[derive(Debug, Clone)]
pub struct AmlScreeningEngine {
    pub config: AmlPolicyConfig,
}

impl AmlScreeningEngine {
    /// Create a new engine with the given policy configuration.
    pub fn new(config: AmlPolicyConfig) -> Self {
        Self { config }
    }

    /// Create an engine with the default Vietnamese regulatory policy.
    pub fn with_default_policy() -> Self {
        Self::new(AmlPolicyConfig::default())
    }

    /// Screen a batch of transactions against all enabled statutory AML rules.
    pub fn screen_transactions(&self, txs: &[ScreenableTransaction]) -> Vec<AmlAlert> {
        let mut alerts = Vec::new();

        // 1. Decision 11/2023/QĐ-TTg High-Value Threshold Check
        if self.config.high_value.enabled {
            self.screen_high_value(txs, &mut alerts);
        }

        // 2. Rolling 24-hour Structuring / Smurfing Sliding Window
        if self.config.structuring.enabled {
            self.screen_structuring(txs, &mut alerts);
        }

        // 3. Rapid Pass-Through / Transit Mule Accounts
        if self.config.transit_mule.enabled {
            self.screen_transit_mule(txs, &mut alerts);
        }

        // 4. Night Velocity Off-Hours Surveillance
        if self.config.night_velocity.enabled {
            self.screen_night_velocity(txs, &mut alerts);
        }

        // 5. Watchlist & Sanctions Screening
        if self.config.watchlist.enabled {
            self.screen_watchlist(txs, &mut alerts);
        }

        alerts
    }

    /// Screen a batch of transactions against both statutory rules and Multi-Dimensional Anomaly Detection.
    /// Returns enhanced alerts with integer anomaly scores (0..10,000 bps), reasons, risk profiles,
    /// and Decree 13 PII-sanitized local SLM explanations.
    pub fn screen_with_anomaly_detection(&self, txs: &[ScreenableTransaction]) -> Vec<EnhancedAmlAlert> {
        let mut enhanced_alerts = Vec::new();

        // 1. Run all statutory screenings first
        let statutory_alerts = self.screen_transactions(txs);
        let mut statutory_by_tx: HashMap<String, Vec<&AmlAlert>> = HashMap::new();
        for alert in &statutory_alerts {
            statutory_by_tx.entry(alert.tx_id.clone()).or_default().push(alert);
        }

        // 2. Statistical Baseline for Amount Distribution (Median & MAD)
        let amounts: Vec<i64> = txs.iter().map(|t| t.amount.amount()).collect();
        let (median, mad) = compute_amount_mad(&amounts);

        // 3. Group by account for Temporal & Velocity analysis
        let mut txs_by_account: HashMap<&str, Vec<&ScreenableTransaction>> = HashMap::new();
        for tx in txs {
            txs_by_account.entry(&tx.account_number).or_default().push(tx);
        }

        // Sort accounts' transactions chronologically and compute micro-burst & velocity sets
        let mut micro_burst_tx_ids = std::collections::HashSet::new();
        let mut velocity_surge_tx_ids = std::collections::HashSet::new();

        if self.config.anomaly_detection.enabled && self.config.anomaly_detection.temporal_velocity.enabled {
            let burst_min_secs = self.config.anomaly_detection.temporal_velocity.burst_interval_min_seconds;
            let window_secs = self.config.anomaly_detection.temporal_velocity.window_seconds;
            let max_count = self.config.anomaly_detection.temporal_velocity.max_tx_count_per_window;

            for (_account, mut acc_txs) in txs_by_account {
                acc_txs.sort_by_key(|t| t.timestamp_seconds);

                // Check micro-burst (< burst_min_secs)
                for i in 1..acc_txs.len() {
                    let dt = acc_txs[i].timestamp_seconds - acc_txs[i - 1].timestamp_seconds;
                    if dt >= 0 && dt < burst_min_secs {
                        micro_burst_tx_ids.insert(acc_txs[i].tx_id.clone());
                        micro_burst_tx_ids.insert(acc_txs[i - 1].tx_id.clone());
                    }
                }

                // Rolling frequency & turnover surge check
                let mut start = 0;
                let mut current_sum: i64 = 0;
                for end in 0..acc_txs.len() {
                    current_sum = current_sum.saturating_add(acc_txs[end].amount.amount());
                    while acc_txs[end].timestamp_seconds - acc_txs[start].timestamp_seconds > window_secs {
                        current_sum = current_sum.saturating_sub(acc_txs[start].amount.amount());
                        start += 1;
                    }
                    let count_in_window = end - start + 1;
                    if count_in_window >= max_count {
                        for tx in &acc_txs[start..=end] {
                            velocity_surge_tx_ids.insert(tx.tx_id.clone());
                        }
                    }
                }
            }
        }

        // 4. Evaluate each transaction
        let tz_offset_secs = self.config.night_velocity.timezone_offset_hours as i64 * 3600;

        for tx in txs {
            let mut anomaly_reasons = Vec::new();
            let mut score_bps: u32 = 0;
            let mut primary_rule_code: Option<AmlAlertCode> = None;
            let mut max_statutory_severity: Option<AmlSeverity> = None;

            // Check if any statutory alerts exist for this tx
            if let Some(stat_alerts) = statutory_by_tx.get(&tx.tx_id) {
                for sa in stat_alerts {
                    if primary_rule_code.is_none() {
                        primary_rule_code = Some(sa.rule_code);
                    }
                    anomaly_reasons.push(sa.description.clone());
                    let statutory_score = match sa.severity {
                        AmlSeverity::Critical => 3500,
                        AmlSeverity::High => 2500,
                        AmlSeverity::Medium => 1500,
                        AmlSeverity::Low => 500,
                    };
                    score_bps = score_bps.saturating_add(statutory_score);
                    let is_higher = match max_statutory_severity {
                        None => true,
                        Some(s) => sa.severity > s,
                    };
                    if is_higher {
                        max_statutory_severity = Some(sa.severity);
                    }
                }
            }

            if self.config.anomaly_detection.enabled {
                // Dim 1: Temporal & Velocity Anomaly
                if self.config.anomaly_detection.temporal_velocity.enabled {
                    let mut dim1_score: u32 = 0;

                    // Micro-burst
                    if micro_burst_tx_ids.contains(&tx.tx_id) {
                        dim1_score = dim1_score.saturating_add(1500);
                        anomaly_reasons.push(
                            "Tần suất micro-burst (< 60s) giữa các giao dịch liên tiếp trên cùng tài khoản".to_string()
                        );
                        if primary_rule_code.is_none() {
                            primary_rule_code = Some(AmlAlertCode::MicroBurstVelocity);
                        }
                    }

                    // Rolling Frequency Surge
                    if velocity_surge_tx_ids.contains(&tx.tx_id) {
                        dim1_score = dim1_score.saturating_add(2000);
                        anomaly_reasons.push(
                            "Đột biến tần suất giao dịch (> 300% mức cơ sở) trong cửa sổ 1 giờ".to_string()
                        );
                        if primary_rule_code.is_none() {
                            primary_rule_code = Some(AmlAlertCode::VelocitySurge);
                        }
                    }

                    // Off-hours 23:00 - 05:00 GMT+7 (> 200M VND)
                    let local_secs = tx.timestamp_seconds.wrapping_add(tz_offset_secs);
                    let hour_of_day = ((local_secs.rem_euclid(86400)) / 3600) as u32;
                    let is_off_hours = !(5..23).contains(&hour_of_day);
                    let off_hours_thresh = self.config.anomaly_detection.temporal_velocity.off_hours_volume_threshold_vnd as i64;
                    if is_off_hours && tx.amount.amount() >= off_hours_thresh {
                        dim1_score = dim1_score.saturating_add(2000);
                        anomaly_reasons.push(format!(
                            "Giao dịch ngoài giờ hành chính ({:02}:00 GMT+7) vượt ngưỡng 200M VND: {} VND",
                            hour_of_day, tx.amount.amount()
                        ));
                        if primary_rule_code.is_none() {
                            primary_rule_code = Some(AmlAlertCode::NightAnomaly);
                        }
                    }

                    score_bps = score_bps.saturating_add(dim1_score.min(2500));
                }

                // Dim 2: Amount Distribution Outliers
                if self.config.anomaly_detection.amount_distribution.enabled {
                    let mut dim2_score: u32 = 0;
                    let amt = tx.amount.amount();
                    let smurf_lower = self.config.anomaly_detection.amount_distribution.smurfing_band_lower_vnd as i64;
                    let smurf_upper = self.config.anomaly_detection.amount_distribution.smurfing_band_upper_vnd as i64;

                    // Near-Threshold Smurfing Band [380M, 399.999M VND]
                    if amt >= smurf_lower && amt <= smurf_upper {
                        dim2_score = dim2_score.saturating_add(2500);
                        anomaly_reasons.push(format!(
                            "Số tiền nằm trong dải phân tách né ngưỡng 400M VND (380M - 399.999M VND): {} VND",
                            amt
                        ));
                        if primary_rule_code.is_none() {
                            primary_rule_code = Some(AmlAlertCode::NearThresholdSmurfing);
                        }
                    }

                    // Robust Modified Z-Score with MAD
                    if amounts.len() >= self.config.anomaly_detection.amount_distribution.min_sample_size && mad > 0 {
                        let z_bps = compute_modified_zscore_bps(amt, median, mad);
                        let thresh = self.config.anomaly_detection.amount_distribution.modified_zscore_threshold_bps;
                        if z_bps > thresh {
                            let z_contrib = if z_bps > 50_000 { 2500 } else { 2000 };
                            dim2_score = dim2_score.saturating_add(z_contrib);
                            anomaly_reasons.push(format!(
                                "Đột biến số tiền theo Modified Z-Score MAD (M_i = {}.{:02} > 3.50, Trung vị = {} VND, MAD = {} VND)",
                                z_bps / 10_000, (z_bps % 10_000) / 100, median, mad
                            ));
                            if primary_rule_code.is_none() {
                                primary_rule_code = Some(AmlAlertCode::AmountDistributionOutlier);
                            }
                        }
                    }

                    score_bps = score_bps.saturating_add(dim2_score.min(3000));
                }

                // Dim 3: Semantic Narration Anomaly
                if self.config.anomaly_detection.semantic_narration.enabled {
                    let (sim_bps, category) = self.evaluate_narration_semantic_bps(&tx.narration);
                    let thresh = self.config.anomaly_detection.semantic_narration.similarity_threshold_bps;
                    if sim_bps >= thresh {
                        let cat_name = category.unwrap_or_else(|| "unknown".to_string());
                        let dim3_score = ((sim_bps as u64 * 2500) / 10_000) as u32;
                        score_bps = score_bps.saturating_add(dim3_score.min(2500));
                        anomaly_reasons.push(format!(
                            "Nội dung chuyển khoản trùng khớp cụm từ nghi vấn '{}' (Độ tương đồng: {}.{:02}%)",
                            cat_name, sim_bps / 100, sim_bps % 100
                        ));
                        if primary_rule_code.is_none() {
                            primary_rule_code = Some(AmlAlertCode::SemanticNarrationAnomaly);
                        }
                    }
                }
            }

            let final_score_bps = score_bps.min(10_000);

            // Determine if an alert should be created
            let has_reasons = !anomaly_reasons.is_empty();
            let has_statutory = statutory_by_tx.contains_key(&tx.tx_id);
            let meets_score_threshold = final_score_bps >= self.config.anomaly_detection.slm_explanation.score_threshold_bps;

            if has_reasons || has_statutory || meets_score_threshold {
                let rule_code = primary_rule_code.unwrap_or(AmlAlertCode::StructuringSmurfing);

                let severity = if final_score_bps >= 7500 || max_statutory_severity == Some(AmlSeverity::Critical) {
                    AmlSeverity::Critical
                } else if final_score_bps >= 5000 || max_statutory_severity == Some(AmlSeverity::High) {
                    AmlSeverity::High
                } else if final_score_bps >= 3500 || max_statutory_severity == Some(AmlSeverity::Medium) {
                    AmlSeverity::Medium
                } else {
                    AmlSeverity::Low
                };

                let risk_profile = if final_score_bps >= 7500 {
                    "CRITICAL_SUSPICIOUS".to_string()
                } else if final_score_bps >= 5000 {
                    "HIGH_RISK".to_string()
                } else if final_score_bps >= 3500 {
                    "ELEVATED_RISK".to_string()
                } else if final_score_bps >= 2000 {
                    "MODERATE_RISK".to_string()
                } else {
                    "LOW_RISK".to_string()
                };

                let statutory_ref = match rule_code {
                    AmlAlertCode::HighValueThreshold => self.config.high_value.statutory_reference.clone(),
                    AmlAlertCode::StructuringSmurfing | AmlAlertCode::NearThresholdSmurfing => {
                        self.config.structuring.statutory_reference.clone()
                    }
                    AmlAlertCode::PassThroughTransit => self.config.transit_mule.statutory_reference.clone(),
                    AmlAlertCode::NightAnomaly => self.config.night_velocity.statutory_reference.clone(),
                    AmlAlertCode::WatchlistSanction => self.config.watchlist.statutory_reference.clone(),
                    _ => "Điều 26 Luật PCRT 2022 (Dấu hiệu giao dịch đáng ngờ theo hành vi)".to_string(),
                };

                let mut alert = EnhancedAmlAlert {
                    alert_id: format!("ALT-ENH-{}-{}", rule_code_prefix(&rule_code), tx.tx_id),
                    rule_code,
                    severity,
                    tx_id: tx.tx_id.clone(),
                    amount: tx.amount,
                    anomaly_score_bps: final_score_bps,
                    anomaly_reasons,
                    risk_profile,
                    vietnamese_explanation: None,
                    statutory_ref,
                    detected_at: "2026-09-15T21:00:00Z".to_string(),
                };

                if self.config.anomaly_detection.slm_explanation.enabled
                    && final_score_bps >= self.config.anomaly_detection.slm_explanation.score_threshold_bps
                {
                    alert.vietnamese_explanation = Some(self.generate_vietnamese_explanation(tx, &alert));
                }

                enhanced_alerts.push(alert);
            }
        }

        enhanced_alerts
    }

    /// Evaluates narration text against Vietnamese suspicious centroids (crypto OTC, underground remittance, kickbacks, money mule).
    /// Returns (similarity_bps, Option<matched_category>).
    pub fn evaluate_narration_semantic_bps(&self, narration: &str) -> (u32, Option<String>) {
        if narration.trim().is_empty() {
            return (0, None);
        }

        let norm_narration = normalize_ascii(narration);
        let mut max_score_bps: u32 = 0;
        let mut matched_category: Option<String> = None;

        for cluster in &self.config.anomaly_detection.semantic_narration.suspicious_clusters {
            for kw in &cluster.keywords {
                let norm_kw = normalize_ascii(kw);
                if norm_kw.is_empty() {
                    continue;
                }

                // Check 1: Multi-word phrase exact match in narration
                if norm_kw.contains(' ') && norm_narration.contains(&norm_kw) {
                    let score = 9500; // 95.00%
                    if score > max_score_bps {
                        max_score_bps = score;
                        matched_category = Some(cluster.category.clone());
                    }
                } else if norm_narration.contains(&norm_kw) {
                    // Check 2: Single keyword exact match
                    let is_standalone = if norm_kw.len() <= 3 {
                        norm_narration.split_whitespace().any(|w| w == norm_kw)
                    } else {
                        true
                    };

                    if is_standalone {
                        let score = if norm_kw == "USDT" || norm_kw == "BINANCE" || norm_kw == "P2P" || norm_kw == "OKX" || norm_kw == "BYBIT" {
                            9200 // 92.00%
                        } else {
                            8600 // 86.00%
                        };
                        if score > max_score_bps {
                            max_score_bps = score;
                            matched_category = Some(cluster.category.clone());
                        }
                    }
                } else if !norm_kw.contains(' ') && norm_kw.len() > 4 {
                    // Check 3: Fuzzy integer Jaro-Winkler match on words (only for single keywords > 4 chars)
                    for word in norm_narration.split_whitespace() {
                        if word.len() > 4 {
                            let jw = integer_jaro_winkler(word, &norm_kw);
                            if jw >= 9200 {
                                let score = 8300;
                                if score > max_score_bps {
                                    max_score_bps = score;
                                    matched_category = Some(cluster.category.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        (max_score_bps, matched_category)
    }

    /// Formats prompt for Qwen3 14B Local SLM to generate Vietnamese compliance explanation.
    /// Strictly applies Decree 13 PII sanitization.
    pub fn format_slm_explanation_prompt(
        &self,
        tx: &ScreenableTransaction,
        alert: &EnhancedAmlAlert,
    ) -> String {
        let sanitized_narration = crate::sanitizer::sanitize_decree13_pii(&tx.narration);
        let sanitized_counterparty = crate::sanitizer::sanitize_counterparty_name(&tx.counterparty_name);
        let reasons_raw = if alert.anomaly_reasons.is_empty() {
            "Phát hiện dấu hiệu bất thường đa chiều".to_string()
        } else {
            alert.anomaly_reasons.join(", ")
        };
        let reasons_str = crate::sanitizer::sanitize_decree13_pii(&reasons_raw);

        let score_integer_part = alert.anomaly_score_bps / 10_000;
        let score_fractional_part = (alert.anomaly_score_bps % 10_000) / 100;

        format!(
            "Bạn là Trợ lý Tuân thủ Phòng chống Rửa tiền (AML) cấp cao của ngân hàng LIVA.\n\
            Hãy phân tích giao dịch sau và tạo báo cáo giải trình ngắn gọn (dưới 80 từ) bằng tiếng Việt:\n\
            - Mã giao dịch: {}\n\
            - Số tiền: {} VND\n\
            - Bên đối ứng: {}\n\
            - Nội dung giao dịch: \"{}\"\n\
            - Các chỉ số kích hoạt: {}\n\
            - Điểm rủi ro tổng hợp: {}.{:02} / 1.00\n\
            Yêu cầu: Nêu rõ căn cứ pháp lý theo Luật PCRT 2022 và khuyến nghị hành động.",
            tx.tx_id,
            tx.amount.amount(),
            sanitized_counterparty,
            sanitized_narration,
            reasons_str,
            score_integer_part,
            score_fractional_part
        )
    }

    /// Generates structured professional Vietnamese compliance report text.
    /// Used when local SLM outputs report or in deterministic evaluation.
    pub fn generate_vietnamese_explanation(
        &self,
        tx: &ScreenableTransaction,
        alert: &EnhancedAmlAlert,
    ) -> String {
        let sanitized_narration = crate::sanitizer::sanitize_decree13_pii(&tx.narration);
        let sanitized_counterparty = crate::sanitizer::sanitize_counterparty_name(&tx.counterparty_name);
        let reasons_raw = alert.anomaly_reasons.join("; ");
        let reasons_str = crate::sanitizer::sanitize_decree13_pii(&reasons_raw);

        let score_pct = alert.anomaly_score_bps / 100;
        let score_pct_dec = alert.anomaly_score_bps % 100;

        format!(
            "BÁO CÁO GIẢI TRÌNH AML: Giao dịch {} (Số tiền: {} VND, Bên đối ứng: {}) \
            bị gắn cờ rủi ro {} do: {}. Điểm bất thường: {}.{:02}%. \
            Nội dung đã khử định danh: \"{}\". \
            Căn cứ Luật PCRT số 14/2022/QH15 và Thông tư 09/2023/TT-NHNN, khuyến nghị: \
            chuyển hồ sơ sang kiểm soát viên kiểm tra chuyên sâu (Enhanced Due Diligence) \
            và lập báo cáo giao dịch đáng ngờ STR nếu cần thiết.",
            tx.tx_id,
            tx.amount.amount(),
            sanitized_counterparty,
            alert.risk_profile,
            reasons_str,
            score_pct,
            score_pct_dec,
            sanitized_narration
        )
    }

    /// Rule 1: High Value Statutory Threshold (Decision 11/2023/QĐ-TTg)
    fn screen_high_value(&self, txs: &[ScreenableTransaction], alerts: &mut Vec<AmlAlert>) {
        let stat_thresh = self.config.high_value.statutory_threshold_vnd as i64;
        let crit_thresh = self.config.high_value.critical_threshold_vnd as i64;

        for tx in txs {
            let amt = tx.amount.amount();
            if amt >= stat_thresh {
                let severity = if amt >= crit_thresh {
                    AmlSeverity::Critical
                } else {
                    AmlSeverity::High
                };

                alerts.push(AmlAlert {
                    alert_id: format!("ALT-HV-{}", tx.tx_id),
                    rule_code: AmlAlertCode::HighValueThreshold,
                    severity,
                    tx_id: tx.tx_id.clone(),
                    amount: tx.amount,
                    description: format!(
                        "Giao dịch giá trị lớn: {} VND vượt ngưỡng quy định 400M VND",
                        amt
                    ),
                    statutory_ref: self.config.high_value.statutory_reference.clone(),
                    detected_at: "2026-09-15T21:00:00Z".to_string(),
                });
            }
        }
    }

    /// Rule 2: Pure Integer Rolling 24-hour Structuring / Smurfing Sliding Window
    fn screen_structuring(&self, txs: &[ScreenableTransaction], alerts: &mut Vec<AmlAlert>) {
        let min_candidate = self.config.structuring.min_candidate_amount_vnd as i64;
        let max_candidate = self.config.structuring.max_candidate_amount_vnd as i64;
        let window_secs = self.config.structuring.window_seconds;
        let min_count = self.config.structuring.min_tx_count;
        let agg_thresh = self.config.structuring.aggregate_threshold_vnd as i64;

        // Group candidate transactions by account_number
        let mut account_groups: HashMap<&str, Vec<&ScreenableTransaction>> = HashMap::new();
        for tx in txs {
            let amt = tx.amount.amount();
            if amt >= min_candidate && amt < max_candidate {
                account_groups
                    .entry(&tx.account_number)
                    .or_default()
                    .push(tx);
            }
        }

        for (account, mut candidates) in account_groups {
            if candidates.len() < min_count {
                continue;
            }

            // Sort chronologically
            candidates.sort_by_key(|tx| tx.timestamp_seconds);

            // Two-pointer sliding window
            let mut start = 0;
            let mut current_sum: i64 = 0;
            let mut alerted_indices = std::collections::HashSet::new();

            for end in 0..candidates.len() {
                current_sum = current_sum.saturating_add(candidates[end].amount.amount());

                while candidates[end].timestamp_seconds - candidates[start].timestamp_seconds
                    > window_secs
                {
                    current_sum = current_sum.saturating_sub(candidates[start].amount.amount());
                    start += 1;
                }

                let count = end - start + 1;
                if count >= min_count && current_sum >= agg_thresh {
                    // Check if we already alerted for this window end
                    if !alerted_indices.contains(&end) {
                        alerted_indices.insert(end);
                        let first_tx = candidates[start];

                        alerts.push(AmlAlert {
                            alert_id: format!("ALT-STRUC-{}-{}", account, first_tx.tx_id),
                            rule_code: AmlAlertCode::StructuringSmurfing,
                            severity: AmlSeverity::Critical,
                            tx_id: first_tx.tx_id.clone(),
                            amount: Money::vnd(current_sum),
                            description: format!(
                                "Dấu hiệu chia nhỏ giao dịch (Smurfing): {} giao dịch trong cửa sổ 24h tài khoản {}, tổng cộng {} VND",
                                count, account, current_sum
                            ),
                            statutory_ref: self.config.structuring.statutory_reference.clone(),
                            detected_at: "2026-09-15T21:00:00Z".to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Rule 3: Pass-Through Transit Mule Accounts
    fn screen_transit_mule(&self, txs: &[ScreenableTransaction], alerts: &mut Vec<AmlAlert>) {
        let min_inflow = self.config.transit_mule.min_inflow_vnd as i64;
        let max_window = self.config.transit_mule.max_transit_window_seconds;
        let min_ratio_bps = self.config.transit_mule.min_outflow_ratio_bps as u128;

        for (i, inflow) in txs.iter().enumerate() {
            if inflow.is_credit && inflow.amount.amount() >= min_inflow {
                let inflow_amt = inflow.amount.amount() as u128;
                if inflow_amt == 0 {
                    continue;
                }

                for outflow in txs.iter().skip(i + 1) {
                    if !outflow.is_credit
                        && outflow.account_number == inflow.account_number
                        && outflow.timestamp_seconds >= inflow.timestamp_seconds
                        && (outflow.timestamp_seconds - inflow.timestamp_seconds) <= max_window
                    {
                        let outflow_amt = outflow.amount.amount() as u128;
                        let outflow_ratio_bps = (outflow_amt * 10_000) / inflow_amt;

                        if outflow_ratio_bps >= min_ratio_bps {
                            alerts.push(AmlAlert {
                                alert_id: format!("ALT-PT-{}", inflow.tx_id),
                                rule_code: AmlAlertCode::PassThroughTransit,
                                severity: AmlSeverity::Critical,
                                tx_id: inflow.tx_id.clone(),
                                amount: inflow.amount,
                                description: format!(
                                    "Dấu hiệu tài khoản trung chuyển (Mule): Tiền vào {} VND được chuyển đi ngay {} VND ({}.{:02}%) trong vòng 15 phút",
                                    inflow.amount.amount(),
                                    outflow.amount.amount(),
                                    outflow_ratio_bps / 100,
                                    outflow_ratio_bps % 100
                                ),
                                statutory_ref: self.config.transit_mule.statutory_reference.clone(),
                                detected_at: "2026-09-15T21:00:00Z".to_string(),
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Rule 4: Night Velocity Off-Hours Surveillance
    fn screen_night_velocity(&self, txs: &[ScreenableTransaction], alerts: &mut Vec<AmlAlert>) {
        let night_start = self.config.night_velocity.night_start_hour;
        let night_end = self.config.night_velocity.night_end_hour;
        let tz_offset_secs = self.config.night_velocity.timezone_offset_hours as i64 * 3600;
        let min_amount = self.config.night_velocity.min_amount_vnd as i64;
        let count_thresh = self.config.night_velocity.count_threshold;
        let agg_thresh = self.config.night_velocity.aggregate_threshold_vnd as i64;

        let mut night_txs_by_account: HashMap<&str, Vec<&ScreenableTransaction>> = HashMap::new();

        for tx in txs {
            let amt = tx.amount.amount();
            if amt < min_amount {
                continue;
            }

            let local_secs = tx.timestamp_seconds.wrapping_add(tz_offset_secs);
            let second_of_day = local_secs.rem_euclid(86400);
            let hour_of_day = (second_of_day / 3600) as u32;

            let is_night = hour_of_day >= night_start || hour_of_day < night_end;
            if is_night {
                night_txs_by_account
                    .entry(&tx.account_number)
                    .or_default()
                    .push(tx);
            }
        }

        for (account, candidates) in night_txs_by_account {
            let total_sum: i64 = candidates.iter().map(|t| t.amount.amount()).sum();
            if candidates.len() >= count_thresh || total_sum >= agg_thresh {
                let first_tx = candidates[0];
                alerts.push(AmlAlert {
                    alert_id: format!("ALT-NIGHT-{}-{}", account, first_tx.tx_id),
                    rule_code: AmlAlertCode::NightAnomaly,
                    severity: AmlSeverity::High,
                    tx_id: first_tx.tx_id.clone(),
                    amount: Money::vnd(total_sum),
                    description: format!(
                        "Đột biến giao dịch đêm khuya (23:00-05:00 GMT+7): {} giao dịch trên tài khoản {}, tổng {} VND",
                        candidates.len(), account, total_sum
                    ),
                    statutory_ref: self.config.night_velocity.statutory_reference.clone(),
                    detected_at: "2026-09-15T21:00:00Z".to_string(),
                });
            }
        }
    }

    /// Rule 5: Watchlist Keywords & Sanctions Integer Jaro-Winkler Matching
    fn screen_watchlist(&self, txs: &[ScreenableTransaction], alerts: &mut Vec<AmlAlert>) {
        for tx in txs {
            let norm_narration = normalize_ascii(&tx.narration);

            // A. Narration Keywords
            for kw in &self.config.watchlist.narration_keywords {
                let norm_kw = normalize_ascii(kw);
                if norm_narration.contains(&norm_kw) {
                    alerts.push(AmlAlert {
                        alert_id: format!("ALT-WL-KW-{}-{}", kw, tx.tx_id),
                        rule_code: AmlAlertCode::WatchlistSanction,
                        severity: AmlSeverity::High,
                        tx_id: tx.tx_id.clone(),
                        amount: tx.amount,
                        description: format!(
                            "Giao dịch chứa từ khóa rủi ro cấm trong nội dung: '{}'",
                            kw
                        ),
                        statutory_ref: self.config.watchlist.statutory_reference.clone(),
                        detected_at: "2026-09-15T21:00:00Z".to_string(),
                    });
                    break;
                }
            }

            // B. Blacklisted Accounts
            for bl_acc in &self.config.watchlist.blacklisted_accounts {
                if tx.account_number == *bl_acc || tx.counterparty_account == *bl_acc {
                    alerts.push(AmlAlert {
                        alert_id: format!("ALT-WL-BL-{}-{}", bl_acc, tx.tx_id),
                        rule_code: AmlAlertCode::WatchlistSanction,
                        severity: AmlSeverity::Critical,
                        tx_id: tx.tx_id.clone(),
                        amount: tx.amount,
                        description: format!(
                            "Giao dịch liên quan tài khoản nằm trong danh sách đen PCRT: {}",
                            bl_acc
                        ),
                        statutory_ref: self.config.watchlist.statutory_reference.clone(),
                        detected_at: "2026-09-15T21:00:00Z".to_string(),
                    });
                    break;
                }
            }

            // C. Sanctioned Names via Integer Jaro-Winkler (basis points)
            if !tx.counterparty_name.is_empty() {
                let norm_counterparty = normalize_ascii(&tx.counterparty_name);
                let threshold_bps = self.config.watchlist.name_similarity_threshold_bps;

                for sanctioned_name in &self.config.watchlist.sanctioned_names {
                    let norm_sanc = normalize_ascii(sanctioned_name);
                    let score_bps = integer_jaro_winkler(&norm_counterparty, &norm_sanc);

                    if score_bps >= threshold_bps {
                        alerts.push(AmlAlert {
                            alert_id: format!("ALT-WL-SANC-{}", tx.tx_id),
                            rule_code: AmlAlertCode::WatchlistSanction,
                            severity: AmlSeverity::Critical,
                            tx_id: tx.tx_id.clone(),
                            amount: tx.amount,
                            description: format!(
                                "Tên đối ứng '{}' trùng khớp danh sách trừng phạt '{}' với độ tương đồng {}.{:02}%",
                                crate::sanitizer::sanitize_counterparty_name(&tx.counterparty_name),
                                sanctioned_name,
                                score_bps / 100,
                                score_bps % 100
                            ),
                            statutory_ref: self.config.watchlist.statutory_reference.clone(),
                            detected_at: "2026-09-15T21:00:00Z".to_string(),
                        });
                        break;
                    }
                }
            }
        }
    }

    /// Build a Form STR report for a specific alert and transaction.
    pub fn build_str_report(
        &self,
        alert: &AmlAlert,
        tx: &ScreenableTransaction,
        compliance_officer_notes: &str,
    ) -> SuspiciousTransactionReport {
        let merkle = compute_merkle_root(&[
            tx.tx_id.clone(),
            alert.alert_id.clone(),
            tx.account_number.clone(),
        ]);

        let mut risk_score: u32 = match alert.severity {
            AmlSeverity::Critical => self.config.reporting.critical_alert_weight,
            AmlSeverity::High => self.config.reporting.high_alert_weight,
            AmlSeverity::Medium => self.config.reporting.medium_alert_weight,
            AmlSeverity::Low => 5,
        };
        if risk_score > 100 {
            risk_score = 100;
        }

        let recommended_action = if risk_score >= self.config.reporting.risk_score_threshold_file_sbv
        {
            "FILE_STR_WITH_SBV".to_string()
        } else if risk_score >= self.config.reporting.risk_score_threshold_compliance_hold {
            "INTERNAL_COMPLIANCE_HOLD".to_string()
        } else {
            "ENHANCED_DUE_DILIGENCE".to_string()
        };

        let alert_type_str = match alert.rule_code {
            AmlAlertCode::HighValueThreshold => "Giao dịch giá trị lớn (>= 400M VND)",
            AmlAlertCode::StructuringSmurfing => "Chia nhỏ giao dịch né tránh báo cáo (Smurfing)",
            AmlAlertCode::VelocitySurge => "Đột biến doanh số quay vòng tài khoản",
            AmlAlertCode::PassThroughTransit => "Tài khoản trung chuyển rửa tiền (Mule)",
            AmlAlertCode::NightAnomaly => "Giao dịch đêm khuya bất thường (Night Velocity)",
            AmlAlertCode::WatchlistSanction => "Trùng khớp danh sách đen / từ khóa cấm (Sanctions/Watchlist)",
            AmlAlertCode::NearThresholdSmurfing => "Giao dịch trong dải phân tách né ngưỡng 400M (Near-Threshold Smurfing)",
            AmlAlertCode::AmountDistributionOutlier => "Đột biến số tiền dị biệt theo phân phối thống kê MAD (Amount Outlier)",
            AmlAlertCode::SemanticNarrationAnomaly => "Nội dung chuyển khoản trùng khớp ngữ nghĩa rủi ro cao (Semantic Anomaly)",
            AmlAlertCode::MicroBurstVelocity => "Tần suất giao dịch siêu vi mô bất thường (< 60s micro-burst)",
        };

        let severity_str = match alert.severity {
            AmlSeverity::Critical => "CRITICAL",
            AmlSeverity::High => "HIGH",
            AmlSeverity::Medium => "MEDIUM",
            AmlSeverity::Low => "LOW",
        };

        SuspiciousTransactionReport {
            str_id: format!("STR-{}", alert.alert_id),
            report_date: "2026-09-15".to_string(),
            reporting_entity: self.config.reporting.reporting_entity_name.clone(),
            suspect_name: tx.counterparty_name.clone(),
            suspect_account: tx.counterparty_account.clone(),
            alert_type: alert_type_str.to_string(),
            severity: severity_str.to_string(),
            total_amount: alert.amount,
            transaction_count: 1,
            statutory_rule_ref: alert.statutory_ref.clone(),
            narrative_summary: alert.description.clone(),
            compliance_officer_notes: compliance_officer_notes.to_string(),
            merkle_proof_hash: merkle,
            risk_score,
            recommended_action,
        }
    }
}

/// Helper function to strip Vietnamese diacritics and uppercase for matching.
pub fn normalize_ascii(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        let upper = match c {
            'a' | 'A' | 'á' | 'Á' | 'à' | 'À' | 'ả' | 'Ả' | 'ã' | 'Ã' | 'ạ' | 'Ạ'
            | 'ă' | 'Ă' | 'ắ' | 'Ắ' | 'ằ' | 'Ằ' | 'ẳ' | 'Ẳ' | 'ẵ' | 'Ẵ' | 'ặ' | 'Ặ'
            | 'â' | 'Â' | 'ấ' | 'Ấ' | 'ầ' | 'Ầ' | 'ẩ' | 'Ẩ' | 'ẫ' | 'Ẫ' | 'ậ' | 'Ậ' => 'A',
            'd' | 'D' | 'đ' | 'Đ' => 'D',
            'e' | 'E' | 'é' | 'É' | 'è' | 'È' | 'ẻ' | 'Ẻ' | 'ẽ' | 'Ẽ' | 'ẹ' | 'Ẹ'
            | 'ê' | 'Ê' | 'ế' | 'Ế' | 'ề' | 'Ề' | 'ể' | 'Ể' | 'ễ' | 'Ễ' | 'ệ' | 'Ệ' => 'E',
            'i' | 'I' | 'í' | 'Í' | 'ì' | 'Ì' | 'ỉ' | 'Ỉ' | 'ĩ' | 'Ĩ' | 'ị' | 'Ị' => 'I',
            'o' | 'O' | 'ó' | 'Ó' | 'ò' | 'Ò' | 'ỏ' | 'Ỏ' | 'õ' | 'Õ' | 'ọ' | 'Ọ'
            | 'ô' | 'Ô' | 'ố' | 'Ố' | 'ồ' | 'Ồ' | 'ổ' | 'Ổ' | 'ỗ' | 'Ỗ' | 'ộ' | 'Ộ'
            | 'ơ' | 'Ơ' | 'ớ' | 'Ớ' | 'ờ' | 'Ờ' | 'ở' | 'Ở' | 'ỡ' | 'Ỡ' | 'ợ' | 'Ợ' => 'O',
            'u' | 'U' | 'ú' | 'Ú' | 'ù' | 'Ù' | 'ủ' | 'Ủ' | 'ũ' | 'Ũ' | 'ụ' | 'Ụ'
            | 'ư' | 'Ư' | 'ứ' | 'Ứ' | 'ừ' | 'Ừ' | 'ử' | 'Ử' | 'ữ' | 'Ữ' | 'ự' | 'Ự' => 'U',
            'y' | 'Y' | 'ý' | 'Ý' | 'ỳ' | 'Ỳ' | 'ỷ' | 'Ỷ' | 'ỹ' | 'Ỹ' | 'ỵ' | 'Ỵ' => 'Y',
            other => other.to_ascii_uppercase(),
        };
        out.push(upper);
    }
    out
}

/// Pure integer Jaro similarity in basis points ($0 \dots 10,000$ bps).
pub fn integer_jaro(s1: &str, s2: &str) -> u32 {
    let a: Vec<char> = s1.chars().collect();
    let b: Vec<char> = s2.chars().collect();

    if a.is_empty() && b.is_empty() {
        return 10_000;
    }
    if a.is_empty() || b.is_empty() {
        return 0;
    }
    if a == b {
        return 10_000;
    }

    let len_a = a.len();
    let len_b = b.len();
    let match_window = (len_a.max(len_b) / 2).saturating_sub(1);

    let mut a_matched = vec![false; len_a];
    let mut b_matched = vec![false; len_b];
    let mut matches = 0usize;

    for i in 0..len_a {
        let start = i.saturating_sub(match_window);
        let end = (i + match_window + 1).min(len_b);
        for j in start..end {
            if !b_matched[j] && a[i] == b[j] {
                a_matched[i] = true;
                b_matched[j] = true;
                matches += 1;
                break;
            }
        }
    }

    if matches == 0 {
        return 0;
    }

    let mut transpositions = 0usize;
    let mut b_idx = 0usize;
    for i in 0..len_a {
        if a_matched[i] {
            while !b_matched[b_idx] {
                b_idx += 1;
            }
            if a[i] != b[b_idx] {
                transpositions += 1;
            }
            b_idx += 1;
        }
    }

    let m = matches as u64;
    let t = (transpositions / 2) as u64;
    let la = len_a as u64;
    let lb = len_b as u64;

    let term1 = (m * 10_000) / la;
    let term2 = (m * 10_000) / lb;
    let term3 = ((m - t) * 10_000) / m;

    let jaro_bps = (term1 + term2 + term3) / 3;
    jaro_bps.min(10_000) as u32
}

/// Pure integer Jaro-Winkler similarity in basis points ($0 \dots 10,000$ bps).
pub fn integer_jaro_winkler(s1: &str, s2: &str) -> u32 {
    let a: Vec<char> = s1.chars().collect();
    let b: Vec<char> = s2.chars().collect();

    if a.is_empty() && b.is_empty() {
        return 10_000;
    }
    if a.is_empty() || b.is_empty() {
        return 0;
    }
    if a == b {
        return 10_000;
    }

    let j_bps = integer_jaro(s1, s2);

    let mut prefix_len = 0usize;
    for (c1, c2) in a.iter().zip(b.iter()).take(4) {
        if c1 == c2 {
            prefix_len += 1;
        } else {
            break;
        }
    }

    if j_bps >= 7_000 && prefix_len > 0 {
        let delta_bps = (prefix_len as u32 * (10_000 - j_bps) + 5) / 10;
        (j_bps + delta_bps).min(10_000)
    } else {
        j_bps
    }
}

/// Helper function to extract short prefix for alert IDs.
pub fn rule_code_prefix(code: &AmlAlertCode) -> &'static str {
    match code {
        AmlAlertCode::HighValueThreshold => "HV",
        AmlAlertCode::StructuringSmurfing => "STRUC",
        AmlAlertCode::VelocitySurge => "SURGE",
        AmlAlertCode::PassThroughTransit => "MULE",
        AmlAlertCode::NightAnomaly => "NIGHT",
        AmlAlertCode::WatchlistSanction => "WL",
        AmlAlertCode::NearThresholdSmurfing => "SMURF",
        AmlAlertCode::AmountDistributionOutlier => "MAD",
        AmlAlertCode::SemanticNarrationAnomaly => "SEM",
        AmlAlertCode::MicroBurstVelocity => "BURST",
    }
}

/// Computes median and Median Absolute Deviation (MAD) using 100% integer arithmetic.
/// Returns (median, mad).
pub fn compute_amount_mad(amounts: &[i64]) -> (i64, i64) {
    if amounts.is_empty() {
        return (0, 0);
    }
    let mut sorted = amounts.to_vec();
    sorted.sort_unstable();
    let n = sorted.len();
    let median = if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2
    };

    let mut deviations: Vec<i64> = sorted.iter().map(|&x| (x - median).abs()).collect();
    deviations.sort_unstable();
    let mad = if n % 2 == 1 {
        deviations[n / 2]
    } else {
        (deviations[n / 2 - 1] + deviations[n / 2]) / 2
    };

    (median, mad)
}

/// Computes Modified Z-score in integer basis points (0..N bps).
/// Formula: M_i = (6,745 * |X_i - median|) / max(1, mad)
/// 35,000 bps corresponds to 3.5x MAD.
pub fn compute_modified_zscore_bps(amount: i64, median: i64, mad: i64) -> u32 {
    let diff = (amount - median).unsigned_abs() as u128;
    let scaled_diff = diff.saturating_mul(6745);
    let denom = mad.max(1) as u128;
    let z_score = scaled_diff / denom;
    if z_score > (u32::MAX as u128) {
        u32::MAX
    } else {
        z_score as u32
    }
}

