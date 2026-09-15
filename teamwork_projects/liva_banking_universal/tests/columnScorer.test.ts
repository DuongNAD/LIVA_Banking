import { describe, it, expect } from 'vitest';
import { classifyHeader, scoreCandidateHeaderRows, extractStatementMetadata } from '../src/engine/ingestion/columnScorer';

describe('ColumnScorer', () => {
  it('should correctly classify common Vietnamese banking column headers', () => {
    expect(classifyHeader('Ngày giao dịch')).toBe('date');
    expect(classifyHeader('Mã GD')).toBe('code');
    expect(classifyHeader('Số tiền ghi nợ')).toBe('debit');
    expect(classifyHeader('Số tiền ghi có')).toBe('credit');
    expect(classifyHeader('Số dư')).toBe('balance');
    expect(classifyHeader('Nội dung chi tiết')).toBe('narration');
    expect(classifyHeader('Tên người gửi/thụ hưởng')).toBe('counterparty');
    expect(classifyHeader('Ngày giá trị')).toBe('valueDate');
  });

  it('should accurately identify the header row in a multi-line bank statement table', () => {
    const table = [
      ['NGAN HANG TMCP NGOAI THUONG VIET NAM'],
      ['Số tài khoản: 0071001234567', '', 'Tên tài khoản: LIVA SOLUTIONS'],
      ['Kỳ sao kê: 01/08/2026 - 31/08/2026'],
      ['Ngày giao dịch', 'Mã GD', 'Số tiền ghi nợ', 'Số tiền ghi có', 'Số dư', 'Nội dung chi tiết', 'Tên người gửi/thụ hưởng'],
      ['01/08/2026 09:15', 'VCB01', '', '145000000', '1595230000', 'Thanh toan tien hang', 'CONG TY AN PHAT'],
    ];

    const result = scoreCandidateHeaderRows(table);
    expect(result.headerRowIndex).toBe(3);
    expect(result.mapping.date).toBe(0);
    expect(result.mapping.code).toBe(1);
    expect(result.mapping.debit).toBe(2);
    expect(result.mapping.credit).toBe(3);
    expect(result.mapping.balance).toBe(4);
    expect(result.mapping.narration).toBe(5);
    expect(result.mapping.counterparty).toBe(6);
  });

  it('should extract metadata from pre-header rows', () => {
    const table = [
      ['Số tài khoản: 0071001234567', '', 'Tên tài khoản: CONG TY TNHH LIVA SOLUTIONS'],
      ['Số dư đầu kỳ: 1.450.230.000 VND'],
      ['Ngày giao dịch', 'Mã GD', 'Số tiền ghi nợ', 'Số tiền ghi có', 'Số dư', 'Nội dung chi tiết'],
    ];

    const metadata = extractStatementMetadata(table, 2, (raw) => {
      const num = raw.replace(/[^\d]/g, '');
      return num ? parseInt(num, 10) : 0;
    });

    expect(metadata.accountNumber).toBe('0071001234567');
    expect(metadata.accountName).toContain('CONG TY TNHH LIVA SOLUTIONS');
    expect(metadata.openingBalance).toBe(1450230000);
  });
});
