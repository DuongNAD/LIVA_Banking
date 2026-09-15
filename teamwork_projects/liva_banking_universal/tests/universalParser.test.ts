import { describe, it, expect } from 'vitest';
import {
  parseVietnameseAmount,
  parseDateAndTimestamp,
  detectBankCode,
  extractReferenceTokens,
  parseCsvOrTsv,
  parsePastedTable,
} from '../src/engine/ingestion/universalParser';

describe('UniversalParser', () => {
  describe('parseVietnameseAmount', () => {
    it('should parse Vietnamese dot-thousands comma-decimals correctly', () => {
      expect(parseVietnameseAmount('15.000.000,00')).toBe(15000000);
      expect(parseVietnameseAmount('145.000.000')).toBe(145000000);
      expect(parseVietnameseAmount('2.200.000,50')).toBe(2200000);
    });

    it('should parse US comma-thousands dot-decimals correctly', () => {
      expect(parseVietnameseAmount('15,000,000.00')).toBe(15000000);
      expect(parseVietnameseAmount('1,450,230,000')).toBe(1450230000);
    });

    it('should handle parenthesized negative amounts with trailing currency tokens', () => {
      expect(parseVietnameseAmount('(50.000.000)')).toBe(-50000000);
      expect(parseVietnameseAmount('(35.000.000) VND')).toBe(-35000000);
      expect(parseVietnameseAmount('(35.000.000) VNĐ')).toBe(-35000000);
      expect(parseVietnameseAmount('(35.000.000) Đ')).toBe(-35000000);
      expect(parseVietnameseAmount('-2.200.000')).toBe(-2200000);
    });

    it('should reject invalid strings, timestamps, and hyphenated dates', () => {
      expect(parseVietnameseAmount('01/08/2026')).toBe(0);
      expect(parseVietnameseAmount('12-08-2026')).toBe(0);
      expect(parseVietnameseAmount('2026-08-12')).toBe(0);
      expect(parseVietnameseAmount('14:25:30')).toBe(0);
      expect(parseVietnameseAmount('INV/2026/88')).toBe(0);
      expect(parseVietnameseAmount('random text')).toBe(0);
    });

    it('should parse amounts from labeled cells with colons', () => {
      expect(parseVietnameseAmount('Cộng: 50.000.000 VNĐ')).toBe(50000000);
      expect(parseVietnameseAmount('Số dư đầu kỳ: 500.000.000 VND')).toBe(500000000);
      expect(parseVietnameseAmount('Tổng nợ: 12.000.000')).toBe(12000000);
    });
  });

  describe('parseDateAndTimestamp', () => {
    it('should normalize VN date DD/MM/YYYY HH:MM:SS', () => {
      const res = parseDateAndTimestamp('01/08/2026 09:15:30');
      expect(res.isoDate).toBe('2026-08-01');
      expect(res.time).toBe('09:15:30');
      expect(res.epochSeconds).toBeGreaterThan(1700000000);
    });

    it('should normalize ISO date YYYY-MM-DD', () => {
      const res = parseDateAndTimestamp('2026-08-05');
      expect(res.isoDate).toBe('2026-08-05');
    });
  });

  describe('detectBankCode', () => {
    it('should detect bank from filename and text', () => {
      expect(detectBankCode('01_VCB_SaoKe.xlsx')).toBe('VCB');
      expect(detectBankCode('techcombank_statement.csv')).toBe('TCB');
      expect(detectBankCode('', 'Ngan hang Dau tu va Phat trien BIDV')).toBe('BIDV');
    });
  });

  describe('extractReferenceTokens', () => {
    it('should extract FT and invoice tokens', () => {
      const res = extractReferenceTokens('Napas VietQR TT HD-2026-88 An Phat FT2621400192');
      expect(res.ftNumber).toBe('FT2621400192');
      expect(res.docRef).toBe('HD-2026-88');
    });
  });

  describe('parseCsvOrTsv and parsePastedTable', () => {
    it('should parse semicolon-delimited CSV with metadata and transactions', () => {
      const csv = `Số tài khoản:;19034567890123;;;;
Số dư đầu kỳ:;785.600.000;;;;
Ngày giao dịch;Mã giao dịch;Số tiền ghi nợ;Số tiền ghi có;Số dư;Nội dung chi tiết;Tên đối tác
01/08/2026 08:30:00;TCB01;;45.000.000;830.600.000;Napas VietQR TT HD131;Công ty An Phát
05/08/2026 15:40:00;TCB02;2.200.000;;828.400.000;Phi duy tri tai khoan;Techcombank`;

      const result = parseCsvOrTsv(csv, 'tcb_sample.csv');
      expect(result.bankCode).toBe('TCB');
      expect(result.accountNumber).toBe('19034567890123');
      expect(result.openingBalance).toBe(785600000);
      expect(result.transactions.length).toBe(2);
      expect(result.transactions[0].credit).toBe(45000000);
      expect(result.transactions[1].debit).toBe(2200000);
      expect(result.balanceInvariantPassed).toBe(true);
    });

    it('should parse pasted tabular clipboard data', () => {
      const pasted = `Ngày giao dịch\tMã giao dịch\tSố tiền ghi nợ\tSố tiền ghi có\tSố dư\tNội dung chi tiết\tTên đối tác\n01/08/2026 09:15\tVCB01\t\t145000000\t1595230000\tCT TT TIEN HANG HOP DONG SO 882026\tCONG TY AN PHAT`;

      const result = parsePastedTable(pasted, 'VCB');
      expect(result.bankCode).toBe('VCB');
      expect(result.transactions.length).toBe(1);
      expect(result.transactions[0].amount).toBe(145000000);
    });

    it('should append multi-row continuation narrations when amount is 0', () => {
      const csv = `Ngày giao dịch;Mã giao dịch;Số tiền ghi có;Số dư;Nội dung chi tiết;Tên đối tác
01/08/2026;TX01;145.000.000;145.000.000;Thanh toan hop dong 88;CONG TY AN PHAT
;;;;giao hang dot 2 kem bien ban ban giao;
02/08/2026;TX02;50.000.000;195.000.000;Thanh toan dot tiep theo;CONG TY HOA BINH`;

      const result = parseCsvOrTsv(csv, 'wrapped_continuation.csv');
      expect(result.transactions.length).toBe(2);
      expect(result.transactions[0].narration).toBe('Thanh toan hop dong 88 giao hang dot 2 kem bien ban ban giao');
      expect(result.transactions[1].narration).toBe('Thanh toan dot tiep theo');
    });
  });
});
