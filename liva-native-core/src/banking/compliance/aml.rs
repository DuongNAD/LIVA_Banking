//! Automated AML/CTF (Anti-Money Laundering & Counter-Terrorist Financing) Screening Engine.
//!
//! Complies with:
//! - Law on Anti-Money Laundering No. 14/2022/QH15.
//! - Decree No. 19/2023/NĐ-CP.
//! - Prime Minister Decision No. 11/2023/QĐ-TTg (Threshold >= 400,000,000 VND).
//!
//! Features:
//! 1. High-Value Transaction Alert (>= 400M VND).
//! 2. Structuring / Smurfing Alert (>= 3 transactions < 400M within 72h summing to >= 800M).
//! 3. Velocity Surge Alert (> 300% historical baseline).
//! 4. Pass-Through / Transit Account Alert (> 95% inflow dispersed in < 15 minutes).
//! 5. Standardized Suspicious Transaction Report (STR) struct and generator.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// High-value transaction threshold in VND pursuant to Decision 11/2023/QĐ-TTg.
pub const AML_HIGH_VALUE_THRESHOLD_VND: u64 = 400_000_000;

/// Critical threshold for instantaneous escalation.
pub const AML_CRITICAL_VALUE_THRESHOLD_VND: u64 = 1_000_000_000;

/// Rolling window for structuring detection: 72 hours (in seconds).
pub const AML_STRUCTURING_WINDOW_SECONDS: i64 = 72 * 3600;

/// Pass-through maximum transit window: 15 minutes (in seconds).
pub const AML_PASS_THROUGH_WINDOW_SECONDS: i64 = 15 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AmlRuleCode {
    #[serde(rename = "AML_HIGH_VALUE")]
    AmlHighValue,
    #[serde(rename = "AML_STRUCTURING")]
    AmlStructuring,
    #[serde(rename = "AML_VELOCITY_SURGE")]
    AmlVelocitySurge,
    #[serde(rename = "AML_PASS_THROUGH")]
    AmlPassThrough,
}

impl AmlRuleCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AmlRuleCode::AmlHighValue => "AML_HIGH_VALUE",
            AmlRuleCode::AmlStructuring => "AML_STRUCTURING",
            AmlRuleCode::AmlVelocitySurge => "AML_VELOCITY_SURGE",
            AmlRuleCode::AmlPassThrough => "AML_PASS_THROUGH",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmlSeverity {
    #[serde(rename = "HIGH")]
    High,
    #[serde(rename = "CRITICAL")]
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlAlert {
    pub tx_id: String,
    pub rule_code: AmlRuleCode,
    pub severity: AmlSeverity,
    pub description: String,
    pub amount_vnd: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionScreeningItem {
    pub tx_id: String,
    #[serde(default)]
    pub account_number: Option<String>,
    #[serde(default)]
    pub counterparty_name: Option<String>,
    #[serde(default)]
    pub counterparty_account: Option<String>,
    pub amount_vnd: u64,
    #[serde(default)]
    pub timestamp: i64,
    pub narration: String,
    #[serde(default)]
    pub is_credit: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousTransactionReport {
    pub report_id: String,
    pub generated_at: i64,
    pub total_screened: usize,
    pub alerts_count: usize,
    pub risk_score: u32,
    pub alerts: Vec<AmlAlert>,
    pub recommended_action: String,
    pub legal_references: Vec<String>,
}

pub struct AmlScreeningEngine;

impl AmlScreeningEngine {
    /// Screens a collection of transactions against statutory Vietnamese AML/CTF rules.
    pub fn screen_transactions(
        transactions: &[TransactionScreeningItem],
        historical_daily_baseline_vnd: Option<u64>,
    ) -> Vec<AmlAlert> {
        let mut alerts = Vec::new();

        // 1. Check Rule 1: High-Value Transaction Threshold (>= 400M VND)
        for tx in transactions {
            if tx.amount_vnd >= AML_HIGH_VALUE_THRESHOLD_VND {
                let severity = if tx.amount_vnd >= AML_CRITICAL_VALUE_THRESHOLD_VND {
                    AmlSeverity::Critical
                } else {
                    AmlSeverity::High
                };
                let desc = format!(
                    "Giao dịch giá trị lớn ({} VND) vượt ngưỡng 400,000,000 VND theo Quyết định 11/2023/QĐ-TTg",
                    format_vnd(tx.amount_vnd)
                );
                alerts.push(AmlAlert {
                    tx_id: tx.tx_id.clone(),
                    rule_code: AmlRuleCode::AmlHighValue,
                    severity,
                    description: desc,
                    amount_vnd: tx.amount_vnd,
                    timestamp: tx.timestamp,
                });
            }
        }

        // 2. Check Rule 2: Structuring / Smurfing
        // >= 3 transactions under 400M within 72h window summing to >= 800M
        let sub_threshold_txs: Vec<&TransactionScreeningItem> = transactions
            .iter()
            .filter(|tx| tx.amount_vnd < AML_HIGH_VALUE_THRESHOLD_VND && tx.amount_vnd >= 50_000_000)
            .collect();

        if sub_threshold_txs.len() >= 3 {
            let mut sorted = sub_threshold_txs.clone();
            sorted.sort_by_key(|tx| tx.timestamp);

            for i in 0..sorted.len() {
                let mut window_txs = vec![sorted[i]];
                let mut window_sum = sorted[i].amount_vnd;

                for j in (i + 1)..sorted.len() {
                    if sorted[j].timestamp - sorted[i].timestamp <= AML_STRUCTURING_WINDOW_SECONDS {
                        window_txs.push(sorted[j]);
                        window_sum = window_sum.saturating_add(sorted[j].amount_vnd);
                    }
                }

                if window_txs.len() >= 3 && window_sum >= 800_000_000 {
                    let desc = format!(
                        "Dấu hiệu chia nhỏ giao dịch (smurfing/structuring): {} giao dịch dưới 400M VND trong 72 giờ, tổng giá trị {} VND (ngưỡng >= 800M)",
                        window_txs.len(),
                        format_vnd(window_sum)
                    );
                    for wtx in &window_txs {
                        // Avoid duplicate structuring alerts for the same tx
                        if !alerts.iter().any(|a| a.tx_id == wtx.tx_id && a.rule_code == AmlRuleCode::AmlStructuring) {
                            alerts.push(AmlAlert {
                                tx_id: wtx.tx_id.clone(),
                                rule_code: AmlRuleCode::AmlStructuring,
                                severity: AmlSeverity::Critical,
                                description: desc.clone(),
                                amount_vnd: wtx.amount_vnd,
                                timestamp: wtx.timestamp,
                            });
                        }
                    }
                }
            }
        }

        // 3. Check Rule 3: Sudden Velocity Surge (> 300% historical baseline)
        if let Some(baseline) = historical_daily_baseline_vnd {
            if baseline > 0 {
                let total_volume: u64 = transactions.iter().map(|tx| tx.amount_vnd).sum();
                // Surge condition: total_volume > 300% surge (i.e. > 4.0 * baseline, or > 3.0 * baseline)
                if total_volume > baseline.saturating_mul(3) {
                    let desc = format!(
                        "Đột biến doanh số giao dịch: Tổng {} VND vượt quá 300% mức trung bình lịch sử ({} VND)",
                        format_vnd(total_volume),
                        format_vnd(baseline)
                    );
                    if let Some(first_tx) = transactions.first() {
                        alerts.push(AmlAlert {
                            tx_id: first_tx.tx_id.clone(),
                            rule_code: AmlRuleCode::AmlVelocitySurge,
                            severity: AmlSeverity::High,
                            description: desc,
                            amount_vnd: total_volume,
                            timestamp: first_tx.timestamp,
                        });
                    }
                }
            }
        }

        // 4. Check Rule 4: Pass-Through / Transit Account
        // Inflow >= 100M VND dispersed > 95% within < 15 minutes (900 seconds)
        let inflows: Vec<&TransactionScreeningItem> = transactions
            .iter()
            .filter(|tx| tx.is_credit == Some(true) && tx.amount_vnd >= 100_000_000)
            .collect();

        let outflows: Vec<&TransactionScreeningItem> = transactions
            .iter()
            .filter(|tx| tx.is_credit == Some(false))
            .collect();

        for in_tx in inflows {
            for out_tx in &outflows {
                if out_tx.timestamp >= in_tx.timestamp
                    && (out_tx.timestamp - in_tx.timestamp) <= AML_PASS_THROUGH_WINDOW_SECONDS
                {
                    // Check if outflow is >= 95% of inflow
                    let min_outflow = (in_tx.amount_vnd as u128 * 95 / 100) as u64;
                    if out_tx.amount_vnd >= min_outflow {
                        let desc = format!(
                            "Dấu hiệu tài khoản trung chuyển (pass-through): Nhận {} VND lúc t={}, giải ngân {} VND (>95%) trong vòng {} giây (< 15 phút)",
                            format_vnd(in_tx.amount_vnd),
                            in_tx.timestamp,
                            format_vnd(out_tx.amount_vnd),
                            out_tx.timestamp - in_tx.timestamp
                        );
                        alerts.push(AmlAlert {
                            tx_id: out_tx.tx_id.clone(),
                            rule_code: AmlRuleCode::AmlPassThrough,
                            severity: AmlSeverity::Critical,
                            description: desc,
                            amount_vnd: out_tx.amount_vnd,
                            timestamp: out_tx.timestamp,
                        });
                    }
                }
            }
        }

        alerts
    }

    /// Generates a standardized Suspicious Transaction Report (STR) for compliance audit.
    pub fn generate_str(
        alerts: Vec<AmlAlert>,
        total_screened: usize,
    ) -> SuspiciousTransactionReport {
        let alerts_count = alerts.len();
        let mut score = 0u32;

        for alert in &alerts {
            match alert.severity {
                AmlSeverity::Critical => score = score.saturating_add(40),
                AmlSeverity::High => score = score.saturating_add(20),
            }
        }
        let risk_score = score.min(100);

        let recommended_action = if risk_score >= 70 || alerts.iter().any(|a| a.severity == AmlSeverity::Critical) {
            "FILE_STR_WITH_SBV".to_string()
        } else if risk_score >= 40 {
            "INTERNAL_COMPLIANCE_HOLD".to_string()
        } else if alerts_count > 0 {
            "ENHANCED_DUE_DILIGENCE".to_string()
        } else {
            "NORMAL_CLEARANCE".to_string()
        };

        let legal_references = vec![
            "Luật Phòng, chống rửa tiền số 14/2022/QH15".to_string(),
            "Nghị định 19/2023/NĐ-CP hướng dẫn Luật PCRT".to_string(),
            "Quyết định 11/2023/QĐ-TTg quy định mức giao dịch có giá trị lớn phải báo cáo".to_string(),
        ];

        SuspiciousTransactionReport {
            report_id: format!("STR-{}", Uuid::new_v4()),
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            total_screened,
            alerts_count,
            risk_score,
            alerts,
            recommended_action,
            legal_references,
        }
    }
}

fn format_vnd(amount: u64) -> String {
    let s = amount.to_string();
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    for (i, &ch) in chars.iter().enumerate() {
        out.push(ch);
        if (len - 1 - i) % 3 == 0 && i < len - 1 {
            out.push('.');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aml_high_value_threshold() {
        let txs = vec![
            TransactionScreeningItem {
                tx_id: "TX-001".to_string(),
                account_number: Some("0011001234567".to_string()),
                counterparty_name: Some("Nguyen Van A".to_string()),
                counterparty_account: None,
                amount_vnd: 500_000_000, // 500M >= 400M
                timestamp: 1725000000,
                narration: "Thanh toan hop dong 01".to_string(),
                is_credit: Some(true),
            },
            TransactionScreeningItem {
                tx_id: "TX-002".to_string(),
                account_number: Some("0011001234567".to_string()),
                counterparty_name: Some("Le Van B".to_string()),
                counterparty_account: None,
                amount_vnd: 50_000_000, // 50M < 400M
                timestamp: 1725001000,
                narration: "Mua hang".to_string(),
                is_credit: Some(true),
            },
        ];

        let alerts = AmlScreeningEngine::screen_transactions(&txs, None);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].tx_id, "TX-001");
        assert_eq!(alerts[0].rule_code, AmlRuleCode::AmlHighValue);
        assert_eq!(alerts[0].severity, AmlSeverity::High);
    }

    #[test]
    fn test_aml_structuring_smurfing() {
        // 3 txs within 72 hours, each < 400M but sum >= 800M
        let base_ts = 1725000000;
        let txs = vec![
            TransactionScreeningItem {
                tx_id: "SMURF-1".to_string(),
                account_number: Some("123".to_string()),
                counterparty_name: None,
                counterparty_account: None,
                amount_vnd: 350_000_000,
                timestamp: base_ts,
                narration: "Split 1".to_string(),
                is_credit: Some(true),
            },
            TransactionScreeningItem {
                tx_id: "SMURF-2".to_string(),
                account_number: Some("123".to_string()),
                counterparty_name: None,
                counterparty_account: None,
                amount_vnd: 350_000_000,
                timestamp: base_ts + 3600 * 24, // 24h later
                narration: "Split 2".to_string(),
                is_credit: Some(true),
            },
            TransactionScreeningItem {
                tx_id: "SMURF-3".to_string(),
                account_number: Some("123".to_string()),
                counterparty_name: None,
                counterparty_account: None,
                amount_vnd: 200_000_000,
                timestamp: base_ts + 3600 * 48, // 48h later
                narration: "Split 3".to_string(),
                is_credit: Some(true),
            },
        ];

        let alerts = AmlScreeningEngine::screen_transactions(&txs, None);
        let structuring_alerts: Vec<_> = alerts
            .into_iter()
            .filter(|a| a.rule_code == AmlRuleCode::AmlStructuring)
            .collect();

        assert_eq!(structuring_alerts.len(), 3);
        assert_eq!(structuring_alerts[0].severity, AmlSeverity::Critical);
    }

    #[test]
    fn test_aml_pass_through_account() {
        let base_ts = 1725000000;
        let txs = vec![
            TransactionScreeningItem {
                tx_id: "INFLOW-1".to_string(),
                account_number: Some("999".to_string()),
                counterparty_name: None,
                counterparty_account: None,
                amount_vnd: 200_000_000,
                timestamp: base_ts,
                narration: "Nhan tien".to_string(),
                is_credit: Some(true),
            },
            TransactionScreeningItem {
                tx_id: "OUTFLOW-1".to_string(),
                account_number: Some("999".to_string()),
                counterparty_name: None,
                counterparty_account: None,
                amount_vnd: 195_000_000, // 195M >= 95% of 200M (190M)
                timestamp: base_ts + 300, // 5 minutes later (< 15 mins)
                narration: "Chuyen ngay".to_string(),
                is_credit: Some(false),
            },
        ];

        let alerts = AmlScreeningEngine::screen_transactions(&txs, None);
        let pt_alerts: Vec<_> = alerts
            .into_iter()
            .filter(|a| a.rule_code == AmlRuleCode::AmlPassThrough)
            .collect();

        assert_eq!(pt_alerts.len(), 1);
        assert_eq!(pt_alerts[0].tx_id, "OUTFLOW-1");
        assert_eq!(pt_alerts[0].severity, AmlSeverity::Critical);
    }

    #[test]
    fn test_aml_velocity_surge() {
        let txs = vec![TransactionScreeningItem {
            tx_id: "SURGE-1".to_string(),
            account_number: Some("555".to_string()),
            counterparty_name: None,
            counterparty_account: None,
            amount_vnd: 50_000_000,
            timestamp: 1725000000,
            narration: "Dot bien".to_string(),
            is_credit: Some(true),
        }];

        // Historical daily baseline is 10M, but tx is 50M (> 300% surge)
        let alerts = AmlScreeningEngine::screen_transactions(&txs, Some(10_000_000));
        let surge_alerts: Vec<_> = alerts
            .into_iter()
            .filter(|a| a.rule_code == AmlRuleCode::AmlVelocitySurge)
            .collect();

        assert_eq!(surge_alerts.len(), 1);
        assert_eq!(surge_alerts[0].severity, AmlSeverity::High);
    }

    #[test]
    fn test_generate_suspicious_transaction_report() {
        let alerts = vec![AmlAlert {
            tx_id: "CRIT-1".to_string(),
            rule_code: AmlRuleCode::AmlStructuring,
            severity: AmlSeverity::Critical,
            description: "Smurfing detected".to_string(),
            amount_vnd: 350_000_000,
            timestamp: 1725000000,
        }];

        let str_report = AmlScreeningEngine::generate_str(alerts, 5);
        assert_eq!(str_report.alerts_count, 1);
        assert_eq!(str_report.recommended_action, "FILE_STR_WITH_SBV");
        assert!(str_report.risk_score >= 40);
        assert_eq!(str_report.legal_references.len(), 3);
    }
}
