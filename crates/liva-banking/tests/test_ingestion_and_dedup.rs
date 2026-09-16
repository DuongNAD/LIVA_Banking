use liva_banking::models::{StatementStatus, TransactionType};
use liva_banking::parser::{ErpInvoiceParser, ParserError};
use liva_banking::service::IngestionService;

#[test]
fn test_tcb_csv_parsing_and_balance_invariant() {
    let csv_data = b"STT,Ng\xc3\xa0y GD,S\xe1\xbb\x91 tham chi\xe1\xba\xbfu,S\xe1\xbb\x91 ti\xe1\xbb\x81n r\xc3\xbat,S\xe1\xbb\x91 ti\xe1\xbb\x81n g\xe1\xbb\xadi,S\xe1\xbb\x91 d\xc6\xb0,N\xe1\xbb\x99i dung\n\
1,15/08/2026,FT26227001,0,50000000,150000000,Cong ty ABC thanh toan HD00102\n\
2,16/08/2026,FT26227002,15000000,0,135000000,Chi phi tiep khach thang 8\n";

    let mut ingestion = IngestionService::new();
    let mut stmt = ingestion
        .ingest_statement(csv_data, "tcb_statement_aug2026.csv")
        .expect("Should parse TCB CSV");

    assert_eq!(stmt.transactions.len(), 2);
    assert_eq!(stmt.transactions[0].amount, 50_000_000);
    assert_eq!(stmt.transactions[0].tx_type, TransactionType::Credit);
    assert_eq!(stmt.transactions[1].amount, 15_000_000);
    assert_eq!(stmt.transactions[1].tx_type, TransactionType::Debit);

    // Set opening and closing balance to test mathematical invariant
    // Opening = 100,000,000; Inflow = +50,000,000; Outflow = -15,000,000 -> Expected Closing = 135,000,000
    stmt.opening_balance = Some(100_000_000);
    stmt.closing_balance = Some(135_000_000);
    assert!(stmt.verify_balance_invariant());
    assert_eq!(stmt.status, StatementStatus::VerifiedBalanced);

    // If closing balance does not match, it must be quarantined
    stmt.closing_balance = Some(130_000_000); // 5,000,000 discrepancy!
    assert!(!stmt.verify_balance_invariant());
    assert_eq!(stmt.status, StatementStatus::QuarantinedUnbalanced);
}

#[test]
fn test_sha256_statement_deduplication() {
    let dummy_data = b"STT,Ng\xc3\xa0y GD,S\xe1\xbb\x91 tham chi\xe1\xba\xbfu,S\xe1\xbb\x91 ti\xe1\xbb\x81n r\xc3\xbat,S\xe1\xbb\x91 ti\xe1\xbb\x81n g\xe1\xbb\xadi,S\xe1\xbb\x91 d\xc6\xb0,N\xe1\xbb\x99i dung\n\
1,10/08/2026,REF001,0,10000000,10000000,Deposit\n";
    let mut ingestion = IngestionService::new();

    // First ingestion succeeds
    let res1 = ingestion.ingest_statement(dummy_data, "statement.csv");
    assert!(res1.is_ok());

    // Second ingestion with identical content must be rejected with DuplicateStatementFile (409 Conflict)
    let res2 = ingestion.ingest_statement(dummy_data, "statement_replay.csv");
    match res2 {
        Err(ParserError::DuplicateStatementFile { hash }) => {
            assert!(!hash.is_empty());
        }
        _ => panic!("Expected DuplicateStatementFile error on duplicate upload"),
    }
}

#[test]
fn test_erp_invoice_csv_parser() {
    let erp_csv = b"Ch\xe1\xbb\xa9ng t\xe1\xbb\xab,H\xc3\xb3a \xc4\x91\xc6\xa1n,M\xc3\xa3 KH,T\xc3\xaan KH,Ng\xc3\xa0y H\xc4\x90,T\xe1\xbb\x95ng ti\xe1\xbb\x81n,C\xc3\xb2n l\xe1\xba\xa1i,Di\xe1\xbb\x85n gi\xe1\xba\xa3i\n\
CT001,HD00102,KH_ABC,Cong ty TNHH ABC,15/08/2026,50.000.000,50.000.000,Tien hang thang 8\n\
CT002,HD00103,KH_XYZ,Cong ty XYZ,16/08/2026,30.000.000,20.000.000,Dich vu phan mem\n";

    let docs = ErpInvoiceParser::parse_csv(erp_csv).expect("Should parse ERP CSV");
    assert_eq!(docs.len(), 2);

    assert_eq!(docs[0].voucher_no, "CT001");
    assert_eq!(docs[0].invoice_no.as_deref(), Some("HD00102"));
    assert_eq!(docs[0].partner_name, "Cong ty TNHH ABC");
    assert_eq!(docs[0].total_amount, 50_000_000);
    assert_eq!(docs[0].open_amount, 50_000_000);

    assert_eq!(docs[1].voucher_no, "CT002");
    assert_eq!(docs[1].open_amount, 20_000_000);
}
