//! Embedded default banking templates for zero-configuration out-of-the-box operation.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

pub const VCB_EXCEL_YAML: &str = include_str!("../templates/vcb_excel.yaml");
pub const TCB_CSV_YAML: &str = include_str!("../templates/tcb_csv.yaml");
pub const BIDV_CSV_YAML: &str = include_str!("../templates/bidv_csv.yaml");
pub const CAMT053_XML_YAML: &str = include_str!("../templates/camt053_xml.yaml");
pub const SWIFT_MT940_YAML: &str = include_str!("../templates/swift_mt940.yaml");

/// Returns the complete list of embedded default YAML templates.
pub fn get_builtin_template_yamls() -> Vec<(&'static str, &'static str)> {
    vec![
        ("vcb_excel.yaml", VCB_EXCEL_YAML),
        ("tcb_csv.yaml", TCB_CSV_YAML),
        ("bidv_csv.yaml", BIDV_CSV_YAML),
        ("camt053_xml.yaml", CAMT053_XML_YAML),
        ("swift_mt940.yaml", SWIFT_MT940_YAML),
    ]
}
