use liva_ledger::{JournalEntry, JournalLine, LedgerError};
use liva_money::Money;
use crate::models::MatchConfig;

/// Standard Vietnamese wire transfer fee schedule (base and with 8%/10% VAT).
pub const KNOWN_VIETNAMESE_WIRE_FEES: &[i64] = &[
    1_000, 1_080, 1_100, // < 500k Napas
    2_000, 2_160, 2_200, // 500k - 2M
    3_000, 3_240, 3_300,
    5_000, 5_400, 5_500, // 2M - 10M
    7_000, 7_560, 7_700,
    8_000, 8_640, 8_800, // 10M - 500M
    9_000, 9_720, 9_900,
    10_000, 10_800, 11_000, // > 10M interbank
    20_000, 21_600, 22_000, // Over counter / large wire
];

/// Helper function to check if pattern occurs in text with word/digit boundary semantics:
/// - Preceding character must NOT be alphanumeric (avoids e.g. "coffee" matching "fee")
/// - Following character must NOT be an ASCII digit (avoids e.g. "phi 1000" matching "phi 10000")
fn contains_bounded_fee_pattern(text: &str, pattern: &str) -> bool {
    let mut search_from = 0;
    while let Some(rel_pos) = text[search_from..].find(pattern) {
        let abs_pos = search_from + rel_pos;
        let match_end = abs_pos + pattern.len();

        let is_preceded_by_alnum = if abs_pos > 0 {
            text[..abs_pos]
                .chars()
                .next_back()
                .map(|c| c.is_alphanumeric())
                .unwrap_or(false)
        } else {
            false
        };

        if is_preceded_by_alnum {
            search_from = abs_pos + 1;
            continue;
        }

        let is_followed_by_digit = text[match_end..]
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false);

        if !is_followed_by_digit {
            return true;
        }

        search_from = abs_pos + 1;
    }
    false
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireFeeDisentanglement {
    pub principal: Money,
    pub fee_amount: Money,
    pub is_pure_fee: bool,
    pub fee_account_code: Option<String>,
    pub fee_account_name: Option<String>,
}

pub struct FeeSplitter;

impl FeeSplitter {
    /// Returns true if the given amount matches a recognized standard wire fee.
    pub fn is_standard_fee(amount_vnd: i64) -> bool {
        KNOWN_VIETNAMESE_WIRE_FEES.contains(&amount_vnd)
    }

    /// Disentangles embedded fee from transaction memo and amount.
    pub fn disentangle_fee(amount: Money, memo: &str) -> WireFeeDisentanglement {
        let currency = amount.currency();
        let val = amount.amount();
        let lower = memo.to_lowercase();

        // 1. Pure bank fee detection
        let is_pure = lower.contains("phi duy tri")
            || lower.contains("phi quan ly")
            || lower.contains("phi dich vu")
            || lower.contains("phi thuong nien");

        if is_pure {
            return WireFeeDisentanglement {
                principal: Money::zero(currency),
                fee_amount: amount,
                is_pure_fee: true,
                fee_account_code: Some("6425".to_string()),
                fee_account_name: Some("Chi phí dịch vụ ngân hàng".to_string()),
            };
        }

        // 2. Search for explicit fee keywords followed by known fee amounts
        // Evaluated in descending order of fee amount to prioritize larger/more specific fees
        let mut detected_fee = 0i64;
        let mut fees_desc = KNOWN_VIETNAMESE_WIRE_FEES.to_vec();
        fees_desc.sort_unstable_by(|a, b| b.cmp(a));

        'fee_search: for &f in &fees_desc {
            let patterns = [
                format!("phi chuyen tien {}", f),
                format!("phi: {}", f),
                format!("phi {}", f),
                format!("fee {}", f),
                format!("cuoc {}", f),
            ];
            for p in &patterns {
                if contains_bounded_fee_pattern(&lower, p) {
                    detected_fee = f;
                    break 'fee_search;
                }
            }
        }

        if detected_fee == 0
            && (lower.contains("phi") || lower.contains("fee"))
            && Self::is_standard_fee(val)
        {
            detected_fee = val;
        }

        detected_fee = detected_fee.min(val);
        let principal = Money::from_minor(val.saturating_sub(detected_fee), currency);
        let fee_money = Money::from_minor(detected_fee, currency);

        WireFeeDisentanglement {
            principal,
            fee_amount: fee_money,
            is_pure_fee: detected_fee == val && val > 0,
            fee_account_code: if detected_fee > 0 {
                Some("6425".to_string())
            } else {
                None
            },
            fee_account_name: if detected_fee > 0 {
                Some("Chi phí dịch vụ ngân hàng".to_string())
            } else {
                None
            },
        }
    }

    /// Generates automated double-entry journal entry booking fee to Account 6425
    /// per Vietnam Circular 200/2014/TT-BTC.
    ///
    /// For customer payment with deducted wire fee:
    /// - Debit TK 1121 (Bank Cash): Net amount received
    /// - Debit TK 6425 (Bank Fees): Wire fee amount
    /// - Credit TK 131 (Customer Receivable): Total invoice amount
    pub fn create_fee_split_journal(
        entry_id: u64,
        timestamp: i64,
        net_bank_amount: Money,
        fee_amount: Money,
        invoice_doc_no: &str,
        config: &MatchConfig,
    ) -> Result<JournalEntry, LedgerError> {
        if !fee_amount.is_positive() {
            return Err(LedgerError::NonPositiveAmount);
        }
        let total_invoice = net_bank_amount.checked_add(fee_amount)?;

        let line_bank = JournalLine::debit(&config.account_bank_cash, net_bank_amount)?;
        let line_fee = JournalLine::debit(&config.account_bank_fee, fee_amount)?;
        let line_ar = JournalLine::credit(&config.account_receivable, total_invoice)?;

        let entry = JournalEntry {
            id: entry_id,
            timestamp,
            description: format!(
                "Hạch toán thu tiền hóa đơn {} bù trừ phí chuyển tiền liên ngân hàng vào TK {}",
                invoice_doc_no, config.account_bank_fee
            ),
            lines: vec![line_bank, line_fee, line_ar],
        };

        // Assert arithmetic equality: Debit (1121) + Debit (6425) == Credit (131)
        entry.verify_balance()?;
        Ok(entry)
    }

    /// Generates pure bank fee debit journal entry:
    /// - Debit TK 6425: Fee amount
    /// - Credit TK 1121: Fee amount
    pub fn create_pure_fee_journal(
        entry_id: u64,
        timestamp: i64,
        fee_amount: Money,
        description: &str,
        config: &MatchConfig,
    ) -> Result<JournalEntry, LedgerError> {
        if !fee_amount.is_positive() {
            return Err(LedgerError::NonPositiveAmount);
        }

        let line_fee = JournalLine::debit(&config.account_bank_fee, fee_amount)?;
        let line_bank = JournalLine::credit(&config.account_bank_cash, fee_amount)?;

        let entry = JournalEntry {
            id: entry_id,
            timestamp,
            description: format!(
                "Hạch toán phí dịch vụ ngân hàng vào TK {}: {}",
                config.account_bank_fee, description
            ),
            lines: vec![line_fee, line_bank],
        };

        entry.verify_balance()?;
        Ok(entry)
    }
}

/// Standalone wrapper for creating a fee split journal entry.
pub fn create_fee_split_journal(
    entry_id: u64,
    timestamp: i64,
    net_bank_amount: Money,
    fee_amount: Money,
    invoice_doc_no: &str,
    config: &MatchConfig,
) -> Result<JournalEntry, LedgerError> {
    FeeSplitter::create_fee_split_journal(
        entry_id,
        timestamp,
        net_bank_amount,
        fee_amount,
        invoice_doc_no,
        config,
    )
}

/// Standalone wrapper for creating a pure bank fee journal entry.
pub fn create_pure_fee_journal(
    entry_id: u64,
    timestamp: i64,
    fee_amount: Money,
    description: &str,
    config: &MatchConfig,
) -> Result<JournalEntry, LedgerError> {
    FeeSplitter::create_pure_fee_journal(entry_id, timestamp, fee_amount, description, config)
}
