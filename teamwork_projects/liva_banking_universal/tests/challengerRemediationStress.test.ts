/**
 * Empirical Challenger Independent Remediation Re-check Suite
 * Milestone 1: Universal Statement Ingestion & 3-Tier Reconciliation Engine
 *
 * Direct adversarial stress verification of all 7 remediations:
 * 1. Parenthesized negative amount with trailing currency suffixes: '(35.000.000) VND' -> -35,000,000
 * 2. Hyphenated dates rejection: '12-08-2026' and '2026-08-12' return 0 (never treated as amounts)
 * 3. Labeled amounts parsing: 'Cộng: 50.000.000 VNĐ' parses as 50,000,000 while timestamps '14:30' return 0
 * 4. Multi-row continuation narrations: blank amount rows merge into previous transaction narration
 * 5. Escaped quotes within CSV lines: 'splitCsvLine' preserves RFC 4180 inner quotes and cell boundaries
 * 6. Subset-sum branch-and-bound solver: 60 identical candidates at k=8 runs in < 5ms (worst-case impossible target)
 * 7. Running balance continuity: detects exact +/- 1 VND discrepancy with zero tolerance
 */

import { describe, it, expect } from 'vitest';
import {
  parseVietnameseAmount,
  parseCsvOrTsv,
} from '../src/engine/ingestion/universalParser';
import { splitCsvLine } from '../src/engine/ingestion/delimiterSniffer';
import {
  solveExactSubsetSumBnb,
} from '../src/engine/reconciliation/tier3SplitSolver';
import {
  verifyBalanceInvariants,
  verifyRunningBalanceContinuity,
} from '../src/engine/reconciliation/balanceValidator';
import type { RawStatementRow } from '../src/types/banking';

describe('Challenger Re-check 1: Parenthesized Negative Amounts with Currency Tokens', () => {
  it('parses (35.000.000) VND as signed negative -35,000,000', () => {
    expect(parseVietnameseAmount('(35.000.000) VND')).toBe(-35_000_000);
    expect(parseVietnameseAmount('(35.000.000) VNĐ')).toBe(-35_000_000);
    expect(parseVietnameseAmount('(35.000.000) Đ')).toBe(-35_000_000);
    expect(parseVietnameseAmount('(35.000.000) đ')).toBe(-35_000_000);
  });

  it('parses various parenthesized negative combinations with accounting suffixes and spaces', () => {
    expect(parseVietnameseAmount('(35.000.000) CR')).toBe(-35_000_000);
    expect(parseVietnameseAmount('(35.000.000) DB')).toBe(-35_000_000);
    expect(parseVietnameseAmount('(35.000.000) NO')).toBe(-35_000_000);
    expect(parseVietnameseAmount('(35.000.000) CO')).toBe(-35_000_000);
    expect(parseVietnameseAmount('(35.000.000) NỢ')).toBe(-35_000_000);
    expect(parseVietnameseAmount('(35.000.000) CÓ')).toBe(-35_000_000);
    expect(parseVietnameseAmount('  ( 35.000.000 )   VND  ')).toBe(-35_000_000);
    expect(parseVietnameseAmount('(35.000.000 VND)')).toBe(-35_000_000);
    expect(parseVietnameseAmount('(35.000.000 VNĐ)')).toBe(-35_000_000);
    expect(parseVietnameseAmount('(35,000,000.00) VND')).toBe(-35_000_000);
    expect(parseVietnameseAmount('(35.000.000,00) VNĐ')).toBe(-35_000_000);
    expect(parseVietnameseAmount('-(35.000.000) VND')).toBe(-35_000_000);
  });

  it('retains positive values when no parens or minus exist', () => {
    expect(parseVietnameseAmount('35.000.000 VND')).toBe(35_000_000);
    expect(parseVietnameseAmount('35.000.000 VNĐ')).toBe(35_000_000);
    expect(parseVietnameseAmount('35.000.000 Đ')).toBe(35_000_000);
    expect(parseVietnameseAmount('35,000,000.00')).toBe(35_000_000);
  });
});

describe('Challenger Re-check 2: Hyphenated Dates Rejection', () => {
  it('strictly returns 0 for hyphenated dates in DD-MM-YYYY and YYYY-MM-DD formats', () => {
    expect(parseVietnameseAmount('12-08-2026')).toBe(0);
    expect(parseVietnameseAmount('2026-08-12')).toBe(0);
    expect(parseVietnameseAmount('01-01-2025')).toBe(0);
    expect(parseVietnameseAmount('31-12-2024')).toBe(0);
    expect(parseVietnameseAmount('1-8-2026')).toBe(0);
    expect(parseVietnameseAmount('12-8-2026')).toBe(0);
    expect(parseVietnameseAmount('01-08-26')).toBe(0);
    expect(parseVietnameseAmount('2026-8-12')).toBe(0);
    expect(parseVietnameseAmount('2026-12-8')).toBe(0);
    expect(parseVietnameseAmount(' 12-08-2026 ')).toBe(0);
  });

  it('rejects slash dates and timestamps while permitting legitimate negative amounts', () => {
    expect(parseVietnameseAmount('12/08/2026')).toBe(0);
    expect(parseVietnameseAmount('2026/08/12')).toBe(0);
    expect(parseVietnameseAmount('14:30:00')).toBe(0);
    expect(parseVietnameseAmount('08:15')).toBe(0);

    // Legitimate negative amounts must NOT be mistaken for dates
    expect(parseVietnameseAmount('-12.000.000')).toBe(-12_000_000);
    expect(parseVietnameseAmount('-35000000')).toBe(-35_000_000);
    expect(parseVietnameseAmount('-500.000 VNĐ')).toBe(-500_000);
  });
});

describe('Challenger Re-check 3: Labeled Amounts Parsing', () => {
  it('correctly parses labeled amounts containing colons', () => {
    expect(parseVietnameseAmount('Cộng: 50.000.000 VNĐ')).toBe(50_000_000);
    expect(parseVietnameseAmount('Cộng: 50.000.000')).toBe(50_000_000);
    expect(parseVietnameseAmount('Số dư đầu kỳ: 500.000.000 VND')).toBe(500_000_000);
    expect(parseVietnameseAmount('Số dư cuối kỳ: 1.250.000.000 VNĐ')).toBe(1_250_000_000);
    expect(parseVietnameseAmount('Tổng cộng phát sinh Có: 350.000.000')).toBe(350_000_000);
    expect(parseVietnameseAmount('Tổng cộng phát sinh Nợ: 120.000.000')).toBe(120_000_000);
    expect(parseVietnameseAmount('Total: 1,500,000.00 USD')).toBe(1_500_000);
  });

  it('correctly parses labeled amounts with negative figures', () => {
    expect(parseVietnameseAmount('Tổng nợ: (35.000.000) VND')).toBe(-35_000_000);
    expect(parseVietnameseAmount('Ghi nợ: -15.000.000 VNĐ')).toBe(-15_000_000);
  });

  it('rejects timestamp cells containing colons and empty labeled cells', () => {
    expect(parseVietnameseAmount('14:30')).toBe(0);
    expect(parseVietnameseAmount('14:30:00')).toBe(0);
    expect(parseVietnameseAmount('08:00:15')).toBe(0);
    expect(parseVietnameseAmount('Cộng:')).toBe(0);
    expect(parseVietnameseAmount('Ghi chú: Khong co tien')).toBe(0);
  });
});

describe('Challenger Re-check 4: Multi-Row Wrapped Continuation Narrations', () => {
  it('preserves and merges multi-row continuation narrations across wrapped rows', () => {
    const csvContent = [
      'Ngày giao dịch;Mã giao dịch;Số tiền ghi có;Số dư;Nội dung chi tiết;Tên đối tác',
      '01/08/2026;TX01;145.000.000;145.000.000;Thanh toan hop dong 88 dot 1;CONG TY AN PHAT',
      ';;;;kem bien ban nghiem thu giai doan A va hoa don GTGT so 00192;',
      ';;;;giao hang tai kho Tong Cty theo bien ban so 45;',
      '02/08/2026;TX02;50.000.000;195.000.000;Tam ung hop dong 89;CONG TY HOA BINH',
      ';;;;ghi chu: chuyen khoan qua cong Napas 247;',
    ].join('\n');

    const result = parseCsvOrTsv(csvContent, 'multi_row_continuation.csv');

    expect(result.transactions.length).toBe(2);

    // TX01 should have lines 2 and 3 appended
    const expectedTx01Narration =
      'Thanh toan hop dong 88 dot 1 kem bien ban nghiem thu giai doan A va hoa don GTGT so 00192 giao hang tai kho Tong Cty theo bien ban so 45';
    expect(result.transactions[0].narration).toBe(expectedTx01Narration);
    expect(result.transactions[0].amount).toBe(145_000_000);
    expect(result.transactions[0].credit).toBe(145_000_000);

    // TX02 should have line 5 appended
    const expectedTx02Narration =
      'Tam ung hop dong 89 ghi chu: chuyen khoan qua cong Napas 247';
    expect(result.transactions[1].narration).toBe(expectedTx02Narration);
    expect(result.transactions[1].amount).toBe(50_000_000);
    expect(result.transactions[1].credit).toBe(50_000_000);

    expect(result.balanceInvariantPassed).toBe(true);
  });

  it('gracefully handles leading metadata lines before first transaction without throwing', () => {
    const csvWithLeadingText = [
      'NGAN HANG TMCP NGOAI THUONG VIET NAM;;;;;',
      'SO PHU TAI KHOAN DOANH NGHIEP;;;;;',
      'Ngày giao dịch;Mã giao dịch;Số tiền ghi có;Số dư;Nội dung chi tiết;Tên đối tác',
      '01/08/2026;TX01;10.000.000;10.000.000;Thanh toan tien dien;CONG TY DIEN LUC',
    ].join('\n');

    const result = parseCsvOrTsv(csvWithLeadingText, 'vcb_header.csv');
    expect(result.transactions.length).toBe(1);
    expect(result.transactions[0].amount).toBe(10_000_000);
    expect(result.transactions[0].narration).toBe('Thanh toan tien dien');
  });
});

describe('Challenger Re-check 5: Escaped Quotes within CSV Lines', () => {
  it('correctly parses RFC 4180 escaped quotes inside cell tokens', () => {
    const line = 'TX01;"CONG TY ""AN PHAT"" TNHH";145000000';
    const cells = splitCsvLine(line, ';');

    expect(cells.length).toBe(3);
    expect(cells[0]).toBe('TX01');
    expect(cells[1]).toBe('CONG TY "AN PHAT" TNHH');
    expect(cells[2]).toBe('145000000');
  });

  it('preserves inner quotes at cell end and cell start boundaries without truncation', () => {
    const lineEnd = 'TX02;"Ghi chu: ""Chuyen khoan""";5000000';
    const cellsEnd = splitCsvLine(lineEnd, ';');
    expect(cellsEnd.length).toBe(3);
    expect(cellsEnd[1]).toBe('Ghi chu: "Chuyen khoan"');

    const lineStart = 'TX03;"""Napas"" chuyen tien";5000000';
    const cellsStart = splitCsvLine(lineStart, ';');
    expect(cellsStart.length).toBe(3);
    expect(cellsStart[1]).toBe('"Napas" chuyen tien');
  });

  it('handles multiple escaped quotes and delimiters inside quoted fields', () => {
    const lineComplex = 'TX04;"Ha Noi; Da Nang; TP.HCM";"Cong ty ""X"" & ""Y""";999999';
    const cells = splitCsvLine(lineComplex, ';');
    expect(cells.length).toBe(4);
    expect(cells[1]).toBe('Ha Noi; Da Nang; TP.HCM');
    expect(cells[2]).toBe('Cong ty "X" & "Y"');
    expect(cells[3]).toBe('999999');
  });
});

describe('Challenger Re-check 6: Subset-Sum BnB Duplicate Pruning Benchmark', () => {
  it('solves worst-case 60 identical candidates at k=8 in < 5ms (target impossible)', () => {
    const candidates = Array.from({ length: 60 }, (_, i) => ({
      index: i,
      amount: 100_000_000,
    }));

    // Target 700,000,001 is impossible for 100M items; depth 8
    // Without duplicate pruning, this explores ~2.5 billion branches.
    // With Branch 2 duplicate skipping, it finishes in < 1ms.
    const iterations = 50;
    const t0 = performance.now();
    for (let i = 0; i < iterations; i++) {
      const solution = solveExactSubsetSumBnb(candidates, 700_000_001, 8);
      expect(solution).toBeNull();
    }
    const totalElapsed = performance.now() - t0;
    const avgMs = totalElapsed / iterations;

    console.log(`[STRESS BENCHMARK] 60 identical items, impossible target, k=8: avg ${avgMs.toFixed(4)}ms per call (total ${totalElapsed.toFixed(2)}ms for ${iterations} calls)`);

    // Strict mandate: runtime MUST be < 5ms
    expect(avgMs).toBeLessThan(5.0);
  });

  it('solves achievable subset of 8 items out of 60 identical items in < 5ms', () => {
    const candidates = Array.from({ length: 60 }, (_, i) => ({
      index: i,
      amount: 100_000_000,
    }));

    const t0 = performance.now();
    const solution = solveExactSubsetSumBnb(candidates, 800_000_000, 8);
    const elapsed = performance.now() - t0;

    expect(solution).not.toBeNull();
    expect(solution!.length).toBe(8);
    const sum = solution!.reduce((acc, idx) => acc + candidates[idx].amount, 0);
    expect(sum).toBe(800_000_000);
    expect(elapsed).toBeLessThan(5.0);
  });

  it('handles 80 candidates partitioned into 8 groups of 10 identical items in < 5ms', () => {
    const candidates: { index: number; amount: number }[] = [];
    let idx = 0;
    for (let group = 1; group <= 8; group++) {
      for (let count = 0; count < 10; count++) {
        candidates.push({ index: idx++, amount: group * 10_000_000 });
      }
    }
    expect(candidates.length).toBe(80);

    // Target ending in 1 VND is impossible
    const t0 = performance.now();
    const solution = solveExactSubsetSumBnb(candidates, 250_000_001, 8);
    const elapsed = performance.now() - t0;

    expect(solution).toBeNull();
    expect(elapsed).toBeLessThan(5.0);
  });
});

describe('Challenger Re-check 7: Running Balance Continuity Discrepancy Detection', () => {
  it('strictly detects +1 VND tampering in running balance continuity', () => {
    const txs: Partial<RawStatementRow>[] = [
      { id: '1', txCode: 'TX01', credit: 50_000_000, debit: 0, balanceAfter: 150_000_000 },
      { id: '2', txCode: 'TX02', credit: 0, debit: 20_000_000, balanceAfter: 130_000_001 }, // +1 VND tamper
      { id: '3', txCode: 'TX03', credit: 10_000_000, debit: 0, balanceAfter: 140_000_000 },
    ];

    const report = verifyRunningBalanceContinuity(txs as RawStatementRow[], 100_000_000);
    expect(report.isContinuous).toBe(false);
    expect(report.brokenRowIndex).toBe(1);
    expect(report.expectedBalance).toBe(130_000_000);
    expect(report.actualBalance).toBe(130_000_001);
    expect(report.errorReason).toContain('1 VND');
  });

  it('strictly detects -1 VND tampering in running balance continuity', () => {
    const txs: Partial<RawStatementRow>[] = [
      { id: '1', txCode: 'TX01', credit: 50_000_000, debit: 0, balanceAfter: 150_000_000 },
      { id: '2', txCode: 'TX02', credit: 0, debit: 20_000_000, balanceAfter: 129_999_999 }, // -1 VND tamper
    ];

    const report = verifyRunningBalanceContinuity(txs as RawStatementRow[], 100_000_000);
    expect(report.isContinuous).toBe(false);
    expect(report.brokenRowIndex).toBe(1);
    expect(report.expectedBalance).toBe(130_000_000);
    expect(report.actualBalance).toBe(129_999_999);
    expect(report.errorReason).toContain('-1 VND');
  });

  it('detects tampering at boundary positions: row 0 and final row in large series', () => {
    const count = 50;
    const txs: Partial<RawStatementRow>[] = [];
    let curBal = 100_000_000;

    for (let i = 0; i < count; i++) {
      curBal += 1_000_000;
      txs.push({
        id: `tx-${i}`,
        txCode: `TX-${i}`,
        credit: 1_000_000,
        debit: 0,
        balanceAfter: curBal,
      });
    }

    // Clean pass
    const cleanReport = verifyRunningBalanceContinuity(txs as RawStatementRow[], 100_000_000);
    expect(cleanReport.isContinuous).toBe(true);

    // Tamper at row 0 by +1 VND
    const tamperedFirst = [...txs.map((t) => ({ ...t }))];
    tamperedFirst[0].balanceAfter = (tamperedFirst[0].balanceAfter || 0) + 1;
    const reportFirst = verifyRunningBalanceContinuity(tamperedFirst as RawStatementRow[], 100_000_000);
    expect(reportFirst.isContinuous).toBe(false);
    expect(reportFirst.brokenRowIndex).toBe(0);

    // Tamper at last row (index 49) by -1 VND
    const tamperedLast = [...txs.map((t) => ({ ...t }))];
    tamperedLast[49].balanceAfter = (tamperedLast[49].balanceAfter || 0) - 1;
    const reportLast = verifyRunningBalanceContinuity(tamperedLast as RawStatementRow[], 100_000_000);
    expect(reportLast.isContinuous).toBe(false);
    expect(reportLast.brokenRowIndex).toBe(49);
  });

  it('macro balance invariant check detects +/- 1 VND discrepancy with exact delta', () => {
    const opening = 500_000_000;
    const credit = 150_000_000;
    const debit = 80_000_000;
    const expectedClosing = 570_000_000;

    // Tamper +1 VND
    const reportPlus1 = verifyBalanceInvariants(opening, expectedClosing + 1, credit, debit);
    expect(reportPlus1.isValid).toBe(false);
    expect(reportPlus1.discrepancy).toBe(1);

    // Tamper -1 VND
    const reportMinus1 = verifyBalanceInvariants(opening, expectedClosing - 1, credit, debit);
    expect(reportMinus1.isValid).toBe(false);
    expect(reportMinus1.discrepancy).toBe(-1);
  });
});
