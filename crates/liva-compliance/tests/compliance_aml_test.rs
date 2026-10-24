use liva_compliance::aml::{
    integer_jaro, integer_jaro_winkler, normalize_ascii, AmlAlertCode, AmlScreeningEngine,
    AmlSeverity, ScreenableTransaction,
};
use liva_money::Money;

#[test]
fn test_decision_11_high_value_statutory_thresholds() {
    let engine = AmlScreeningEngine::with_default_policy();

    let txs = vec![
        // 1. Exactly 400M VND -> Statutory threshold trigger
        ScreenableTransaction {
            tx_id: "TX-400M".to_string(),
            account_number: "00710001".to_string(),
            counterparty_name: "Cong ty ABC".to_string(),
            counterparty_account: "19034001".to_string(),
            amount: Money::vnd(400_000_000),
            is_credit: true,
            timestamp_seconds: 1726300000,
            narration: "Thanh toan hop dong 400M".to_string(),
        },
        // 2. Just below threshold: 399,999,999 VND -> No high value alert
        ScreenableTransaction {
            tx_id: "TX-SUB-400M".to_string(),
            account_number: "00710002".to_string(),
            counterparty_name: "Cong ty XYZ".to_string(),
            counterparty_account: "19034002".to_string(),
            amount: Money::vnd(399_999_999),
            is_credit: true,
            timestamp_seconds: 1726300100,
            narration: "Thanh toan duoi nguong".to_string(),
        },
        // 3. Critical threshold: 1,200,000,000 VND (>= 1 Billion)
        ScreenableTransaction {
            tx_id: "TX-1B2".to_string(),
            account_number: "00710003".to_string(),
            counterparty_name: "Cong ty Big Corp".to_string(),
            counterparty_account: "19034003".to_string(),
            amount: Money::vnd(1_200_000_000),
            is_credit: false,
            timestamp_seconds: 1726300200,
            narration: "Chuyen tien dau tu".to_string(),
        },
    ];

    let alerts = engine.screen_transactions(&txs);

    // TX-400M must trigger HighValueThreshold with High severity
    let alert_400m = alerts.iter().find(|a| a.tx_id == "TX-400M").expect("Must alert on 400M");
    assert_eq!(alert_400m.rule_code, AmlAlertCode::HighValueThreshold);
    assert_eq!(alert_400m.severity, AmlSeverity::High);

    // TX-SUB-400M must NOT trigger HighValueThreshold
    assert!(
        alerts.iter().all(|a| a.tx_id != "TX-SUB-400M"),
        "Sub-threshold transaction must not trigger high value alert"
    );

    // TX-1B2 must trigger HighValueThreshold with Critical severity
    let alert_1b2 = alerts.iter().find(|a| a.tx_id == "TX-1B2").expect("Must alert on 1.2B");
    assert_eq!(alert_1b2.rule_code, AmlAlertCode::HighValueThreshold);
    assert_eq!(alert_1b2.severity, AmlSeverity::Critical);
}

#[test]
fn test_rolling_24h_structuring_sliding_window() {
    let engine = AmlScreeningEngine::with_default_policy();

    // Scenario A: 3 transactions in 24 hours (each 150M, sum 450M >= 400M)
    let t0 = 1726300000;
    let txs_structuring = vec![
        ScreenableTransaction {
            tx_id: "STRUC-1".to_string(),
            account_number: "ACCOUNT-STRUC-01".to_string(),
            counterparty_name: "Doi tac A".to_string(),
            counterparty_account: "CP-01".to_string(),
            amount: Money::vnd(150_000_000),
            is_credit: true,
            timestamp_seconds: t0,
            narration: "Dot 1".to_string(),
        },
        ScreenableTransaction {
            tx_id: "STRUC-2".to_string(),
            account_number: "ACCOUNT-STRUC-01".to_string(),
            counterparty_name: "Doi tac B".to_string(),
            counterparty_account: "CP-02".to_string(),
            amount: Money::vnd(150_000_000),
            is_credit: true,
            timestamp_seconds: t0 + 10_000,
            narration: "Dot 2".to_string(),
        },
        ScreenableTransaction {
            tx_id: "STRUC-3".to_string(),
            account_number: "ACCOUNT-STRUC-01".to_string(),
            counterparty_name: "Doi tac C".to_string(),
            counterparty_account: "CP-03".to_string(),
            amount: Money::vnd(150_000_000),
            is_credit: true,
            timestamp_seconds: t0 + 40_000, // Within 86,400s (24h)
            narration: "Dot 3".to_string(),
        },
    ];

    let alerts = engine.screen_transactions(&txs_structuring);
    let struc_alert = alerts
        .iter()
        .find(|a| a.rule_code == AmlAlertCode::StructuringSmurfing);
    assert!(struc_alert.is_some(), "Must detect structuring in rolling 24h");
    let a = struc_alert.unwrap();
    assert_eq!(a.severity, AmlSeverity::Critical);
    assert_eq!(a.amount.amount(), 450_000_000);

    // Scenario B: Transactions spread beyond 24 hours (t0, t0 + 50k, t0 + 100k)
    let txs_spread = vec![
        ScreenableTransaction {
            tx_id: "SPREAD-1".to_string(),
            account_number: "ACCOUNT-SPREAD-01".to_string(),
            counterparty_name: "Doi tac A".to_string(),
            counterparty_account: "CP-01".to_string(),
            amount: Money::vnd(150_000_000),
            is_credit: true,
            timestamp_seconds: t0,
            narration: "Dot 1".to_string(),
        },
        ScreenableTransaction {
            tx_id: "SPREAD-2".to_string(),
            account_number: "ACCOUNT-SPREAD-01".to_string(),
            counterparty_name: "Doi tac B".to_string(),
            counterparty_account: "CP-02".to_string(),
            amount: Money::vnd(150_000_000),
            is_credit: true,
            timestamp_seconds: t0 + 50_000,
            narration: "Dot 2".to_string(),
        },
        ScreenableTransaction {
            tx_id: "SPREAD-3".to_string(),
            account_number: "ACCOUNT-SPREAD-01".to_string(),
            counterparty_name: "Doi tac C".to_string(),
            counterparty_account: "CP-03".to_string(),
            amount: Money::vnd(150_000_000),
            is_credit: true,
            timestamp_seconds: t0 + 100_000, // Beyond 86,400s!
            narration: "Dot 3".to_string(),
        },
    ];

    let alerts_spread = engine.screen_transactions(&txs_spread);
    assert!(
        alerts_spread
            .iter()
            .all(|a| a.rule_code != AmlAlertCode::StructuringSmurfing),
        "Transactions spanning beyond 24h must not trigger structuring alert"
    );
}

#[test]
fn test_rapid_pass_through_transit_mule() {
    let engine = AmlScreeningEngine::with_default_policy();
    let t0 = 1726300000;

    // Test A: Inflow 200M, Outflow 196M (98% ratio >= 95%) in 600s (<= 900s)
    let txs_mule = vec![
        ScreenableTransaction {
            tx_id: "MULE-IN".to_string(),
            account_number: "MULE-ACC-999".to_string(),
            counterparty_name: "Nguon tien".to_string(),
            counterparty_account: "SRC-01".to_string(),
            amount: Money::vnd(200_000_000),
            is_credit: true,
            timestamp_seconds: t0,
            narration: "Nhan tien".to_string(),
        },
        ScreenableTransaction {
            tx_id: "MULE-OUT".to_string(),
            account_number: "MULE-ACC-999".to_string(),
            counterparty_name: "Dich den".to_string(),
            counterparty_account: "DST-01".to_string(),
            amount: Money::vnd(196_000_000),
            is_credit: false,
            timestamp_seconds: t0 + 600,
            narration: "Chuyen tiep ngay".to_string(),
        },
    ];

    let alerts = engine.screen_transactions(&txs_mule);
    let mule_alert = alerts
        .iter()
        .find(|a| a.rule_code == AmlAlertCode::PassThroughTransit);
    assert!(mule_alert.is_some(), "Must detect rapid transit mule");
    assert_eq!(mule_alert.unwrap().severity, AmlSeverity::Critical);

    // Test B: Outflow only 80% (160M) -> Does not meet 95% threshold
    let txs_low_ratio = vec![
        ScreenableTransaction {
            tx_id: "LOW-IN".to_string(),
            account_number: "MULE-ACC-888".to_string(),
            counterparty_name: "Nguon tien".to_string(),
            counterparty_account: "SRC-01".to_string(),
            amount: Money::vnd(200_000_000),
            is_credit: true,
            timestamp_seconds: t0,
            narration: "Nhan tien".to_string(),
        },
        ScreenableTransaction {
            tx_id: "LOW-OUT".to_string(),
            account_number: "MULE-ACC-888".to_string(),
            counterparty_name: "Dich den".to_string(),
            counterparty_account: "DST-01".to_string(),
            amount: Money::vnd(160_000_000), // 80%
            is_credit: false,
            timestamp_seconds: t0 + 600,
            narration: "Chuyen mot phan".to_string(),
        },
    ];

    let alerts_low = engine.screen_transactions(&txs_low_ratio);
    assert!(
        alerts_low
            .iter()
            .all(|a| a.rule_code != AmlAlertCode::PassThroughTransit),
        "Outflow under 95% must not trigger transit mule alert"
    );

    // Test C: Outflow occurs after 1200s (> 900s) -> Does not meet window
    let txs_slow = vec![
        ScreenableTransaction {
            tx_id: "SLOW-IN".to_string(),
            account_number: "MULE-ACC-777".to_string(),
            counterparty_name: "Nguon tien".to_string(),
            counterparty_account: "SRC-01".to_string(),
            amount: Money::vnd(200_000_000),
            is_credit: true,
            timestamp_seconds: t0,
            narration: "Nhan tien".to_string(),
        },
        ScreenableTransaction {
            tx_id: "SLOW-OUT".to_string(),
            account_number: "MULE-ACC-777".to_string(),
            counterparty_name: "Dich den".to_string(),
            counterparty_account: "DST-01".to_string(),
            amount: Money::vnd(196_000_000),
            is_credit: false,
            timestamp_seconds: t0 + 1200, // 20 minutes (> 15 minutes)
            narration: "Chuyen cham".to_string(),
        },
    ];

    let alerts_slow = engine.screen_transactions(&txs_slow);
    assert!(
        alerts_slow
            .iter()
            .all(|a| a.rule_code != AmlAlertCode::PassThroughTransit),
        "Outflow after 900s must not trigger transit mule alert"
    );
}

#[test]
fn test_night_velocity_off_hours_surveillance() {
    let engine = AmlScreeningEngine::with_default_policy();

    // 02:00 AM GMT+7 = 19:00 UTC previous day
    let t_night_exact: i64 = 1726272000 - (5 * 3600);

    let txs_night = vec![
        ScreenableTransaction {
            tx_id: "NIGHT-1".to_string(),
            account_number: "NIGHT-ACC-01".to_string(),
            counterparty_name: "A".to_string(),
            counterparty_account: "CA-1".to_string(),
            amount: Money::vnd(70_000_000),
            is_credit: true,
            timestamp_seconds: t_night_exact,
            narration: "Dem 1".to_string(),
        },
        ScreenableTransaction {
            tx_id: "NIGHT-2".to_string(),
            account_number: "NIGHT-ACC-01".to_string(),
            counterparty_name: "B".to_string(),
            counterparty_account: "CA-2".to_string(),
            amount: Money::vnd(80_000_000),
            is_credit: true,
            timestamp_seconds: t_night_exact + 300,
            narration: "Dem 2".to_string(),
        },
        ScreenableTransaction {
            tx_id: "NIGHT-3".to_string(),
            account_number: "NIGHT-ACC-01".to_string(),
            counterparty_name: "C".to_string(),
            counterparty_account: "CA-3".to_string(),
            amount: Money::vnd(90_000_000),
            is_credit: true,
            timestamp_seconds: t_night_exact + 600,
            narration: "Dem 3".to_string(),
        },
    ];

    let alerts = engine.screen_transactions(&txs_night);
    let night_alert = alerts
        .iter()
        .find(|a| a.rule_code == AmlAlertCode::NightAnomaly);
    assert!(night_alert.is_some(), "Must detect night velocity anomaly");
    assert_eq!(night_alert.unwrap().severity, AmlSeverity::High);
}

#[test]
fn test_watchlist_and_sanction_screening() {
    let engine = AmlScreeningEngine::with_default_policy();

    let txs = vec![
        // A. Narration keyword "TIEN AO"
        ScreenableTransaction {
            tx_id: "WL-KW".to_string(),
            account_number: "00710001".to_string(),
            counterparty_name: "Nguyen Van A".to_string(),
            counterparty_account: "19034001".to_string(),
            amount: Money::vnd(20_000_000),
            is_credit: true,
            timestamp_seconds: 1726300000,
            narration: "Chuyen khoan mua TIEN AO tren san".to_string(),
        },
        // B. Blacklisted account "007100987654"
        ScreenableTransaction {
            tx_id: "WL-BL".to_string(),
            account_number: "007100987654".to_string(),
            counterparty_name: "Tran B".to_string(),
            counterparty_account: "19034002".to_string(),
            amount: Money::vnd(15_000_000),
            is_credit: false,
            timestamp_seconds: 1726300100,
            narration: "Thanh toan tien mat".to_string(),
        },
        // C. Sanctioned name "NGUYEN VAN TOAN"
        ScreenableTransaction {
            tx_id: "WL-SANC".to_string(),
            account_number: "00710003".to_string(),
            counterparty_name: "NGUYEN VAN TOAN".to_string(),
            counterparty_account: "19034003".to_string(),
            amount: Money::vnd(30_000_000),
            is_credit: true,
            timestamp_seconds: 1726300200,
            narration: "Hop dong dich vu".to_string(),
        },
        // D. Benign transaction
        ScreenableTransaction {
            tx_id: "WL-CLEAN".to_string(),
            account_number: "00710004".to_string(),
            counterparty_name: "LE THI MAI".to_string(),
            counterparty_account: "19034004".to_string(),
            amount: Money::vnd(10_000_000),
            is_credit: true,
            timestamp_seconds: 1726300300,
            narration: "Tra tien com trua".to_string(),
        },
    ];

    let alerts = engine.screen_transactions(&txs);

    let kw_alert = alerts.iter().find(|a| a.tx_id == "WL-KW");
    assert!(kw_alert.is_some(), "Must flag narration keyword alert");

    let bl_alert = alerts.iter().find(|a| a.tx_id == "WL-BL");
    assert!(bl_alert.is_some(), "Must flag blacklisted account alert");
    assert_eq!(bl_alert.unwrap().severity, AmlSeverity::Critical);

    let sanc_alert = alerts.iter().find(|a| a.tx_id == "WL-SANC");
    assert!(sanc_alert.is_some(), "Must flag sanctioned name alert");
    assert_eq!(sanc_alert.unwrap().severity, AmlSeverity::Critical);

    let clean_alert = alerts.iter().find(|a| a.tx_id == "WL-CLEAN");
    assert!(clean_alert.is_none(), "Clean transaction must have 0 alerts");
}

#[test]
fn test_integer_jaro_winkler_basis_points() {
    assert_eq!(integer_jaro("MARTHA", "MARHTA"), 9444);
    assert_eq!(integer_jaro_winkler("MARTHA", "MARHTA"), 9611);
    assert_eq!(integer_jaro_winkler("DWAYNE", "DUANE"), 8400);
    assert_eq!(integer_jaro_winkler("SAME", "SAME"), 10_000);
    assert_eq!(integer_jaro_winkler("ABC", "XYZ"), 0);
    assert_eq!(normalize_ascii("Nguyễn Văn Bình"), "NGUYEN VAN BINH");
}

#[test]
fn test_str_report_drafting_and_risk_routing() {
    let engine = AmlScreeningEngine::with_default_policy();

    let tx = ScreenableTransaction {
        tx_id: "TX-STR-TEST".to_string(),
        account_number: "0071000999".to_string(),
        counterparty_name: "Nguyen Van Toan".to_string(),
        counterparty_account: "1903400999".to_string(),
        amount: Money::vnd(800_000_000),
        is_credit: true,
        timestamp_seconds: 1726300000,
        narration: "Giao dich dang ngo".to_string(),
    };

    let alerts = engine.screen_transactions(&[tx.clone()]);
    assert!(!alerts.is_empty());

    let alert = &alerts[0];
    let str_rep = engine.build_str_report(alert, &tx, "Phat hien rui ro can bao cao NHNN.");

    assert!(str_rep.str_id.starts_with("STR-"));
    assert_eq!(str_rep.reporting_entity, "LIVA Commercial Banking Operations Center");
    assert!(str_rep.merkle_proof_hash.starts_with("0x"));
    assert!(str_rep.risk_score >= 20);
    assert!(!str_rep.recommended_action.is_empty());
}
