//! Bank statement parsing module for Vietnamese Banks:
//! - Big 4: Vietcombank (VCB), VietinBank (CTG), BIDV, Agribank (VBA)
//! - Joint-stock commercial banks: Techcombank (TCB), MBBank (MB)
//!
//! Features:
//! - Container detection via Magic Bytes (Office Open XML / XLSX, OLE / XLS, PDF, HTML, CSV / Plaintext).
//! - Semantic header and signature scoring for bank detection.
//! - 4-phase intelligent `sniff_and_parse` dispatcher.

pub mod agribank_parser;
pub mod bidv_pdf;
pub mod iso20022_xml;
pub mod mbbank_parser;
pub mod tcb_csv;
pub mod vcb_excel;
pub mod vietinbank_parser;

pub use agribank_parser::AgribankParser;
pub use bidv_pdf::BidvPdfParser;
pub use iso20022_xml::Iso20022XmlParser;
pub use mbbank_parser::MbBankParser;
pub use tcb_csv::TcbCsvParser;
pub use vcb_excel::VcbExcelParser;
pub use vietinbank_parser::VietinBankParser;

use crate::banking::models::{BankType, ParsedStatement, StatementFormat};
use std::fmt;

#[derive(Debug, Clone)]
pub enum ParserError {
    UnsupportedFormat { filename: String },
    ExcelError(String),
    CsvError(String),
    PdfError(String),
    IoError(String),
    InvalidStructure(String),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::UnsupportedFormat { filename } => {
                write!(
                    f,
                    "Unsupported or unrecognized statement format for file: {filename}"
                )
            }
            ParserError::ExcelError(msg) => write!(f, "Excel parser error: {msg}"),
            ParserError::CsvError(msg) => write!(f, "CSV parser error: {msg}"),
            ParserError::PdfError(msg) => write!(f, "PDF parser error: {msg}"),
            ParserError::IoError(msg) => write!(f, "I/O error: {msg}"),
            ParserError::InvalidStructure(msg) => write!(f, "Invalid statement structure: {msg}"),
        }
    }
}

impl std::error::Error for ParserError {}

/// Core trait for Bank Statement Parsers.
pub trait BankStatementParser: Send + Sync {
    /// Sniffs bytes and filename to determine if this parser can handle the statement.
    fn sniff(&self, bytes: &[u8], filename: &str) -> bool;

    /// Parses the raw statement bytes into a standardized `ParsedStatement`.
    fn parse(&self, bytes: &[u8], filename: &str) -> Result<ParsedStatement, ParserError>;
}

/// Container format detected from magic bytes or file structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerFormat {
    Pdf,
    ExcelZip,  // XLSX (Office Open XML, ZIP header)
    ExcelOle,  // XLS (OLE Compound File)
    HtmlTable, // Agribank HTML table exported as .xls
    TextCsv,   // CSV / TSV / Delimited Plaintext
    Xml,       // ISO 20022 camt.053 XML / Open Banking XML
    Unknown,
}

impl ContainerFormat {
    pub fn to_statement_format(&self) -> StatementFormat {
        match self {
            ContainerFormat::Pdf => StatementFormat::Pdf,
            ContainerFormat::ExcelZip | ContainerFormat::ExcelOle => StatementFormat::Excel,
            ContainerFormat::HtmlTable => StatementFormat::Html,
            ContainerFormat::TextCsv => StatementFormat::Csv,
            ContainerFormat::Xml => StatementFormat::Xml,
            ContainerFormat::Unknown => StatementFormat::Unknown,
        }
    }
}

/// Phase 1: Detect container format using magic bytes and structural heuristics.
pub fn detect_container_format(bytes: &[u8], filename: &str) -> ContainerFormat {
    // 1. PDF magic bytes (%PDF-)
    if bytes.starts_with(b"%PDF-") {
        return ContainerFormat::Pdf;
    }

    // 2. ZIP / XLSX magic bytes (PK\x03\x04)
    if bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
        return ContainerFormat::ExcelZip;
    }

    // 3. OLE Compound Document magic bytes (\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1)
    if bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]) {
        return ContainerFormat::ExcelOle;
    }

    let prefix_len = bytes.len().min(1024);
    let prefix = String::from_utf8_lossy(&bytes[..prefix_len]).to_lowercase();

    // 4. ISO 20022 XML / Generic XML
    if prefix.contains("camt.053")
        || prefix.contains("bktocstmrstmt")
        || (prefix.contains("<?xml") && !prefix.contains("<html"))
        || (prefix.contains("<document") && prefix.contains("iso:20022"))
    {
        return ContainerFormat::Xml;
    }

    // 5. HTML Table magic (<html, <!doctype, <table)
    if prefix.contains("<html") || prefix.contains("<!doctype html") || prefix.contains("<table") {
        return ContainerFormat::HtmlTable;
    }

    // 6. UTF-8 or UTF-16 BOM text detection
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF])
        || bytes.starts_with(&[0xFF, 0xFE])
        || bytes.starts_with(&[0xFE, 0xFF])
    {
        return ContainerFormat::TextCsv;
    }

    // 7. Filename extension fallbacks
    let lower_name = filename.to_lowercase();
    if lower_name.ends_with(".pdf") {
        ContainerFormat::Pdf
    } else if lower_name.ends_with(".xlsx") {
        ContainerFormat::ExcelZip
    } else if lower_name.ends_with(".xml") || lower_name.ends_with(".camt") || lower_name.ends_with(".camt053") {
        ContainerFormat::Xml
    } else if lower_name.ends_with(".xls") {
        // Agribank often exports HTML table as .xls without standard OLE header
        if prefix.contains("<table") || prefix.contains("<tr") {
            ContainerFormat::HtmlTable
        } else {
            ContainerFormat::ExcelOle
        }
    } else if lower_name.ends_with(".csv")
        || lower_name.ends_with(".txt")
        || lower_name.ends_with(".tsv")
    {
        ContainerFormat::TextCsv
    } else {
        // Inspect if bytes look like text (printable ASCII or valid UTF-8)
        let sample = &bytes[..bytes.len().min(512)];
        let non_printable = sample
            .iter()
            .filter(|&&b| b < 0x09 || (b > 0x0D && b < 0x20))
            .count();
        if non_printable == 0 && sample.len() > 0 {
            ContainerFormat::TextCsv
        } else {
            ContainerFormat::Unknown
        }
    }
}

/// Phase 2: Semantic Bank Signature Scoring.
/// Inspects file contents and filename for Vietnamese banking signatures.
pub fn score_bank(bytes: &[u8], filename: &str) -> Option<BankType> {
    let lower_filename = filename.to_lowercase();

    // Check filename tokens first
    let fn_vcb = lower_filename.contains("vcb") || lower_filename.contains("vietcombank");
    let fn_tcb = lower_filename.contains("tcb") || lower_filename.contains("techcombank");
    let fn_bidv = lower_filename.contains("bidv");
    let fn_ctg = lower_filename.contains("ctg")
        || lower_filename.contains("vietinbank")
        || lower_filename.contains("icb");
    let fn_mb = lower_filename.contains("mbbank")
        || lower_filename.contains("mb_")
        || lower_filename.contains("mb-");
    let fn_vba = lower_filename.contains("agribank")
        || lower_filename.contains("vba")
        || lower_filename.contains("nongnghiep");

    // Scan the first 8KB of file content as text
    let head_len = bytes.len().min(8192);
    let head_text = String::from_utf8_lossy(&bytes[..head_len]).to_uppercase();

    let mut score_vcb = if fn_vcb { 5 } else { 0 };
    let mut score_tcb = if fn_tcb { 5 } else { 0 };
    let mut score_bidv = if fn_bidv { 5 } else { 0 };
    let mut score_ctg = if fn_ctg { 5 } else { 0 };
    let mut score_mb = if fn_mb { 5 } else { 0 };
    let mut score_vba = if fn_vba { 5 } else { 0 };

    // Vietcombank signatures
    if head_text.contains("VIETCOMBANK") || head_text.contains("NGOAI THUONG") {
        score_vcb += 10;
    }
    if head_text.contains("SO TK / ACCOUNT NO") || head_text.contains("SAO KE TAI KHOAN") {
        score_vcb += 3;
    }

    // Techcombank signatures
    if head_text.contains("TECHCOMBANK") || head_text.contains("KY THUONG") {
        score_tcb += 10;
    }
    if head_text.contains("TIEN GUI THANH TOAN") || head_text.contains("TKTT") {
        score_tcb += 3;
    }

    // BIDV signatures
    if head_text.contains("BIDV") || head_text.contains("DAU TU VA PHAT TRIEN") {
        score_bidv += 10;
    }
    if head_text.contains("NGAY GIAO DICH / TRANS DATE") || head_text.contains("NGAY GD") {
        score_bidv += 3;
    }

    // VietinBank signatures
    if head_text.contains("VIETINBANK") || head_text.contains("CONG THUONG") {
        score_ctg += 10;
    }
    if head_text.contains("SO PHIEU")
        || head_text.contains("PHIEU THU")
        || head_text.contains("PHIEU CHI")
    {
        score_ctg += 3;
    }

    // MBBank signatures
    if head_text.contains("MBBANK")
        || head_text.contains("MB BANK")
        || head_text.contains("QUAN DOI")
    {
        score_mb += 10;
    }
    if head_text.contains("TAI KHOAN NGUOI THU HUONG") || head_text.contains("NGAN HANG THU HUONG")
    {
        score_mb += 4;
    }

    // Agribank signatures
    if head_text.contains("AGRIBANK")
        || head_text.contains("NONG NGHIEP")
        || head_text.contains("PHAT TRIEN NONG THON")
    {
        score_vba += 10;
    }
    if head_text.contains("SO DU DAU KY") || head_text.contains("SO DU CUOI KY") {
        score_vba += 2;
    }

    let scores = [
        (BankType::Vietcombank, score_vcb),
        (BankType::Techcombank, score_tcb),
        (BankType::Bidv, score_bidv),
        (BankType::VietinBank, score_ctg),
        (BankType::MbBank, score_mb),
        (BankType::Agribank, score_vba),
    ];

    scores
        .into_iter()
        .filter(|(_, s)| *s > 0)
        .max_by_key(|(_, s)| *s)
        .map(|(b, _)| b)
}

/// 4-Phase Dispatcher:
/// Phase 1: Container magic bytes detection.
/// Phase 2: Semantic bank signature scoring.
/// Phase 3: Candidate parser ranking.
/// Phase 4: Execution with graceful fallback.
pub fn sniff_and_parse(bytes: &[u8], filename: &str) -> Result<ParsedStatement, ParserError> {
    let container = detect_container_format(bytes, filename);
    let bank_hint = score_bank(bytes, filename);

    let vcb = vcb_excel::VcbExcelParser;
    let tcb = tcb_csv::TcbCsvParser;
    let bidv = bidv_pdf::BidvPdfParser;
    let ctg = vietinbank_parser::VietinBankParser;
    let mb = mbbank_parser::MbBankParser;
    let vba = agribank_parser::AgribankParser;
    let iso_xml = iso20022_xml::Iso20022XmlParser;

    // Phase 3: Rank candidates based on container and bank score
    let mut candidates: Vec<&dyn BankStatementParser> = Vec::new();

    match container {
        ContainerFormat::Xml => {
            candidates.push(&iso_xml);
        }
        ContainerFormat::Pdf => match bank_hint {
            Some(BankType::Bidv) => {
                candidates.push(&bidv);
                candidates.push(&ctg);
            }
            Some(BankType::VietinBank) => {
                candidates.push(&ctg);
                candidates.push(&bidv);
            }
            _ => {
                candidates.push(&bidv);
                candidates.push(&ctg);
            }
        },
        ContainerFormat::ExcelZip | ContainerFormat::ExcelOle => match bank_hint {
            Some(BankType::Vietcombank) => {
                candidates.push(&vcb);
                candidates.push(&ctg);
                candidates.push(&mb);
                candidates.push(&vba);
            }
            Some(BankType::VietinBank) => {
                candidates.push(&ctg);
                candidates.push(&vcb);
                candidates.push(&mb);
                candidates.push(&vba);
            }
            Some(BankType::MbBank) => {
                candidates.push(&mb);
                candidates.push(&vcb);
                candidates.push(&ctg);
                candidates.push(&vba);
            }
            Some(BankType::Agribank) => {
                candidates.push(&vba);
                candidates.push(&vcb);
                candidates.push(&ctg);
                candidates.push(&mb);
            }
            _ => {
                candidates.push(&vcb);
                candidates.push(&ctg);
                candidates.push(&mb);
                candidates.push(&vba);
            }
        },
        ContainerFormat::HtmlTable => {
            candidates.push(&vba);
            candidates.push(&vcb);
        }
        ContainerFormat::TextCsv => match bank_hint {
            Some(BankType::Techcombank) => {
                candidates.push(&tcb);
                candidates.push(&mb);
                candidates.push(&ctg);
                candidates.push(&vba);
            }
            Some(BankType::MbBank) => {
                candidates.push(&mb);
                candidates.push(&tcb);
                candidates.push(&ctg);
                candidates.push(&vba);
            }
            Some(BankType::VietinBank) => {
                candidates.push(&ctg);
                candidates.push(&tcb);
                candidates.push(&mb);
                candidates.push(&vba);
            }
            Some(BankType::Agribank) => {
                candidates.push(&vba);
                candidates.push(&tcb);
                candidates.push(&mb);
                candidates.push(&ctg);
            }
            _ => {
                candidates.push(&tcb);
                candidates.push(&mb);
                candidates.push(&ctg);
                candidates.push(&vba);
            }
        },
        ContainerFormat::Unknown => {
            // Test all parsers based on sniff()
            candidates.push(&iso_xml);
            candidates.push(&vcb);
            candidates.push(&tcb);
            candidates.push(&bidv);
            candidates.push(&ctg);
            candidates.push(&mb);
            candidates.push(&vba);
        }
    }

    // Phase 4: Execution with graceful fallback
    // 1. Try parsers that explicitly sniff positive first
    for parser in &candidates {
        if parser.sniff(bytes, filename) {
            if let Ok(statement) = parser.parse(bytes, filename) {
                return Ok(statement);
            }
        }
    }

    // 2. Try candidates even if sniff returned false (lenient fallback)
    for parser in &candidates {
        if let Ok(statement) = parser.parse(bytes, filename) {
            return Ok(statement);
        }
    }

    // 3. Last-resort fallback: try all parsers
    let all_parsers: [&dyn BankStatementParser; 7] = [&iso_xml, &vcb, &tcb, &bidv, &ctg, &mb, &vba];
    for parser in &all_parsers {
        if let Ok(statement) = parser.parse(bytes, filename) {
            return Ok(statement);
        }
    }

    Err(ParserError::UnsupportedFormat {
        filename: filename.to_string(),
    })
}
