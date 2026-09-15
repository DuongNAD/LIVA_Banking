//! LIVA Banking — `erp_posting` Module (P50–P53)
//!
//! Automated Journal Entry Synthesis, ERP Payload Bridge (MISA / FAST),
//! and Bank vs GL Balance Reconciliation Certification.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

use crate::{JournalLine, LedgerError, PostingType};
use liva_money::Money;
use serde::{Deserialize, Serialize};

/// Target enterprise ERP system for ledger integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErpTarget {
    /// MISA AMIS Enterprise Accounting
    MisaAmis,
    /// FAST Business Online
    FastBusiness,
    /// BRAVO ERP 8R3
    Bravo,
    /// SAP S/4HANA Finance
    Sap,
}

impl ErpTarget {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::MisaAmis => "MISA_AMIS",
            Self::FastBusiness => "FAST_BUSINESS",
            Self::Bravo => "BRAVO_ERP",
            Self::Sap => "SAP_S4HANA",
        }
    }
}

/// Lifecycle status of an automated journal voucher proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoucherStatus {
    /// Đề xuất mới tạo từ kết quả đối soát
    Draft,
    /// Chờ Kế toán trưởng ký duyệt trước khi gửi ERP
    PendingPost,
    /// Đã đồng bộ thành công sang ERP
    PostedToErp,
    /// Đồng bộ thất bại (được đưa vào Dead Letter Queue để retry)
    SyncFailed,
}

/// Automated balanced journal entry proposal (P50).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationVoucherProposal {
    /// UUIDv4 Idempotency Key (Single-use per transaction batch)
    pub voucher_guid: String,
    pub batch_id: String,
    pub match_id: String,
    pub voucher_number: String,
    pub voucher_date: String,
    pub partner_name: String,
    pub partner_tax_id: Option<String>,
    pub description: String,
    pub target_erp: ErpTarget,
    pub status: VoucherStatus,
    pub lines: Vec<JournalLine>,
    pub total_debit: Money,
    pub total_credit: Money,
    pub idempotency_hash: String,
    pub erp_reference: Option<String>,
    pub posted_at: Option<String>,
}

impl ReconciliationVoucherProposal {
    /// Helper to compute deterministic idempotency hash from key attributes
    fn compute_idempotency_hash(guid: &str, batch_id: &str, match_id: &str, amount: i64) -> String {
        format!("0x{:016x}{:016x}", amount.wrapping_mul(31) as u64, (guid.len() + batch_id.len() + match_id.len()) as u64)
    }

    /// Constructs a 1:1 Customer Receipt Voucher (Nợ 1121 / Có 131).
    pub fn new_customer_receipt(
        voucher_guid: impl Into<String>,
        batch_id: impl Into<String>,
        match_id: impl Into<String>,
        voucher_number: impl Into<String>,
        voucher_date: impl Into<String>,
        partner_name: impl Into<String>,
        partner_tax_id: Option<String>,
        description: impl Into<String>,
        amount: Money,
        bank_account: impl Into<String>,
        customer_account: impl Into<String>,
        target_erp: ErpTarget,
    ) -> Result<Self, LedgerError> {
        if !amount.is_positive() {
            return Err(LedgerError::NonPositiveAmount);
        }

        let guid = voucher_guid.into();
        let b_id = batch_id.into();
        let m_id = match_id.into();
        let hash = Self::compute_idempotency_hash(&guid, &b_id, &m_id, amount.amount());

        let lines = vec![
            JournalLine::new(bank_account, PostingType::Debit, amount)?,
            JournalLine::new(customer_account, PostingType::Credit, amount)?,
        ];

        Ok(Self {
            voucher_guid: guid,
            batch_id: b_id,
            match_id: m_id,
            voucher_number: voucher_number.into(),
            voucher_date: voucher_date.into(),
            partner_name: partner_name.into(),
            partner_tax_id,
            description: description.into(),
            target_erp,
            status: VoucherStatus::Draft,
            lines,
            total_debit: amount,
            total_credit: amount,
            idempotency_hash: hash,
            erp_reference: None,
            posted_at: None,
        })
    }

    /// Constructs a 1:1 Vendor Payment Voucher (Nợ 331 / Có 1121).
    pub fn new_vendor_payment(
        voucher_guid: impl Into<String>,
        batch_id: impl Into<String>,
        match_id: impl Into<String>,
        voucher_number: impl Into<String>,
        voucher_date: impl Into<String>,
        partner_name: impl Into<String>,
        partner_tax_id: Option<String>,
        description: impl Into<String>,
        amount: Money,
        vendor_account: impl Into<String>,
        bank_account: impl Into<String>,
        target_erp: ErpTarget,
    ) -> Result<Self, LedgerError> {
        if !amount.is_positive() {
            return Err(LedgerError::NonPositiveAmount);
        }

        let guid = voucher_guid.into();
        let b_id = batch_id.into();
        let m_id = match_id.into();
        let hash = Self::compute_idempotency_hash(&guid, &b_id, &m_id, amount.amount());

        let lines = vec![
            JournalLine::new(vendor_account, PostingType::Debit, amount)?,
            JournalLine::new(bank_account, PostingType::Credit, amount)?,
        ];

        Ok(Self {
            voucher_guid: guid,
            batch_id: b_id,
            match_id: m_id,
            voucher_number: voucher_number.into(),
            voucher_date: voucher_date.into(),
            partner_name: partner_name.into(),
            partner_tax_id,
            description: description.into(),
            target_erp,
            status: VoucherStatus::Draft,
            lines,
            total_debit: amount,
            total_credit: amount,
            idempotency_hash: hash,
            erp_reference: None,
            posted_at: None,
        })
    }

    /// Constructs a 3-line Customer Receipt with Intermediary Wire Fee (Nợ 1121, Nợ 6425 / Có 131).
    pub fn new_customer_receipt_with_fee(
        voucher_guid: impl Into<String>,
        batch_id: impl Into<String>,
        match_id: impl Into<String>,
        voucher_number: impl Into<String>,
        voucher_date: impl Into<String>,
        partner_name: impl Into<String>,
        partner_tax_id: Option<String>,
        description: impl Into<String>,
        net_bank_amount: Money,
        fee_amount: Money,
        invoice_amount: Money,
        bank_account: impl Into<String>,
        fee_account: impl Into<String>,
        customer_account: impl Into<String>,
        target_erp: ErpTarget,
    ) -> Result<Self, LedgerError> {
        let total_debit = net_bank_amount.checked_add(fee_amount)?;
        if total_debit != invoice_amount {
            return Err(LedgerError::UnbalancedEntry {
                total_debit: total_debit.amount(),
                total_credit: invoice_amount.amount(),
            });
        }

        let guid = voucher_guid.into();
        let b_id = batch_id.into();
        let m_id = match_id.into();
        let hash = Self::compute_idempotency_hash(&guid, &b_id, &m_id, invoice_amount.amount());

        let lines = vec![
            JournalLine::new(bank_account, PostingType::Debit, net_bank_amount)?,
            JournalLine::new(fee_account, PostingType::Debit, fee_amount)?,
            JournalLine::new(customer_account, PostingType::Credit, invoice_amount)?,
        ];

        Ok(Self {
            voucher_guid: guid,
            batch_id: b_id,
            match_id: m_id,
            voucher_number: voucher_number.into(),
            voucher_date: voucher_date.into(),
            partner_name: partner_name.into(),
            partner_tax_id,
            description: description.into(),
            target_erp,
            status: VoucherStatus::Draft,
            lines,
            total_debit,
            total_credit: invoice_amount,
            idempotency_hash: hash,
            erp_reference: None,
            posted_at: None,
        })
    }

    /// Constructs a Composite Split Voucher (1 Bank Debit vs N Credit lines).
    pub fn new_split_receipt(
        voucher_guid: impl Into<String>,
        batch_id: impl Into<String>,
        match_id: impl Into<String>,
        voucher_number: impl Into<String>,
        voucher_date: impl Into<String>,
        partner_name: impl Into<String>,
        partner_tax_id: Option<String>,
        description: impl Into<String>,
        lump_sum_amount: Money,
        split_invoices: &[(String, Money)], // (Account code, amount)
        bank_account: impl Into<String>,
        target_erp: ErpTarget,
    ) -> Result<Self, LedgerError> {
        let mut lines = Vec::with_capacity(split_invoices.len() + 1);
        lines.push(JournalLine::new(bank_account, PostingType::Debit, lump_sum_amount)?);

        let mut total_credit = Money::zero(lump_sum_amount.currency());
        for (acc, amt) in split_invoices {
            total_credit = total_credit.checked_add(*amt)?;
            lines.push(JournalLine::new(acc.clone(), PostingType::Credit, *amt)?);
        }

        if lump_sum_amount != total_credit {
            return Err(LedgerError::UnbalancedEntry {
                total_debit: lump_sum_amount.amount(),
                total_credit: total_credit.amount(),
            });
        }

        let guid = voucher_guid.into();
        let b_id = batch_id.into();
        let m_id = match_id.into();
        let hash = Self::compute_idempotency_hash(&guid, &b_id, &m_id, lump_sum_amount.amount());

        Ok(Self {
            voucher_guid: guid,
            batch_id: b_id,
            match_id: m_id,
            voucher_number: voucher_number.into(),
            voucher_date: voucher_date.into(),
            partner_name: partner_name.into(),
            partner_tax_id,
            description: description.into(),
            target_erp,
            status: VoucherStatus::Draft,
            lines,
            total_debit: lump_sum_amount,
            total_credit,
            idempotency_hash: hash,
            erp_reference: None,
            posted_at: None,
        })
    }

    /// Verifies whether Total Debit == Total Credit.
    pub fn is_balanced(&self) -> bool {
        self.total_debit == self.total_credit
    }

    /// Serializes to MISA AMIS REST API JSON payload (P51).
    pub fn to_misa_json(&self) -> String {
        let lines_json: Vec<serde_json::Value> = self
            .lines
            .iter()
            .map(|l| {
                serde_json::json!({
                    "account_code": l.account_code,
                    "posting_type": if l.posting_type == PostingType::Debit { "DEBIT" } else { "CREDIT" },
                    "amount": l.amount.amount(),
                    "currency": l.amount.currency().code(),
                })
            })
            .collect();

        let payload = serde_json::json!({
            "app_id": "LIVA_BANKING_CORE_V2",
            "org_company_code": "LIVA_ENTERPRISE_VN",
            "voucher_guid": self.voucher_guid,
            "voucher_no": self.voucher_number,
            "voucher_date": self.voucher_date,
            "posted_date": self.voucher_date,
            "currency_id": self.total_debit.currency().code(),
            "partner_name": self.partner_name,
            "partner_tax_id": self.partner_tax_id,
            "description": self.description,
            "total_amount": self.total_debit.amount(),
            "idempotency_hash": self.idempotency_hash,
            "journal_lines": lines_json,
        });

        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
    }

    /// Serializes to FAST Business Online XML document (P51).
    pub fn to_fast_xml(&self) -> String {
        let mut lines_xml = String::new();
        for l in &self.lines {
            lines_xml.push_str(&format!(
                "    <Line>\n      <AccountCode>{}</AccountCode>\n      <PostingType>{}</PostingType>\n      <Amount>{}</Amount>\n    </Line>\n",
                l.account_code,
                if l.posting_type == PostingType::Debit { "Nợ" } else { "Có" },
                l.amount.amount()
            ));
        }

        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<VoucherData xmlns=\"urn:fast:business:online\">\n  <Header>\n    <VoucherGUID>{}</VoucherGUID>\n    <VoucherNo>{}</VoucherNo>\n    <VoucherDate>{}</VoucherDate>\n    <PartnerName>{}</PartnerName>\n    <TotalAmount>{}</TotalAmount>\n    <IdempotencyHash>{}</IdempotencyHash>\n  </Header>\n  <Details>\n{}  </Details>\n</VoucherData>",
            self.voucher_guid,
            self.voucher_number,
            self.voucher_date,
            self.partner_name,
            self.total_debit.amount(),
            self.idempotency_hash,
            lines_xml
        )
    }

    /// Marks the voucher as successfully posted to ERP with returning reference.
    pub fn mark_posted(&mut self, erp_ref: impl Into<String>, timestamp: impl Into<String>) {
        self.status = VoucherStatus::PostedToErp;
        self.erp_reference = Some(erp_ref.into());
        self.posted_at = Some(timestamp.into());
    }

    /// Marks the voucher as failed with error details.
    pub fn mark_failed(&mut self) {
        self.status = VoucherStatus::SyncFailed;
    }
}

/// Bank vs GL Balance Reconciliation Certificate (P53).
///
/// Attests that:
/// `Closing Bank Balance + In-Transit Deposits - In-Transit Withdrawals == Closing GL Balance`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationCertificate {
    pub certificate_id: String,
    pub bank_account: String,
    pub period_start: String,
    pub period_end: String,
    pub bank_opening_balance: Money,
    pub bank_total_credits: Money,
    pub bank_total_debits: Money,
    pub bank_closing_balance: Money,
    pub gl_opening_balance: Money,
    pub gl_total_debits: Money,
    pub gl_total_credits: Money,
    pub gl_closing_balance: Money,
    pub in_transit_deposits: Money,
    pub in_transit_withdrawals: Money,
    pub adjusted_bank_balance: Money,
    pub variance: Money,
    pub is_certified: bool,
    pub merkle_root_hash: String,
    pub certified_by_checker: Option<String>,
    pub certified_at: Option<String>,
}

impl ReconciliationCertificate {
    /// Computes adjusted balances and verifies mathematical reconciliation invariant.
    pub fn create_and_verify(
        certificate_id: impl Into<String>,
        bank_account: impl Into<String>,
        period_start: impl Into<String>,
        period_end: impl Into<String>,
        bank_opening: Money,
        bank_credits: Money,
        bank_debits: Money,
        bank_closing: Money,
        gl_opening: Money,
        gl_debits: Money,
        gl_credits: Money,
        gl_closing: Money,
        in_transit_deposits: Money,
        in_transit_withdrawals: Money,
        merkle_root_hash: impl Into<String>,
    ) -> Result<Self, LedgerError> {
        // Adjusted Bank Balance = Bank Closing + In-Transit Deposits - In-Transit Withdrawals
        let adjusted_bank = bank_closing
            .checked_add(in_transit_deposits)?
            .checked_sub(in_transit_withdrawals)?;

        // Variance = Adjusted Bank - GL Closing
        let variance = adjusted_bank.checked_sub(gl_closing)?;
        let is_certified = variance.is_zero();

        Ok(Self {
            certificate_id: certificate_id.into(),
            bank_account: bank_account.into(),
            period_start: period_start.into(),
            period_end: period_end.into(),
            bank_opening_balance: bank_opening,
            bank_total_credits: bank_credits,
            bank_total_debits: bank_debits,
            bank_closing_balance: bank_closing,
            gl_opening_balance: gl_opening,
            gl_total_debits: gl_debits,
            gl_total_credits: gl_credits,
            gl_closing_balance: gl_closing,
            in_transit_deposits,
            in_transit_withdrawals,
            adjusted_bank_balance: adjusted_bank,
            variance,
            is_certified,
            merkle_root_hash: merkle_root_hash.into(),
            certified_by_checker: None,
            certified_at: None,
        })
    }

    /// Signs the certification as Checker (R04 Kế toán trưởng).
    pub fn sign_off(&mut self, checker_id: impl Into<String>, timestamp: impl Into<String>) {
        self.certified_by_checker = Some(checker_id.into());
        self.certified_at = Some(timestamp.into());
    }
}
