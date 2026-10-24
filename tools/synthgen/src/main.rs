//! LIVA Banking — Synthetic Data Generator (`synthgen`)
//!
//! Deterministic generator for Vietnamese banking transactions and internal ledger entries.
//! Enforces exact mathematical balance invariants Closing = Opening + Credit - Debit.

#![deny(clippy::float_arithmetic)]

use liva_ledger::{PostingType, Ledger};
use liva_money::Money;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub const DEFAULT_BENCHMARK_SEED: u64 = 0x0005_0000_2026;

/// Realistic company in Vietnamese commerce.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VietnameseCompany {
    pub partner_code: String,
    pub name: String,
    pub short_name: String,
    pub tax_code: String,
    pub contract_no: String,
    pub sector: String,
}

pub fn build_corporate_registry() -> Vec<VietnameseCompany> {
    let legal_forms = [
        "CÔNG TY CỔ PHẦN",
        "CÔNG TY TNHH",
        "CÔNG TY TNHH MTV",
        "TẬP ĐOÀN",
        "TỔNG CÔNG TY",
    ];

    let sectors = [
        ("THƯƠNG MẠI VÀ XUẤT NHẬP KHẨU", "TM & XNK"),
        ("CÔNG NGHỆ VÀ TRUYỀN THÔNG SỐ", "CN & TT"),
        ("XÂY DỰNG VÀ BẤT ĐỘNG SẢN", "XD & BĐS"),
        ("DƯỢC PHẨM VÀ THIẾT BỊ Y TẾ", "DƯỢC & Y TẾ"),
        ("TIẾP VẬN VÀ KHO VẬN LOGISTICS", "LOGISTICS"),
        ("NÔNG SẢN VÀ THỰC PHẨM CHẾ BIẾN", "NÔNG SẢN"),
        ("CƠ KHÍ CHẾ TẠO VÀ TỰ ĐỘNG HÓA", "CƠ KHÍ"),
        ("NĂNG LƯỢNG VÀ VẬT LIỆU MỚI", "NĂNG LƯỢNG"),
        ("DỊCH VỤ DU LỊCH VÀ KHÁCH SẠN", "DU LỊCH"),
        ("TÀI CHÍNH VÀ ĐẦU TƯ PHÁT TRIỂN", "ĐẦU TƯ"),
    ];

    let brands = [
        "MINH LONG", "AN PHÁT", "ĐẠI NAM", "HOÀNG KIM", "ĐÔNG ĐÔ",
        "BẮC Á", "SÀI GÒN", "HÀ NỘI", "THĂNG LONG", "ÂU LẠC",
        "VIỆT Á", "TÂM PHÁT", "PHÚC THỊNH", "HƯNG THỊNH", "TOÀN CẦU",
        "BẢO AN", "HÒA BÌNH", "VĨNH PHÁT", "TÂN THỊNH", "NAM VIỆT",
    ];

    let mut registry = Vec::with_capacity(200);
    for k in 0..200 {
        let sector_idx = k / 20;
        let brand_idx = k % 20;
        let legal_idx = (k / 40) % legal_forms.len();

        let legal = legal_forms[legal_idx];
        let (sector_name, sector_short) = sectors[sector_idx];
        let brand = brands[brand_idx];

        let partner_code = format!("KH{:03}", k + 1);
        let tax_code = format!("01{:08}", 10000000 + k * 137);
        let contract_no = format!("HD-2026/{:03}", k + 1);
        let name = format!("{legal} {sector_name} {brand}");
        let short_name = format!("{sector_short} {brand}");

        registry.push(VietnameseCompany {
            partner_code,
            name,
            short_name,
            tax_code,
            contract_no,
            sector: sector_name.to_string(),
        });
    }
    registry
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticTx {
    pub row_id: usize,
    pub bank_code: String,
    pub ref_code: String,
    pub tx_type: String, // "CREDIT" or "DEBIT"
    pub amount_vnd: i64,
    pub balance_after_vnd: i64,
    pub counterparty_name: String,
    pub narration: String,
    pub invoice_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticDataset {
    pub seed: u64,
    pub total_transactions: usize,
    pub bank_summary: HashMap<String, usize>,
    pub opening_balance_vnd: i64,
    pub closing_balance_vnd: i64,
    pub total_credit_vnd: i64,
    pub total_debit_vnd: i64,
    pub balance_invariant_verified: bool,
    pub transactions: Vec<SyntheticTx>,
}

pub fn generate_dataset(total_count: usize, seed: u64, target_bank: &str) -> SyntheticDataset {
    let mut rng = fastrand::Rng::with_seed(seed);
    let registry = build_corporate_registry();

    let supported_banks = ["VCB", "TCB", "BIDV", "CTG", "MB", "VBA"];
    let mut bank_summary = HashMap::new();

    let opening_balance_vnd: i64 = 1_000_000_000; // 1 tỷ VND
    let mut running_balance = opening_balance_vnd;
    let mut total_credit: i64 = 0;
    let mut total_debit: i64 = 0;

    let mut transactions = Vec::with_capacity(total_count);

    for idx in 1..=total_count {
        let bank_code = if target_bank == "ALL" || target_bank.is_empty() {
            supported_banks[rng.usize(0..supported_banks.len())].to_string()
        } else {
            target_bank.to_uppercase()
        };

        *bank_summary.entry(bank_code.clone()).or_insert(0) += 1;

        let company = &registry[rng.usize(0..registry.len())];
        let is_credit = rng.bool(); // 50% inflow, 50% outflow
        let raw_amount = (rng.i64(50..50_000) * 10_000).max(100_000); // 100k to 500M

        let (amount, tx_type) = if is_credit {
            total_credit += raw_amount;
            running_balance += raw_amount;
            (raw_amount, "CREDIT")
        } else {
            // Guard against negative balance
            let debit_amt = if running_balance > raw_amount {
                raw_amount
            } else {
                running_balance / 2
            }.max(10_000);
            total_debit += debit_amt;
            running_balance -= debit_amt;
            (debit_amt, "DEBIT")
        };

        let ref_code = match bank_code.as_str() {
            "TCB" => format!("FT26{:08}", rng.u32(10000000..99999999)),
            "VCB" => format!("VCB{:08}", rng.u32(10000000..99999999)),
            "BIDV" => format!("BIDV{:08}", rng.u32(10000000..99999999)),
            "CTG" => format!("CTG{:08}", rng.u32(10000000..99999999)),
            "MB" => format!("MB{:08}", rng.u32(10000000..99999999)),
            _ => format!("NPS{:08}", rng.u32(10000000..99999999)),
        };

        let invoice_ref = format!("INV-2026-{:05}", rng.u32(1000..99999));
        let narration = if is_credit {
            format!("Napas VietQR TT HD {} TU: {}", invoice_ref, company.short_name)
        } else {
            format!("Thanh toan tien hang hop dong {} CHO: {}", company.contract_no, company.short_name)
        };

        transactions.push(SyntheticTx {
            row_id: idx,
            bank_code,
            ref_code,
            tx_type: tx_type.to_string(),
            amount_vnd: amount,
            balance_after_vnd: running_balance,
            counterparty_name: company.name.clone(),
            narration,
            invoice_ref: Some(invoice_ref),
        });
    }

    // Verify mathematical balance invariant using liva-ledger
    let opening_m = Money::vnd(opening_balance_vnd);
    let closing_m = Money::vnd(running_balance);
    let ledger_txs: Vec<(PostingType, Money)> = transactions
        .iter()
        .map(|t| {
            let p_type = if t.tx_type == "CREDIT" {
                PostingType::Credit
            } else {
                PostingType::Debit
            };
            (p_type, Money::vnd(t.amount_vnd))
        })
        .collect();

    let balance_invariant_verified = Ledger::verify_statement_balance(opening_m, closing_m, &ledger_txs).unwrap_or(false);

    SyntheticDataset {
        seed,
        total_transactions: total_count,
        bank_summary,
        opening_balance_vnd,
        closing_balance_vnd: running_balance,
        total_credit_vnd: total_credit,
        total_debit_vnd: total_debit,
        balance_invariant_verified,
        transactions,
    }
}

pub fn export_csv(dataset: &SyntheticDataset, path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    // Write UTF-8 BOM
    file.write_all(&[0xEF, 0xBB, 0xBF])?;
    writeln!(file, "STT,NganHang,MaGiaoDich,LoaiGiaoDich,SoTien,SoDuSau,DoiTac,NoiDung,MaHoaDon")?;

    for tx in &dataset.transactions {
        writeln!(
            file,
            "{},{},{},{},{},{},\"{}\",\"{}\",\"{}\"",
            tx.row_id,
            tx.bank_code,
            tx.ref_code,
            tx.tx_type,
            tx.amount_vnd,
            tx.balance_after_vnd,
            tx.counterparty_name,
            tx.narration,
            tx.invoice_ref.as_deref().unwrap_or("")
        )?;
    }
    Ok(())
}

fn print_usage() {
    println!("LIVA Banking Synthetic Generator (synthgen) v0.1.0");
    println!("Usage:");
    println!("  synthgen [--count <N>] [--bank <VCB|TCB|BIDV|CTG|MB|VBA|ALL>] [--format <json|csv>] [--out <PATH>] [--seed <U64>]");
    println!("Default:");
    println!("  --count 1000 --bank ALL --format json --out synthetic_statements.json --seed 3276802026");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut count = 1000usize;
    let mut bank = "ALL".to_string();
    let mut format = "json".to_string();
    let mut out_path = "synthetic_statements.json".to_string();
    let mut seed = DEFAULT_BENCHMARK_SEED;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--count" => {
                if i + 1 < args.len() {
                    count = args[i + 1].parse().unwrap_or(1000);
                    i += 1;
                }
            }
            "--bank" => {
                if i + 1 < args.len() {
                    bank = args[i + 1].to_uppercase();
                    i += 1;
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    format = args[i + 1].to_lowercase();
                    i += 1;
                }
            }
            "--out" => {
                if i + 1 < args.len() {
                    out_path = args[i + 1].clone();
                    i += 1;
                }
            }
            "--seed" => {
                if i + 1 < args.len() {
                    seed = args[i + 1].parse().unwrap_or(DEFAULT_BENCHMARK_SEED);
                    i += 1;
                }
            }
            "--help" | "-h" => {
                print_usage();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    println!("[synthgen] Generating {} synthetic transactions (Bank: {}, Seed: {})...", count, bank, seed);
    let dataset = generate_dataset(count, seed, &bank);

    println!("[synthgen] Balance Invariant Verified: {}", dataset.balance_invariant_verified);
    println!("[synthgen] Opening: {} VND | Closing: {} VND", dataset.opening_balance_vnd, dataset.closing_balance_vnd);
    println!("[synthgen] Total Credit: {} VND | Total Debit: {} VND", dataset.total_credit_vnd, dataset.total_debit_vnd);

    if format == "csv" {
        export_csv(&dataset, Path::new(&out_path)).expect("Failed to export CSV");
        println!("[synthgen] Exported CSV to: {}", out_path);
    } else {
        let json = serde_json::to_string_pretty(&dataset).expect("Failed to serialize JSON");
        std::fs::write(&out_path, json).expect("Failed to write JSON");
        println!("[synthgen] Exported JSON to: {}", out_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthetic_dataset_balance_invariant() {
        let dataset = generate_dataset(500, 0x12345, "ALL");
        assert_eq!(dataset.total_transactions, 500);
        assert!(dataset.balance_invariant_verified);
        assert_eq!(
            dataset.closing_balance_vnd,
            dataset.opening_balance_vnd + dataset.total_credit_vnd - dataset.total_debit_vnd
        );
    }

    #[test]
    fn test_specific_bank_generation() {
        let dataset = generate_dataset(200, 0x999, "VCB");
        for tx in &dataset.transactions {
            assert_eq!(tx.bank_code, "VCB");
            assert!(tx.ref_code.starts_with("VCB"));
        }
        assert!(dataset.balance_invariant_verified);
    }
}
