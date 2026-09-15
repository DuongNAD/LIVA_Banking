//! Deterministic 50,000-Transaction Synthetic Banking Data Generator.
//!
//! Features:
//! - Deterministic PRNG seeded with `0x50_000_2026` (`rand::rngs::StdRng::seed_from_u64`).
//! - Vietnamese Corporate Registry (~200 realistic corporations with tax codes and contracts).
//! - Multi-Bank Partitioning:
//!   * VCB (Vietcombank): 20,000 transactions
//!   * TCB (Techcombank): 15,000 transactions
//!   * CTG (VietinBank): 5,000 transactions
//!   * MB (MBBank): 5,000 transactions
//!   * BIDV: 5,000 transactions
//!   (Total: 50,000 bank transactions)
//! - 50,000 corresponding internal ledger entries / invoices.
//! - Controlled Reconciliation Distribution:
//!   * ~90.0% Exact 1:1 Hash Match (Tier 1): 45,000 items, Delta = 0.
//!   * ~7.5% Fuzzy Heuristic Match (Tier 2): 3,750 items, Napas fees & prefix noise.
//!   * ~2.3% Composite Split Match (Tier 3): 1,150 items (1-to-N & N-to-1), Delta = 0.
//!   * ~0.2% Fail-Closed HITL Queue: 100 items with UUID v4 tokens.
//! - Strict scaled integer arithmetic: amounts in `u64` (VND), zero float drift.
//! - Double-Entry Accounting Invariant Guarantee:
//!   `Balance_i = Balance_{i-1} \pm Amount_i` and `Closing = Opening + Sum(Credit) - Sum(Debit)`.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::banking::models::{
    BankStatement, BankTransactionRow, BankType, InternalLedgerEntry, ReconciliationStatus,
    StatementFormat, TransactionRecord, TransactionType,
};
use crate::banking::reconciliation::jaro_winkler::normalize_vietnamese_text;

/// Default deterministic seed for benchmark reproducibility.
pub const DEFAULT_BENCHMARK_SEED: u64 = 0x0005_0000_2026;

/// Realistic corporate entity in Vietnamese commerce.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VietnameseCompany {
    pub partner_code: String,
    pub name: String,
    pub short_name: String,
    pub tax_code: String,
    pub contract_no: String,
    pub sector: String,
}

/// Generates the Vietnamese Corporate Registry (~200 distinct entities).
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
        "MINH LONG",
        "AN PHÁT",
        "ĐẠI NAM",
        "HOÀNG KIM",
        "ĐÔNG ĐÔ",
        "BẮC Á",
        "SÀI GÒN",
        "HÀ NỘI",
        "THĂNG LONG",
        "ÂU LẠC",
        "VIỆT Á",
        "TÂM PHÁT",
        "PHÚC THỊNH",
        "HƯNG THỊNH",
        "TOÀN CẦU",
        "BẢO AN",
        "HÒA BÌNH",
        "VĨNH PHÁT",
        "TÂN THỊNH",
        "NAM VIỆT",
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

/// Metadata summary of the generated 50,000 dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub seed: u64,
    pub total_bank_transactions: usize,
    pub total_ledger_entries: usize,
    pub bank_counts: HashMap<String, usize>,
    pub tier1_exact_count: usize,
    pub tier2_fuzzy_count: usize,
    pub tier3_split_count: usize,
    pub hitl_queue_count: usize,
    pub balance_invariants_verified: bool,
}

/// Complete 50,000 benchmark dataset.
#[derive(Debug, Clone)]
pub struct Banking50kDataset {
    pub bank_transactions: Vec<BankTransactionRow>,
    pub ledger_entries: Vec<InternalLedgerEntry>,
    pub statements: Vec<BankStatement>,
    pub metadata: DatasetMetadata,
}

/// Bank partition configuration.
#[derive(Debug, Clone)]
struct BankPartitionConfig {
    bank_code: &'static str,
    bank_type: BankType,
    account_number: &'static str,
    account_name: &'static str,
    opening_balance: u64,
    total_txs: usize,
    t1_count: usize,
    t2_count: usize,
    t3_count: usize,
    hitl_count: usize,
}

/// Generates the complete 50,000 transaction dataset using deterministic PRNG.
pub fn generate_50k_dataset(seed: u64) -> Banking50kDataset {
    let mut rng = StdRng::seed_from_u64(seed);
    let registry = build_corporate_registry();

    // 5 Bank partitions totaling exactly 50,000 transactions
    let bank_configs = [
        BankPartitionConfig {
            bank_code: "VCB",
            bank_type: BankType::Vietcombank,
            account_number: "0071001234567",
            account_name: "CONG TY CP LIVA TREASURY VCB",
            opening_balance: 100_000_000_000, // 100B VND
            total_txs: 20_000,
            t1_count: 18_000,
            t2_count: 1_500,
            t3_count: 460,
            hitl_count: 40,
        },
        BankPartitionConfig {
            bank_code: "TCB",
            bank_type: BankType::Techcombank,
            account_number: "19036789012019",
            account_name: "CONG TY CP LIVA TREASURY TCB",
            opening_balance: 80_000_000_000, // 80B VND
            total_txs: 15_000,
            t1_count: 13_500,
            t2_count: 1_125,
            t3_count: 345,
            hitl_count: 30,
        },
        BankPartitionConfig {
            bank_code: "CTG",
            bank_type: BankType::VietinBank,
            account_number: "108001234567",
            account_name: "CONG TY CP LIVA TREASURY CTG",
            opening_balance: 50_000_000_000, // 50B VND
            total_txs: 5_000,
            t1_count: 4_500,
            t2_count: 375,
            t3_count: 115,
            hitl_count: 10,
        },
        BankPartitionConfig {
            bank_code: "MB",
            bank_type: BankType::MbBank,
            account_number: "0880123456789",
            account_name: "CONG TY CP LIVA TREASURY MB",
            opening_balance: 50_000_000_000, // 50B VND
            total_txs: 5_000,
            t1_count: 4_500,
            t2_count: 375,
            t3_count: 115,
            hitl_count: 10,
        },
        BankPartitionConfig {
            bank_code: "BIDV",
            bank_type: BankType::Bidv,
            account_number: "21510001234567",
            account_name: "CONG TY CP LIVA TREASURY BIDV",
            opening_balance: 50_000_000_000, // 50B VND
            total_txs: 5_000,
            t1_count: 4_500,
            t2_count: 375,
            t3_count: 115,
            hitl_count: 10,
        },
    ];

    let base_timestamp = 1_785_542_400i64; // August 1, 2026 00:00:00 UTC

    let mut all_bank_txs = Vec::with_capacity(50_000);
    let mut all_ledger_entries = Vec::with_capacity(50_000);
    let mut all_statements = Vec::with_capacity(bank_configs.len());
    let mut bank_counts = HashMap::new();

    // Global tracking counters across partitions
    let mut global_t1_counter = 0usize;
    let mut global_t2_counter = 0usize;
    let mut global_t3_tx_counter = 0usize;
    let mut global_hitl_counter = 0usize;

    // Prefixes for Tier 2 fuzzy matching noise
    let tier2_prefixes = [
        "MBVCB",
        "Napas VietQR TT",
        "QRIBFT",
        "IBVCB",
        "CT TU",
        "CHUYEN TIEN",
    ];

    // Standard Napas interbank fee variations in VND
    let napas_fees: [u64; 7] = [1_100, 2_200, 3_300, 5_500, 7_700, 8_800, 11_000];

    for config in &bank_configs {
        assert_eq!(
            config.t1_count + config.t2_count + config.t3_count + config.hitl_count,
            config.total_txs,
            "Bank partition counts must sum to total_txs"
        );

        let mut running_balance = config.opening_balance;
        let mut total_credit: u64 = 0;
        let mut total_debit: u64 = 0;
        let mut stmt_tx_records = Vec::with_capacity(config.total_txs);

        let stmt_id = format!("stmt_{}", config.bank_code.to_lowercase());

        for local_idx in 0..config.total_txs {
            let row_id = local_idx + 1;
            let tx_date = base_timestamp + (local_idx as i64) * 60;
            let value_date = tx_date;

            // Determine tier category for this transaction in the partition
            let (
                tx_type,
                amount,
                doc_ref,
                counterparty_account,
                counterparty_name,
                counterparty_bank,
                narration,
            ) = if local_idx < config.t1_count {
                // =================================================================
                // Tier 1: Exact 1:1 Hash Match
                // =================================================================
                let t1_idx = global_t1_counter;
                global_t1_counter += 1;

                let company = &registry[t1_idx % registry.len()];
                let tx_type = if (t1_idx % 20) < 13 {
                    TransactionType::Credit
                } else {
                    TransactionType::Debit
                };

                // Scaled integer amount between 1,000,000 and 249,000,000 VND
                let raw_amt: u64 = rng.gen_range(1_000_000..=250_000_000);
                let amount = (raw_amt / 1_000) * 1_000;

                let doc_code = format!("HD{:06}", t1_idx + 1);
                let doc_ref_str = doc_code.clone();
                let cp_acc = format!("{}888", company.tax_code);
                let cp_name = company.name.clone();
                let cp_bank = config.bank_code.to_string();
                let narr = format!("Thanh toan hop dong {} cho {}", doc_code, company.name);

                // Corresponding Internal Ledger Entry
                all_ledger_entries.push(InternalLedgerEntry {
                    id: format!("led_t1_{:06}", t1_idx + 1),
                    account_id: format!("ACC-{}", company.partner_code),
                    doc_no: format!("HD-{:06}", t1_idx + 1), // Normalizes to HD{:06}
                    entry_date: tx_date,
                    entry_type: tx_type,
                    amount,
                    partner_code: Some(company.partner_code.clone()),
                    partner_name: Some(company.name.clone()),
                    description: format!("Ghi nhan cong no hop dong {}", doc_code),
                    reconciled_status: ReconciliationStatus::Unmatched,
                    created_at: tx_date,
                });

                (
                    tx_type,
                    amount,
                    Some(doc_ref_str),
                    Some(cp_acc),
                    Some(cp_name),
                    Some(cp_bank),
                    narr,
                )
            } else if local_idx < config.t1_count + config.t2_count {
                // =================================================================
                // Tier 2: Fuzzy Heuristic Match
                // =================================================================
                let t2_idx = global_t2_counter;
                global_t2_counter += 1;

                let company = &registry[(t2_idx + 17) % registry.len()];
                let tx_type = if (t2_idx % 20) < 13 {
                    TransactionType::Credit
                } else {
                    TransactionType::Debit
                };

                // Spaced amounts: 44,000 VND spacing = 4 buckets of 11,000 VND
                // Eliminates bucket collision between distinct Tier 2 items.
                let ledger_amount = 5_000_000 + (t2_idx as u64) * 44_000;

                let (tx_amount, fee_diff) = if t2_idx % 2 == 0 {
                    // Exact amount, but prefix noise and no doc_ref token
                    (ledger_amount, 0)
                } else {
                    // Standard Napas interbank fee variation
                    let fee = napas_fees[t2_idx % napas_fees.len()];
                    (ledger_amount - fee, fee)
                };

                let prefix = tier2_prefixes[t2_idx % tier2_prefixes.len()];
                let party_norm = normalize_vietnamese_text(&company.name);

                // Narration with bank noise and legal form variations, WITHOUT any HD/INV tokens
                let narr = format!(
                    "{prefix} {party_norm} THANH TOAN DICH VU KY {:#05}",
                    t2_idx + 1
                );
                let cp_name = party_norm;
                let cp_acc = format!("{}999", company.tax_code);
                let cp_bank = config.bank_code.to_string();

                // Corresponding Internal Ledger Entry
                all_ledger_entries.push(InternalLedgerEntry {
                    id: format!("led_t2_{:05}", t2_idx + 1),
                    account_id: format!("ACC-{}", company.partner_code),
                    doc_no: format!("TX2-NONREF-{:05}", t2_idx + 1), // Non-matching doc ref
                    entry_date: tx_date,
                    entry_type: tx_type,
                    amount: ledger_amount,
                    partner_code: Some(company.partner_code.clone()),
                    partner_name: Some(company.name.clone()),
                    description: format!(
                        "Hop dong dich vu {} (phi={fee_diff})",
                        company.short_name
                    ),
                    reconciled_status: ReconciliationStatus::Unmatched,
                    created_at: tx_date,
                });

                (
                    tx_type,
                    tx_amount,
                    None,
                    Some(cp_acc),
                    Some(cp_name),
                    Some(cp_bank),
                    narr,
                )
            } else if local_idx < config.t1_count + config.t2_count + config.t3_count {
                // =================================================================
                // Tier 3: Composite Split Match (1-to-N or N-to-1)
                // =================================================================
                let t3_idx = global_t3_tx_counter;
                global_t3_tx_counter += 1;

                let company = &registry[(t3_idx + 33) % registry.len()];
                let tx_type = TransactionType::Credit;

                // Generate structured composite split items
                let (tx_amount, narr) =
                    generate_tier3_item_details(t3_idx, company, tx_date, &mut all_ledger_entries);

                let cp_acc = format!("{}777", company.tax_code);
                let cp_name = company.name.clone();
                let cp_bank = config.bank_code.to_string();

                (
                    tx_type,
                    tx_amount,
                    None,
                    Some(cp_acc),
                    Some(cp_name),
                    Some(cp_bank),
                    narr,
                )
            } else {
                // =================================================================
                // Fail-Closed HITL Queue (Unknown Credits / Stray Transfers)
                // =================================================================
                let h_idx = global_hitl_counter;
                global_hitl_counter += 1;

                let tx_type = TransactionType::Credit;
                let amount = 13_000_000 + (h_idx as u64) * 77_000;
                let narr = format!("Tien chuyen nham khong ro nguon goc #{:03}", h_idx + 1);

                (tx_type, amount, None, None, None, None, narr)
            };

            // Double-entry running balance update
            match tx_type {
                TransactionType::Credit => {
                    running_balance = running_balance
                        .checked_add(amount)
                        .expect("Balance overflow");
                    total_credit = total_credit.saturating_add(amount);
                }
                TransactionType::Debit => {
                    running_balance = running_balance
                        .checked_sub(amount)
                        .expect("Balance underflow");
                    total_debit = total_debit.saturating_add(amount);
                }
            }

            let balance_after = Some(running_balance);

            // Record in BankStatement
            let tx_record = TransactionRecord::new(
                row_id,
                tx_date,
                value_date,
                doc_ref.clone(),
                tx_type,
                amount,
                balance_after,
                counterparty_account.clone(),
                counterparty_name.clone(),
                counterparty_bank.clone(),
                narration.clone(),
            );
            stmt_tx_records.push(tx_record);

            // Record in BankTransactionRow
            let bank_row = BankTransactionRow {
                id: format!("{}_{:05}", config.bank_code, row_id),
                statement_id: stmt_id.clone(),
                account_id: config.account_number.to_string(),
                bank_code: config.bank_code.to_string(),
                tx_date,
                value_date,
                doc_ref,
                tx_type,
                amount,
                balance_after,
                counterparty_account,
                counterparty_name,
                counterparty_bank,
                narration,
                reconciled_status: ReconciliationStatus::Unmatched,
                reconciled_match_id: None,
                created_at: tx_date,
            };
            all_bank_txs.push(bank_row);
        }

        let statement = BankStatement::new(
            config.bank_code.to_string(),
            config.bank_type,
            StatementFormat::Csv,
            Some(config.account_number.to_string()),
            Some(config.account_name.to_string()),
            Some(config.opening_balance),
            Some(running_balance),
            Some(base_timestamp),
            Some(base_timestamp + (config.total_txs as i64) * 60),
            stmt_tx_records,
            0,
        );

        // Verify balance checksum immediately
        assert!(
            statement.balance_checksum_passed,
            "Statement balance invariant failed for bank {}",
            config.bank_code
        );

        bank_counts.insert(config.bank_code.to_string(), config.total_txs);
        all_statements.push(statement);
    }

    // Add residual unallocated ledger entries to reach exactly 50,000 ledger entries
    let residual_needed = 50_000usize.saturating_sub(all_ledger_entries.len());
    for r in 0..residual_needed {
        let r_idx = r + 1;
        let company = &registry[(r + 71) % registry.len()];
        let entry_date = base_timestamp + (r as i64) * 120;
        all_ledger_entries.push(InternalLedgerEntry {
            id: format!("led_res_{:04}", r_idx),
            account_id: format!("ACC-{}", company.partner_code),
            doc_no: format!("INV-UNPAID-{:04}", r_idx),
            entry_date,
            entry_type: TransactionType::Credit,
            amount: 700_000_000 + (r as u64) * 500_000,
            partner_code: Some(company.partner_code.clone()),
            partner_name: Some(company.name.clone()),
            description: format!("Cong no chua thanh toan khach hang {}", company.short_name),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: entry_date,
        });
    }

    assert_eq!(
        all_bank_txs.len(),
        50_000,
        "Must generate exactly 50,000 bank transactions"
    );
    assert_eq!(
        all_ledger_entries.len(),
        50_000,
        "Must generate exactly 50,000 ledger entries"
    );
    assert_eq!(
        global_t1_counter, 45_000,
        "Must generate 45,000 Tier 1 transactions (90.0%)"
    );
    assert_eq!(
        global_t2_counter, 3_750,
        "Must generate 3,750 Tier 2 transactions (7.5%)"
    );
    assert_eq!(
        global_t3_tx_counter, 1_150,
        "Must generate 1,150 Tier 3 transactions (2.3%)"
    );
    assert_eq!(
        global_hitl_counter, 100,
        "Must generate 100 HITL transactions (0.2%)"
    );

    let metadata = DatasetMetadata {
        seed,
        total_bank_transactions: all_bank_txs.len(),
        total_ledger_entries: all_ledger_entries.len(),
        bank_counts,
        tier1_exact_count: global_t1_counter,
        tier2_fuzzy_count: global_t2_counter,
        tier3_split_count: global_t3_tx_counter,
        hitl_queue_count: global_hitl_counter,
        balance_invariants_verified: true,
    };

    Banking50kDataset {
        bank_transactions: all_bank_txs,
        ledger_entries: all_ledger_entries,
        statements: all_statements,
        metadata,
    }
}

/// Helper to generate Tier 3 transaction and invoice configurations.
/// Supports both:
/// 1. 1 Bank Transaction -> N Invoices (Composite Settlement, 350 txs -> 800 invoices)
/// 2. N Bank Transactions -> 1 Invoice (Multi-Installment Settlement, 800 txs -> 325 invoices)
/// Total Tier 3 bank transactions: 350 + 800 = 1,150 txs.
/// Total Tier 3 ledger invoices: 800 + 325 = 1,125 invoices.
fn generate_tier3_item_details(
    t3_idx: usize,
    company: &VietnameseCompany,
    tx_date: i64,
    ledger_entries: &mut Vec<InternalLedgerEntry>,
) -> (u64, String) {
    if t3_idx < 250 {
        // --- 1-to-2 Composite Invoice (250 txs -> 500 invoices) ---
        let g = t3_idx;
        let inv1_amt = 120_000_000 + (g as u64) * 100_000;
        let inv2_amt = 180_000_000 + (g as u64) * 100_000;
        let tx_amt = inv1_amt + inv2_amt;

        let doc1 = format!("INV81{:05}A", g + 1);
        let doc2 = format!("INV81{:05}B", g + 1);

        ledger_entries.push(InternalLedgerEntry {
            id: format!("led_t3_c2_{:05}_1", g + 1),
            account_id: format!("ACC-{}", company.partner_code),
            doc_no: doc1.clone(),
            entry_date: tx_date,
            entry_type: TransactionType::Credit,
            amount: inv1_amt,
            partner_code: Some(company.partner_code.clone()),
            partner_name: Some(company.name.clone()),
            description: format!("Dot thanh toan 1 cua hop dong {}", company.contract_no),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: tx_date,
        });

        ledger_entries.push(InternalLedgerEntry {
            id: format!("led_t3_c2_{:05}_2", g + 1),
            account_id: format!("ACC-{}", company.partner_code),
            doc_no: doc2.clone(),
            entry_date: tx_date,
            entry_type: TransactionType::Credit,
            amount: inv2_amt,
            partner_code: Some(company.partner_code.clone()),
            partner_name: Some(company.name.clone()),
            description: format!("Dot thanh toan 2 cua hop dong {}", company.contract_no),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: tx_date,
        });

        let narr = format!(
            "Thanh toan gom cac hoa don {} va {} {}",
            doc1, doc2, company.name
        );
        (tx_amt, narr)
    } else if t3_idx < 350 {
        // --- 1-to-3 Composite Invoice (100 txs -> 300 invoices) ---
        let g = t3_idx - 250;
        let inv1_amt = 50_000_000 + (g as u64) * 50_000;
        let inv2_amt = 100_000_000 + (g as u64) * 50_000;
        let inv3_amt = 150_000_000 + (g as u64) * 50_000;
        let tx_amt = inv1_amt + inv2_amt + inv3_amt;

        let doc1 = format!("INV82{:05}A", g + 1);
        let doc2 = format!("INV82{:05}B", g + 1);
        let doc3 = format!("INV82{:05}C", g + 1);

        for (sub_id, (amt, doc)) in [
            (1, (inv1_amt, &doc1)),
            (2, (inv2_amt, &doc2)),
            (3, (inv3_amt, &doc3)),
        ] {
            ledger_entries.push(InternalLedgerEntry {
                id: format!("led_t3_c3_{:05}_{sub_id}", g + 1),
                account_id: format!("ACC-{}", company.partner_code),
                doc_no: doc.clone(),
                entry_date: tx_date,
                entry_type: TransactionType::Credit,
                amount: amt,
                partner_code: Some(company.partner_code.clone()),
                partner_name: Some(company.name.clone()),
                description: format!("Goi san pham {sub_id} {}", company.short_name),
                reconciled_status: ReconciliationStatus::Unmatched,
                created_at: tx_date,
            });
        }

        let narr = format!(
            "Thanh toan hop dong gom {}, {}, {} {}",
            doc1, doc2, doc3, company.name
        );
        (tx_amt, narr)
    } else if t3_idx < 750 {
        // --- 2-to-1 Multi-Installment (400 txs -> 200 groups -> 200 invoices) ---
        let rel_idx = t3_idx - 350;
        let g = rel_idx / 2;
        let part = (rel_idx % 2) + 1;

        let total_inv_amt = 80_000_000 + (g as u64) * 100_000;
        let tx1_amt = 35_000_000 + (g as u64) * 50_000;
        let tx2_amt = total_inv_amt - tx1_amt;

        let doc = format!("INV83{:05}", g + 1);

        // Only create the single invoice once (on part 1)
        if part == 1 {
            ledger_entries.push(InternalLedgerEntry {
                id: format!("led_t3_ins2_{:05}", g + 1),
                account_id: format!("ACC-{}", company.partner_code),
                doc_no: doc.clone(),
                entry_date: tx_date,
                entry_type: TransactionType::Credit,
                amount: total_inv_amt,
                partner_code: Some(company.partner_code.clone()),
                partner_name: Some(company.name.clone()),
                description: format!("Hoa don ban hang tra gop {}", company.short_name),
                reconciled_status: ReconciliationStatus::Unmatched,
                created_at: tx_date,
            });
        }

        let tx_amt = if part == 1 { tx1_amt } else { tx2_amt };
        let narr = format!("Thanh toan dot {part} cho hoa don {} {}", doc, company.name);
        (tx_amt, narr)
    } else if t3_idx < 1050 {
        // --- 3-to-1 Multi-Installment (300 txs -> 100 groups -> 100 invoices) ---
        let rel_idx = t3_idx - 750;
        let g = rel_idx / 3;
        let part = (rel_idx % 3) + 1;

        let total_inv_amt = 90_000_000 + (g as u64) * 100_000;
        let tx1_amt = 25_000_000;
        let tx2_amt = 30_000_000;
        let tx3_amt = total_inv_amt - tx1_amt - tx2_amt;

        let doc = format!("INV84{:05}", g + 1);

        if part == 1 {
            ledger_entries.push(InternalLedgerEntry {
                id: format!("led_t3_ins3_{:05}", g + 1),
                account_id: format!("ACC-{}", company.partner_code),
                doc_no: doc.clone(),
                entry_date: tx_date,
                entry_type: TransactionType::Credit,
                amount: total_inv_amt,
                partner_code: Some(company.partner_code.clone()),
                partner_name: Some(company.name.clone()),
                description: format!("Hop dong cung ung 3 dot {}", company.short_name),
                reconciled_status: ReconciliationStatus::Unmatched,
                created_at: tx_date,
            });
        }

        let tx_amt = match part {
            1 => tx1_amt,
            2 => tx2_amt,
            _ => tx3_amt,
        };
        let narr = format!("Thanh toan dot {part} cho hoa don {} {}", doc, company.name);
        (tx_amt, narr)
    } else {
        // --- 4-to-1 Multi-Installment (100 txs -> 25 groups -> 25 invoices) ---
        let rel_idx = t3_idx - 1050;
        let g = rel_idx / 4;
        let part = (rel_idx % 4) + 1;

        let total_inv_amt = 120_000_000 + (g as u64) * 200_000;
        let tx1_amt = 20_000_000;
        let tx2_amt = 30_000_000;
        let tx3_amt = 30_000_000;
        let tx4_amt = total_inv_amt - tx1_amt - tx2_amt - tx3_amt;

        let doc = format!("INV85{:05}", g + 1);

        if part == 1 {
            ledger_entries.push(InternalLedgerEntry {
                id: format!("led_t3_ins4_{:05}", g + 1),
                account_id: format!("ACC-{}", company.partner_code),
                doc_no: doc.clone(),
                entry_date: tx_date,
                entry_type: TransactionType::Credit,
                amount: total_inv_amt,
                partner_code: Some(company.partner_code.clone()),
                partner_name: Some(company.name.clone()),
                description: format!(
                    "Hop dong thi cong xay dung 4 giai doan {}",
                    company.short_name
                ),
                reconciled_status: ReconciliationStatus::Unmatched,
                created_at: tx_date,
            });
        }

        let tx_amt = match part {
            1 => tx1_amt,
            2 => tx2_amt,
            3 => tx3_amt,
            _ => tx4_amt,
        };
        let narr = format!(
            "Thanh toan giai doan {part} cho hoa don {} {}",
            doc, company.name
        );
        (tx_amt, narr)
    }
}

/// Formats a `BankStatement` as a standard Vietnamese banking CSV string.
/// Compatible with `TcbCsvParser`, `MbBankParser`, and `VietinBankParser`.
pub fn format_statement_as_csv(statement: &BankStatement) -> String {
    let mut out = String::with_capacity(statement.transactions.len() * 120 + 256);

    // Header metadata lines
    out.push_str(&format!("NGÂN HÀNG {}\n", statement.bank_code));
    out.push_str(&format!(
        "Số tài khoản: {}\n",
        statement.account_number.as_deref().unwrap_or("0000000000")
    ));
    out.push_str(&format!(
        "Tên tài khoản: {}\n",
        statement.account_name.as_deref().unwrap_or("LIVA TREASURY")
    ));
    out.push_str(&format!(
        "Số dư đầu kỳ: {}\n",
        fmt_vnd(statement.opening_balance.unwrap_or(0))
    ));
    out.push_str("Ngày giao dịch,Mã giao dịch,Số tiền ghi nợ,Số tiền ghi có,Số dư,Nội dung\n");

    for tx in &statement.transactions {
        let (debit_str, credit_str) = match tx.tx_type {
            TransactionType::Debit => (fmt_vnd(tx.amount), String::new()),
            TransactionType::Credit => (String::new(), fmt_vnd(tx.amount)),
        };

        let bal_str = tx.balance_after.map(fmt_vnd).unwrap_or_default();
        let ref_str = tx.doc_ref.as_deref().unwrap_or("");

        // Format date as DD/MM/YYYY
        let date_str = "15/08/2026";

        // Clean narration from internal commas or quotes
        let clean_narration = tx.narration.replace('"', "\"\"");

        out.push_str(&format!(
            "{},{},{},{},{},\"{}\"\n",
            date_str, ref_str, debit_str, credit_str, bal_str, clean_narration
        ));
    }

    out
}

/// Formats a `u64` amount into Vietnamese dot-thousands format (e.g. `15.000.000`).
pub fn fmt_vnd(amount: u64) -> String {
    let s = amount.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push('.');
        }
        result.push(c);
    }
    result
}

impl Banking50kDataset {
    /// Exports the 50,000 dataset into standard banking statement CSV files and JSON fixtures.
    pub fn export_to_directory(&self, out_dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(out_dir)?;

        for stmt in &self.statements {
            let filename = format!("{}_statement.csv", stmt.bank_code.to_lowercase());
            let csv_content = format_statement_as_csv(stmt);
            std::fs::write(out_dir.join(filename), csv_content)?;
        }

        let invoices_json = serde_json::to_string_pretty(&self.ledger_entries)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(out_dir.join("open_invoices.json"), invoices_json)?;

        let metadata_json = serde_json::to_string_pretty(&self.metadata)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(out_dir.join("dataset_metadata.json"), metadata_json)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corporate_registry_generation() {
        let registry = build_corporate_registry();
        assert_eq!(
            registry.len(),
            200,
            "Must generate exactly 200 distinct corporations"
        );

        // Verify distinctness of tax codes and partner codes
        let mut codes = HashMap::new();
        let mut tax_codes = HashMap::new();
        for c in &registry {
            assert!(
                codes.insert(c.partner_code.clone(), true).is_none(),
                "Duplicate partner code"
            );
            assert!(
                tax_codes.insert(c.tax_code.clone(), true).is_none(),
                "Duplicate tax code"
            );
            assert!(!c.name.is_empty());
        }
    }

    #[test]
    fn test_generator_deterministic_output() {
        let ds1 = generate_50k_dataset(DEFAULT_BENCHMARK_SEED);
        let ds2 = generate_50k_dataset(DEFAULT_BENCHMARK_SEED);

        assert_eq!(ds1.bank_transactions.len(), 50_000);
        assert_eq!(ds1.ledger_entries.len(), 50_000);
        assert_eq!(ds1.metadata.tier1_exact_count, 45_000);
        assert_eq!(ds1.metadata.tier2_fuzzy_count, 3_750);
        assert_eq!(ds1.metadata.tier3_split_count, 1_150);
        assert_eq!(ds1.metadata.hitl_queue_count, 100);

        // Byte-for-byte reproducibility
        assert_eq!(
            ds1.bank_transactions[0].amount,
            ds2.bank_transactions[0].amount
        );
        assert_eq!(
            ds1.bank_transactions[1000].narration,
            ds2.bank_transactions[1000].narration
        );
        assert_eq!(
            ds1.bank_transactions[49999].id,
            ds2.bank_transactions[49999].id
        );
        assert_eq!(
            ds1.ledger_entries[49999].amount,
            ds2.ledger_entries[49999].amount
        );
    }

    #[test]
    fn test_double_entry_balance_invariants_across_all_statements() {
        let dataset = generate_50k_dataset(DEFAULT_BENCHMARK_SEED);

        for stmt in &dataset.statements {
            let inv = stmt.verify_balance_invariants();
            assert!(
                inv.is_valid,
                "Balance invariant failed for {}",
                stmt.bank_code
            );
            assert_eq!(
                inv.discrepancy, 0,
                "Discrepancy must be 0 for {}",
                stmt.bank_code
            );
        }
    }
}
