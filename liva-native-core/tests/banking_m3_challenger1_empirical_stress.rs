//! Milestone 3 (M3) Empirical Adversarial Stress Test Suite (Challenger 1)
//!
//! Empirical validation of 50,000 Synthetic Generator & Benchmark Harness:
//! 1. Invariant & PRNG Stress:
//!    - Arbitrary PRNG seeds (0, 1, u64::MAX, random seeds, benchmark seed).
//!    - Confirms that for EVERY seed, double-entry balance invariants hold 100% with strictly Delta = 0 VND.
//! 2. Invariant Tampering / Negative Verification:
//!    - Verifies that if any transaction amount, running balance, opening balance, or closing balance
//!      is perturbed by even 1 VND, both `verify_balance_invariants` and `check_balance_invariants`
//!      immediately detect and flag the discrepancy.
//! 3. Edge Case Amounts & Extreme Volumes:
//!    - Tests large treasury amounts (100 Billion VND, 1 Trillion VND, 10 Trillion VND) and near-boundary
//!      values to verify absolute absence of integer overflow, arithmetic panics, or float drift.
//! 4. CSV Statement Serialization & Re-parsing Invariant Round-trip:
//!    - Verifies balance invariants survive CSV export and streaming parser re-ingestion.
//! 5. Anti-Cheating & Integrity Audit:
//!    - Confirms all generated bank transactions and ledger entries initialize strictly as Unmatched.

use std::collections::HashSet;

use liva_native_core::banking::generator::{
    DEFAULT_BENCHMARK_SEED, format_statement_as_csv, generate_50k_dataset,
};
use liva_native_core::banking::models::{
    BankStatement, BankTransactionRow, BankType, ReconciliationStatus, StatementFormat,
    TransactionRecord, TransactionType, verify_balance_invariants,
};
use liva_native_core::banking::parser::sniff_and_parse;
use liva_native_core::banking::reconciliation::ReconciliationEngine;

// ===========================================================================
// 1. Invariant & PRNG Stress Across Arbitrary Seeds
// ===========================================================================

#[test]
fn test_challenger1_prng_seed_invariance_and_delta_zero() {
    // Test diverse PRNG seeds including extreme boundary values:
    // - 0 (zero boundary)
    // - 1 (unit boundary)
    // - u64::MAX (18_446_744_073_709_551_615)
    // - 0xDEAD_BEEF_CAFE_BABE
    // - DEFAULT_BENCHMARK_SEED (0x50_000_2026)
    let test_seeds: [u64; 5] = [
        0,
        1,
        u64::MAX,
        0xDEAD_BEEF_CAFE_BABE,
        DEFAULT_BENCHMARK_SEED,
    ];

    for seed in test_seeds {
        let dataset = generate_50k_dataset(seed);

        assert_eq!(
            dataset.bank_transactions.len(),
            50_000,
            "Seed {seed} must generate exactly 50,000 bank transactions"
        );
        assert_eq!(
            dataset.ledger_entries.len(),
            50_000,
            "Seed {seed} must generate exactly 50,000 ledger entries"
        );
        assert_eq!(
            dataset.statements.len(),
            5,
            "Seed {seed} must partition into 5 bank statements"
        );

        // Verify statement-level double-entry invariants
        for stmt in &dataset.statements {
            let report = stmt.verify_balance_invariants();
            assert!(
                report.is_valid,
                "Seed {seed}: Statement invariant failed for bank {}. Discrepancy: {} VND",
                stmt.bank_code, report.discrepancy
            );
            assert!(
                report.is_balanced,
                "Seed {seed}: Statement balance flag failed for bank {}",
                stmt.bank_code
            );
            assert_eq!(
                report.discrepancy, 0,
                "Seed {seed}: Discrepancy must be strictly 0 VND for bank {}",
                stmt.bank_code
            );
            assert_eq!(
                report.closing_balance as i128,
                (report.opening_balance as i128) + (report.total_credit as i128)
                    - (report.total_debit as i128),
                "Seed {seed}: Closing != Opening + Credit - Debit for bank {}",
                stmt.bank_code
            );
        }

        // Verify transaction-level running balance transitions
        let (checked, passed, max_disc) =
            ReconciliationEngine::check_balance_invariants(&dataset.bank_transactions);
        assert!(
            checked,
            "Seed {seed}: Running balance check must be evaluated"
        );
        assert!(
            passed,
            "Seed {seed}: Running balance check must pass on all accounts"
        );
        assert_eq!(
            max_disc, 0,
            "Seed {seed}: Running balance max discrepancy must be strictly 0 VND"
        );
    }
}

// ===========================================================================
// 2. Invariant Tampering / Negative Verification (1 VND Perturbations)
// ===========================================================================

#[test]
fn test_challenger1_negative_verification_statement_tampering() {
    let dataset = generate_50k_dataset(DEFAULT_BENCHMARK_SEED);
    let original_stmt = &dataset.statements[0]; // Vietcombank

    // Baseline: Original statement must pass
    let base_report = original_stmt.verify_balance_invariants();
    assert!(base_report.is_valid);
    assert_eq!(base_report.discrepancy, 0);

    // --- Perturbation 2.1: Closing Balance +1 VND ---
    let mut tampered_closing_plus = original_stmt.clone();
    tampered_closing_plus.closing_balance = Some(original_stmt.closing_balance.unwrap() + 1);
    let rep_c_plus = tampered_closing_plus.verify_balance_invariants();
    assert!(
        !rep_c_plus.is_valid,
        "Closing balance +1 VND must be detected as invalid"
    );
    assert_eq!(
        rep_c_plus.discrepancy, 1,
        "Closing balance +1 VND must yield exactly +1 VND discrepancy"
    );

    // --- Perturbation 2.2: Closing Balance -1 VND ---
    let mut tampered_closing_minus = original_stmt.clone();
    tampered_closing_minus.closing_balance = Some(original_stmt.closing_balance.unwrap() - 1);
    let rep_c_minus = tampered_closing_minus.verify_balance_invariants();
    assert!(
        !rep_c_minus.is_valid,
        "Closing balance -1 VND must be detected as invalid"
    );
    assert_eq!(
        rep_c_minus.discrepancy, -1,
        "Closing balance -1 VND must yield exactly -1 VND discrepancy"
    );

    // --- Perturbation 2.3: Opening Balance +1 VND ---
    let mut tampered_opening_plus = original_stmt.clone();
    tampered_opening_plus.opening_balance = Some(original_stmt.opening_balance.unwrap() + 1);
    let rep_o_plus = tampered_opening_plus.verify_balance_invariants();
    assert!(
        !rep_o_plus.is_valid,
        "Opening balance +1 VND must be detected as invalid"
    );
    assert_eq!(
        rep_o_plus.discrepancy, -1,
        "Opening balance +1 VND must yield -1 discrepancy against calculated"
    );

    // --- Perturbation 2.4: Total Credit +1 VND ---
    let mut tampered_credit = original_stmt.clone();
    tampered_credit.total_credit += 1;
    let rep_cr = tampered_credit.verify_balance_invariants();
    assert!(!rep_cr.is_valid);
    assert_eq!(rep_cr.discrepancy, -1);

    // --- Perturbation 2.5: Total Debit +1 VND ---
    let mut tampered_debit = original_stmt.clone();
    tampered_debit.total_debit += 1;
    let rep_db = tampered_debit.verify_balance_invariants();
    assert!(!rep_db.is_valid);
    assert_eq!(rep_db.discrepancy, 1);

    // --- Perturbation 2.6: Transaction Record amount altered by 1 VND in standalone verifier ---
    let mut tampered_tx_records = original_stmt.transactions.clone();
    tampered_tx_records[50].amount += 1; // Perturb 1 transaction out of 20,000 by 1 VND
    let rep_tx_pert = ReconciliationEngine::verify_balance_invariants(
        original_stmt.opening_balance,
        original_stmt.closing_balance,
        &tampered_tx_records,
    );
    assert!(
        !rep_tx_pert.is_valid,
        "Transaction list with 1 VND perturbation must fail verify_balance_invariants"
    );
    assert_ne!(
        rep_tx_pert.discrepancy, 0,
        "Discrepancy must be non-zero when single transaction amount is altered"
    );
}

#[test]
fn test_challenger1_negative_verification_running_balance_tampering() {
    let dataset = generate_50k_dataset(DEFAULT_BENCHMARK_SEED);

    // Isolate VCB transactions (20,000 transactions)
    let mut vcb_txs: Vec<BankTransactionRow> = dataset
        .bank_transactions
        .into_iter()
        .filter(|t| t.bank_code == "VCB")
        .collect();
    assert_eq!(vcb_txs.len(), 20_000);

    // Baseline: Unperturbed batch must pass 100%
    let (checked, passed, max_disc) = ReconciliationEngine::check_balance_invariants(&vcb_txs);
    assert!(checked);
    assert!(passed);
    assert_eq!(max_disc, 0);

    // --- Perturbation 2.7: Middle transaction amount perturbed by +1 VND ---
    let original_amt = vcb_txs[10_000].amount;
    vcb_txs[10_000].amount += 1;
    let (_, passed_amt, disc_amt) = ReconciliationEngine::check_balance_invariants(&vcb_txs);
    assert!(
        !passed_amt,
        "check_balance_invariants must FAIL when a transaction amount is perturbed by 1 VND"
    );
    assert_ne!(
        disc_amt, 0,
        "Discrepancy must be detected when transaction amount is perturbed"
    );
    vcb_txs[10_000].amount = original_amt; // Restore

    // --- Perturbation 2.8: Middle transaction balance_after perturbed by +1 VND ---
    let original_bal = vcb_txs[10_000].balance_after;
    vcb_txs[10_000].balance_after = original_bal.map(|b| b + 1);
    let (_, passed_bal, disc_bal) = ReconciliationEngine::check_balance_invariants(&vcb_txs);
    assert!(
        !passed_bal,
        "check_balance_invariants must FAIL when balance_after is perturbed by +1 VND"
    );
    assert_ne!(disc_bal, 0);
    vcb_txs[10_000].balance_after = original_bal; // Restore

    // --- Perturbation 2.9: First transaction balance_after perturbed by -1 VND ---
    let orig_first_bal = vcb_txs[0].balance_after;
    vcb_txs[0].balance_after = orig_first_bal.map(|b| b - 1);
    let (_, passed_first, disc_first) = ReconciliationEngine::check_balance_invariants(&vcb_txs);
    assert!(
        !passed_first,
        "check_balance_invariants must FAIL when first balance_after is perturbed by -1 VND"
    );
    assert_ne!(disc_first, 0);
    vcb_txs[0].balance_after = orig_first_bal; // Restore

    // --- Perturbation 2.10: Invert transaction direction (Credit -> Debit) ---
    let orig_type = vcb_txs[500].tx_type;
    vcb_txs[500].tx_type = match orig_type {
        TransactionType::Credit => TransactionType::Debit,
        TransactionType::Debit => TransactionType::Credit,
    };
    let (_, passed_type, disc_type) = ReconciliationEngine::check_balance_invariants(&vcb_txs);
    assert!(
        !passed_type,
        "check_balance_invariants must FAIL when transaction direction is swapped"
    );
    assert_ne!(disc_type, 0);
    vcb_txs[500].tx_type = orig_type; // Restore

    // --- Perturbation 2.11: Swap two adjacent transactions with different amounts ---
    if vcb_txs[100].amount != vcb_txs[101].amount {
        vcb_txs.swap(100, 101);
        let (_, _passed_swap, _) = ReconciliationEngine::check_balance_invariants(&vcb_txs);
        // Because check_balance_invariants sorts by tx_date, if dates differ it will sort them back,
        // so let's swap their timestamps to simulate out-of-order sequence:
        let t100 = vcb_txs[100].tx_date;
        let t101 = vcb_txs[101].tx_date;
        vcb_txs[100].tx_date = t101;
        vcb_txs[101].tx_date = t100;
        let (_, passed_order, disc_order) =
            ReconciliationEngine::check_balance_invariants(&vcb_txs);
        assert!(
            !passed_order,
            "check_balance_invariants must FAIL when transaction sequence violates running balances"
        );
        assert_ne!(disc_order, 0);
    }
}

// ===========================================================================
// 3. Edge Case Amounts & Extreme Volumes (100 Billion VND Treasury)
// ===========================================================================

#[test]
fn test_challenger1_edge_case_treasury_amounts_and_overflow_protection() {
    let base_time = 1_785_542_400i64;

    // Extreme Treasury Opening: 100 Billion VND (100_000_000_000)
    let opening_treasury: u64 = 100_000_000_000;

    // Massive transactions up to 10 Trillion VND
    let massive_txs = vec![
        TransactionRecord::new(
            1,
            base_time,
            base_time,
            Some("HD-CORP-01".to_string()),
            TransactionType::Credit,
            50_000_000_000, // +50B
            Some(150_000_000_000),
            None,
            Some("CONG TY DIEN LUC".to_string()),
            Some("VCB".to_string()),
            "Thanh toan tien dien toan quoc".to_string(),
        ),
        TransactionRecord::new(
            2,
            base_time + 60,
            base_time + 60,
            Some("HD-CORP-02".to_string()),
            TransactionType::Debit,
            30_000_000_000, // -30B
            Some(120_000_000_000),
            None,
            Some("KHO BAC NHA NUOC".to_string()),
            Some("VCB".to_string()),
            "Nop thue thu nhap doanh nghiep".to_string(),
        ),
        TransactionRecord::new(
            3,
            base_time + 120,
            base_time + 120,
            Some("HD-CORP-03".to_string()),
            TransactionType::Credit,
            880_000_000_000, // +880B (reaching 1 Trillion VND total)
            Some(1_000_000_000_000),
            None,
            Some("TAP DOAN DAU KHI".to_string()),
            Some("VCB".to_string()),
            "Giai ngan tin dung hop vo".to_string(),
        ),
        TransactionRecord::new(
            4,
            base_time + 180,
            base_time + 180,
            Some("HD-CORP-04".to_string()),
            TransactionType::Debit,
            900_000_000_000, // -900B
            Some(100_000_000_000),
            None,
            Some("NGAN HANG TRUNG UONG".to_string()),
            Some("VCB".to_string()),
            "Dao han thi truong mo OMO".to_string(),
        ),
    ];

    let expected_credit: u64 = 50_000_000_000 + 880_000_000_000;
    let expected_debit: u64 = 30_000_000_000 + 900_000_000_000;
    let expected_closing: u64 = opening_treasury + expected_credit - expected_debit;
    assert_eq!(expected_closing, 100_000_000_000);

    let treasury_stmt = BankStatement::new(
        "VCB_TREASURY".to_string(),
        BankType::Vietcombank,
        StatementFormat::Csv,
        Some("007100_TREASURY_MASTER".to_string()),
        Some("LIVA TREASURY MASTER ACCOUNT".to_string()),
        Some(opening_treasury),
        Some(expected_closing),
        Some(base_time),
        Some(base_time + 180),
        massive_txs,
        0,
    );

    let inv = treasury_stmt.verify_balance_invariants();
    assert!(
        inv.is_valid,
        "Treasury 100B VND statement invariant must be valid"
    );
    assert_eq!(inv.discrepancy, 0, "Discrepancy must be strictly 0 VND");
    assert_eq!(inv.total_credit, 930_000_000_000);
    assert_eq!(inv.total_debit, 930_000_000_000);
    assert_eq!(inv.closing_balance, 100_000_000_000);

    // Test extreme boundary values approaching u64::MAX
    // Opening balance = 10^18 VND (~1 Quintillion VND)
    let ultra_high_opening: u64 = 1_000_000_000_000_000_000;
    let ultra_credit: u64 = 500_000_000_000_000_000;
    let ultra_debit: u64 = 200_000_000_000_000_000;
    let ultra_closing: u64 = ultra_high_opening + ultra_credit - ultra_debit;

    let ultra_stmt = BankStatement::new(
        "ULTRA_HIGH".to_string(),
        BankType::Vietcombank,
        StatementFormat::Csv,
        Some("ULTRA_ACC".to_string()),
        Some("ULTRA TREASURY".to_string()),
        Some(ultra_high_opening),
        Some(ultra_closing),
        Some(base_time),
        Some(base_time + 60),
        vec![
            TransactionRecord::new(
                1,
                base_time,
                base_time,
                None,
                TransactionType::Credit,
                ultra_credit,
                Some(ultra_high_opening + ultra_credit),
                None,
                None,
                None,
                "".to_string(),
            ),
            TransactionRecord::new(
                2,
                base_time + 60,
                base_time + 60,
                None,
                TransactionType::Debit,
                ultra_debit,
                Some(ultra_closing),
                None,
                None,
                None,
                "".to_string(),
            ),
        ],
        0,
    );

    let ultra_inv = ultra_stmt.verify_balance_invariants();
    assert!(
        ultra_inv.is_valid,
        "Ultra-high treasury invariant must remain valid without integer overflow"
    );
    assert_eq!(ultra_inv.discrepancy, 0);
}

// ===========================================================================
// 4. CSV Statement Export & Re-parsing Invariant Round-Trip
// ===========================================================================

#[test]
fn test_challenger1_statement_csv_reingestion_invariant_preservation() {
    let dataset = generate_50k_dataset(DEFAULT_BENCHMARK_SEED);

    for stmt in &dataset.statements {
        let csv_text = format_statement_as_csv(stmt);
        let filename = match stmt.bank_type {
            BankType::Techcombank => "tcb.csv",
            BankType::VietinBank => "ctg.csv",
            BankType::MbBank => "mb.csv",
            BankType::Vietcombank => "vcb.csv",
            BankType::Bidv => "bidv.csv",
            _ => "bank.csv",
        };

        let parsed = sniff_and_parse(csv_text.as_bytes(), filename)
            .unwrap_or_else(|e| panic!("Failed to parse CSV for bank {}: {e}", stmt.bank_code));

        // Ensure parser computed balance invariants on the re-ingested data
        assert!(
            parsed.balance_checksum_passed,
            "Re-parsed statement for {} must pass balance checksum",
            stmt.bank_code
        );

        let report = verify_balance_invariants(&parsed);
        assert!(
            report.is_valid,
            "Re-parsed statement invariant report for {} must be valid",
            stmt.bank_code
        );
        assert_eq!(
            report.discrepancy, 0,
            "Re-parsed statement discrepancy for {} must be strictly 0 VND",
            stmt.bank_code
        );
    }
}

// ===========================================================================
// 5. Anti-Cheating & Integrity Audit (Unmatched Initial State)
// ===========================================================================

#[test]
fn test_challenger1_integrity_unmatched_initial_state() {
    let dataset = generate_50k_dataset(DEFAULT_BENCHMARK_SEED);

    // 1. Bank transactions must be pristine (Unmatched, no match ID)
    for (idx, tx) in dataset.bank_transactions.iter().enumerate() {
        assert_eq!(
            tx.reconciled_status,
            ReconciliationStatus::Unmatched,
            "Bank transaction at index {idx} leaked matched status"
        );
        assert!(
            tx.reconciled_match_id.is_none(),
            "Bank transaction at index {idx} leaked match ID"
        );
    }

    // 2. Ledger entries must be pristine
    for (idx, led) in dataset.ledger_entries.iter().enumerate() {
        assert_eq!(
            led.reconciled_status,
            ReconciliationStatus::Unmatched,
            "Ledger entry at index {idx} leaked matched status"
        );
    }

    // 3. Document numbers and partner codes must be well-formed
    let mut unique_ledger_ids = HashSet::with_capacity(50_000);
    for led in &dataset.ledger_entries {
        assert!(
            unique_ledger_ids.insert(led.id.clone()),
            "Duplicate ledger entry ID detected: {}",
            led.id
        );
    }
    assert_eq!(unique_ledger_ids.len(), 50_000);

    let mut unique_bank_ids = HashSet::with_capacity(50_000);
    for tx in &dataset.bank_transactions {
        assert!(
            unique_bank_ids.insert(tx.id.clone()),
            "Duplicate bank transaction ID detected: {}",
            tx.id
        );
    }
    assert_eq!(unique_bank_ids.len(), 50_000);
}
