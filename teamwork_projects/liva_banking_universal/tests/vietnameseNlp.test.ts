import { describe, it, expect } from 'vitest';
import {
  normalizeVietnameseIntent,
  removeVietnameseAccents,
  matchRemarksToLedger,
  VIETNAMESE_BANKING_ABBREVIATIONS,
} from '../src/engine/intelligence/vietnameseNlp';
import type { LedgerEntry } from '../src/types/banking';

describe('Vietnamese Semantic Intent Normalizer (Feature F09)', () => {
  it('should expand common Vietnamese banking abbreviations', () => {
    expect(VIETNAMESE_BANKING_ABBREVIATIONS['ck']).toBe('chuyển khoản');
    expect(VIETNAMESE_BANKING_ABBREVIATIONS['tt']).toBe('thanh toán');
    expect(VIETNAMESE_BANKING_ABBREVIATIONS['ung']).toBe('tạm ứng');
    expect(VIETNAMESE_BANKING_ABBREVIATIONS['hd']).toBe('hóa đơn');
    expect(VIETNAMESE_BANKING_ABBREVIATIONS['mst']).toBe('mã số thuế');
  });

  it('should expand abbreviations and classify payment intent', () => {
    const res = normalizeVietnameseIntent('ck tien may bom hd 88');
    expect(res.normalized).toContain('chuyển khoản');
    expect(res.intent).toBe('PAYMENT');
    expect(res.invoiceNumbers).toContain('88');
  });

  it('should extract invoice references with alphanumeric patterns', () => {
    const res1 = normalizeVietnameseIntent('tt hd 131');
    expect(res1.invoiceNumbers).toContain('131');

    const res2 = normalizeVietnameseIntent('Napas VietQR TT HD-2026-88 An Phat');
    expect(res2.invoiceNumbers).toContain('2026-88');

    const res3 = normalizeVietnameseIntent('INV-0099 thanh toan hop dong');
    expect(res3.invoiceNumbers).toContain('0099');
  });

  it('should extract Vietnamese tax codes (MST) 10 and 13 digits', () => {
    const res = normalizeVietnameseIntent('Thanh toan tien hang MST: 0101234567-001 cong ty X');
    expect(res.taxCodes).toContain('0101234567-001');

    const res2 = normalizeVietnameseIntent('MST 0312345678 thanh toan dich vu');
    expect(res2.taxCodes).toContain('0312345678');
  });

  it('should extract mobile phone numbers (10 digits)', () => {
    const res = normalizeVietnameseIntent('ZaloPay 0912345678 chuyen tien an trua');
    expect(res.phoneNumbers).toContain('0912345678');

    const res2 = normalizeVietnameseIntent('Lien he +84987654321 de nhan hang');
    expect(res2.phoneNumbers).toContain('+84987654321');
  });

  it('should extract Napas/VietQR trace codes', () => {
    const res = normalizeVietnameseIntent('NPS9876543210 thanh toan qua Napas 247');
    expect(res.traceCodes).toContain('NPS9876543210');

    const res2 = normalizeVietnameseIntent('FT26228899120011 chuyen tien noi bo VCB');
    expect(res2.traceCodes).toContain('FT26228899120011');
  });

  it('should classify advance payment intent correctly', () => {
    const res = normalizeVietnameseIntent('tam ung chi phi cong tac');
    expect(res.intent).toBe('ADVANCE');

    const res2 = normalizeVietnameseIntent('ung tien vat tu dot 1');
    expect(res2.intent).toBe('ADVANCE');
  });

  it('should classify bank fee intent correctly', () => {
    const res = normalizeVietnameseIntent('phi duy tri quan ly tai khoan doanh nghiep');
    expect(res.intent).toBe('BANK_FEE');
  });

  it('should classify payroll intent correctly', () => {
    const res = normalizeVietnameseIntent('thanh toan tien luong thang 8 cho can bo nhan vien');
    expect(res.intent).toBe('PAYROLL');
  });

  it('should handle all-uppercase input without distortion', () => {
    const res = normalizeVietnameseIntent('CK TIEN MAY BOM HD 88');
    expect(res.normalized).toContain('chuyển khoản');
    expect(res.invoiceNumbers).toContain('88');
  });

  it('should handle punctuation-dense input', () => {
    const res = normalizeVietnameseIntent('CK...TIEN,,MAY--BOM//HD::88');
    expect(res.invoiceNumbers).toContain('88');
  });

  it('should safely handle empty or whitespace-only memo', () => {
    const res = normalizeVietnameseIntent('   ');
    expect(res.normalized).toBe('');
    expect(res.invoiceNumbers.length).toBe(0);
    expect(res.intent).toBe('OTHER');
  });

  it('should handle 1,000 character memo without performance issues', () => {
    const longMemo = 'ck tien hang '.repeat(100);
    const start = performance.now();
    const res = normalizeVietnameseIntent(longMemo);
    const duration = performance.now() - start;

    expect(duration).toBeLessThan(50);
    expect(res.intent).toBe('PAYMENT');
  });

  it('should remove Vietnamese accents accurately', () => {
    expect(removeVietnameseAccents('Nguyễn Anh Dương')).toBe('Nguyen Anh Duong');
    expect(removeVietnameseAccents('Đồng tiền phát sinh')).toBe('Dong tien phat sinh');
  });

  it('should match remarks to candidate ledger invoices with high confidence', () => {
    const ledgers: LedgerEntry[] = [
      {
        id: 'l1',
        docNo: 'HD-88',
        entryDate: '2026-08-10',
        entryTimestamp: 1723248000,
        amount: 145_000_000,
        entryType: 'CREDIT',
        partnerName: 'Cong ty An Phat',
        description: 'Tien may bom',
        status: 'UNMATCHED',
      },
      {
        id: 'l2',
        docNo: 'HD-99',
        entryDate: '2026-08-10',
        entryTimestamp: 1723248000,
        amount: 50_000_000,
        entryType: 'CREDIT',
        partnerName: 'Cong ty Binh Minh',
        description: 'Tien vat tu',
        status: 'UNMATCHED',
      },
    ];

    const matchRes = matchRemarksToLedger('ck tien may bom cong ty an phat theo hd 88', ledgers);
    expect(matchRes.matchedLedger?.id).toBe('l1');
    expect(matchRes.confidence).toBeGreaterThanOrEqual(0.85);
    expect(matchRes.explanation).toContain('HD-88');
  });
});
