//! Comprehensive test suite for liva-normalize crate.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use liva_money::Currency;
use liva_normalize::{
    extract_reference_codes, normalize_datetime, normalize_partner_name, parse_monetary_amount,
    strip_vietnamese_diacritics, verify_statement_balance, BankIdentifier, NormalizedAmount,
    NormalizedStatement, NormalizedTransaction,
};
use proptest::prelude::*;

#[test]
fn test_temporal_normalization_formats() {
    // 1. DD/MM/YYYY
    let d1 = normalize_datetime("15/08/2026").unwrap();
    assert_eq!(d1.iso_date, "2026-08-15");
    assert!(d1.epoch_seconds > 0);

    // 2. DD/MM/YYYY HH:mm:ss
    let d2 = normalize_datetime("15/08/2026 14:30:45").unwrap();
    assert_eq!(d2.iso_date, "2026-08-15T14:30:45+07:00");

    // 3. YYYY-MM-DD
    let d3 = normalize_datetime("2026-08-31").unwrap();
    assert_eq!(d3.iso_date, "2026-08-31");

    // 4. ISO 8601 full string
    let d4 = normalize_datetime("2026-09-01T08:00:00Z").unwrap();
    assert_eq!(d4.epoch_seconds, 1788249600);

    // 5. SWIFT MT940 YYMMDD
    let d5 = normalize_datetime("260801").unwrap();
    assert_eq!(d5.iso_date, "2026-08-01");

    // 6. Invalid date format
    assert!(normalize_datetime("invalid-date-string").is_err());
    assert!(normalize_datetime("").is_err());
}

#[test]
fn test_monetary_normalization_formats() {
    // 1. Vietnamese dot thousands and comma decimals
    let a1 = parse_monetary_amount("1.450.230.000,00").unwrap();
    assert_eq!(a1.minor_units, 1_450_230_000);
    assert!(!a1.is_negative);
    assert_eq!(a1.currency, Currency::VND);

    // 2. Dot thousands without decimal part
    let a2 = parse_monetary_amount("785.600.000").unwrap();
    assert_eq!(a2.minor_units, 785_600_000);

    // 3. US/UK format
    let a3 = parse_monetary_amount("1,250.50 USD").unwrap();
    assert_eq!(a3.minor_units, 125_050);
    assert_eq!(a3.currency, Currency::USD);

    // 4. Negative amounts
    let a4 = parse_monetary_amount("-205.000.000").unwrap();
    assert_eq!(a4.minor_units, 205_000_000);
    assert!(a4.is_negative);

    let a5 = parse_monetary_amount("(15.000.000)").unwrap();
    assert_eq!(a5.minor_units, 15_000_000);
    assert!(a5.is_negative);

    // 5. SWIFT MT940 trailing comma
    let a6 = parse_monetary_amount("11500000,").unwrap();
    assert_eq!(a6.minor_units, 11_500_000);

    // 6. Money conversion
    let money = a1.to_money().unwrap();
    assert_eq!(money.amount(), 1_450_230_000);
    assert_eq!(money.currency(), Currency::VND);

    let norm_back: NormalizedAmount = money.into();
    assert_eq!(norm_back.minor_units, 1_450_230_000);
}

#[test]
fn test_partner_name_folding_diacritics_and_prefixes() {
    // 1. NFC / NFD diacritic stripping
    let p1 = "Ngân hàng Thương mại Cổ phần Ngoại thương Việt Nam";
    assert_eq!(
        strip_vietnamese_diacritics(p1),
        "Ngan hang Thuong mai Co phan Ngoai thuong Viet Nam"
    );
    let f1 = normalize_partner_name(p1);
    assert_eq!(f1, "NGAN HANG THUONG MAI CO PHAN NGOAI THUONG VIET NAM");

    // 2. Complex company name with abbreviations
    let p2 = "Cty TNHH TMDV An Phát";
    let f2 = normalize_partner_name(p2);
    assert_eq!(f2, "CONG TY TNHH THUONG MAI DICH VU AN PHAT");

    // 3. Spaced TM DV
    let p3 = "Cty CP TM DV Đầu tư và Xây dựng Hòa Bình";
    let f3 = normalize_partner_name(p3);
    assert_eq!(
        f3,
        "CONG TY CO PHAN THUONG MAI DICH VU DAU TU VA XAY DUNG HOA BINH"
    );

    // 4. Trách nhiệm hữu hạn
    let p4 = "Công ty Trách nhiệm hữu hạn Thép Việt Nhật";
    let f4 = normalize_partner_name(p4);
    assert_eq!(f4, "CONG TY TNHH THEP VIET NHAT");

    // 5. Punctuation noise and symbols
    let p5 = "   *** CTY CP XNK HOANG GIA (VIETNAM) - CHI NHANH 1 ***   ";
    let f5 = normalize_partner_name(p5);
    assert_eq!(f5, "CONG TY CO PHAN XUAT NHAP KHAU HOANG GIA VIETNAM CHI NHANH 1");
}

#[test]
fn test_reference_tokenization_all_patterns() {
    // 1. Core banking FT and invoice
    let n1 = "Napas VietQR TT HD131 Tu Cong ty An Phat FT2624000001";
    let tok1 = extract_reference_codes(n1, None);
    assert!(tok1.contains(&"FT2624000001".to_string()));
    assert!(tok1.contains(&"HD131".to_string()));

    // 2. Interbank Napas and Debit order
    let n2 = "NPS26080001 chuyen khoan VN2624DEB01 thanh toan INV-9988";
    let tok2 = extract_reference_codes(n2, None);
    assert!(tok2.contains(&"NPS26080001".to_string()));
    assert!(tok2.contains(&"VN2624DEB01".to_string()));
    assert!(tok2.contains(&"HD9988".to_string()));

    // 3. Multi-invoice batch: "HD208 VA HD209"
    let n3 = "Napas VietQR TT HD208 VA HD209 TU CONG TY HOA BINH";
    let tok3 = extract_reference_codes(n3, None);
    assert!(tok3.contains(&"HD208".to_string()));
    assert!(tok3.contains(&"HD209".to_string()));

    // 4. Multi-invoice list: "HD210 HD211 HD212"
    let n4 = "Napas VietQR chuyen khoan thanh toan hoa don HD210 HD211 HD212 THEP VIET NHAT";
    let tok4 = extract_reference_codes(n4, None);
    assert!(tok4.contains(&"HD210".to_string()));
    assert!(tok4.contains(&"HD211".to_string()));
    assert!(tok4.contains(&"HD212".to_string()));

    // 5. Bank fee codes
    let n5 = "Phi quan ly tai khoan TCBFEE20260831";
    let tok5 = extract_reference_codes(n5, Some("TCBFEE20260831"));
    assert!(tok5.contains(&"TCBFEE20260831".to_string()));
}

#[test]
fn test_statement_balance_invariant_validation() {
    let stmt = NormalizedStatement {
        bank: BankIdentifier::Techcombank,
        account_no: "19034567890123".to_string(),
        opening_cents: 785_600_000,
        closing_cents: 2_246_754_900,
        transactions: vec![
            NormalizedTransaction {
                id: "tx-1".to_string(),
                date: "2026-08-01".to_string(),
                booking_date: None,
                voucher_no: None,
                amount_cents: 1_693_954_900,
                is_credit: true,
                balance_cents: 2_479_554_900,
                narration: "Credits total".to_string(),
                counterparty_name: None,
                reference_codes: vec![],
                tx_timestamp: 1785517200,
                value_timestamp: None,
            },
            NormalizedTransaction {
                id: "tx-2".to_string(),
                date: "2026-08-31".to_string(),
                booking_date: None,
                voucher_no: None,
                amount_cents: 232_800_000,
                is_credit: false,
                balance_cents: 2_246_754_900,
                narration: "Debits total".to_string(),
                counterparty_name: None,
                reference_codes: vec![],
                tx_timestamp: 1788109200,
                value_timestamp: None,
            },
        ],
    };

    let check = verify_statement_balance(&stmt).unwrap();
    assert!(check.is_valid);
    assert_eq!(check.total_credit_cents, 1_693_954_900);
    assert_eq!(check.total_debit_cents, 232_800_000);
    assert_eq!(check.calculated_closing_cents, 2_246_754_900);
}

proptest! {
    #[test]
    fn proptest_vietnamese_integer_amount_parsing(val in 0u64..1_000_000_000_000u64) {
        let s = format!("{val}");
        let parsed = parse_monetary_amount(&s).unwrap();
        prop_assert_eq!(parsed.minor_units, val);
        prop_assert!(!parsed.is_negative);
    }
}
