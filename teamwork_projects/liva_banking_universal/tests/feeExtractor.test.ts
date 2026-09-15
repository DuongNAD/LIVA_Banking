import { describe, it, expect } from 'vitest';
import {
  disentangleWireFee,
  checkWireFeeDiscrepancy,
  KNOWN_VIETNAMESE_WIRE_FEES,
} from '../src/engine/intelligence/feeExtractor';

describe('Wire Fee Disentanglement (Feature F10)', () => {
  it('should list all standard Vietnamese wire fee amounts', () => {
    expect(KNOWN_VIETNAMESE_WIRE_FEES).toContain(1100);
    expect(KNOWN_VIETNAMESE_WIRE_FEES).toContain(2200);
    expect(KNOWN_VIETNAMESE_WIRE_FEES).toContain(3300);
    expect(KNOWN_VIETNAMESE_WIRE_FEES).toContain(5500);
    expect(KNOWN_VIETNAMESE_WIRE_FEES).toContain(7700);
    expect(KNOWN_VIETNAMESE_WIRE_FEES).toContain(8800);
    expect(KNOWN_VIETNAMESE_WIRE_FEES).toContain(9900);
    expect(KNOWN_VIETNAMESE_WIRE_FEES).toContain(11000);
    expect(KNOWN_VIETNAMESE_WIRE_FEES).toContain(22000);
  });

  it('should disentangle 2,200 VND Napas wire fee from transaction principal', () => {
    const res = disentangleWireFee(45_000_000, 'ck tien hang phi 2200');
    expect(res.feeAmount).toBe(2200);
    expect(res.principal).toBe(44_997_800);
    expect(res.accountCode).toBe('TK 6425');
    expect(res.accountName).toBe('Chi phí dịch vụ ngân hàng');
    expect(res.sumCheckPassed).toBe(true);
  });

  it('should disentangle 1,100 VND wire fee and allocate to TK 6425', () => {
    const res = disentangleWireFee(10_000_000, 'chuyen tien phi 1100');
    expect(res.feeAmount).toBe(1100);
    expect(res.principal).toBe(9_998_900);
    expect(res.accountCode).toBe('TK 6425');
    expect(res.sumCheckPassed).toBe(true);
  });

  it('should handle pure bank account fee as 100% fee without principal', () => {
    const res = disentangleWireFee(2_200_000, 'Phi duy tri quan ly tai khoan');
    expect(res.isPureFee).toBe(true);
    expect(res.feeAmount).toBe(2_200_000);
    expect(res.principal).toBe(0);
    expect(res.accountCode).toBe('TK 6425');
    expect(res.sumCheckPassed).toBe(true);
  });

  it('should verify arithmetic invariant: Principal + Fee == Gross Amount', () => {
    const gross = 85_000_000;
    const res = disentangleWireFee(gross, 'Phi chuyen khoan 5500');
    expect(res.sumCheckPassed).toBe(true);
    expect(res.principal + res.feeAmount).toBe(gross);
  });

  it('should return zero fee when transaction contains no wire fee marker', () => {
    const res = disentangleWireFee(120_000_000, 'Thanh toan tien may bien ap');
    expect(res.feeAmount).toBe(0);
    expect(res.principal).toBe(120_000_000);
    expect(res.accountCode).toBeNull();
    expect(res.sumCheckPassed).toBe(true);
  });

  it('should handle transaction equal to fee amount (0 principal)', () => {
    const res = disentangleWireFee(2200, 'phi 2200');
    expect(res.feeAmount).toBe(2200);
    expect(res.principal).toBe(0);
    expect(res.sumCheckPassed).toBe(true);
  });

  it('should retain exact precision on 10 billion VND gross amount with 2,200 fee', () => {
    const gross = 10_000_000_000;
    const res = disentangleWireFee(gross, 'Phi chuyen tien 2200');
    expect(res.feeAmount).toBe(2200);
    expect(res.principal).toBe(9_999_997_800);
    expect(res.sumCheckPassed).toBe(true);
  });

  it('should not confuse invoice number containing 2200 as a fee without fee keyword', () => {
    const res = disentangleWireFee(50_000_000, 'Thanh toan tien hang HD-2200');
    expect(res.feeAmount).toBe(0);
    expect(res.principal).toBe(50_000_000);
  });

  it('should handle zero gross amount cleanly', () => {
    const res = disentangleWireFee(0, '');
    expect(res.feeAmount).toBe(0);
    expect(res.principal).toBe(0);
    expect(res.sumCheckPassed).toBe(true);
  });

  it('should match bank transaction and ledger invoice delta', () => {
    // Case 1: Bank deducted 2,200 VND fee from invoice payment
    const diff1 = checkWireFeeDiscrepancy(99_997_800, 100_000_000);
    expect(diff1.isFeeDiscrepancy).toBe(true);
    expect(diff1.feeAmount).toBe(2200);
    expect(diff1.direction).toBe('BANK_DEDUCTED_FEE');

    // Case 2: Payer covered 5,500 VND fee
    const diff2 = checkWireFeeDiscrepancy(100_005_500, 100_000_000);
    expect(diff2.isFeeDiscrepancy).toBe(true);
    expect(diff2.feeAmount).toBe(5500);
    expect(diff2.direction).toBe('PAYER_COVERED_FEE');

    // Case 3: Arbitrary discrepancy (not a standard bank fee)
    const diff3 = checkWireFeeDiscrepancy(95_000_000, 100_000_000);
    expect(diff3.isFeeDiscrepancy).toBe(false);
  });
});
