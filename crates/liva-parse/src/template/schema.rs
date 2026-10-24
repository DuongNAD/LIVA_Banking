//! Statement Template DSL schema definitions.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

use serde::{Deserialize, Serialize};

/// High-level Statement Template definition loaded from YAML or JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatementTemplate {
    pub name: String,
    pub version: String,
    pub bank_code: String,
    pub bank_name: String,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default)]
    pub description: Option<String>,
    pub sniffing: SniffingConfig,
    #[serde(default)]
    pub container_options: ContainerOptions,
    #[serde(default)]
    pub metadata: MetadataConfig,
    pub table: TableConfig,
    #[serde(default)]
    pub formatting: FormattingConfig,
    #[serde(default)]
    pub invariants: InvariantConfig,
}

fn default_currency() -> String {
    "VND".to_string()
}

fn default_true() -> bool {
    true
}

// -----------------------------------------------------------------------------
// 1. Sniffing Rules
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SniffingConfig {
    #[serde(default)]
    pub container: ContainerSniffingConfig,
    #[serde(default)]
    pub content_rules: ContentRulesConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ContainerSniffingConfig {
    #[serde(default)]
    pub allowed_formats: Vec<ContainerFormat>,
    #[serde(default)]
    pub file_extensions: Vec<String>,
    #[serde(default)]
    pub magic_bytes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContainerFormat {
    TextCsv,
    ExcelZip,
    ExcelOle,
    Xml,
    SwiftMt,
    HtmlTable,
    Pdf,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ContentRulesConfig {
    #[serde(default = "default_min_score")]
    pub min_score: u32,
    #[serde(default)]
    pub rules: Vec<ContentRule>,
}

fn default_min_score() -> u32 {
    10
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentRule {
    pub pattern: String,
    #[serde(default = "default_target")]
    pub target: String, // "header_text", "filename", "full_text", "sheet_name"
    #[serde(default = "default_score")]
    pub score: u32,
}

fn default_target() -> String {
    "header_text".to_string()
}

fn default_score() -> u32 {
    5
}

// -----------------------------------------------------------------------------
// 2. Container Options
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ContainerOptions {
    #[serde(default)]
    pub csv: CsvOptions,
    #[serde(default)]
    pub excel: ExcelOptions,
    #[serde(default)]
    pub xml: XmlOptions,
    #[serde(default)]
    pub swift: SwiftOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CsvOptions {
    #[serde(default = "default_delimiter")]
    pub delimiter: String, // "auto", ",", ";", "\t", "|"
    #[serde(default = "default_quote_char")]
    pub quote_char: char,
    #[serde(default)]
    pub has_header: bool,
    #[serde(default = "default_encoding")]
    pub encoding: String,
    #[serde(default = "default_true")]
    pub strip_bom: bool,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            delimiter: "auto".to_string(),
            quote_char: '"',
            has_header: false,
            encoding: "auto".to_string(),
            strip_bom: true,
        }
    }
}

fn default_delimiter() -> String {
    "auto".to_string()
}

fn default_quote_char() -> char {
    '"'
}

fn default_encoding() -> String {
    "auto".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExcelOptions {
    #[serde(default = "default_sheet_selection")]
    pub sheet_selection: String, // "first", "by_index", "by_name"
    #[serde(default)]
    pub sheet_index: usize,
    #[serde(default)]
    pub sheet_name_pattern: Option<String>,
    #[serde(default = "default_merged_cells")]
    pub merged_cells: String, // "forward_fill", "blank", "none"
}

impl Default for ExcelOptions {
    fn default() -> Self {
        Self {
            sheet_selection: "first".to_string(),
            sheet_index: 0,
            sheet_name_pattern: None,
            merged_cells: "forward_fill".to_string(),
        }
    }
}

fn default_sheet_selection() -> String {
    "first".to_string()
}

fn default_merged_cells() -> String {
    "forward_fill".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct XmlOptions {
    #[serde(default)]
    pub statement_path: Option<String>,
    #[serde(default)]
    pub entry_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwiftOptions {
    #[serde(default = "default_block_id")]
    pub block_id: String, // "4"
}

impl Default for SwiftOptions {
    fn default() -> Self {
        Self {
            block_id: "4".to_string(),
        }
    }
}

fn default_block_id() -> String {
    "4".to_string()
}

// -----------------------------------------------------------------------------
// 3. Metadata Extraction
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MetadataConfig {
    #[serde(default)]
    pub scan_range: ScanRangeConfig,
    #[serde(default)]
    pub fields: MetadataFieldsConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanRangeConfig {
    #[serde(default = "default_header_rows")]
    pub header_rows: usize,
    #[serde(default = "default_footer_rows")]
    pub footer_rows: usize,
}

impl Default for ScanRangeConfig {
    fn default() -> Self {
        Self {
            header_rows: 25,
            footer_rows: 30,
        }
    }
}

fn default_header_rows() -> usize {
    25
}

fn default_footer_rows() -> usize {
    30
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MetadataFieldsConfig {
    #[serde(default)]
    pub account_number: Option<FieldExtractor>,
    #[serde(default)]
    pub account_name: Option<FieldExtractor>,
    #[serde(default)]
    pub opening_balance: Option<FieldExtractor>,
    #[serde(default)]
    pub closing_balance: Option<FieldExtractor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FieldExtractor {
    #[serde(default = "default_extractor")]
    pub extractor: String, // "regex", "key_value", "cell_coordinate", "xml_xpath", "xml_balance", "swift_tag", "swift_tag_balance"
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub value_offset: Option<(i32, i32)>, // [row_delta, col_delta]
    #[serde(default)]
    pub balance_type: Option<String>, // e.g. "OPBD", "CLBD"
    #[serde(default)]
    pub tag: Option<String>, // e.g. ":25:", ":60F:"
    #[serde(default)]
    pub fallback_tag: Option<String>,
}

fn default_extractor() -> String {
    "regex".to_string()
}

// -----------------------------------------------------------------------------
// 4. Table Header & Column Mapping
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableConfig {
    #[serde(default)]
    pub header_detection: HeaderDetectionConfig,
    pub columns: ColumnsConfig,
    #[serde(default)]
    pub row_filters: RowFiltersConfig,
    #[serde(default)]
    pub continuation_lines: ContinuationLinesConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderDetectionConfig {
    #[serde(default = "default_detection_mode")]
    pub detection_mode: String, // "keyword_match", "row_index", "regex"
    #[serde(default = "default_min_matching_columns")]
    pub min_matching_columns: usize,
    #[serde(default = "default_stop_at_row")]
    pub stop_at_row: usize,
    #[serde(default)]
    pub fixed_row_index: Option<usize>,
}

impl Default for HeaderDetectionConfig {
    fn default() -> Self {
        Self {
            detection_mode: "keyword_match".to_string(),
            min_matching_columns: 3,
            stop_at_row: 30,
            fixed_row_index: None,
        }
    }
}

fn default_detection_mode() -> String {
    "keyword_match".to_string()
}

fn default_min_matching_columns() -> usize {
    3
}

fn default_stop_at_row() -> usize {
    30
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmountMode {
    #[default]
    SeparateDebitCredit,
    SingleWithIndicator,
    SignedAmount,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ColumnsConfig {
    #[serde(default)]
    pub date: Option<ColumnSpec>,
    #[serde(default)]
    pub value_date: Option<ColumnSpec>,
    #[serde(default)]
    pub ref_code: Option<ColumnSpec>,
    #[serde(default)]
    pub amount_mode: AmountMode,
    #[serde(default)]
    pub debit: Option<ColumnSpec>,
    #[serde(default)]
    pub credit: Option<ColumnSpec>,
    #[serde(default)]
    pub amount: Option<ColumnSpec>,
    #[serde(default)]
    pub indicator: Option<IndicatorColumnSpec>,
    #[serde(default)]
    pub balance_after: Option<ColumnSpec>,
    #[serde(default)]
    pub narration: Option<ColumnSpec>,
    #[serde(default)]
    pub counterparty_account: Option<ColumnSpec>,
    #[serde(default)]
    pub counterparty_name: Option<ColumnSpec>,
    #[serde(default)]
    pub counterparty_bank: Option<ColumnSpec>,
    #[serde(default)]
    pub swift_statement_line: Option<SwiftLineConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ColumnSpec {
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub fixed_index: Option<usize>,
    #[serde(default)]
    pub fallback_from_narration_regex: Vec<String>,
    #[serde(default)]
    pub xml_tag: Option<String>,
    #[serde(default)]
    pub fallback_xml_tag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct IndicatorColumnSpec {
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub credit_tokens: Vec<String>,
    #[serde(default)]
    pub debit_tokens: Vec<String>,
    #[serde(default)]
    pub xml_tag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwiftLineConfig {
    #[serde(default = "default_swift_tag")]
    pub tag: String,
    #[serde(default = "default_swift_date")]
    pub date_subfield: String,
    #[serde(default = "default_swift_indicator")]
    pub indicator_subfield: String,
    #[serde(default = "default_swift_amount")]
    pub amount_subfield: String,
    #[serde(default = "default_swift_ref")]
    pub ref_subfield: String,
}

impl Default for SwiftLineConfig {
    fn default() -> Self {
        Self {
            tag: ":61:".to_string(),
            date_subfield: "value_date".to_string(),
            indicator_subfield: "credit_debit".to_string(),
            amount_subfield: "amount".to_string(),
            ref_subfield: "reference".to_string(),
        }
    }
}

fn default_swift_tag() -> String {
    ":61:".to_string()
}
fn default_swift_date() -> String {
    "value_date".to_string()
}
fn default_swift_indicator() -> String {
    "credit_debit".to_string()
}
fn default_swift_amount() -> String {
    "amount".to_string()
}
fn default_swift_ref() -> String {
    "reference".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowFiltersConfig {
    #[serde(default = "default_true")]
    pub ignore_empty_rows: bool,
    #[serde(default = "default_true")]
    pub skip_if_date_missing: bool,
    #[serde(default)]
    pub stop_conditions: Vec<String>,
}

impl Default for RowFiltersConfig {
    fn default() -> Self {
        Self {
            ignore_empty_rows: true,
            skip_if_date_missing: true,
            stop_conditions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuationLinesConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_narration")]
    pub append_to: String,
    #[serde(default = "default_space")]
    pub delimiter: String,
    #[serde(default = "default_condition")]
    pub condition: String,
    #[serde(default = "default_max_continuation_lines")]
    pub max_lines: usize,
}

impl Default for ContinuationLinesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            append_to: "narration".to_string(),
            delimiter: " ".to_string(),
            condition: "date_is_empty_and_no_amount".to_string(),
            max_lines: 6,
        }
    }
}

fn default_narration() -> String {
    "narration".to_string()
}

fn default_space() -> String {
    " ".to_string()
}

fn default_condition() -> String {
    "date_is_empty_and_no_amount".to_string()
}

fn default_max_continuation_lines() -> usize {
    6
}

// -----------------------------------------------------------------------------
// 5. Formatting (Date & Zero-Float Number)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FormattingConfig {
    #[serde(default)]
    pub date: DateFormattingConfig,
    #[serde(default)]
    pub number: NumberFormattingConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateFormattingConfig {
    #[serde(default = "default_date_formats")]
    pub formats: Vec<String>,
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

impl Default for DateFormattingConfig {
    fn default() -> Self {
        Self {
            formats: default_date_formats(),
            timezone: default_timezone(),
        }
    }
}

fn default_date_formats() -> Vec<String> {
    vec![
        "%d/%m/%Y %H:%M:%S".to_string(),
        "%d/%m/%Y".to_string(),
        "%Y-%m-%d %H:%M:%S".to_string(),
        "%Y-%m-%d".to_string(),
    ]
}

fn default_timezone() -> String {
    "Asia/Ho_Chi_Minh".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NumberFormattingConfig {
    #[serde(default = "default_thousand_sep")]
    pub thousand_separator: String, // ".", ",", " ", "none", "auto"
    #[serde(default = "default_decimal_sep")]
    pub decimal_separator: String, // ",", ".", "none", "auto"
    #[serde(default)]
    pub decimal_places: u32,
    #[serde(default = "default_negative_format")]
    pub negative_format: String, // "prefix_minus", "parentheses", "trailing_minus", "both"
    #[serde(default)]
    pub strip_tokens: Vec<String>,
}

impl Default for NumberFormattingConfig {
    fn default() -> Self {
        Self {
            thousand_separator: default_thousand_sep(),
            decimal_separator: default_decimal_sep(),
            decimal_places: 0,
            negative_format: default_negative_format(),
            strip_tokens: Vec::new(),
        }
    }
}

fn default_thousand_sep() -> String {
    ".".to_string()
}

fn default_decimal_sep() -> String {
    ",".to_string()
}

fn default_negative_format() -> String {
    "both".to_string()
}

// -----------------------------------------------------------------------------
// 6. Invariants
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantConfig {
    #[serde(default = "default_verification_formula")]
    pub verification_formula: String, // "closing_eq_opening_plus_credit_minus_debit"
    #[serde(default = "default_policy")]
    pub policy: String, // "strict", "warn", "relaxed"
    #[serde(default = "default_true")]
    pub allow_missing_balances: bool,
    #[serde(default)]
    pub tolerance_minor_units: i64,
}

impl Default for InvariantConfig {
    fn default() -> Self {
        Self {
            verification_formula: default_verification_formula(),
            policy: default_policy(),
            allow_missing_balances: true,
            tolerance_minor_units: 0,
        }
    }
}

fn default_verification_formula() -> String {
    "closing_eq_opening_plus_credit_minus_debit".to_string()
}

fn default_policy() -> String {
    "strict".to_string()
}
