//! Container format detection and template matching score evaluator.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use crate::template::schema::{ContainerFormat, StatementTemplate};
use regex::Regex;

/// Detects the underlying physical container format of a raw file payload.
pub fn detect_container_format(raw: &[u8], filename: &str) -> ContainerFormat {
    let lower_name = filename.to_lowercase();

    // Check magic bytes
    if raw.len() >= 4 {
        if raw[0..4] == [0x50, 0x4B, 0x03, 0x04] {
            return ContainerFormat::ExcelZip;
        }
        if raw.len() >= 8 && raw[0..4] == [0xD0, 0xCF, 0x11, 0xE0] {
            return ContainerFormat::ExcelOle;
        }
        if raw[0..4] == [0x25, 0x50, 0x44, 0x46] {
            // %PDF
            return ContainerFormat::Pdf;
        }
    }

    // Check XML markers
    let prefix_sample = String::from_utf8_lossy(&raw[..raw.len().min(2048)]).to_lowercase();
    if prefix_sample.starts_with("<?xml")
        || prefix_sample.contains("<bktocstmrstmt>")
        || prefix_sample.contains("camt.053")
        || lower_name.ends_with(".xml")
        || lower_name.ends_with(".camt053")
    {
        return ContainerFormat::Xml;
    }

    // Check SWIFT MT markers (:20:, {4:, :60F:)
    if prefix_sample.starts_with(":20:")
        || prefix_sample.contains("\n:20:")
        || prefix_sample.contains("\r\n:20:")
        || prefix_sample.contains(":60f:")
        || prefix_sample.contains("{4:")
        || lower_name.ends_with(".940")
        || lower_name.ends_with(".sta")
    {
        return ContainerFormat::SwiftMt;
    }

    // Check HTML tables disguised as spreadsheet
    if prefix_sample.contains("<html")
        || prefix_sample.contains("<!doctype html")
        || prefix_sample.contains("<table")
    {
        return ContainerFormat::HtmlTable;
    }

    // Extension fallbacks
    if lower_name.ends_with(".xlsx") {
        return ContainerFormat::ExcelZip;
    }
    if lower_name.ends_with(".xls") {
        return ContainerFormat::ExcelOle;
    }

    // Default to CSV / delimited text
    ContainerFormat::TextCsv
}

/// Evaluates how well a given file matches a StatementTemplate, returning a match score.
pub fn evaluate_template_score(
    template: &StatementTemplate,
    raw: &[u8],
    filename: &str,
) -> u32 {
    let detected_format = detect_container_format(raw, filename);
    let lower_filename = filename.to_lowercase();

    // 1. Container format compatibility
    let allowed_formats = &template.sniffing.container.allowed_formats;
    if !allowed_formats.is_empty() && !allowed_formats.contains(&detected_format) {
        return 0;
    }

    // 2. Extension check
    let file_extensions = &template.sniffing.container.file_extensions;
    let mut score: u32 = 0;
    if !file_extensions.is_empty() {
        let matches_ext = file_extensions
            .iter()
            .any(|ext| lower_filename.ends_with(&ext.to_lowercase()));
        if !matches_ext {
            return 0;
        }
        score = score.saturating_add(5);
    }

    // 3. Magic bytes check
    let magic_bytes = &template.sniffing.container.magic_bytes;
    if !magic_bytes.is_empty() {
        let mut matched_magic = false;
        for mb_hex in magic_bytes {
            if let Ok(bytes) = hex_to_bytes(mb_hex) {
                if raw.len() >= bytes.len() && &raw[..bytes.len()] == bytes.as_slice() {
                    matched_magic = true;
                    score = score.saturating_add(10);
                    break;
                }
            }
        }
        if !matched_magic && detected_format != ContainerFormat::TextCsv {
            return 0;
        }
    }

    // Text samples for content rules
    let header_sample_len = raw.len().min(4096);
    let header_text = String::from_utf8_lossy(&raw[..header_sample_len]);

    let full_sample_len = raw.len().min(65536);
    let full_text = if full_sample_len > header_sample_len {
        String::from_utf8_lossy(&raw[..full_sample_len])
    } else {
        header_text.clone()
    };

    // 4. Content rules scoring
    for rule in &template.sniffing.content_rules.rules {
        if let Ok(re) = Regex::new(&rule.pattern) {
            let matched = match rule.target.as_str() {
                "filename" => re.is_match(filename),
                "header_text" => re.is_match(&header_text),
                "full_text" => re.is_match(&full_text),
                _ => re.is_match(&header_text) || re.is_match(filename),
            };

            if matched {
                score = score.saturating_add(rule.score);
            }
        }
    }

    score
}

fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, ()> {
    let clean = hex_str.trim().replace("0x", "").replace(' ', "");
    if clean.len() % 2 != 0 {
        return Err(());
    }
    (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16).map_err(|_| ()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_container_formats() {
        let zip_magic = [0x50, 0x4B, 0x03, 0x04, 0x14, 0x00];
        assert_eq!(
            detect_container_format(&zip_magic, "test.xlsx"),
            ContainerFormat::ExcelZip
        );

        let ole_magic = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        assert_eq!(
            detect_container_format(&ole_magic, "test.xls"),
            ContainerFormat::ExcelOle
        );

        let xml_bytes = b"<?xml version=\"1.0\"?><Document>";
        assert_eq!(
            detect_container_format(xml_bytes, "test.xml"),
            ContainerFormat::Xml
        );

        let csv_bytes = b"So tai khoan:;123;;";
        assert_eq!(
            detect_container_format(csv_bytes, "test.csv"),
            ContainerFormat::TextCsv
        );
    }
}
