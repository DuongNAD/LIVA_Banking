import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useLedgerStore } from '../../src/stores/ledgerStore';
import { useWorkbenchStore } from '../../src/stores/workbenchStore';

describe('ledgerStore (P50–P53 Sổ Cái & Bút Toán ERP)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('initializes and automatically synthesizes vouchers from workbench matches', () => {
    const ledger = useLedgerStore();
    expect(ledger.vouchers.length).toBeGreaterThan(0);
    expect(ledger.voucherStats.total).toBeGreaterThan(0);
    expect(ledger.voucherStats.totalAmount).toBeGreaterThan(0);

    // Verify all generated vouchers are strictly balanced (Delta == 0)
    for (const v of ledger.vouchers) {
      expect(v.difference).toBe(0);
      expect(v.totalDebit).toBe(v.totalCredit);
      expect(v.lines.length).toBeGreaterThanOrEqual(2);
      expect(v.idempotencyHash).toMatch(/^0x[0-9a-f]{16,32}$/);
    }
  });

  it('filters vouchers by status, ERP target, and search text', () => {
    const ledger = useLedgerStore();
    expect(ledger.filteredVouchers.length).toBe(ledger.vouchers.length);

    ledger.filterStatus = 'POSTED_TO_ERP';
    expect(ledger.filteredVouchers.every((v) => v.status === 'POSTED_TO_ERP')).toBe(true);

    ledger.filterStatus = 'ALL';
    ledger.filterErp = 'MISA';
    expect(ledger.filteredVouchers.every((v) => v.targetErp === 'MISA')).toBe(true);

    ledger.searchQuery = 'PKT';
    expect(ledger.filteredVouchers.every((v) => v.voucherNumber.includes('PKT'))).toBe(true);
  });

  it('posts a single voucher to ERP and updates status and reference', async () => {
    const ledger = useLedgerStore();
    const first = ledger.vouchers[0];
    expect(first.status).toBe('DRAFT');

    await ledger.postVoucherToErp(first.id, 'MISA');
    expect(first.status).toBe('POSTED_TO_ERP');
    expect(first.erpReference).toBeDefined();
    expect(first.erpReference).toContain('MISA-VOUCHER');
    expect(first.postedAt).toBeDefined();
  });

  it('performs batch posting to ERP with success count', async () => {
    const ledger = useLedgerStore();
    const count = await ledger.batchPostToErp(undefined, 'FAST');
    expect(count).toBeGreaterThan(0);
    expect(ledger.voucherStats.posted).toBe(ledger.voucherStats.total);
    expect(ledger.voucherStats.pending).toBe(0);
  });

  it('generates valid MISA AMIS REST API JSON payload', () => {
    const ledger = useLedgerStore();
    const first = ledger.vouchers[0];
    const jsonStr = ledger.getMisaJson(first);
    expect(jsonStr).toBeDefined();

    const parsed = JSON.parse(jsonStr);
    expect(parsed.app_id).toBe('LIVA_BANKING_CORE_V2');
    expect(parsed.voucher_guid).toBe(first.voucherGuid);
    expect(parsed.total_amount).toBe(first.totalDebit);
    expect(parsed.journal_lines.length).toBe(first.lines.length);
  });

  it('generates valid FAST Business Online XML payload', () => {
    const ledger = useLedgerStore();
    const first = ledger.vouchers[0];
    const xmlStr = ledger.getFastXml(first);
    expect(xmlStr).toContain('<VoucherData');
    expect(xmlStr).toContain(`<VoucherGUID>${first.voucherGuid}</VoucherGUID>`);
    expect(xmlStr).toContain(`<TotalAmount>${first.totalDebit}</TotalAmount>`);
    expect(xmlStr).toContain('</VoucherData>');
  });

  it('verifies P53 Bank vs GL Reconciliation Certificate (Zero Variance)', () => {
    const ledger = useLedgerStore();
    expect(ledger.certificate.isCertified).toBe(true);
    expect(ledger.certificate.variance).toBe(0);
    expect(ledger.certificate.merkleRootHash).toBeDefined();

    // Checker sign-off
    ledger.certifyCertificate('USR-CHECKER-KTT-01');
    expect(ledger.certificate.certifiedByChecker).toBe('USR-CHECKER-KTT-01');
    expect(ledger.certificate.certifiedAt).toBeDefined();
  });

  it('synthesizes voucher from approved quarantine item', () => {
    const workbench = useWorkbenchStore();
    const ledger = useLedgerStore();

    // Approve q-02 in workbench
    workbench.switchOperatorRole('CHECKER');
    workbench.approveQuarantineItem('q-02', 'Duyet chi phi chuyen tien ngoai te TK 6425');

    // Re-generate vouchers
    ledger.generateVouchersFromWorkbench();
    const qVoucher = ledger.vouchers.find((v) => v.sourceType === 'QUARANTINE_RELEASE');
    expect(qVoucher).toBeDefined();
    expect(qVoucher?.difference).toBe(0);
    expect(qVoucher?.lines.some((l) => l.accountCode === '6425')).toBe(true);
  });
});
