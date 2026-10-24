//! Performance Benchmarks and Resource Guardrails for LIVA Banking Local AI.
//!
//! Benchmarks & Assertions:
//! 1. P90 Latency: ONNX Embedding & Multi-Dimensional Anomaly Detection < 50ms.
//! 2. Time-To-First-Token (TTFT) / Qualitative Synthesis Latency: Local SLM < 500ms.
//! 3. Memory Guardrail: Process physical RAM consumption (RSS) <= 4GB under batch workload.

use liva_native_core::banking::mcp::{
    ComplianceAmlScreenArgs, CreditRiskScoringArgs, ScreeningTxInput,
    handle_compliance_aml_screen, handle_credit_risk_scoring,
};
use liva_native_core::sysinfo::process_memory_bytes;
use std::time::{Duration, Instant};

fn calculate_percentile(durations: &mut [Duration], percentile: f64) -> Duration {
    assert!(!durations.is_empty(), "Durations slice must not be empty");
    durations.sort();
    let rank = ((durations.len() as f64) * (percentile / 100.0)).ceil() as usize;
    let idx = if rank == 0 { 0 } else { rank - 1 };
    durations[idx.min(durations.len() - 1)]
}

// ---------------------------------------------------------------------------
// 1. P90 Latency Benchmark: Multi-Dimensional Anomaly Detection < 50ms
// ---------------------------------------------------------------------------

#[test]
fn test_benchmark_onnx_anomaly_detection_latency_p90() {
    let t_day = 1789527600i64;
    let t_night = 1789498800i64;

    // Batch of 10 transactions spanning normal, near-threshold smurfing, off-hours, and semantic keywords
    let test_batch = vec![
        ScreeningTxInput {
            tx_id: "BENCH-01".to_string(),
            account_number: Some("00110001".to_string()),
            counterparty_name: Some("Cong ty TNHH Nong San Dalat".to_string()),
            counterparty_account: Some("CP-001".to_string()),
            amount_vnd: 45_000_000,
            timestamp: Some(t_day),
            narration: "Thanh toan tien rau cu thang 9".to_string(),
            is_credit: Some(true),
        },
        ScreeningTxInput {
            tx_id: "BENCH-02".to_string(),
            account_number: Some("00110002".to_string()),
            counterparty_name: Some("Nguyen Van A".to_string()),
            counterparty_account: Some("CP-002".to_string()),
            amount_vnd: 388_000_000, // Near-threshold smurfing
            timestamp: Some(t_day + 100),
            narration: "Chuyen tien hop dong vat tu sdt 0912345678 CCCD 001095012345".to_string(),
            is_credit: Some(true),
        },
        ScreeningTxInput {
            tx_id: "BENCH-03".to_string(),
            account_number: Some("00110003".to_string()),
            counterparty_name: Some("Doi Tac Dem Khuya".to_string()),
            counterparty_account: Some("CP-003".to_string()),
            amount_vnd: 250_000_000, // Night off-hours anomaly
            timestamp: Some(t_night),
            narration: "Chuyen khoan dem".to_string(),
            is_credit: Some(true),
        },
        ScreeningTxInput {
            tx_id: "BENCH-04".to_string(),
            account_number: Some("00110004".to_string()),
            counterparty_name: Some("Le Van B".to_string()),
            counterparty_account: Some("CP-004".to_string()),
            amount_vnd: 60_000_000,
            timestamp: Some(t_day + 200),
            narration: "Chuyen tien mua ban tien ao usdt bitcoin dao rut rua tien".to_string(), // Semantic anomaly
            is_credit: Some(true),
        },
        ScreeningTxInput {
            tx_id: "BENCH-05".to_string(),
            account_number: Some("00110005".to_string()),
            counterparty_name: Some("CTY CP LOGISTICS VIET NAM".to_string()),
            counterparty_account: Some("CP-005".to_string()),
            amount_vnd: 120_000_000,
            timestamp: Some(t_day + 300),
            narration: "Cuoc van chuyen hang hoa thang 9".to_string(),
            is_credit: Some(false),
        },
    ];

    // Warm-up: 5 iterations
    for _ in 0..5 {
        let args = ComplianceAmlScreenArgs {
            transactions: test_batch.clone(),
            redact_pii: Some(true),
        };
        let _ = handle_compliance_aml_screen(args).expect("Warmup screening");
    }

    // Benchmark: 50 measured iterations
    const ITERATIONS: usize = 50;
    let mut durations = Vec::with_capacity(ITERATIONS);

    for _ in 0..ITERATIONS {
        let args = ComplianceAmlScreenArgs {
            transactions: test_batch.clone(),
            redact_pii: Some(true),
        };

        let start = Instant::now();
        let result = handle_compliance_aml_screen(args).expect("Measured screening");
        let elapsed = start.elapsed();

        assert_eq!(result.total_screened, test_batch.len());
        assert!(result.zero_egress_verified);
        durations.push(elapsed);
    }

    let p50 = calculate_percentile(&mut durations, 50.0);
    let p90 = calculate_percentile(&mut durations, 90.0);
    let p99 = calculate_percentile(&mut durations, 99.0);
    let min = durations.iter().min().copied().unwrap();
    let max = durations.iter().max().copied().unwrap();

    println!("\n========================================================");
    println!("--- LOCAL AI ANOMALY DETECTION LATENCY BENCHMARK ---");
    println!("Iterations:        {}", ITERATIONS);
    println!("Batch Size:        {} transactions", test_batch.len());
    println!("Min Latency:       {:?}", min);
    println!("P50 (Median):      {:?}", p50);
    println!("P90 Latency:       {:?}", p90);
    println!("P99 Latency:       {:?}", p99);
    println!("Max Latency:       {:?}", max);
    println!("========================================================\n");

    // Strict SLA Requirement: P90 latency < 50ms
    assert!(
        p90 < Duration::from_millis(50),
        "P90 Latency ({:?}) exceeded SLA limit of 50ms",
        p90
    );
}

// ---------------------------------------------------------------------------
// 2. Local SLM TTFT / Qualitative Synthesis Latency < 500ms
// ---------------------------------------------------------------------------

#[test]
fn test_benchmark_slm_ttft_or_qualitative_synthesis_latency() {
    let args = CreditRiskScoringArgs {
        ebitda_vnd: Some(2_000_000_000),
        capex_vnd: Some(300_000_000),
        debt_service_principal_vnd: Some(500_000_000),
        debt_service_interest_vnd: Some(150_000_000),
        cash_and_equivalents_vnd: Some(600_000_000),
        marketable_securities_vnd: Some(150_000_000),
        accounts_receivable_vnd: Some(400_000_000),
        current_liabilities_vnd: Some(700_000_000),
        historical_cashflow: None,
        forecast_days: Some(30),
    };

    // Warm-up: 5 iterations
    for _ in 0..5 {
        let _ = handle_credit_risk_scoring(args.clone()).expect("Warmup scoring");
    }

    // Benchmark: 40 measured iterations
    const ITERATIONS: usize = 40;
    let mut durations = Vec::with_capacity(ITERATIONS);

    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let res = handle_credit_risk_scoring(args.clone()).expect("Measured scoring");
        let elapsed = start.elapsed();

        assert!(!res.qualitative_risk_summary.is_empty());
        durations.push(elapsed);
    }

    let p50 = calculate_percentile(&mut durations, 50.0);
    let p90 = calculate_percentile(&mut durations, 90.0);
    let max = durations.iter().max().copied().unwrap();

    println!("\n========================================================");
    println!("--- LOCAL SLM QUALITATIVE SYNTHESIS LATENCY BENCHMARK ---");
    println!("Iterations:        {}", ITERATIONS);
    println!("P50 (Median):      {:?}", p50);
    println!("P90 Latency:       {:?}", p90);
    println!("Max Latency:       {:?}", max);
    println!("========================================================\n");

    // Strict SLA Requirement: TTFT / qualitative synthesis latency < 500ms
    assert!(
        p90 < Duration::from_millis(500),
        "P90 Synthesis Latency ({:?}) exceeded SLA limit of 500ms",
        p90
    );
}

// ---------------------------------------------------------------------------
// 3. Memory Guardrail: Process Physical RAM Consumption <= 4GB
// ---------------------------------------------------------------------------

#[test]
fn test_benchmark_memory_guardrail_process_ram_under_4gb() {
    const FOUR_GIGABYTES: u64 = 4 * 1024 * 1024 * 1024;

    // Run an intensive batch workload:
    // Screen 100 transactions and execute 50 risk calculations in memory
    let mut batch = Vec::with_capacity(100);
    for i in 0..100 {
        batch.push(ScreeningTxInput {
            tx_id: format!("MEM-TX-{i}"),
            account_number: Some(format!("ACC-{:04}", i % 10)),
            counterparty_name: Some(format!("Counterparty {:04}", i)),
            counterparty_account: Some(format!("CP-{:04}", i)),
            amount_vnd: 350_000_000 + ((i as u64 * 1_000_000) % 60_000_000),
            timestamp: Some(1789527600 + (i as i64 * 30)),
            narration: format!("Thanh toan batch memory test giao dich {i}"),
            is_credit: Some(i % 2 == 0),
        });
    }

    let screen_args = ComplianceAmlScreenArgs {
        transactions: batch,
        redact_pii: Some(true),
    };
    let _ = handle_compliance_aml_screen(screen_args).expect("Screening batch");

    for i in 0..50 {
        let risk_args = CreditRiskScoringArgs {
            ebitda_vnd: Some(2_000_000_000 + (i as i64 * 10_000_000)),
            capex_vnd: Some(300_000_000),
            debt_service_principal_vnd: Some(500_000_000),
            debt_service_interest_vnd: Some(150_000_000),
            cash_and_equivalents_vnd: Some(600_000_000),
            marketable_securities_vnd: Some(150_000_000),
            accounts_receivable_vnd: Some(400_000_000),
            current_liabilities_vnd: Some(700_000_000),
            historical_cashflow: None,
            forecast_days: Some(30),
        };
        let _ = handle_credit_risk_scoring(risk_args).expect("Risk scoring");
    }

    // Inspect process memory via sysinfo
    if let Some((working_set, commit_charge)) = process_memory_bytes() {
        let working_set_mb = working_set / (1024 * 1024);
        let commit_charge_mb = commit_charge / (1024 * 1024);

        println!("\n========================================================");
        println!("--- PROCESS MEMORY GUARDRAIL INSPECTION ---");
        println!("Working Set (RSS):    {} MB ({} bytes)", working_set_mb, working_set);
        println!("Commit Charge:        {} MB ({} bytes)", commit_charge_mb, commit_charge);
        println!("Hard Ceiling Limit:   4096 MB ({} bytes)", FOUR_GIGABYTES);
        println!("========================================================\n");

        // Assert physical RAM consumption does not exceed 4GB
        assert!(
            working_set <= FOUR_GIGABYTES,
            "Process Working Set ({} MB) exceeded 4GB RAM guardrail!",
            working_set_mb
        );
    } else {
        println!("Notice: process_memory_bytes() is available on Windows platforms.");
    }
}
