//! Statement format and bank dialect sniffer.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::models::{ContainerFormat, StatementSniffer};
use liva_normalize::BankIdentifier;

/// Default autonomous statement sniffer.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultSniffer;

impl StatementSniffer for DefaultSniffer {
    fn sniff_container(&self, bytes: &[u8], filename: &str) -> ContainerFormat {
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

        let prefix_len = bytes.len().min(2048);
        let prefix = String::from_utf8_lossy(&bytes[..prefix_len]).to_lowercase();

        // 4. SWIFT MT940 statement message
        if prefix.contains("{1:f01")
            || prefix.contains("{2:i940")
            || (prefix.contains(":20:") && (prefix.contains(":60f:") || prefix.contains(":61:")))
        {
            return ContainerFormat::SwiftMt;
        }

        // 5. ISO 20022 CAMT.053 XML / Financial XML
        if prefix.contains("camt.053")
            || prefix.contains("bktocstmrstmt")
            || (prefix.contains("<?xml") && !prefix.contains("<html"))
            || (prefix.contains("<document") && prefix.contains("iso:20022"))
        {
            return ContainerFormat::Xml;
        }

        // 6. HTML Table disguised as spreadsheet (<html, <!doctype, <table)
        if prefix.contains("<html")
            || prefix.contains("<!doctype html")
            || prefix.contains("<table")
            || (prefix.contains("<tr") && prefix.contains("<td"))
        {
            return ContainerFormat::HtmlTable;
        }

        // 7. UTF-8 or UTF-16 BOM text detection
        if bytes.starts_with(&[0xEF, 0xBB, 0xBF])
            || bytes.starts_with(&[0xFF, 0xFE])
            || bytes.starts_with(&[0xFE, 0xFF])
        {
            return ContainerFormat::TextCsv;
        }

        // 8. Filename extension fallback
        let lower_name = filename.to_lowercase();
        if lower_name.ends_with(".pdf") {
            ContainerFormat::Pdf
        } else if lower_name.ends_with(".xlsx") {
            ContainerFormat::ExcelZip
        } else if lower_name.ends_with(".xml")
            || lower_name.ends_with(".camt")
            || lower_name.ends_with(".camt053")
        {
            ContainerFormat::Xml
        } else if lower_name.ends_with(".xls") {
            if prefix.contains("<table") || prefix.contains("<tr") {
                ContainerFormat::HtmlTable
            } else {
                ContainerFormat::ExcelOle
            }
        } else if lower_name.ends_with(".csv")
            || lower_name.ends_with(".txt")
            || lower_name.ends_with(".tsv")
        {
            if prefix.contains(":20:") && prefix.contains(":60f:") {
                ContainerFormat::SwiftMt
            } else {
                ContainerFormat::TextCsv
            }
        } else {
            // Text heuristics
            let sample = &bytes[..bytes.len().min(512)];
            let non_printable = sample
                .iter()
                .filter(|&&b| b < 0x09 || (b > 0x0D && b < 0x20))
                .count();
            if non_printable == 0 && !sample.is_empty() {
                ContainerFormat::TextCsv
            } else {
                ContainerFormat::Unknown
            }
        }
    }

    fn sniff_bank(&self, bytes: &[u8], filename: &str) -> BankIdentifier {
        let lower_fn = filename.to_lowercase();
        let head_len = bytes.len().min(8192);
        let head = String::from_utf8_lossy(&bytes[..head_len]).to_uppercase();

        let mut score_vcb: i32 = if lower_fn.contains("vcb") || lower_fn.contains("vietcombank") {
            10
        } else {
            0
        };
        let mut score_tcb: i32 = if lower_fn.contains("tcb") || lower_fn.contains("techcombank") {
            10
        } else {
            0
        };
        let mut score_bidv: i32 = if lower_fn.contains("bidv") { 10 } else { 0 };
        let mut score_ctg: i32 = if lower_fn.contains("ctg")
            || lower_fn.contains("vietinbank")
            || lower_fn.contains("icb")
        {
            10
        } else {
            0
        };
        let mut score_mb: i32 = if lower_fn.contains("mbbank")
            || lower_fn.contains("mb_")
            || lower_fn.contains("mb-")
        {
            10
        } else {
            0
        };
        let mut score_vba: i32 = if lower_fn.contains("agribank")
            || lower_fn.contains("vba")
            || lower_fn.contains("nongnghiep")
        {
            10
        } else {
            0
        };
        let mut score_iso: i32 = if lower_fn.contains("camt") || lower_fn.contains("iso20022") {
            10
        } else {
            0
        };
        let mut score_swift: i32 = if lower_fn.contains("mt940") || lower_fn.contains("swift") {
            10
        } else {
            0
        };

        // Textual content scoring
        if head.contains("VIETCOMBANK") || head.contains("NGOAI THUONG") {
            score_vcb += 15;
        }
        if head.contains("TECHCOMBANK") || head.contains("KY THUONG") || head.contains("F@ST EBANK")
        {
            score_tcb += 15;
        }
        if head.contains("BIDV") || head.contains("DAU TU VA PHAT TRIEN") {
            score_bidv += 15;
        }
        if head.contains("VIETINBANK") || head.contains("CONG THUONG") || head.contains("EFAST") {
            score_ctg += 15;
        }
        if head.contains("MBBANK") || head.contains("QUAN DOI") || head.contains("MB BIZ") {
            score_mb += 15;
        }
        if head.contains("AGRIBANK")
            || head.contains("NONG NGHIEP")
            || head.contains("PHAT TRIEN NONG THON")
        {
            score_vba += 15;
        }
        if head.contains("CAMT.053") || head.contains("BKTOCSTMRSTMT") {
            score_iso += 20;
        }
        if head.contains("I940") || (head.contains(":20:") && head.contains(":60F:")) {
            score_swift += 20;
        }

        let scores = [
            (BankIdentifier::Iso20022, score_iso),
            (BankIdentifier::Swift, score_swift),
            (BankIdentifier::Vietcombank, score_vcb),
            (BankIdentifier::Techcombank, score_tcb),
            (BankIdentifier::Bidv, score_bidv),
            (BankIdentifier::VietinBank, score_ctg),
            (BankIdentifier::MbBank, score_mb),
            (BankIdentifier::Agribank, score_vba),
        ];

        scores
            .into_iter()
            .filter(|(_, s)| *s > 0)
            .max_by_key(|(_, s)| *s)
            .map(|(b, _)| b)
            .unwrap_or(BankIdentifier::Unknown)
    }
}
