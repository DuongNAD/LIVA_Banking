//! Gold Validation Dataset Benchmark for Multi-Dimensional Anomaly Detection & Decree 13 PII Masking.
//!
//! Evaluates 200 synthetic banking transactions (100 Benign, 100 Multi-Dimensional Anomalies).
//! Covers:
//! 1. Temporal & Velocity: rolling surge (>300%), off-hours (23:00-05:00 >200M VND), micro-burst (<60s).
//! 2. Amount Distribution: near-threshold smurfing band [380M, 399.999M VND], robust Modified Z-score with MAD.
//! 3. Semantic Narration: Crypto OTC, underground remittance, kickbacks, money mule obfuscation.
//! 4. Multi-Vector Blended Anomalies.
//!
//! Asserts F1-score >= 0.90 and Decree 13 PII Sanitization invariants.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use liva_compliance::aml::{AmlScreeningEngine, ScreenableTransaction};
use liva_money::Money;

struct LabeledTx {
    tx: ScreenableTransaction,
    is_anomaly: bool,
    #[allow(dead_code)]
    expected_category: &'static str,
}

fn generate_gold_dataset() -> Vec<LabeledTx> {
    let mut dataset = Vec::with_capacity(200);

    // Baseline reference timestamp: Wednesday 2026-09-16 10:00:00 GMT+7
    // In GMT+7, 10:00 AM corresponds to 03:00 AM UTC.
    let t_day = 1789527600; // Normal daytime business hours

    // Night timestamp: 02:00 AM GMT+7 corresponds to 19:00 UTC previous day.
    // (t_night + 7*3600) % 86400 = 2 * 3600 = 7200s (02:00 AM)
    let t_night = 1789498800;

    // =========================================================================
    // PART 1: 100 MULTI-DIMENSIONAL ANOMALIES (True Positives)
    // =========================================================================

    // Group 1.1: Temporal & Velocity Anomalies (25 cases)
    // 10 Off-Hours Night Anomalies (23:00 - 05:00 GMT+7, > 200M VND)
    for i in 1..=10 {
        let amt_vnd = 210_000_000 + (i as i64 * 15_000_000); // 225M .. 360M
        dataset.push(LabeledTx {
            tx: ScreenableTransaction {
                tx_id: format!("ANOM-NIGHT-{:02}", i),
                account_number: format!("ACC-NIGHT-{:02}", i),
                counterparty_name: format!("Doi tac Ban Dem {:02}", i),
                counterparty_account: format!("CP-NIGHT-{:02}", i),
                amount: Money::vnd(amt_vnd),
                is_credit: true,
                timestamp_seconds: t_night + (i as i64 * 600), // Around 02:00 - 03:30 AM
                narration: format!("Chuyen tien dem khuya dot {}", i),
            },
            is_anomaly: true,
            expected_category: "temporal_off_hours",
        });
    }

    // 8 Micro-Burst Anomalies (< 60s successive transactions on same account)
    for i in 1..=8 {
        dataset.push(LabeledTx {
            tx: ScreenableTransaction {
                tx_id: format!("ANOM-BURST-{:02}", i),
                account_number: "ACC-BOT-BURST-01".to_string(),
                counterparty_name: "Trieu Van Mau".to_string(),
                counterparty_account: "CP-BURST-01".to_string(),
                amount: Money::vnd(45_000_000 + (i as i64 * 2_000_000)),
                is_credit: false,
                timestamp_seconds: t_day + 10_000 + (i as i64 * 25), // Every 25 seconds (< 60s!)
                narration: format!("Lenh chi tu dong burst {}", i),
            },
            is_anomaly: true,
            expected_category: "temporal_micro_burst",
        });
    }

    // 7 Rolling Frequency & Turnover Surge (> 300% baseline / >= 5 txs in 1h window)
    for i in 1..=7 {
        dataset.push(LabeledTx {
            tx: ScreenableTransaction {
                tx_id: format!("ANOM-SURGE-{:02}", i),
                account_number: "ACC-SURGE-VIP-99".to_string(),
                counterparty_name: "Nguyen Hoang Long".to_string(),
                counterparty_account: "CP-SURGE-99".to_string(),
                amount: Money::vnd(80_000_000 + (i as i64 * 10_000_000)),
                is_credit: true,
                timestamp_seconds: t_day + 20_000 + (i as i64 * 300), // 7 txs in 35 mins
                narration: format!("Giao dich dot bien tan suat cao {}", i),
            },
            is_anomaly: true,
            expected_category: "temporal_velocity_surge",
        });
    }

    // Group 1.2: Amount Distribution Outliers (25 cases)
    // 15 Near-Threshold Smurfing Band [380M, 399.999M VND]
    let smurfing_amounts = [
        381_000_000, 383_500_000, 385_000_000, 388_000_000, 390_000_000,
        392_500_000, 394_000_000, 395_000_000, 396_500_000, 397_000_000,
        398_200_000, 398_900_000, 399_000_000, 399_500_000, 399_900_000,
    ];
    for (i, &amt) in smurfing_amounts.iter().enumerate() {
        dataset.push(LabeledTx {
            tx: ScreenableTransaction {
                tx_id: format!("ANOM-SMURF-{:02}", i + 1),
                account_number: format!("ACC-SMURF-{:02}", i + 1),
                counterparty_name: format!("Doi tac Smurf {:02}", i + 1),
                counterparty_account: format!("CP-SMURF-{:02}", i + 1),
                amount: Money::vnd(amt),
                is_credit: true,
                timestamp_seconds: t_day + 30_000 + (i as i64 * 1200),
                narration: format!("Thanh toan phan tach hop dong {}", i + 1),
            },
            is_anomaly: true,
            expected_category: "amount_near_threshold",
        });
    }

    // 10 Robust Modified Z-Score with MAD Outliers
    let mad_outlier_amounts = [
        650_000_000, 720_000_000, 850_000_000, 920_000_000, 1_100_000_000,
        1_250_000_000, 1_400_000_000, 1_600_000_000, 1_850_000_000, 2_200_000_000,
    ];
    for (i, &amt) in mad_outlier_amounts.iter().enumerate() {
        dataset.push(LabeledTx {
            tx: ScreenableTransaction {
                tx_id: format!("ANOM-MAD-{:02}", i + 1),
                account_number: format!("ACC-MAD-{:02}", i + 1),
                counterparty_name: format!("Doi tac So Tien Lon {:02}", i + 1),
                counterparty_account: format!("CP-MAD-{:02}", i + 1),
                amount: Money::vnd(amt),
                is_credit: true,
                timestamp_seconds: t_day + 40_000 + (i as i64 * 1500),
                narration: format!("Chuyen khoan dot bien gia tri bat thuong {}", i + 1),
            },
            is_anomaly: true,
            expected_category: "amount_mad_outlier",
        });
    }

    // Group 1.3: Semantic Narration Anomalies (25 cases)
    let semantic_narrations = [
        // Crypto OTC (7 cases)
        ("ANOM-SEM-01", "Mua 5000 USDT tren san Binance P2P"),
        ("ANOM-SEM-02", "Chuyen khoan mua USDT p2p ck nhanh"),
        ("ANOM-SEM-03", "Thanh toan mua coin tren san OKX"),
        ("ANOM-SEM-04", "Nap tien vi crypto bybit p2p"),
        ("ANOM-SEM-05", "Giao dich tien ao binance usdt"),
        ("ANOM-SEM-06", "Ban USDT nhan tien mat qua ngan hang"),
        ("ANOM-SEM-07", "Mua usdt qua p2p"),
        // Underground Remittance (6 cases)
        ("ANOM-SEM-08", "Chuyen ho tien sang Campuchia lay tien mat"),
        ("ANOM-SEM-09", "Doi ngoai te chui lay tien do ve"),
        ("ANOM-SEM-10", "Day tien ho qua bien gioi"),
        ("ANOM-SEM-11", "Chuyen tien tay ba nhanh khong can giay to"),
        ("ANOM-SEM-12", "Chuyen ho tien sang Trung Quoc"),
        ("ANOM-SEM-13", "Rut tien ho tai quay"),
        // Kickbacks & Bribery (6 cases)
        ("ANOM-SEM-14", "Chi hoa hong du an kin k xuat hoa don"),
        ("ANOM-SEM-15", "Tien cam on sep phe duyet goi thau so 4"),
        ("ANOM-SEM-16", "Chi ngoai so tien chenh lech hop dong"),
        ("ANOM-SEM-17", "Tien lot tay can bo ban quan ly"),
        ("ANOM-SEM-18", "Tien boi tron thu tuc cap phep nhanh"),
        ("ANOM-SEM-19", "Phan tram du an mat cho truong phong"),
        // Money Mule Obfuscation (6 cases)
        ("ANOM-SEM-20", "Don dep tai khoan gap trong ngay"),
        ("ANOM-SEM-21", "Tai khoan muon chuyen tiep ngay cho khach"),
        ("ANOM-SEM-22", "Keo tien ve khong ghi noi dung"),
        ("ANOM-SEM-23", "Chuyen gap khong ghi noi dung de phong"),
        ("ANOM-SEM-24", "Chuyen tiep ngay cho tai khoan khac"),
        ("ANOM-SEM-25", "Nhan roi ban ngay chuyen sang stk khac"),
    ];

    for (tx_id, narration) in semantic_narrations {
        dataset.push(LabeledTx {
            tx: ScreenableTransaction {
                tx_id: tx_id.to_string(),
                account_number: format!("ACC-{}", tx_id),
                counterparty_name: "Tran Van Nghi Van".to_string(),
                counterparty_account: format!("CP-{}", tx_id),
                amount: Money::vnd(75_000_000),
                is_credit: true,
                timestamp_seconds: t_day + 50_000,
                narration: narration.to_string(),
            },
            is_anomaly: true,
            expected_category: "semantic_narration",
        });
    }

    // Group 1.4: Multi-Vector Blended Anomalies (25 cases)
    // Combines Off-hours + Smurfing / Micro-burst + Semantic
    for i in 1..=25 {
        let (narr, amt, ts) = match i % 4 {
            0 => ("Chuyen tien mua USDT p2p ban dem", 388_000_000, t_night + (i as i64 * 300)),
            1 => ("Day tien ho sang Campuchia gap", 395_000_000, t_night + (i as i64 * 400)),
            2 => ("Don dep tai khoan chuyen gap", 280_000_000, t_night + (i as i64 * 200)),
            _ => ("Chi hoa hong du an kin", 392_000_000, t_day + (i as i64 * 800)),
        };
        dataset.push(LabeledTx {
            tx: ScreenableTransaction {
                tx_id: format!("ANOM-BLEND-{:02}", i),
                account_number: format!("ACC-BLEND-{:02}", i),
                counterparty_name: format!("Doi tac Da Chieu {:02}", i),
                counterparty_account: format!("CP-BLEND-{:02}", i),
                amount: Money::vnd(amt),
                is_credit: true,
                timestamp_seconds: ts,
                narration: narr.to_string(),
            },
            is_anomaly: true,
            expected_category: "multi_vector_blended",
        });
    }

    // =========================================================================
    // PART 2: 100 BENIGN NORMAL TRANSACTIONS (True Negatives)
    // =========================================================================

    let normal_corporates = [
        "CONG TY TNHH PHAT TRIEN PHAN MEM MINH QUAN",
        "CONG TY CO PHAN TAP DOAN HOA PHAT",
        "CONG TY TNHH THUONG MAI DICH VU AN PHU",
        "TRUONG DAI HOC BACH KHOA HA NOI",
        "BENH VIEN DA KHOA QUOC TE VINMEC",
        "NGAN HANG TMCP NGOAI THUONG VIET NAM",
        "CONG TY CO PHAN SUA VIET NAM VINAMILK",
        "CONG TY TNHH LOGISTICS DONG DUONG",
        "CONG TY CO PHAN CO DIEN LANH REE",
        "CONG TY TNHH SAN XUAT BAO BI TOAN CAU",
    ];

    let normal_retail = [
        "Tran Dai Nghia", "Tran Thi Mai", "Le Van Cuong", "Hoang Thi Lan",
        "Vu Dinh Trong", "Dang Minh Tuan", "Bui Thu Trang", "Pham Quang Huy",
        "Doan Van Hau", "Ngo Bao Chau",
    ];

    let normal_narrations = [
        "Thanh toan tien hang thang 9 hoa don 1024",
        "Tra luong can bo nhan vien ky 1 thang 9",
        "Thanh toan tien dien thang 8 van phong",
        "Tien thue mat bang van phong thang 9",
        "Mua van phong pham va giay in A4",
        "Thanh toan tien nuoc sinh hoat thang 8",
        "Chi phi ve may bay cong tac Da Nang",
        "Phi bao tri he thong thang 9 nam 2026",
        "Thanh toan cuoc vien thong internet thang 8",
        "Chuyen tien hoc phi cho con",
    ];

    for i in 1..=100 {
        let corp_idx = (i - 1) % normal_corporates.len();
        let retail_idx = (i - 1) % normal_retail.len();
        let narr_idx = (i - 1) % normal_narrations.len();

        let is_corp = i % 2 == 0;
        let counterparty = if is_corp {
            normal_corporates[corp_idx].to_string()
        } else {
            normal_retail[retail_idx].to_string()
        };

        // Modest normal amounts: 8M .. 120M VND (within normal operational bounds)
        let amt_vnd = 8_000_000 + ((i as i64 * 1_120_000) % 110_000_000);

        // Normal business hours: 09:00 AM to 16:30 PM
        let daytime_offset = (i as i64 * 1800) % 25200; // within 7 business hours

        dataset.push(LabeledTx {
            tx: ScreenableTransaction {
                tx_id: format!("BENIGN-{:03}", i),
                account_number: format!("ACC-CORP-{:03}", (i % 20) + 1),
                counterparty_name: counterparty,
                counterparty_account: format!("CP-CORP-{:03}", i),
                amount: Money::vnd(amt_vnd),
                is_credit: i % 3 != 0,
                timestamp_seconds: t_day + daytime_offset,
                narration: normal_narrations[narr_idx].to_string(),
            },
            is_anomaly: false,
            expected_category: "benign",
        });
    }

    dataset
}

#[test]
fn test_gold_dataset_f1_benchmark_score_greater_than_90_percent() {
    let engine = AmlScreeningEngine::with_default_policy();
    let labeled_dataset = generate_gold_dataset();

    assert_eq!(
        labeled_dataset.len(),
        200,
        "Gold validation dataset must contain exactly 200 synthetic banking transactions"
    );

    let benign_count = labeled_dataset.iter().filter(|x| !x.is_anomaly).count();
    let anomaly_count = labeled_dataset.iter().filter(|x| x.is_anomaly).count();
    assert_eq!(benign_count, 100, "Dataset must have exactly 100 benign transactions");
    assert_eq!(anomaly_count, 100, "Dataset must have exactly 100 anomaly transactions");

    // Extract transaction slice for engine screening
    let txs: Vec<ScreenableTransaction> = labeled_dataset.iter().map(|l| l.tx.clone()).collect();

    let start_time = std::time::Instant::now();
    let enhanced_alerts = engine.screen_with_anomaly_detection(&txs);
    let elapsed = start_time.elapsed();

    // Map detected alerts by tx_id
    let mut detected_tx_ids = std::collections::HashSet::new();
    for alert in &enhanced_alerts {
        detected_tx_ids.insert(alert.tx_id.clone());
    }

    // Evaluate Confusion Matrix
    let mut true_positives: usize = 0;
    let mut false_positives: usize = 0;
    let mut true_negatives: usize = 0;
    let mut false_negatives: usize = 0;

    for item in &labeled_dataset {
        let is_detected = detected_tx_ids.contains(&item.tx.tx_id);
        if item.is_anomaly {
            if is_detected {
                true_positives += 1;
            } else {
                false_negatives += 1;
            }
        } else {
            if is_detected {
                false_positives += 1;
            } else {
                true_negatives += 1;
            }
        }
    }

    // Integer arithmetic precision, recall, and F1 calculation in basis points (0..10,000 bps)
    let total_pred_positives = (true_positives + false_positives) as u64;
    let total_actual_positives = (true_positives + false_negatives) as u64;

    let precision_bps = if total_pred_positives > 0 {
        ((true_positives as u64 * 10_000) / total_pred_positives) as u32
    } else {
        0
    };

    let recall_bps = if total_actual_positives > 0 {
        ((true_positives as u64 * 10_000) / total_actual_positives) as u32
    } else {
        0
    };

    let f1_bps = if (precision_bps + recall_bps) > 0 {
        ((2 * precision_bps as u64 * recall_bps as u64) / (precision_bps + recall_bps) as u64) as u32
    } else {
        0
    };

    println!("\n========================================================");
    println!("--- LIVA BANKING AML ANOMALY DETECTION F1 BENCHMARK ---");
    println!("Total Samples:               {}", labeled_dataset.len());
    println!("Execution Duration:          {:?}", elapsed);
    println!("True Positives (TP):         {}", true_positives);
    println!("False Positives (FP):        {}", false_positives);
    println!("True Negatives (TN):         {}", true_negatives);
    println!("False Negatives (FN):        {}", false_negatives);
    println!(
        "Precision:                   {}.{:02}% ({} bps)",
        precision_bps / 100,
        precision_bps % 100,
        precision_bps
    );
    println!(
        "Recall:                      {}.{:02}% ({} bps)",
        recall_bps / 100,
        recall_bps % 100,
        recall_bps
    );
    println!(
        "F1-Score:                    {}.{:02}% ({} bps)",
        f1_bps / 100,
        f1_bps % 100,
        f1_bps
    );
    println!("========================================================\n");

    // Strict Acceptance Criteria: F1-Score >= 0.90 (9,000 bps)
    assert!(
        f1_bps >= 9_000,
        "F1-score must be >= 0.90 (9,000 bps), achieved: {}.{:02}%",
        f1_bps / 100,
        f1_bps % 100
    );
    assert!(
        precision_bps >= 9_000,
        "Precision must be >= 0.90, achieved: {}.{:02}%",
        precision_bps / 100,
        precision_bps % 100
    );
    assert!(
        recall_bps >= 9_000,
        "Recall must be >= 0.90, achieved: {}.{:02}%",
        recall_bps / 100,
        recall_bps % 100
    );

    // Verify Decree 13 PII sanitization across all alerts
    for alert in &enhanced_alerts {
        assert!(alert.anomaly_score_bps > 0);
        assert!(!alert.anomaly_reasons.is_empty());
        assert!(!alert.risk_profile.is_empty());

        if let Some(ref explanation) = alert.vietnamese_explanation {
            // Must NOT leak unmasked personal patronymic names
            assert!(
                !explanation.contains("Nguyen Van Binh"),
                "Explanation leaked unmasked natural person name"
            );
            assert!(
                !explanation.contains("Tran Van Nghi Van"),
                "Explanation leaked unmasked natural person name"
            );
            // Must preserve corporate legal entities
            if alert.anomaly_reasons.iter().any(|r| r.contains("CONG TY")) {
                assert!(
                    explanation.contains("CONG TY"),
                    "Explanation failed to preserve corporate entity name"
                );
            }
        }
    }
}

#[test]
fn test_slm_explanation_formatter_and_decree13_preservation() {
    let engine = AmlScreeningEngine::with_default_policy();

    let tx_person = ScreenableTransaction {
        tx_id: "TX-PII-01".to_string(),
        account_number: "0071000888".to_string(),
        counterparty_name: "Nguyen Van Binh".to_string(),
        counterparty_account: "1903400888".to_string(),
        amount: Money::vnd(395_000_000),
        is_credit: true,
        timestamp_seconds: 1789498800, // Night 02:00 AM GMT+7
        narration: "0908123456 chuyen tien gap mua USDT tren san Binance".to_string(),
    };

    let alerts = engine.screen_with_anomaly_detection(&[tx_person.clone()]);
    assert_eq!(alerts.len(), 1);
    let alert = &alerts[0];

    // Check prompt formatted for Qwen3 14B
    let prompt = engine.format_slm_explanation_prompt(&tx_person, alert);
    assert!(prompt.contains("TX-PII-01"));
    assert!(prompt.contains("[REDACTED_NAME]"));
    assert!(prompt.contains("[REDACTED_PHONE]"));
    assert!(!prompt.contains("0908123456"));
    assert!(!prompt.contains("Nguyen Van Binh"));

    // Check generated Vietnamese compliance explanation
    let expl = alert.vietnamese_explanation.as_ref().expect("Must have explanation");
    assert!(expl.contains("BÁO CÁO GIẢI TRÌNH AML"));
    assert!(expl.contains("[REDACTED_NAME]"));
    assert!(expl.contains("[REDACTED_PHONE]"));
    assert!(!expl.contains("0908123456"));
    assert!(!expl.contains("Nguyen Van Binh"));

    // Corporate entity test: Corporate name MUST BE PRESERVED
    let tx_corp = ScreenableTransaction {
        tx_id: "TX-CORP-01".to_string(),
        account_number: "0071000777".to_string(),
        counterparty_name: "CONG TY TNHH PHAT TRIEN PHAN MEM MINH QUAN".to_string(),
        counterparty_account: "1903400777".to_string(),
        amount: Money::vnd(398_000_000),
        is_credit: true,
        timestamp_seconds: 1789498800,
        narration: "Thanh toan tien phan mem".to_string(),
    };

    let corp_alerts = engine.screen_with_anomaly_detection(&[tx_corp.clone()]);
    assert_eq!(corp_alerts.len(), 1);
    let corp_alert = &corp_alerts[0];
    let corp_prompt = engine.format_slm_explanation_prompt(&tx_corp, corp_alert);
    assert!(corp_prompt.contains("CONG TY TNHH PHAT TRIEN PHAN MEM MINH QUAN"));
}

#[test]
fn test_pure_integer_mad_calculation() {
    use liva_compliance::aml::{compute_amount_mad, compute_modified_zscore_bps};

    let normal_amounts = vec![
        20_000_000, 22_000_000, 25_000_000, 28_000_000, 30_000_000,
        32_000_000, 35_000_000, 38_000_000, 40_000_000,
    ];
    let (median, mad) = compute_amount_mad(&normal_amounts);
    assert_eq!(median, 30_000_000);
    assert_eq!(mad, 5_000_000);

    // Normal amount Modified Z-Score: |35M - 30M| * 6745 / 5M = 6,745 bps (< 35,000 bps)
    let z_normal = compute_modified_zscore_bps(35_000_000, median, mad);
    assert!(z_normal < 35_000);

    // Outlier Modified Z-Score: |400M - 30M| * 6745 / 5M = 499,130 bps (>> 35,000 bps)
    let z_outlier = compute_modified_zscore_bps(400_000_000, median, mad);
    assert!(z_outlier > 35_000);
}
