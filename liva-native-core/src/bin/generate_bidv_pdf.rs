//! Generates a realistic BIDV PDF Statement fixture (40 transactions)
//! fully compatible with `bidv_pdf.rs` in `liva-native-core`.

use lopdf::content::{Content, Operation};
use lopdf::{Dictionary, Document, Object, Stream, StringFormat};
use std::fs;
use std::path::Path;

fn fmt_vnd(amount: u64) -> String {
    let s = amount.to_string();
    let mut result = String::new();
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating bidv_aug2026.pdf fixture...");

    let output_dir = Path::new("fixtures/statements");
    fs::create_dir_all(output_dir)?;
    let output_path = output_dir.join("bidv_aug2026.pdf");

    let mut doc = Document::with_version("1.4");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", "Font".into()),
        ("Subtype", "Type1".into()),
        ("BaseFont", "Helvetica".into()),
    ]));

    let resources_id = doc.add_object(Dictionary::from_iter(vec![(
        "Font",
        Dictionary::from_iter(vec![("F1", font_id.into())]).into(),
    )]));

    let mut ops: Vec<Operation> = Vec::new();
    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec!["F1".into(), 10.into()]));

    let mut emit_line = |x: f32, y: f32, tokens: &[&str]| {
        let mut cur_x = x;
        for &tok in tokens {
            ops.push(Operation::new(
                "Tm",
                vec![
                    1.into(),
                    0.into(),
                    0.into(),
                    1.into(),
                    cur_x.into(),
                    y.into(),
                ],
            ));
            ops.push(Operation::new(
                "Tj",
                vec![Object::String(
                    tok.as_bytes().to_vec(),
                    StringFormat::Literal,
                )],
            ));
            cur_x += (tok.len() as f32) * 6.5 + 10.0;
        }
    };

    let opening_balance: u64 = 520_000_000;
    let mut running_balance: u64 = opening_balance;
    let mut sum_credits: u64 = 0;
    let mut sum_debits: u64 = 0;

    let mut y = 1450.0;

    // 1. Metadata Block
    emit_line(
        50.0,
        y,
        &["NGAN HANG TMCP DAU TU VA PHAT TRIEN VIET NAM (BIDV)"],
    );
    y -= 20.0;
    emit_line(50.0, y, &["SAO KE TAI KHOAN TIEN GUI DOANH NGHIEP"]);
    y -= 18.0;
    emit_line(50.0, y, &["Số tài khoản: 12410001234567"]);
    y -= 16.0;
    emit_line(50.0, y, &["Tên khách hàng: CONG TY TNHH LIVA SOLUTIONS"]);
    y -= 16.0;
    emit_line(
        50.0,
        y,
        &["Ky sao ke: 01/08/2026 den 31/08/2026 - Tien te: VND"],
    );
    y -= 16.0;
    emit_line(50.0, y, &["Số dư đầu kỳ:", &fmt_vnd(opening_balance)]);
    y -= 25.0;

    // 2. Table Header Line
    emit_line(
        50.0,
        y,
        &[
            "Ngày GD",
            "Chứng từ",
            "Số tiền ghi nợ",
            "Số tiền ghi có",
            "Số dư",
            "Nội dung giao dịch",
        ],
    );
    y -= 18.0;

    // 3. Transactions Specification (40 transactions)
    // 18 Exact Matches (HD165 to HD182)
    let exact_amounts: Vec<(u64, &str, &str)> = vec![
        (13500000, "HD165", "CONG TY TNHH LOGISTICS BIEN DONG"),
        (21500000, "HD166", "CONG TY CP TAP DOAN MASAN"),
        (33500000, "HD167", "CONG TY TNHH THEP VIET NHAT"),
        (41500000, "HD168", "CONG TY CP HOA CHAT DUC GIANG"),
        (54000000, "HD169", "CONG TY TNHH XNK HOANG GIA"),
        (16500000, "HD170", "CONG TY CP THIET BI Y TE PHUONG DONG"),
        (29000000, "HD171", "CONG TY TNHH DET MAY PHONG PHU"),
        (39000000, "HD172", "CONG TY CP NHUA TIEN PHONG"),
        (51500000, "HD173", "CONG TY TNHH NONG SAN TRUNG AN"),
        (22500000, "HD174", "CONG TY CP TAP DOAN DEO CA"),
        (17000000, "HD175", "CONG TY TNHH CO DIEN LANH DAI VIET"),
        (26000000, "HD176", "CONG TY TNHH TM DV AN PHAT"),
        (11500000, "HD177", "CONG TY CP MASAN HIGH TECH"),
        (23000000, "HD178", "CONG TY TNHH DIEN CO THONG NHAT"),
        (32500000, "HD179", "CONG TY CP TONG CONG TY MAY 10"),
        (44000000, "HD180", "CONG TY TNHH SX TM TAN A DAI THANH"),
        (57000000, "HD181", "CONG TY CP RANG DONG"),
        (14000000, "HD182", "CONG TY TNHH PHU THAI CAT"),
    ];

    let mut tx_count = 0;

    // A. Emit 18 Exact Matches
    for (i, (amt, doc, cp)) in exact_amounts.into_iter().enumerate() {
        tx_count += 1;
        sum_credits += amt;
        running_balance += amt;
        let day = 1 + (i % 28);
        let date_str = format!("{day:02}/08/2026");
        let ref_str = format!("FT26214589{:02}", i + 1);
        let amt_str = fmt_vnd(amt);
        let bal_str = fmt_vnd(running_balance);
        let narr_str = format!("BIDV TT TIEN HANG {doc} TU {cp}");

        emit_line(
            50.0,
            y,
            &[&date_str, &ref_str, &amt_str, &bal_str, &narr_str],
        );
        y -= 14.0;

        // Add multi-line narration on some transactions
        if i % 4 == 0 {
            emit_line(
                120.0,
                y,
                &["Chi tiet: Hop dong thiet bi cong nghe dot 2 nam 2026"],
            );
            y -= 14.0;
        }
    }

    // B. Emit 5 Fuzzy Matches (HD198 to HD202)
    let fuzzy_spec: Vec<(u64, u64, &str, &str)> = vec![
        (62000000, 2200, "HD198", "CONG TY CP DET MAY PHONG PHU"),
        (44000000, 3300, "HD199", "CONG TY CP NHUA TIEN PHONG"),
        (46000000, 5500, "HD200", "CONG TY TNHH NONG SAN TRUNG AN"),
        (54000000, 7700, "HD201", "CONG TY CP TAP DOAN DEO CA"),
        (
            68000000,
            11000,
            "HD202",
            "CONG TY TNHH CO DIEN LANH DAI VIET",
        ),
    ];

    for (i, (base_amt, fee, doc, cp)) in fuzzy_spec.into_iter().enumerate() {
        tx_count += 1;
        let net_amt = base_amt - fee;
        sum_credits += net_amt;
        running_balance += net_amt;
        let day = 3 + i * 4;
        let date_str = format!("{day:02}/08/2026");
        let ref_str = format!("FT26214599{:02}", i + 1);
        let amt_str = fmt_vnd(net_amt);
        let bal_str = fmt_vnd(running_balance);
        let narr_str = format!("BIDV CK TT {doc} DA TRU PHI GD TU {cp}");

        emit_line(
            50.0,
            y,
            &[&date_str, &ref_str, &amt_str, &bal_str, &narr_str],
        );
        y -= 14.0;
    }

    // C. Emit 2 Composite Split Matches
    // Split E: HD213 (30M) + HD214 (30M) + HD215 (30M) = 90M
    {
        tx_count += 1;
        let split_e_amt = 90_000_000u64;
        sum_credits += split_e_amt;
        running_balance += split_e_amt;
        let date_str = "14/08/2026";
        let ref_str = "FT2621889901";
        let amt_str = fmt_vnd(split_e_amt);
        let bal_str = fmt_vnd(running_balance);
        let narr_str = "BIDV TT TIEN HANG HD213 HD214 HD215 CTY DAI VIET";
        emit_line(50.0, y, &[date_str, ref_str, &amt_str, &bal_str, narr_str]);
        y -= 14.0;
        emit_line(
            120.0,
            y,
            &["Ghi chu: Thanh toan tong hop 3 don hang thang 8"],
        );
        y -= 14.0;
    }

    // Split F: HD216 (35M) + HD217 (30M) = 65M
    {
        tx_count += 1;
        let split_f_amt = 65_000_000u64;
        sum_credits += split_f_amt;
        running_balance += split_f_amt;
        let date_str = "20/08/2026";
        let ref_str = "FT2621889902";
        let amt_str = fmt_vnd(split_f_amt);
        let bal_str = fmt_vnd(running_balance);
        let narr_str = "BIDV CK HD216 VA HD217 CONG TY HUY HOANG";
        emit_line(50.0, y, &[date_str, ref_str, &amt_str, &bal_str, narr_str]);
        y -= 14.0;
    }

    // D. Emit 1 Discrepancy Credit (1,250,000 VND - Fails closed to HITL)
    {
        tx_count += 1;
        let disc_amt = 1_250_000u64;
        sum_credits += disc_amt;
        running_balance += disc_amt;
        let date_str = "25/08/2026";
        let ref_str = "FT2621999999";
        let amt_str = fmt_vnd(disc_amt);
        let bal_str = fmt_vnd(running_balance);
        let narr_str = "Tien chuyen khoan nham tai khoan chua xac dinh nguon goc";
        emit_line(50.0, y, &[date_str, ref_str, &amt_str, &bal_str, narr_str]);
        y -= 14.0;
    }

    // E. Emit 14 Operational Debits
    let debits_spec: Vec<(u64, &str)> = vec![
        (25000000, "THANH TOAN TIEN THUE VAN PHONG BIDV TOWER"),
        (15000000, "CHI PHI DAO TAO NOI BO THANG 8 NGUON NHAN LUC"),
        (8000000, "CHI PHI KHAM SUC KHOE DINH KY NHAN VIEN"),
        (12500000, "THANH TOAN NHA CUNG CAP MUC IN VA THIET BI"),
        (30000000, "CHI PHI MARKETING VA QUANG CAO DU AN INNOSTART"),
        (7500000, "CHI PHI DONG PHUC NHAN VIEN CONG TY"),
        (18000000, "CHI PHI BAO DUONG HE THONG MAY LANH TOA NHA"),
        (4000000, "PHI DICH VU TAI KHOAN DOANH NGHIEP BIDV"),
        (20000000, "THANH TOAN HOA DON TIEP KHACH HOI NGHỊ"),
        (11000000, "CHI PHI VAN CHUYEN HANG HOA VA TIEP VAN"),
        (6500000, "CHI PHI DICH THUAT HO SO PHAP LY SANDBOX"),
        (2500000, "PHI KIEM TOAN DOC LAP NGAN HANG"),
        (14500000, "CHI PHI DANG KY BAN QUYEN SANG CHE AI"),
        (16500000, "CHI PHI THUE CHUYEN GIA AN TOAN THONG TIN"),
    ];

    for (idx, (amt, narr)) in debits_spec.into_iter().enumerate() {
        tx_count += 1;
        sum_debits += amt;
        running_balance -= amt;
        let day = 2 + idx * 2;
        let date_str = format!("{day:02}/08/2026");
        let ref_str = format!("FT2621DEB{:02}", idx + 1);
        let amt_str = format!("-{}", fmt_vnd(amt)); // Debit indicated by minus sign
        let bal_str = fmt_vnd(running_balance);

        emit_line(50.0, y, &[&date_str, &ref_str, &amt_str, &bal_str, narr]);
        y -= 14.0;
    }

    assert_eq!(tx_count, 40, "Expected exactly 40 BIDV transactions");

    // 4. Summary and Balance Checksum Footer
    y -= 15.0;
    emit_line(
        50.0,
        y,
        &[
            "Tổng phát sinh trong kỳ:",
            "Ghi nợ:",
            &fmt_vnd(sum_debits),
            "Ghi có:",
            &fmt_vnd(sum_credits),
        ],
    );
    y -= 16.0;
    emit_line(50.0, y, &["Số dư cuối kỳ:", &fmt_vnd(running_balance)]);

    ops.push(Operation::new("ET", vec![]));

    let content = Content { operations: ops };
    let content_bytes = content.encode()?;
    let content_id = doc.add_object(Stream::new(Dictionary::new(), content_bytes));

    let page_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", "Page".into()),
        ("Parent", pages_id.into()),
        ("Contents", content_id.into()),
    ]));

    let pages_dict = Dictionary::from_iter(vec![
        ("Type", "Pages".into()),
        ("Kids", vec![page_id.into()].into()),
        ("Count", 1.into()),
        ("Resources", resources_id.into()),
        (
            "MediaBox",
            vec![0.into(), 0.into(), 595.into(), 1600.into()].into(),
        ),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", "Catalog".into()),
        ("Pages", pages_id.into()),
    ]));
    doc.trailer.set("Root", catalog_id);

    doc.save(&output_path)?;

    println!(
        "[OK] Generated 40 BIDV transactions in {}",
        output_path.display()
    );
    println!(
        "     Opening: 520,000,000 | Credits: {sum_credits} | Debits: {sum_debits} | Closing: {running_balance}"
    );
    println!(
        "     Checksum invariant: {} + {} - {} == {} -> {}",
        opening_balance,
        sum_credits,
        sum_debits,
        running_balance,
        opening_balance + sum_credits - sum_debits == running_balance
    );

    Ok(())
}
