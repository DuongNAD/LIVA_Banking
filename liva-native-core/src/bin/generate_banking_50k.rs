//! Standalone 50,000-Transaction Synthetic Banking Generator CLI.
//!
//! Generates 50,000 realistic Vietnamese bank transactions and 50,000 internal ledger entries
//! with controlled match rates across Tier 1, Tier 2, Tier 3, and Fail-Closed HITL queues.
//!
//! Usage:
//! ```bash
//! cargo run -p liva-native-core --bin generate_banking_50k -j 2
//! cargo run -p liva-native-core --bin generate_banking_50k -j 2 -- --out-dir fixtures/synthetic_50k
//! ```

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use liva_native_core::banking::generator::{DEFAULT_BENCHMARK_SEED, fmt_vnd, generate_50k_dataset};

fn print_usage() {
    println!("LIVA Banking 50k Synthetic Data Generator");
    println!("Usage: generate_banking_50k [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --out-dir <PATH>   Export generated statement CSVs and ledger JSON to directory");
    println!("  --seed <U64>       Custom PRNG seed (default: 0x50_000_2026 / 343597392422)");
    println!("  --help, -h         Show this help message");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut out_dir: Option<PathBuf> = None;
    let mut seed: u64 = DEFAULT_BENCHMARK_SEED;

    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            "--out-dir" => {
                if idx + 1 < args.len() {
                    out_dir = Some(PathBuf::from(&args[idx + 1]));
                    idx += 2;
                } else {
                    eprintln!("Error: --out-dir requires a directory path");
                    std::process::exit(1);
                }
            }
            "--seed" => {
                if idx + 1 < args.len() {
                    seed = args[idx + 1]
                        .parse::<u64>()
                        .unwrap_or(DEFAULT_BENCHMARK_SEED);
                    idx += 2;
                } else {
                    eprintln!("Error: --seed requires a 64-bit unsigned integer");
                    std::process::exit(1);
                }
            }
            unknown => {
                eprintln!("Unknown argument: {unknown}");
                print_usage();
                std::process::exit(1);
            }
        }
    }

    println!("================================================================================");
    println!("         LIVA BANKING RECONCILIATION — 50,000 DATASET GENERATOR                 ");
    println!("================================================================================");
    println!("  PRNG Seed                     : 0x{:X} ({})", seed, seed);
    println!("  Target Total Bank Txs         : 50,000");
    println!("  Target Total Ledger Entries   : 50,000");
    println!("  Target Corporate Registry     : ~200 Vietnamese Entities");
    println!("--------------------------------------------------------------------------------");

    let start = Instant::now();
    let dataset = generate_50k_dataset(seed);
    let gen_duration = start.elapsed();

    println!(
        "  Generation Status             : SUCCESS in {:.3} seconds",
        gen_duration.as_secs_f64()
    );
    println!();
    println!("  +--------------------+----------------+---------------+");
    println!("  | Bank Partner       | Account Number | Transaction   |");
    println!("  +--------------------+----------------+---------------+");
    for stmt in &dataset.statements {
        let acc = stmt.account_number.as_deref().unwrap_or("N/A");
        println!(
            "  | {:<18} | {:<14} | {:>10} tx |",
            stmt.bank_code,
            acc,
            stmt.transactions.len()
        );
    }
    println!("  +--------------------+----------------+---------------+");
    println!(
        "  | TOTAL BANK TXS     |                | {:>10} tx |",
        dataset.bank_transactions.len()
    );
    println!("  +--------------------+----------------+---------------+");
    println!();

    println!("  Distribution Breakdown across Reconciliation Tiers:");
    println!("  ------------------------------------------------------------------");
    let total_tx = dataset.bank_transactions.len() as f64;
    println!(
        "  * Tier 1 (Exact 1:1 Hash)   : {:>6} tx ({:>5.1}%) [Target: 90.0%]",
        dataset.metadata.tier1_exact_count,
        (dataset.metadata.tier1_exact_count as f64 / total_tx) * 100.0
    );
    println!(
        "  * Tier 2 (Fuzzy Heuristic)  : {:>6} tx ({:>5.1}%) [Target:  7.5%]",
        dataset.metadata.tier2_fuzzy_count,
        (dataset.metadata.tier2_fuzzy_count as f64 / total_tx) * 100.0
    );
    println!(
        "  * Tier 3 (Composite Split)  : {:>6} tx ({:>5.1}%) [Target:  2.3%]",
        dataset.metadata.tier3_split_count,
        (dataset.metadata.tier3_split_count as f64 / total_tx) * 100.0
    );
    println!(
        "  * HITL Queue (Fail-Closed)  : {:>6} tx ({:>5.1}%) [Target:  0.2%]",
        dataset.metadata.hitl_queue_count,
        (dataset.metadata.hitl_queue_count as f64 / total_tx) * 100.0
    );
    println!("  ------------------------------------------------------------------");
    println!(
        "  Total Internal Ledger Entries : {}",
        dataset.ledger_entries.len()
    );
    println!();

    println!("  Verifying Double-Entry Balance Invariants:");
    let mut all_invariants_ok = true;
    for stmt in &dataset.statements {
        let inv = stmt.verify_balance_invariants();
        let status = if inv.is_valid { "PASS" } else { "FAIL" };
        if !inv.is_valid {
            all_invariants_ok = false;
        }
        println!(
            "    [{}] {}: Opening={} VND, Closing={} VND, Delta={} VND",
            status,
            stmt.bank_code,
            fmt_vnd(inv.opening_balance),
            fmt_vnd(inv.closing_balance),
            inv.discrepancy
        );
    }

    if !all_invariants_ok {
        eprintln!("FATAL: Double-entry balance invariant check failed!");
        std::process::exit(1);
    }

    if let Some(ref dir) = out_dir {
        println!();
        println!(
            "  Exporting statement CSV files and invoices to: {}",
            dir.display()
        );
        let export_start = Instant::now();
        dataset.export_to_directory(dir)?;
        println!(
            "  Export completed in {:.3} seconds.",
            export_start.elapsed().as_secs_f64()
        );
    }

    println!();
    println!("================================================================================");
    println!("  STATUS: 50,000 DATASET GENERATION VERIFIED & READY FOR BENCHMARK              ");
    println!("================================================================================");

    Ok(())
}
