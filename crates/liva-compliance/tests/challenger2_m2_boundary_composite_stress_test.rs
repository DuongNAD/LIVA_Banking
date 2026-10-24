//! Challenger 2 Adversarial Stress Test Suite for Milestone M2
//! Focus: Off-Hours Night Velocity Boundary Conditions & Composite Risk Scoring Bounding

use liva_compliance::aml::{
    AmlAlertCode, AmlScreeningEngine, AmlSeverity, ScreenableTransaction,
};
use liva_money::Money;

/// Reference timestamp T corresponding to 00:00:00 GMT+7 (17:00:00 UTC previous day).
/// (1726333200 + 7 * 3600) % 86400 = 1726358400 % 86400 = 0.
const REF_MIDNIGHT_GMT7_UTC: i64 = 1726333200;

#[test]
fn test_off_hours_timezone_boundary_gmt7_exact_seconds() {
    let engine = AmlScreeningEngine::with_default_policy();

    // 1. 22:59:59 GMT+7 (82,799 seconds past midnight GMT+7)
    // Hour of day = 82799 / 3600 = 22. Outside night window [23:00 - 05:00).
    let t_22_59_59 = REF_MIDNIGHT_GMT7_UTC + 82799;

    // 2. 23:00:00 GMT+7 (82,800 seconds past midnight GMT+7)
    // Hour of day = 82800 / 3600 = 23. Inside night window.
    let t_23_00_00 = REF_MIDNIGHT_GMT7_UTC + 82800;

    // 3. 04:59:59 GMT+7 (Next morning: 86,400 + 17,999 seconds past midnight GMT+7)
    // Hour of day = 17999 / 3600 = 4. Inside night window.
    let t_04_59_59 = REF_MIDNIGHT_GMT7_UTC + 86400 + 17999;

    // 4. 05:00:00 GMT+7 (Next morning: 86,400 + 18,000 seconds past midnight GMT+7)
    // Hour of day = 18000 / 3600 = 5. Outside night window.
    let t_05_00_00 = REF_MIDNIGHT_GMT7_UTC + 86400 + 18000;

    // All transactions have amount 250,000,000 VND (> 200M VND threshold)
    let boundary_txs = vec![
        ScreenableTransaction {
            tx_id: "TX-TZ-22-59-59".to_string(),
            account_number: "ACC-NIGHT-EDGE-01".to_string(),
            counterparty_name: "CONG TY TNHH TEST 1".to_string(),
            counterparty_account: "CP-EDGE-01".to_string(),
            amount: Money::vnd(250_000_000),
            is_credit: true,
            timestamp_seconds: t_22_59_59,
            narration: "Chuyen tien cuoi gio toi 22h59m59s".to_string(),
        },
        ScreenableTransaction {
            tx_id: "TX-TZ-23-00-00".to_string(),
            account_number: "ACC-NIGHT-EDGE-02".to_string(),
            counterparty_name: "CONG TY TNHH TEST 2".to_string(),
            counterparty_account: "CP-EDGE-02".to_string(),
            amount: Money::vnd(250_000_000),
            is_credit: true,
            timestamp_seconds: t_23_00_00,
            narration: "Chuyen tien bat dau gio dem 23h00m00s".to_string(),
        },
        ScreenableTransaction {
            tx_id: "TX-TZ-04-59-59".to_string(),
            account_number: "ACC-NIGHT-EDGE-03".to_string(),
            counterparty_name: "CONG TY TNHH TEST 3".to_string(),
            counterparty_account: "CP-EDGE-03".to_string(),
            amount: Money::vnd(250_000_000),
            is_credit: true,
            timestamp_seconds: t_04_59_59,
            narration: "Chuyen tien giay cuoi cung gio dem 04h59m59s".to_string(),
        },
        ScreenableTransaction {
            tx_id: "TX-TZ-05-00-00".to_string(),
            account_number: "ACC-NIGHT-EDGE-04".to_string(),
            counterparty_name: "CONG TY TNHH TEST 4".to_string(),
            counterparty_account: "CP-EDGE-04".to_string(),
            amount: Money::vnd(250_000_000),
            is_credit: true,
            timestamp_seconds: t_05_00_00,
            narration: "Chuyen tien dau gio sang 05h00m00s".to_string(),
        },
    ];

    let enhanced_alerts = engine.screen_with_anomaly_detection(&boundary_txs);

    let has_night_alert = |tx_id: &str| -> bool {
        enhanced_alerts.iter().any(|a| {
            a.tx_id == tx_id
                && (a.rule_code == AmlAlertCode::NightAnomaly
                    || a.anomaly_reasons.iter().any(|r| r.contains("ngoài giờ hành chính")))
        })
    };

    // Assertion 1: 22:59:59 GMT+7 is OUTSIDE night window
    assert!(
        !has_night_alert("TX-TZ-22-59-59"),
        "22:59:59 GMT+7 is before 23:00:00, must NOT trigger NightAnomaly"
    );

    // Assertion 2: 23:00:00 GMT+7 is INSIDE night window
    assert!(
        has_night_alert("TX-TZ-23-00-00"),
        "23:00:00 GMT+7 is exact start of night window, MUST trigger NightAnomaly"
    );
    let a_23 = enhanced_alerts.iter().find(|a| a.tx_id == "TX-TZ-23-00-00").unwrap();
    assert!(
        a_23.anomaly_reasons.iter().any(|r| r.contains("23:00 GMT+7")),
        "Reason must state 23:00 GMT+7"
    );

    // Assertion 3: 04:59:59 GMT+7 is INSIDE night window
    assert!(
        has_night_alert("TX-TZ-04-59-59"),
        "04:59:59 GMT+7 is within night window [23:00..05:00), MUST trigger NightAnomaly"
    );
    let a_04 = enhanced_alerts.iter().find(|a| a.tx_id == "TX-TZ-04-59-59").unwrap();
    assert!(
        a_04.anomaly_reasons.iter().any(|r| r.contains("04:00 GMT+7")),
        "Reason must state 04:00 GMT+7"
    );

    // Assertion 4: 05:00:00 GMT+7 is OUTSIDE night window
    assert!(
        !has_night_alert("TX-TZ-05-00-00"),
        "05:00:00 GMT+7 is boundary exit of night window, must NOT trigger NightAnomaly"
    );

    println!("PASSED: Exact timezone boundary verified for 22:59:59 (OUT), 23:00:00 (IN), 04:59:59 (IN), 05:00:00 (OUT).");
}

#[test]
fn test_off_hours_amount_threshold_boundaries() {
    let engine = AmlScreeningEngine::with_default_policy();

    // 02:00:00 AM GMT+7 (Deep within night window)
    let t_night_2am = REF_MIDNIGHT_GMT7_UTC + 86400 + (2 * 3600);

    let amount_boundary_txs = vec![
        ScreenableTransaction {
            tx_id: "TX-AMT-199M".to_string(),
            account_number: "ACC-AMT-01".to_string(),
            counterparty_name: "CONG TY TNHH TEST AMT 1".to_string(),
            counterparty_account: "CP-AMT-01".to_string(),
            amount: Money::vnd(199_999_999), // 199,999,999 VND (< 200M VND)
            is_credit: true,
            timestamp_seconds: t_night_2am,
            narration: "Giao dich 199.999.999 VND ban dem".to_string(),
        },
        ScreenableTransaction {
            tx_id: "TX-AMT-200M".to_string(),
            account_number: "ACC-AMT-02".to_string(),
            counterparty_name: "CONG TY TNHH TEST AMT 2".to_string(),
            counterparty_account: "CP-AMT-02".to_string(),
            amount: Money::vnd(200_000_000), // Exactly 200,000,000 VND
            is_credit: true,
            timestamp_seconds: t_night_2am + 100,
            narration: "Giao dich 200.000.000 VND tron ban dem".to_string(),
        },
        ScreenableTransaction {
            tx_id: "TX-AMT-200M-PLUS1".to_string(),
            account_number: "ACC-AMT-03".to_string(),
            counterparty_name: "CONG TY TNHH TEST AMT 3".to_string(),
            counterparty_account: "CP-AMT-03".to_string(),
            amount: Money::vnd(200_000_001), // 200,000,001 VND (> 200M VND)
            is_credit: true,
            timestamp_seconds: t_night_2am + 200,
            narration: "Giao dich 200.000.001 VND ban dem".to_string(),
        },
        ScreenableTransaction {
            tx_id: "TX-AMT-50M".to_string(),
            account_number: "ACC-AMT-04".to_string(),
            counterparty_name: "CONG TY TNHH TEST AMT 4".to_string(),
            counterparty_account: "CP-AMT-04".to_string(),
            amount: Money::vnd(50_000_000), // Normal sub-threshold retail amount
            is_credit: true,
            timestamp_seconds: t_night_2am + 300,
            narration: "Giao dich binh thuong 50M ban dem".to_string(),
        },
    ];

    let alerts = engine.screen_with_anomaly_detection(&amount_boundary_txs);

    let has_night_alert = |tx_id: &str| -> bool {
        alerts.iter().any(|a| {
            a.tx_id == tx_id
                && (a.rule_code == AmlAlertCode::NightAnomaly
                    || a.anomaly_reasons.iter().any(|r| r.contains("ngoài giờ hành chính")))
        })
    };

    // Sub-threshold amount (199,999,999 VND) must NOT trigger NightAnomaly
    assert!(
        !has_night_alert("TX-AMT-199M"),
        "199,999,999 VND is strictly below 200M VND threshold and must NOT trigger NightAnomaly"
    );

    // Normal amount (50,000,000 VND) must NOT trigger NightAnomaly
    assert!(
        !has_night_alert("TX-AMT-50M"),
        "50,000,000 VND is far below 200M VND threshold and must NOT trigger NightAnomaly"
    );

    // Exact threshold (200,000,000 VND) triggers due to >= comparison
    let triggers_at_200m = has_night_alert("TX-AMT-200M");
    println!("Empirical observation: Amount exactly 200,000,000 VND triggers NightAnomaly? {}", triggers_at_200m);
    assert!(
        triggers_at_200m,
        "Implementation uses '>= off_hours_thresh' so exactly 200,000,000 VND triggers"
    );

    // Strictly above threshold (200,000,001 VND) MUST trigger NightAnomaly
    assert!(
        has_night_alert("TX-AMT-200M-PLUS1"),
        "200,000,001 VND is strictly above 200M VND and MUST trigger NightAnomaly"
    );

    println!("PASSED: Night velocity amount threshold tested: 199.999M (NO), 200.000M (YES, >=), 200.001M (YES).");
}

#[test]
fn test_composite_risk_score_accumulation_and_clamp_at_10000_bps() {
    let engine = AmlScreeningEngine::with_default_policy();

    // 01:30:00 AM GMT+7
    let t_night = REF_MIDNIGHT_GMT7_UTC + 3600 + (30 * 60);

    // 1. Single Trigger: Only Off-hours (250M VND, benign narration, daytime baseline)
    let tx_only_night = ScreenableTransaction {
        tx_id: "TX-ONLY-NIGHT".to_string(),
        account_number: "ACC-SINGLE-01".to_string(),
        counterparty_name: "CONG TY TNHH HOP DONG".to_string(),
        counterparty_account: "CP-S-01".to_string(),
        amount: Money::vnd(250_000_000),
        is_credit: true,
        timestamp_seconds: t_night,
        narration: "Thanh toan hop dong cung ung dich vu".to_string(),
    };
    let alerts_single = engine.screen_with_anomaly_detection(&[tx_only_night]);
    assert_eq!(alerts_single.len(), 1);
    println!(
        "Observation on tx_only_night (250M at 01:30 AM): score = {} bps, reasons: {:?}",
        alerts_single[0].anomaly_score_bps, alerts_single[0].anomaly_reasons
    );
    // Statutory Night Velocity (>= 200M agg): +2500 bps (High)
    // Dim 1 Off-hours anomaly (> 200M): +2000 bps
    // Composite sum: 2500 + 2000 = 4500 bps
    assert_eq!(
        alerts_single[0].anomaly_score_bps, 4500,
        "Statutory High (2500) + Dim 1 Night (2000) gives exactly 4500 bps"
    );
    assert_eq!(alerts_single[0].risk_profile, "ELEVATED_RISK");
    assert_eq!(alerts_single[0].severity, AmlSeverity::High);

    // 2. Multi-Trigger: Off-hours + Smurfing + Semantic Evasion ("TIEN AO" + "RUA TIEN")
    // Amount: 388,000,000 VND (in smurfing band [380M..399.999M] AND >= 200M off-hours)
    // Narration contains suspicious keywords: "TIEN AO", "RUA TIEN"
    let tx_multi_trigger = ScreenableTransaction {
        tx_id: "TX-MULTI-TRIGGER".to_string(),
        account_number: "ACC-MULTI-01".to_string(),
        counterparty_name: "CONG TY TNHH DAU TU".to_string(),
        counterparty_account: "CP-M-01".to_string(),
        amount: Money::vnd(388_000_000),
        is_credit: true,
        timestamp_seconds: t_night,
        narration: "Chuyen tien mua TIEN AO va RUA TIEN nhanh".to_string(),
    };
    let alerts_multi = engine.screen_with_anomaly_detection(&[tx_multi_trigger]);
    assert_eq!(alerts_multi.len(), 1);
    let multi_alert = &alerts_multi[0];

    // Verification of simultaneous trigger detection
    println!(
        "Multi-trigger anomaly_reasons: {:?}",
        multi_alert.anomaly_reasons
    );
    let has_night_reason = multi_alert.anomaly_reasons.iter().any(|r| r.contains("ngoài giờ hành chính"));
    let has_smurf_reason = multi_alert.anomaly_reasons.iter().any(|r| r.contains("dải phân tách né ngưỡng"));
    let has_semantic_reason = multi_alert.anomaly_reasons.iter().any(|r| r.contains("TIEN AO") || r.contains("RUA TIEN"));
    let has_statutory_wl = multi_alert.anomaly_reasons.iter().any(|r| r.contains("từ khóa rủi ro cấm"));

    assert!(has_night_reason, "Must flag off-hours trigger");
    assert!(has_smurf_reason, "Must flag smurfing trigger");
    assert!(has_semantic_reason, "Must flag semantic narration trigger");
    assert!(has_statutory_wl, "Must flag statutory watchlist trigger");

    // Theoretical accumulation for multi_alert:
    // Statutory Night: +2500 (High)
    // Statutory Watchlist: +2500 (High)
    // Dim 1 (Night Anomaly): +2000
    // Dim 2 (Smurfing Band): +2500
    // Dim 3 (Semantic Narration): +2375
    // Raw sum = 2500 + 2500 + 2000 + 2500 + 2375 = 11,875 bps.
    // Clamped by .min(10_000) -> 10,000 bps!
    println!(
        "Multi-trigger composite score: {} bps (clamped from raw 11,875 bps), profile: {}, severity: {:?}",
        multi_alert.anomaly_score_bps, multi_alert.risk_profile, multi_alert.severity
    );
    assert_eq!(multi_alert.anomaly_score_bps, 10_000, "Raw 11,875 bps must be clamped to exactly 10,000 bps");
    assert_eq!(multi_alert.risk_profile, "CRITICAL_SUSPICIOUS");
    assert_eq!(multi_alert.severity, AmlSeverity::Critical);
    assert!(multi_alert.anomaly_score_bps <= 10_000);

    // 2B. Intermediate Multi-Trigger (Daytime: Smurfing 2500 + Watchlist 2500 + Semantic 2375 = 7375 bps < 10,000)
    let t_daytime = REF_MIDNIGHT_GMT7_UTC + (14 * 3600); // 14:00 PM GMT+7
    let tx_intermediate = ScreenableTransaction {
        tx_id: "TX-INTERMEDIATE".to_string(),
        account_number: "ACC-INTER-01".to_string(),
        counterparty_name: "CONG TY TNHH DAU TU".to_string(),
        counterparty_account: "CP-INT-01".to_string(),
        amount: Money::vnd(388_000_000), // Smurfing band [380M..399.999M]
        is_credit: true,
        timestamp_seconds: t_daytime,
        narration: "Chuyen tien mua TIEN AO hop phap".to_string(),
    };
    let alerts_inter = engine.screen_with_anomaly_detection(&[tx_intermediate]);
    assert_eq!(alerts_inter.len(), 1);
    let inter_alert = &alerts_inter[0];
    println!(
        "Intermediate multi-trigger score: {} bps, reasons: {:?}",
        inter_alert.anomaly_score_bps, inter_alert.anomaly_reasons
    );
    // Statutory Watchlist: 2500
    // Dim 2 Smurfing: 2500
    // Dim 3 Semantic ('crypto_otc' 95%): 2375
    // Total = 2500 + 2500 + 2375 = 7375 bps
    assert_eq!(inter_alert.anomaly_score_bps, 7375);
    assert_eq!(inter_alert.risk_profile, "HIGH_RISK");
    assert_eq!(inter_alert.severity, AmlSeverity::High);
    assert!(inter_alert.anomaly_score_bps < 10_000);

    // 3. Extreme Saturation Multi-Trigger: Simultaneous Over-Capacity (Sum > 15,000 bps)
    // We construct a scenario where:
    // - Statutory High-Value (>= 400M): +2500
    // - Statutory Watchlist ("TIEN AO"): +2500
    // - Micro-Burst (< 60s): +1500
    // - Frequency Surge (>= 5 txs in 1h): +2000
    //   -> Dim 1 capped at 2500
    // - Night Anomaly (> 200M at 02:00): +2000
    // - MAD Outlier (Modified Z > 5.0): +2500
    //   -> Dim 2 capped at 3000
    // - Semantic Anomaly ("RUA TIEN"): +2500
    //   -> Dim 3 capped at 2500
    // Unclamped sum = 2500 + 2500 + 2500 + 3000 + 2500 = 13,000 bps.
    // The final score MUST be clamped at exactly 10,000 bps!
    let mut sat_batch: Vec<ScreenableTransaction> = (0..15)
        .map(|i| ScreenableTransaction {
            tx_id: format!("TX-SAT-HIST-{:02}", i),
            account_number: "ACC-SAT-01".to_string(),
            counterparty_name: "CONG TY BAN LE".to_string(),
            counterparty_account: format!("CP-SAT-{:02}", i),
            amount: Money::vnd(10_000_000), // Historical baseline 10M
            is_credit: true,
            timestamp_seconds: t_night - 7200 + (i * 300), // Daytime/evening
            narration: "Giao dich tap hoa".to_string(),
        })
        .collect();

    // Add rapid burst transactions in night window
    for j in 1..=4 {
        sat_batch.push(ScreenableTransaction {
            tx_id: format!("TX-SAT-BURST-{:02}", j),
            account_number: "ACC-SAT-01".to_string(),
            counterparty_name: "CONG TY TNHH TIEN AO".to_string(),
            counterparty_account: format!("CP-SAT-BURST-{:02}", j),
            amount: Money::vnd(15_000_000),
            is_credit: true,
            timestamp_seconds: t_night + (j * 15), // Every 15 seconds (< 60s micro-burst)
            narration: "Mua crypto nhanh".to_string(),
        });
    }

    // Add the monster saturated transaction
    sat_batch.push(ScreenableTransaction {
        tx_id: "TX-SAT-MONSTER".to_string(),
        account_number: "ACC-SAT-01".to_string(),
        counterparty_name: "NGUYEN VAN TOI PHAM".to_string(),
        counterparty_account: "CP-SAT-MONSTER".to_string(),
        amount: Money::vnd(800_000_000), // >= 400M statutory, huge MAD outlier, night > 200M
        is_credit: true,
        timestamp_seconds: t_night + 70, // In burst window, 01:31 AM
        narration: "Chuyen khoan TIEN AO RUA TIEN quy mo lon".to_string(),
    });

    let sat_alerts = engine.screen_with_anomaly_detection(&sat_batch);
    let monster_alert = sat_alerts
        .iter()
        .find(|a| a.tx_id == "TX-SAT-MONSTER")
        .expect("Monster transaction must trigger enhanced alert");

    println!(
        "Monster saturated transaction score: {} bps (capped at 10,000 bps). Reasons count: {}",
        monster_alert.anomaly_score_bps,
        monster_alert.anomaly_reasons.len()
    );

    // CRITICAL INVARIANT: The composite score MUST NEVER exceed 10,000 bps (1.0000)
    assert!(
        monster_alert.anomaly_score_bps <= 10_000,
        "Score must not exceed 10,000 bps"
    );
    assert_eq!(
        monster_alert.anomaly_score_bps, 10_000,
        "Extreme saturated multi-trigger transaction must be clamped at exactly 10,000 bps"
    );
    assert_eq!(monster_alert.risk_profile, "CRITICAL_SUSPICIOUS");
    assert_eq!(monster_alert.severity, AmlSeverity::Critical);

    println!("PASSED: Composite risk score accumulation and 10,000 bps upper bound verified.");
}

#[test]
fn test_adversarial_timestamp_wrapping_and_negative_timestamps() {
    let engine = AmlScreeningEngine::with_default_policy();

    // 1. Unix epoch 0 (1970-01-01 00:00:00 UTC = 07:00:00 GMT+7, daytime)
    let tx_epoch_0 = ScreenableTransaction {
        tx_id: "TX-EPOCH-0".to_string(),
        account_number: "ACC-EPOCH-0".to_string(),
        counterparty_name: "CONG TY TNHH EPOCH".to_string(),
        counterparty_account: "CP-0".to_string(),
        amount: Money::vnd(300_000_000),
        is_credit: true,
        timestamp_seconds: 0,
        narration: "Giao dich tai moc epoch 0".to_string(),
    };

    // 2. Negative timestamp (e.g. 1969-12-31 23:59:59 UTC = 1970-01-01 06:59:59 GMT+7)
    let tx_neg = ScreenableTransaction {
        tx_id: "TX-NEG-TS".to_string(),
        account_number: "ACC-NEG-0".to_string(),
        counterparty_name: "CONG TY TNHH NEGATIVE".to_string(),
        counterparty_account: "CP-NEG".to_string(),
        amount: Money::vnd(300_000_000),
        is_credit: true,
        timestamp_seconds: -1,
        narration: "Giao dich timestamp am -1".to_string(),
    };

    // 3. Near max i64 timestamp to test wrapping_add
    let tx_near_max = ScreenableTransaction {
        tx_id: "TX-NEAR-MAX".to_string(),
        account_number: "ACC-MAX-0".to_string(),
        counterparty_name: "CONG TY TNHH MAX".to_string(),
        counterparty_account: "CP-MAX".to_string(),
        amount: Money::vnd(300_000_000),
        is_credit: true,
        timestamp_seconds: i64::MAX - 1000,
        narration: "Giao dich timestamp gan cuc dai".to_string(),
    };

    // Engine must not panic on negative, zero, or large timestamps
    let alerts = engine.screen_with_anomaly_detection(&[tx_epoch_0, tx_neg, tx_near_max]);
    println!("Adversarial timestamp test completed without panic. Alerts generated: {}", alerts.len());
}
