//! LIVA Banking — Declarative Posting Engine
//!
//! Evaluates incoming statement events against priority-ordered dynamic rules
//! and synthesizes double-entry balanced journal vouchers.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.
//! Guarantees double-entry balance ($\sum \text{Debit} \equiv \sum \text{Credit}$, $\Delta = 0$).

use super::coa_schema::{CoaError, CompiledCoaTree};
use super::fee_unbundling::unbundle_gross_fee;
use super::posting_rules::{
    AmountFormula, PostingAction, PostingCondition, PostingEventType, PostingRuleConfig,
};
use crate::{
    ErpTarget, JournalEntry, JournalLine, LedgerError, PostingType,
    ReconciliationVoucherProposal, VoucherStatus,
};
use liva_money::Money;
use regex::Regex;
use thiserror::Error;

/// Errors arising during rule engine compilation or execution.
#[derive(Debug, Error)]
pub enum RuleEngineError {
    #[error("Regex compilation error: {0}")]
    Regex(#[from] regex::Error),

    #[error("YAML deserialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Ledger error during rule execution: {0}")]
    Ledger(#[from] LedgerError),

    #[error("Chart of Accounts validation error: {0}")]
    Coa(#[from] CoaError),
}

/// Runtime context supplied to the dynamic posting rules engine.
#[derive(Debug, Clone)]
pub struct PostingContext<'a> {
    /// Bank code (e.g. "VCB", "TCB", "BIDV")
    pub bank_code: &'a str,
    /// Bank transaction reference or trace code
    pub ref_code: &'a str,
    /// Transaction posting direction (Debit = out, Credit = in)
    pub posting_type: PostingType,
    /// Gross transaction amount
    pub amount: Money,
    /// Raw or normalized transaction narration
    pub narration: &'a str,
    /// Counterparty legal name if extracted
    pub counterparty_name: Option<&'a str>,
    /// Counterparty bank account number if extracted
    pub counterparty_account: Option<&'a str>,
    /// High-level event classification if pre-identified
    pub event_type: Option<PostingEventType>,
    /// Organization bank GL account code (e.g. "1121.VCB.01" or "1121")
    pub bank_account_code: &'a str,
    /// Counterparty GL account code (e.g. "131.CUST_01", "331.VEND_01")
    pub partner_account_code: Option<&'a str>,
    /// Counterparty display name
    pub partner_name: &'a str,
    /// Counterparty tax ID / MST
    pub partner_tax_id: Option<&'a str>,
    /// Unique voucher idempotency GUID
    pub voucher_guid: &'a str,
    /// Batch identifier
    pub batch_id: &'a str,
    /// Reconciliation match identifier
    pub match_id: &'a str,
    /// Statutory voucher date (YYYY-MM-DD)
    pub voucher_date: &'a str,
    /// Statutory voucher number (e.g. "PT00001", "PC00001")
    pub voucher_number: &'a str,
    /// Target enterprise ERP system
    pub target_erp: ErpTarget,
    /// Intermediary or payment gateway deducted fee if applicable
    pub deducted_fee: Option<Money>,
    /// Original invoice amount being settled if known
    pub invoice_amount: Option<Money>,
}

impl<'a> PostingContext<'a> {
    /// Constructs a standard posting context.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bank_code: &'a str,
        ref_code: &'a str,
        posting_type: PostingType,
        amount: Money,
        narration: &'a str,
        bank_account_code: &'a str,
        partner_name: &'a str,
        voucher_guid: &'a str,
        batch_id: &'a str,
        match_id: &'a str,
        voucher_date: &'a str,
        voucher_number: &'a str,
        target_erp: ErpTarget,
    ) -> Self {
        Self {
            bank_code,
            ref_code,
            posting_type,
            amount,
            narration,
            counterparty_name: None,
            counterparty_account: None,
            event_type: None,
            bank_account_code,
            partner_account_code: None,
            partner_name,
            partner_tax_id: None,
            voucher_guid,
            batch_id,
            match_id,
            voucher_date,
            voucher_number,
            target_erp,
            deducted_fee: None,
            invoice_amount: None,
        }
    }
}

/// Precompiled pattern indices for efficient matching.
struct CompiledRulePatterns {
    narration_regexes: Vec<Regex>,
    counterparty_regexes: Vec<Regex>,
}

/// Dynamic posting engine evaluating incoming banking events into balanced vouchers.
pub struct DeclarativePostingEngine {
    config: PostingRuleConfig,
    compiled_patterns: Vec<CompiledRulePatterns>,
}

impl DeclarativePostingEngine {
    /// Compiles a DeclarativePostingEngine from a `PostingRuleConfig`.
    pub fn from_config(mut config: PostingRuleConfig) -> Result<Self, RuleEngineError> {
        // Sort rules by priority descending
        config.rules.sort_by_key(|a| std::cmp::Reverse(a.priority));

        let mut compiled_patterns = Vec::with_capacity(config.rules.len());
        for rule in &config.rules {
            let mut narration_regexes = Vec::with_capacity(rule.when.narration_patterns.len());
            for pat in &rule.when.narration_patterns {
                let re = Regex::new(pat)?;
                narration_regexes.push(re);
            }

            let mut counterparty_regexes =
                Vec::with_capacity(rule.when.counterparty_patterns.len());
            for pat in &rule.when.counterparty_patterns {
                let re = Regex::new(pat)?;
                counterparty_regexes.push(re);
            }

            compiled_patterns.push(CompiledRulePatterns {
                narration_regexes,
                counterparty_regexes,
            });
        }

        Ok(Self {
            config,
            compiled_patterns,
        })
    }

    /// Compiles a DeclarativePostingEngine from a YAML string.
    pub fn from_yaml(yaml_str: &str) -> Result<Self, RuleEngineError> {
        let config = PostingRuleConfig::from_yaml(yaml_str)?;
        Self::from_config(config)
    }

    /// Returns the underlying posting rule configuration.
    pub const fn config(&self) -> &PostingRuleConfig {
        &self.config
    }

    /// Evaluates context against rules and synthesizes a balanced journal voucher proposal.
    pub fn evaluate(
        &self,
        ctx: &PostingContext,
    ) -> Result<ReconciliationVoucherProposal, LedgerError> {
        for (idx, rule) in self.config.rules.iter().enumerate() {
            if !rule.enabled {
                continue;
            }
            if !self.matches_condition(&rule.when, idx, ctx) {
                continue;
            }

            return self.execute_action(&rule.then, ctx);
        }

        // Fallback default action
        self.execute_default_fallback(ctx)
    }

    /// Evaluates context against rules and validates that all generated accounts
    /// exist and are leaf accounts within the provided Chart of Accounts tree.
    pub fn evaluate_with_coa(
        &self,
        ctx: &PostingContext,
        coa: &CompiledCoaTree,
    ) -> Result<ReconciliationVoucherProposal, RuleEngineError> {
        let voucher = self.evaluate(ctx).map_err(RuleEngineError::Ledger)?;
        for line in &voucher.lines {
            coa.validate_postable(&line.account_code)
                .map_err(RuleEngineError::Coa)?;
        }
        Ok(voucher)
    }

    fn matches_condition(
        &self,
        cond: &PostingCondition,
        rule_idx: usize,
        ctx: &PostingContext,
    ) -> bool {
        // 1. Direction check
        if let Some(dir) = cond.direction {
            if ctx.posting_type != dir {
                return false;
            }
        }

        // 2. Bank codes filter
        if !cond.bank_codes.is_empty()
            && !cond
                .bank_codes
                .iter()
                .any(|b| b.eq_ignore_ascii_case(ctx.bank_code))
        {
            return false;
        }

        // 3. Amount thresholds
        let amt = ctx.amount.amount();
        if let Some(min) = cond.min_amount {
            if amt < min {
                return false;
            }
        }
        if let Some(max) = cond.max_amount {
            if amt > max {
                return false;
            }
        }

        // 4. Explicit event type
        if let Some(et) = cond.event_type {
            if ctx.event_type != Some(et) {
                return false;
            }
        }

        // 5. Narration regexes
        let patterns = &self.compiled_patterns[rule_idx];
        if !patterns.narration_regexes.is_empty() {
            let matched = patterns
                .narration_regexes
                .iter()
                .any(|re| re.is_match(ctx.narration));
            if !matched {
                return false;
            }
        }

        // 6. Counterparty regexes
        if !patterns.counterparty_regexes.is_empty() {
            let cp_name = ctx.counterparty_name.unwrap_or(ctx.partner_name);
            let cp_tax = ctx.partner_tax_id.unwrap_or("");
            let matched = patterns
                .counterparty_regexes
                .iter()
                .any(|re| re.is_match(cp_name) || re.is_match(cp_tax));
            if !matched {
                return false;
            }
        }

        true
    }

    fn execute_action(
        &self,
        action: &PostingAction,
        ctx: &PostingContext,
    ) -> Result<ReconciliationVoucherProposal, LedgerError> {
        let mut lines = Vec::with_capacity(action.lines.len());

        for line_tmpl in &action.lines {
            let account_code = match line_tmpl.account_expr.as_str() {
                "{{bank_account}}" => ctx.bank_account_code.to_string(),
                "{{partner_account}}" => {
                    if let Some(partner_acc) = ctx.partner_account_code {
                        partner_acc.to_string()
                    } else if ctx.posting_type == PostingType::Credit {
                        self.config.defaults.customer_default_account.clone()
                    } else {
                        self.config.defaults.vendor_default_account.clone()
                    }
                }
                "{{fee_expense_account}}" => self.config.defaults.fee_expense_account.clone(),
                "{{fee_tax_account}}" => self.config.defaults.fee_tax_account.clone(),
                "{{input_vat_account}}" => self.config.defaults.input_vat_account.clone(),
                literal_code => literal_code.to_string(),
            };

            let amount = match line_tmpl.amount_expr {
                AmountFormula::FullAmount => ctx.amount,
                AmountFormula::NetExclusive { vat_rate_bps } => {
                    let (net, _) = unbundle_gross_fee(ctx.amount, vat_rate_bps)?;
                    net
                }
                AmountFormula::VatPortion { vat_rate_bps } => {
                    let (_, vat) = unbundle_gross_fee(ctx.amount, vat_rate_bps)?;
                    vat
                }
                AmountFormula::DeductedFee => {
                    ctx.deducted_fee.unwrap_or_else(|| Money::zero(ctx.amount.currency()))
                }
                AmountFormula::NetReceived => {
                    if let Some(fee) = ctx.deducted_fee {
                        ctx.amount.checked_sub(fee)?
                    } else {
                        ctx.amount
                    }
                }
                AmountFormula::InvoiceAmount => ctx.invoice_amount.unwrap_or(ctx.amount),
                AmountFormula::Fixed { amount_minor } => {
                    Money::from_minor(amount_minor, ctx.amount.currency())
                }
            };

            // Only add positive lines (e.g. skip VAT if VAT portion rounds to 0)
            if amount.is_positive() {
                lines.push(JournalLine::new(account_code, line_tmpl.posting_type, amount)?);
            }
        }

        // Verify that entry contains at least 2 lines and is balanced
        if lines.len() < 2 {
            return Err(LedgerError::InsufficientLines);
        }

        let mut total_debit = Money::zero(ctx.amount.currency());
        let mut total_credit = Money::zero(ctx.amount.currency());

        for line in &lines {
            match line.posting_type {
                PostingType::Debit => total_debit = total_debit.checked_add(line.amount)?,
                PostingType::Credit => total_credit = total_credit.checked_add(line.amount)?,
            }
        }

        if total_debit != total_credit {
            return Err(LedgerError::UnbalancedEntry {
                total_debit: total_debit.amount(),
                total_credit: total_credit.amount(),
            });
        }

        let description = self.interpolate_description(&action.description_template, ctx);
        let idempotency_hash = format!(
            "0x{:016x}{:016x}",
            (total_debit.amount() as u64).wrapping_mul(31),
            (ctx.voucher_guid.len() + ctx.batch_id.len() + ctx.match_id.len()) as u64
        );

        Ok(ReconciliationVoucherProposal {
            voucher_guid: ctx.voucher_guid.to_string(),
            batch_id: ctx.batch_id.to_string(),
            match_id: ctx.match_id.to_string(),
            voucher_number: ctx.voucher_number.to_string(),
            voucher_date: ctx.voucher_date.to_string(),
            partner_name: ctx.partner_name.to_string(),
            partner_tax_id: ctx.partner_tax_id.map(|s| s.to_string()),
            description,
            target_erp: ctx.target_erp,
            status: VoucherStatus::Draft,
            lines,
            total_debit,
            total_credit,
            idempotency_hash,
            erp_reference: None,
            posted_at: None,
        })
    }

    fn execute_default_fallback(
        &self,
        ctx: &PostingContext,
    ) -> Result<ReconciliationVoucherProposal, LedgerError> {
        match ctx.posting_type {
            PostingType::Credit => {
                // Inbound cash: Nợ 1121 / Có 131
                let customer_acc = ctx
                    .partner_account_code
                    .unwrap_or(&self.config.defaults.customer_default_account);
                ReconciliationVoucherProposal::new_customer_receipt(
                    ctx.voucher_guid,
                    ctx.batch_id,
                    ctx.match_id,
                    ctx.voucher_number,
                    ctx.voucher_date,
                    ctx.partner_name,
                    ctx.partner_tax_id.map(|s| s.to_string()),
                    ctx.narration,
                    ctx.amount,
                    ctx.bank_account_code,
                    customer_acc,
                    ctx.target_erp,
                )
            }
            PostingType::Debit => {
                // Outbound cash: Nợ 331 / Có 1121
                let vendor_acc = ctx
                    .partner_account_code
                    .unwrap_or(&self.config.defaults.vendor_default_account);
                ReconciliationVoucherProposal::new_vendor_payment(
                    ctx.voucher_guid,
                    ctx.batch_id,
                    ctx.match_id,
                    ctx.voucher_number,
                    ctx.voucher_date,
                    ctx.partner_name,
                    ctx.partner_tax_id.map(|s| s.to_string()),
                    ctx.narration,
                    ctx.amount,
                    vendor_acc,
                    ctx.bank_account_code,
                    ctx.target_erp,
                )
            }
        }
    }

    fn interpolate_description(&self, template: &str, ctx: &PostingContext) -> String {
        template
            .replace("{{bank}}", ctx.bank_code)
            .replace("{{ref}}", ctx.ref_code)
            .replace("{{narration}}", ctx.narration)
            .replace("{{partner}}", ctx.partner_name)
            .replace("{{date}}", ctx.voucher_date)
    }
}

/// Converts a `ReconciliationVoucherProposal` into a verified atomic `JournalEntry`.
pub fn voucher_to_journal_entry(
    voucher: &ReconciliationVoucherProposal,
    entry_id: u64,
    timestamp: i64,
) -> Result<JournalEntry, LedgerError> {
    let entry = JournalEntry {
        id: entry_id,
        timestamp,
        description: voucher.description.clone(),
        lines: voucher.lines.clone(),
    };
    entry.verify_balance()?;
    Ok(entry)
}
