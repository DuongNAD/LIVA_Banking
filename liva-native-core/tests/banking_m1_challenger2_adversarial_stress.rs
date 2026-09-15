//! Adversarial Stress & Empirical Challenge Test Suite (Challenger 2)
//!
//! Milestone 1: Bank Parsers Architecture & Expansion
//! Covers:
//! 1. Remediation 1: TCB CSV footer closing balance extraction & transaction balance fallback.
//! 2. Remediation 2: Windows-1258 Unicode NFC normalization across TCB, CTG, MB, and Agribank parsers.
//! 3. Remediation 3: VietinBank CSV English header detection & debit/credit column priority.
//! 4. Extreme Edge Cases: Zero-amount and negative amount handling.
//! 5. Large Treasury Numbers: 500 Billion VND calculations without overflow.
//! 6. Balance Invariant Oracle: Sensitivity to any discrepancy != 0.
//! 7. Multi-bank and multi-format container detection & sniffing matrix across all 6 banks.

use liva_native_core::banking::models::*;
use liva_native_core::banking::parser::agribank_parser::AgribankParser;
use liva_native_core::banking::parser::mbbank_parser::MbBankParser;
use liva_native_core::banking::parser::tcb_csv::TcbCsvParser;
use liva_native_core::banking::parser::vietinbank_parser::VietinBankParser;
use liva_native_core::banking::parser::{BankStatementParser, sniff_and_parse};
use std::path::Path;

// ===========================================================================
// 1. Remediation 1: TCB CSV Footer Closing Balance & Fallback
// ===========================================================================

#[test]
fn test_challenger2_m1_remediation_tcb_csv_footer_closing_balance_and_fallback() {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let fixtures_dir = base_dir.join("fixtures").join("statements");

    // A. Real Fixture with summary footer row at line 66 (tcb_aug2026.csv)
    let tcb_file = fixtures_dir.join("tcb_aug2026.csv");
    assert!(tcb_file.exists(), "tcb_aug2026.csv must exist");
    let tcb_bytes = std::fs::read(&tcb_file).unwrap();
    let stmt = TcbCsvParser.parse(&tcb_bytes, "tcb_aug2026.csv").unwrap();

    assert_eq!(stmt.bank_code, "TCB");
    assert_eq!(stmt.account_number.as_deref(), Some("19034567890123"));
    assert_eq!(stmt.opening_balance, Some(785_600_000));
    assert_eq!(
        stmt.closing_balance,
        Some(2_246_754_900),
        "Footer closing balance must be extracted"
    );
    assert_eq!(stmt.transactions.len(), 60);

    let inv = verify_balance_invariants(&stmt);
    assert!(
        inv.is_balanced,
        "TCB statement with footer closing balance must be balanced"
    );
    assert_eq!(inv.discrepancy, 0);
    assert_eq!(inv.computed_closing, 2_246_754_900);

    // B. Synthetic Statement with footer row using different wording ("Closing Balance: 350.000.000")
    let tcb_eng_footer = "\
Số tài khoản: 19031122334455\n\
Số dư đầu kỳ: 200.000.000\n\
Ngày giao dịch,Mã giao dịch,Số tiền ghi nợ,Số tiền ghi có,Số dư,Nội dung\n\
10/08/2026,FT101,,200.000.000,400.000.000,Doanh thu ban hang\n\
11/08/2026,FT102,50.000.000,,350.000.000,Tien thue van phong\n\
Tổng phát sinh;;50.000.000;200.000.000;350.000.000\n\
Closing Balance: 350.000.000\n";
    let eng_stmt = TcbCsvParser
        .parse(tcb_eng_footer.as_bytes(), "tcb_eng.csv")
        .unwrap();
    assert_eq!(eng_stmt.closing_balance, Some(350_000_000));
    let eng_inv = verify_balance_invariants(&eng_stmt);
    assert!(eng_inv.is_balanced);
    assert_eq!(eng_inv.discrepancy, 0);

    // C. Synthetic Statement WITHOUT footer row -> Fallback to transactions.last().balance_after
    let tcb_no_footer = "\
Số tài khoản: 19039988776655\n\
Số dư đầu kỳ: 100.000.000\n\
Ngày giao dịch,Mã giao dịch,Số tiền ghi nợ,Số tiền ghi có,Số dư,Nội dung\n\
15/08/2026,FT201,,50.000.000,150.000.000,Tien hang\n\
16/08/2026,FT202,30.000.000,,120.000.000,Chi phi van hanh\n";
    let fallback_stmt = TcbCsvParser
        .parse(tcb_no_footer.as_bytes(), "tcb_fallback.csv")
        .unwrap();
    assert_eq!(
        fallback_stmt.closing_balance,
        Some(120_000_000),
        "Must fallback to last transaction balance"
    );
    let fallback_inv = verify_balance_invariants(&fallback_stmt);
    assert!(fallback_inv.is_balanced);
    assert_eq!(fallback_inv.discrepancy, 0);
}

// ===========================================================================
// 2. Remediation 2: Windows-1258 Unicode NFC Normalization Stress
// ===========================================================================

#[test]
fn test_challenger2_m1_remediation_windows_1258_nfc_normalization_stress() {
    fn to_cp1258_decomposed(s: &str) -> String {
        let mut out = String::new();
        for ch in s.chars() {
            match ch {
                'ố' => {
                    out.push('ô');
                    out.push('\u{0301}');
                }
                'Ố' => {
                    out.push('Ô');
                    out.push('\u{0301}');
                }
                'ồ' => {
                    out.push('ô');
                    out.push('\u{0300}');
                }
                'ổ' => {
                    out.push('ô');
                    out.push('\u{0309}');
                }
                'ỗ' => {
                    out.push('ô');
                    out.push('\u{0303}');
                }
                'ộ' => {
                    out.push('ô');
                    out.push('\u{0323}');
                }
                'ế' => {
                    out.push('ê');
                    out.push('\u{0301}');
                }
                'ề' => {
                    out.push('ê');
                    out.push('\u{0300}');
                }
                'ể' => {
                    out.push('ê');
                    out.push('\u{0309}');
                }
                'ễ' => {
                    out.push('ê');
                    out.push('\u{0303}');
                }
                'ệ' => {
                    out.push('ê');
                    out.push('\u{0323}');
                }
                'ấ' => {
                    out.push('â');
                    out.push('\u{0301}');
                }
                'ầ' => {
                    out.push('â');
                    out.push('\u{0300}');
                }
                'ẩ' => {
                    out.push('â');
                    out.push('\u{0309}');
                }
                'ẫ' => {
                    out.push('â');
                    out.push('\u{0303}');
                }
                'ậ' => {
                    out.push('â');
                    out.push('\u{0323}');
                }
                'ắ' => {
                    out.push('ă');
                    out.push('\u{0301}');
                }
                'ằ' => {
                    out.push('ă');
                    out.push('\u{0300}');
                }
                'ẳ' => {
                    out.push('ă');
                    out.push('\u{0309}');
                }
                'ẵ' => {
                    out.push('ă');
                    out.push('\u{0303}');
                }
                'ặ' => {
                    out.push('ă');
                    out.push('\u{0323}');
                }
                'ứ' => {
                    out.push('ư');
                    out.push('\u{0301}');
                }
                'ừ' => {
                    out.push('ư');
                    out.push('\u{0300}');
                }
                'ử' => {
                    out.push('ư');
                    out.push('\u{0309}');
                }
                'ữ' => {
                    out.push('ư');
                    out.push('\u{0303}');
                }
                'ự' => {
                    out.push('ư');
                    out.push('\u{0323}');
                }
                'ớ' => {
                    out.push('ơ');
                    out.push('\u{0301}');
                }
                'ờ' => {
                    out.push('ơ');
                    out.push('\u{0300}');
                }
                'ở' => {
                    out.push('ơ');
                    out.push('\u{0309}');
                }
                'ỡ' => {
                    out.push('ơ');
                    out.push('\u{0303}');
                }
                'ợ' => {
                    out.push('ơ');
                    out.push('\u{0323}');
                }
                'Ả' => {
                    out.push('A');
                    out.push('\u{0309}');
                }
                'ả' => {
                    out.push('a');
                    out.push('\u{0309}');
                }
                'ạ' => {
                    out.push('a');
                    out.push('\u{0323}');
                }
                'ì' => {
                    out.push('i');
                    out.push('\u{0300}');
                }
                'í' => {
                    out.push('i');
                    out.push('\u{0301}');
                }
                'ĩ' => {
                    out.push('i');
                    out.push('\u{0303}');
                }
                'ỉ' => {
                    out.push('i');
                    out.push('\u{0309}');
                }
                'ị' => {
                    out.push('i');
                    out.push('\u{0323}');
                }
                'ỏ' => {
                    out.push('o');
                    out.push('\u{0309}');
                }
                'ọ' => {
                    out.push('o');
                    out.push('\u{0323}');
                }
                'ủ' => {
                    out.push('u');
                    out.push('\u{0309}');
                }
                'ụ' => {
                    out.push('u');
                    out.push('\u{0323}');
                }
                'ỷ' => {
                    out.push('y');
                    out.push('\u{0309}');
                }
                'ỳ' => {
                    out.push('y');
                    out.push('\u{0300}');
                }
                'ỵ' => {
                    out.push('y');
                    out.push('\u{0323}');
                }
                _ => out.push(ch),
            }
        }
        out
    }

    // Comprehensive Vietnamese accented statement template
    let text = "\
Số tài khoản: 1234567890\n\
Tên tài khoản: CÔNG TY TNHH GIẢI PHÁP LIVA\n\
Số dư đầu kỳ: 1.000.000.000\n\
Ngày giao dịch,Số chứng từ,Số tiền ghi nợ,Số tiền ghi có,Số dư,Diễn giải\n\
15/08/2026,GD-001,,250.000.000,1.250.000.000,Thanh toán dịch vụ phần mềm LIVA\n\
16/08/2026,GD-002,50.000.000,,1.200.000.000,Phí duy trì tài khoản doanh nghiệp\n\
Số dư cuối kỳ: 1.200.000.000\n";

    let decomposed = to_cp1258_decomposed(text);
    let (win1258_bytes, _, had_errors) = encoding_rs::WINDOWS_1258.encode(&decomposed);
    assert!(!had_errors, "Must encode cleanly to Windows-1258");
    assert!(
        std::str::from_utf8(&win1258_bytes).is_err(),
        "Must be non-UTF8 to test decoder"
    );

    // 1. TcbCsvParser
    let tcb = TcbCsvParser
        .parse(&win1258_bytes, "tcb_cp1258.csv")
        .unwrap();
    assert_eq!(tcb.account_number.as_deref(), Some("1234567890"));
    assert_eq!(tcb.opening_balance, Some(1_000_000_000));
    assert_eq!(tcb.closing_balance, Some(1_200_000_000));
    assert_eq!(tcb.transactions.len(), 2);
    assert!(verify_balance_invariants(&tcb).is_balanced);

    // 2. VietinBankParser
    let ctg = VietinBankParser
        .parse(&win1258_bytes, "ctg_cp1258.csv")
        .unwrap();
    assert_eq!(ctg.account_number.as_deref(), Some("1234567890"));
    assert_eq!(ctg.opening_balance, Some(1_000_000_000));
    assert_eq!(ctg.closing_balance, Some(1_200_000_000));
    assert_eq!(ctg.transactions.len(), 2);
    assert!(verify_balance_invariants(&ctg).is_balanced);

    // 3. MbBankParser
    let mb = MbBankParser.parse(&win1258_bytes, "mb_cp1258.csv").unwrap();
    assert_eq!(mb.account_number.as_deref(), Some("1234567890"));
    assert_eq!(mb.opening_balance, Some(1_000_000_000));
    assert_eq!(mb.closing_balance, Some(1_200_000_000));
    assert_eq!(mb.transactions.len(), 2);
    assert!(verify_balance_invariants(&mb).is_balanced);

    // 4. AgribankParser
    let agri = AgribankParser
        .parse(&win1258_bytes, "agri_cp1258.csv")
        .unwrap();
    assert_eq!(agri.account_number.as_deref(), Some("1234567890"));
    assert_eq!(agri.opening_balance, Some(1_000_000_000));
    assert_eq!(agri.closing_balance, Some(1_200_000_000));
    assert_eq!(agri.transactions.len(), 2);
    assert!(verify_balance_invariants(&agri).is_balanced);
}

// ===========================================================================
// 3. Remediation 3: VietinBank English Header & Direction Stress
// ===========================================================================

#[test]
fn test_challenger2_m1_remediation_vietinbank_english_header_and_direction() {
    // English headers with dedicated Debit and Credit columns
    let ctg_eng = "\
Account No: 102000123456\n\
Account Name: LIVA BANKING CORP\n\
Opening Balance: 500.000.000\n\
Date,Doc No,Amount,Credit,Debit,Balance,Description\n\
10/08/2026,REF001,150.000.000,150.000.000,,650.000.000,Invoice payment from client\n\
11/08/2026,REF002,50.000.000,,50.000.000,600.000.000,Cloud server hosting fee\n\
12/08/2026,REF003,200.000.000,200.000.000,,800.000.000,Consulting retainer\n\
Closing Balance: 800.000.000\n";

    let ctg = VietinBankParser
        .parse(ctg_eng.as_bytes(), "ctg_corp_en.csv")
        .unwrap();
    assert_eq!(ctg.bank_code, "CTG");
    assert_eq!(ctg.account_number.as_deref(), Some("102000123456"));
    assert_eq!(ctg.opening_balance, Some(500_000_000));
    assert_eq!(ctg.closing_balance, Some(800_000_000));
    assert_eq!(ctg.transactions.len(), 3);

    assert_eq!(ctg.transactions[0].tx_type, TransactionType::Credit);
    assert_eq!(ctg.transactions[0].amount, 150_000_000);
    assert_eq!(ctg.transactions[1].tx_type, TransactionType::Debit);
    assert_eq!(ctg.transactions[1].amount, 50_000_000);
    assert_eq!(ctg.transactions[2].tx_type, TransactionType::Credit);
    assert_eq!(ctg.transactions[2].amount, 200_000_000);

    let inv = verify_balance_invariants(&ctg);
    assert!(inv.is_balanced);
    assert_eq!(inv.discrepancy, 0);
    assert_eq!(inv.computed_closing, 800_000_000);
}

// ===========================================================================
// 4. Extreme Edge Cases: Zero-Amount & Negative Amount Handling
// ===========================================================================

#[test]
fn test_challenger2_m1_zero_and_negative_amount_handling() {
    // 1. Amount Parser: Zero values
    assert_eq!(parse_vietnamese_amount("0"), Some(0));
    assert_eq!(parse_vietnamese_amount("0.00"), Some(0));
    assert_eq!(parse_vietnamese_amount("0,00"), Some(0));
    assert_eq!(parse_vietnamese_amount(" 0 VND "), Some(0));
    assert_eq!(parse_vietnamese_amount("0,000"), Some(0));

    // 2. Amount Parser: Negative values (returns absolute scaled u64)
    assert_eq!(parse_vietnamese_amount("-500.000"), Some(500_000));
    assert_eq!(parse_vietnamese_amount("-50.000.000,00"), Some(50_000_000));
    assert_eq!(parse_vietnamese_amount("(15.000.000)"), Some(15_000_000));
    assert_eq!(parse_vietnamese_amount("-1.500.000 VND"), Some(1_500_000));

    // 3. Amount Parser: Invalid symbols rejected
    assert_eq!(parse_vietnamese_amount("--500.000"), None);
    assert_eq!(parse_vietnamese_amount("50-000"), None);
    assert_eq!(parse_vietnamese_amount("((50.000))"), None);
    assert_eq!(parse_vietnamese_amount("50.000%"), None);
    assert_eq!(parse_vietnamese_amount("50.000$"), None);
    assert_eq!(parse_vietnamese_amount("N/A"), None);

    // 4. Zero-amount rows must NOT produce bogus transactions in parsers
    let csv_with_zero = "\
Số tài khoản: 19034455667788\n\
Số dư đầu kỳ: 100.000.000\n\
Ngày giao dịch,Mã giao dịch,Số tiền ghi nợ,Số tiền ghi có,Số dư,Nội dung\n\
15/08/2026,FT101,,50.000.000,150.000.000,Thanh toan tien hang\n\
15/08/2026,FT102,0,0,150.000.000,Giao dich khong phat sinh phi\n\
16/08/2026,FT103,20.000.000,,130.000.000,Tien dien nuoc\n\
16/08/2026,FT104,,,130.000.000,Dong mo ta khoan phu\n\
Số dư cuối kỳ: 130.000.000\n";

    let stmt = TcbCsvParser
        .parse(csv_with_zero.as_bytes(), "tcb_zero.csv")
        .unwrap();
    // Verify only the 2 non-zero transactions were added
    assert_eq!(
        stmt.transactions.len(),
        2,
        "Zero and blank amount rows must be skipped"
    );
    assert_eq!(stmt.transactions[0].amount, 50_000_000);
    assert_eq!(stmt.transactions[1].amount, 20_000_000);
    assert!(verify_balance_invariants(&stmt).is_balanced);

    // 5. Negative amount row in generic amount column -> mapped to Debit
    let ctg_neg_csv = "\
NGAN HANG CONG THUONG VIETINBANK\n\
Số tài khoản: 102000887766\n\
Số dư đầu kỳ: 50.000.000\n\
Ngày GD,Số phiếu,Số tiền GD,Dư cuối,Diễn giải\n\
15/08/2026,CT101,20.000.000,70.000.000,Nop tien mat\n\
16/08/2026,CT102,-10.000.000,60.000.000,Rut tien mat ATM\n\
Số dư cuối kỳ: 60.000.000\n";

    let ctg = VietinBankParser
        .parse(ctg_neg_csv.as_bytes(), "ctg_neg.csv")
        .unwrap();
    assert_eq!(ctg.transactions.len(), 2);
    assert_eq!(ctg.transactions[0].tx_type, TransactionType::Credit);
    assert_eq!(ctg.transactions[0].amount, 20_000_000);
    assert_eq!(
        ctg.transactions[1].tx_type,
        TransactionType::Debit,
        "Negative amount must be Debit"
    );
    assert_eq!(ctg.transactions[1].amount, 10_000_000);
    assert!(verify_balance_invariants(&ctg).is_balanced);
}

// ===========================================================================
// 5. Extreme Edge Cases: Large Treasury Numbers (500 Billion VND)
// ===========================================================================

#[test]
fn test_challenger2_m1_large_treasury_500_billion_vnd() {
    // 1. Amount parser with massive treasury amounts
    assert_eq!(
        parse_vietnamese_amount("500.000.000.000"),
        Some(500_000_000_000)
    );
    assert_eq!(
        parse_vietnamese_amount("1.250.000.000.000"),
        Some(1_250_000_000_000)
    );
    assert_eq!(
        parse_vietnamese_amount("999.999.999.999.999"),
        Some(999_999_999_999_999)
    );

    // 2. Full statement parse with Treasury scale transactions
    let treasury_csv = "\
Số tài khoản: 19039999888877\n\
Số dư đầu kỳ: 500.000.000.000\n\
Ngày giao dịch,Mã giao dịch,Số tiền ghi nợ,Số tiền ghi có,Số dư,Nội dung\n\
01/08/2026,TR001,,250.000.000.000,750.000.000.000,Thu hoi von dau tu trai phieu\n\
05/08/2026,TR002,100.000.000.000,,650.000.000.000,Thanh toan tien vay lien ngan hang\n\
10/08/2026,TR003,,150.000.000.000,800.000.000.000,Chuyen tien tu ngan hang nha nuoc\n\
15/08/2026,TR004,300.000.000.000,,500.000.000.000,Giai ngan goi tin dung du an\n\
Số dư cuối kỳ: 500.000.000.000\n";

    let stmt = TcbCsvParser
        .parse(treasury_csv.as_bytes(), "treasury.csv")
        .unwrap();
    assert_eq!(stmt.opening_balance, Some(500_000_000_000));
    assert_eq!(stmt.closing_balance, Some(500_000_000_000));
    assert_eq!(stmt.total_credit, 400_000_000_000);
    assert_eq!(stmt.total_debit, 400_000_000_000);

    let inv = verify_balance_invariants(&stmt);
    assert!(
        inv.is_balanced,
        "Treasury 500 Billion calculation must balance without overflow"
    );
    assert_eq!(inv.discrepancy, 0);
    assert_eq!(inv.computed_closing, 500_000_000_000);
}

// ===========================================================================
// 6. Balance Invariant Verification Oracle Sensitivity
// ===========================================================================

#[test]
fn test_challenger2_m1_balance_invariant_oracle_sensitivity() {
    let make_statement = |open: u64, credits: &[u64], debits: &[u64], close: u64| {
        let mut txs = Vec::new();
        let mut cur_time = 1_725_000_000i64;
        let mut id = 1;
        for &c in credits {
            txs.push(TransactionRecord::new(
                id,
                cur_time,
                cur_time,
                Some(format!("CR_{id}")),
                TransactionType::Credit,
                c,
                None,
                None,
                None,
                None,
                "Credit".to_string(),
            ));
            cur_time += 100;
            id += 1;
        }
        for &d in debits {
            txs.push(TransactionRecord::new(
                id,
                cur_time,
                cur_time,
                Some(format!("DB_{id}")),
                TransactionType::Debit,
                d,
                None,
                None,
                None,
                None,
                "Debit".to_string(),
            ));
            cur_time += 100;
            id += 1;
        }

        BankStatement::new(
            "VCB".to_string(),
            BankType::Vietcombank,
            StatementFormat::Excel,
            Some("0011001234567".to_string()),
            Some("LIVA CORP".to_string()),
            Some(open),
            Some(close),
            Some(1_725_000_000),
            Some(cur_time),
            txs,
            1,
        )
    };

    // A. Discrepancy == 0 -> Balanced
    let s0 = make_statement(100_000_000, &[50_000_000], &[20_000_000], 130_000_000);
    let r0 = verify_balance_invariants(&s0);
    assert!(r0.is_balanced);
    assert!(r0.is_valid);
    assert_eq!(r0.discrepancy, 0);

    // B. Discrepancy == +1 VND -> UNBALANCED
    let s_p1 = make_statement(100_000_000, &[50_000_000], &[20_000_000], 130_000_001);
    let r_p1 = verify_balance_invariants(&s_p1);
    assert!(!r_p1.is_balanced, "+1 VND discrepancy must flag unbalanced");
    assert!(!r_p1.is_valid);
    assert_eq!(r_p1.discrepancy, 1);

    // C. Discrepancy == -1 VND -> UNBALANCED
    let s_m1 = make_statement(100_000_000, &[50_000_000], &[20_000_000], 129_999_999);
    let r_m1 = verify_balance_invariants(&s_m1);
    assert!(!r_m1.is_balanced, "-1 VND discrepancy must flag unbalanced");
    assert!(!r_m1.is_valid);
    assert_eq!(r_m1.discrepancy, -1);

    // D. Missing opening / closing defaults to 0
    let mut s_none = make_statement(100_000_000, &[50_000_000], &[20_000_000], 130_000_000);
    s_none.opening_balance = None;
    s_none.closing_balance = None;
    let r_none = verify_balance_invariants(&s_none);
    // Calculated = 0 + 50M - 20M = 30M; Discrepancy = 0 - 30M = -30M
    assert!(!r_none.is_balanced);
    assert_eq!(r_none.discrepancy, -30_000_000);
}

// ===========================================================================
// 7. Multi-bank and Multi-format Sniff & Parse Coverage Matrix
// ===========================================================================

#[test]
fn test_challenger2_m1_sniff_and_parse_all_six_banks_and_formats() {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let fixtures_dir = base_dir.join("fixtures").join("statements");

    // 1A. Vietcombank Adversarial Merged (Excel .xlsx)
    let vcb_merged_bytes = std::fs::read(fixtures_dir.join("vcb_adversarial_merged.xlsx")).unwrap();
    let vcb_merged = sniff_and_parse(&vcb_merged_bytes, "vcb_adversarial_merged.xlsx").unwrap();
    assert_eq!(vcb_merged.bank, BankType::Vietcombank);
    assert_eq!(vcb_merged.bank_code, "VCB");
    assert_eq!(vcb_merged.format, StatementFormat::Excel);
    assert_eq!(vcb_merged.closing_balance, Some(160_000_000));
    assert!(verify_balance_invariants(&vcb_merged).is_balanced);

    // 1B. Vietcombank August 2026 Fixture (Excel .xlsx)
    // REMEDIATION 4: Multi-column summary row 3-pass extraction ensures
    // closing balance (2,658,070,400) is prioritized over debit/credit totals.
    let vcb_bytes = std::fs::read(fixtures_dir.join("vcb_aug2026.xlsx")).unwrap();
    let vcb = sniff_and_parse(&vcb_bytes, "vcb_aug2026.xlsx").unwrap();
    assert_eq!(vcb.bank, BankType::Vietcombank);
    assert_eq!(vcb.bank_code, "VCB");
    assert_eq!(vcb.format, StatementFormat::Excel);
    assert_eq!(vcb.opening_balance, Some(1_450_230_000));
    assert_eq!(vcb.transactions.len(), 50);
    assert_eq!(
        vcb.closing_balance,
        Some(2_658_070_400),
        "Closing balance must be correctly extracted"
    );
    let vcb_inv = verify_balance_invariants(&vcb);
    assert_eq!(vcb_inv.discrepancy, 0, "Discrepancy must be exactly 0");
    assert!(
        vcb_inv.is_balanced,
        "vcb_aug2026 must pass balance invariant with remediated 3-pass extraction"
    );

    // 2. Techcombank (CSV .csv)
    let tcb_bytes = std::fs::read(fixtures_dir.join("tcb_aug2026.csv")).unwrap();
    let tcb = sniff_and_parse(&tcb_bytes, "tcb_aug2026.csv").unwrap();
    assert_eq!(tcb.bank, BankType::Techcombank);
    assert_eq!(tcb.bank_code, "TCB");
    assert_eq!(tcb.format, StatementFormat::Csv);
    assert!(verify_balance_invariants(&tcb).is_balanced);

    // 3. BIDV (PDF .pdf)
    let bidv_bytes = std::fs::read(fixtures_dir.join("bidv_aug2026.pdf")).unwrap();
    let bidv = sniff_and_parse(&bidv_bytes, "bidv_aug2026.pdf").unwrap();
    assert_eq!(bidv.bank, BankType::Bidv);
    assert_eq!(bidv.bank_code, "BIDV");
    assert_eq!(bidv.format, StatementFormat::Pdf);
    assert!(verify_balance_invariants(&bidv).is_balanced);

    // 4. VietinBank (CSV .csv)
    let ctg_csv = "\
NGÂN HÀNG TMCP CÔNG THƯƠNG VIỆT NAM (VIETINBANK)\n\
Số tài khoản: 102000887766\n\
Số dư đầu kỳ: 100.000.000\n\
Ngày GD,Số phiếu,Số tiền GD,Nợ/Có,Dư cuối,Diễn giải\n\
15/08/2026,CT881,30.000.000,C,130.000.000,Khach hang thanh toan tien hang\n\
16/08/2026,CT882,10.000.000,N,120.000.000,Phi chuyen tien\n\
Số dư cuối kỳ: 120.000.000\n";
    let ctg = sniff_and_parse(ctg_csv.as_bytes(), "ctg_saoke.csv").unwrap();
    assert_eq!(ctg.bank, BankType::VietinBank);
    assert_eq!(ctg.bank_code, "CTG");
    assert_eq!(ctg.format, StatementFormat::Csv);
    assert!(verify_balance_invariants(&ctg).is_balanced);

    // 5. MBBank (CSV .csv)
    let mb_csv = "\
NGÂN HÀNG TMCP QUÂN ĐỘI (MBBANK)\n\
Số tài khoản: 0880199887766\n\
Số dư đầu kỳ: 200.000.000\n\
Ngày,Số GD,Ghi có,Ghi nợ,Số dư,Nội dung\n\
15/08/2026,MB901,80.000.000,,280.000.000,Thu tien hop dong so 1\n\
16/08/2026,MB902,,30.000.000,250.000.000,Chi phi van phong pham\n\
Số dư cuối kỳ: 250.000.000\n";
    let mb = sniff_and_parse(mb_csv.as_bytes(), "mbbank_saoke.csv").unwrap();
    assert_eq!(mb.bank, BankType::MbBank);
    assert_eq!(mb.bank_code, "MB");
    assert_eq!(mb.format, StatementFormat::Csv);
    assert!(verify_balance_invariants(&mb).is_balanced);

    // 6. Agribank (HTML Table .xls export)
    let vba_html = "\
<html><head><meta charset=\"utf-8\"></head><body>\
<table>\
<tr><td colspan=\"6\">NGÂN HÀNG NÔNG NGHIỆP VÀ PHÁT TRIỂN NÔNG THÔN VIỆT NAM (AGRIBANK)</td></tr>\
<tr><td>Số tài khoản: 1500200889911</td></tr>\
<tr><td>Số dư đầu kỳ: 300.000.000</td></tr>\
<tr><th>Ngày GD</th><th>Mã GD</th><th>Diễn giải</th><th>Số tiền ghi nợ</th><th>Số tiền ghi có</th><th>Số dư</th></tr>\
<tr><td>10/08/2026</td><td>VB101</td><td>Thu tien cung cap vat tu</td><td></td><td>100.000.000</td><td>400.000.000</td></tr>\
<tr><td>11/08/2026</td><td>VB102</td><td>Phi dich vu ngan hang</td><td>10.000.000</td><td></td><td>390.000.000</td></tr>\
<tr><td>Số dư cuối kỳ: 390.000.000</td></tr>\
</table></body></html>";
    let vba = sniff_and_parse(vba_html.as_bytes(), "agribank_statement.xls").unwrap();
    assert_eq!(vba.bank, BankType::Agribank);
    assert_eq!(vba.bank_code, "VBA");
    assert_eq!(vba.format, StatementFormat::Html);
    assert!(verify_balance_invariants(&vba).is_balanced);
}
