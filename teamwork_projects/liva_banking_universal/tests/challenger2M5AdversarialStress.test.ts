/**
 * Milestone 5 — Challenger 2 Empirical Adversarial Stress Suite
 * Focus: Ingestion Fuzzing, NLP Adversarial Strings, AML Boundary Conditions & Form STR Schema
 *
 * Mandated Verification Objectives:
 * 1. Ingestion Fuzzing:
 *    - Malformed tables, ragged rows, uneven columns, empty/whitespace cells.
 *    - Mixed delimiters (semicolon, comma, tab, pipe, escaped quotes, whitespace padding).
 *    - UTF-8 BOM (\uFEFF, \xEF\xBB\xBF) and UTF-16 BOM with various delimiters.
 *    - Multi-line narrations (narration continued on next row with 0 amounts).
 *    - Parentheses with trailing currencies: (35.000.000) VND, (35,000,000.00) VNĐ, etc.
 *    - European dot / English comma chaos: 15.000.000,00 vs 15,000,000.00 vs 15000.50 vs 15000,50.
 *    - Reference datasets in data/demo_ready/ and verify >= 99.8% match rate.
 * 2. NLP Adversarial Strings:
 *    - Mixed uppercase/lowercase slang (cK tIeN hAng, tToaN hDon, uNg LuOnG dOt 1, etc.).
 *    - Punctuation-dense and typo-ridden narrations (ckeck, chuyen khoang, thabg toan, etc.).
 *    - Special characters in invoice numbers (HD#99/2026-VAT, INV@2026_01!, HD::88, etc.).
 *    - Complex bank wire fees (pure fees, embedded fees, fee direction and delta checks).
 * 3. AML Boundary Stress:
 *    - High-Value: 399,999,999 vs 400,000,000 vs 400,000,001 VND; 999,999,999 vs 1,000,000,000 VND.
 *    - Structuring / Smurfing: 3 txs totaling 399M vs 400M vs 401M within 24h; count < 3; txs >= 400M.
 *    - Night Velocity: 22:59:59 vs 23:00:00 vs 04:59:59 vs 05:00:00 vs 05:00:01; 49,999,999 vs 50,000,000 VND.
 *    - Rapid Pass-Through: Inflow < 100M; 89.9% drain vs 90.0% drain vs 100.0% drain.
 * 4. Statutory Form STR Schema:
 *    - Phụ lục II Thông tư 09/2023/TT-NHNN schema conformance.
 *    - Strict absence of undefined, NaN, or null in structured fields and formatted documents.
 */

import { describe, it, expect } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { performance } from 'node:perf_hooks';

import {
  parseVietnameseAmount,
  parseTableRows,
  parseCsvOrTsv,
  parsePastedTable,
  parseExcelFile,
  parseLedgerCsv,
} from '../src/engine/ingestion/universalParser';
import {
  sniffDelimiter,
} from '../src/engine/ingestion/delimiterSniffer';
import {
  normalizeVietnameseIntent,
  matchRemarksToLedger,
} from '../src/engine/intelligence/vietnameseNlp';
import {
  disentangleWireFee,
  checkWireFeeDiscrepancy,
  KNOWN_VIETNAMESE_WIRE_FEES,
} from '../src/engine/intelligence/feeExtractor';
import {
  detectAmlHighValue,
  detectAmlStructuring,
  detectAmlNightVelocity,
  detectAmlRapidPassThrough,
  runFullAmlSurveillance,
  inspectSystemPrompt,
} from '../src/engine/intelligence/amlSurveillance';
import {
  generateFormStr,
  formatOfficialStrDocument,
} from '../src/engine/intelligence/strGenerator';
import { runReconciliationEngine } from '../src/engine/reconciliation/reconciliationEngine';

import type { RawStatementRow, LedgerEntry } from '../src/types/banking';
import type { AmlAlert } from '../src/types/aml';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

describe('Milestone 5 Challenger 2: Empirical Adversarial Stress Suite', () => {
  const demoDir = path.resolve(__dirname, '../../../data/demo_ready');

  // ==========================================================================
  // SECTION 1: INGESTION FUZZING & MALFORMED TABLE STRESS
  // ==========================================================================
  describe('1. Ingestion Fuzzing & Malformed Table Stress', () => {

    describe('1.1 Malformed Tables, Ragged Rows & Uneven Columns', () => {
      it('parses ragged tables with highly uneven column lengths (2 to 11 cols per row)', () => {
        const raggedRows: string[][] = [
          ['NGAN HANG TMCP NGOAI THUONG VIET NAM'], // 1 col
          ['Số tài khoản: 0071001234567', 'Tên: CONG TY TNHH LIVA', 'Loại tiền: VND'], // 3 cols
          ['Số dư đầu kỳ: 50.000.000 VND'], // 1 col
          ['Ngày GD', 'Mã GD', 'Số tiền ghi nợ', 'Số tiền ghi có', 'Số dư', 'Nội dung giao dịch', 'Đối tác', 'TK đối tác', 'Ghi chú extra', 'Ký hiệu'], // 10 cols
          ['01/08/2026', 'TX01', '', '10.000.000', '60.000.000', 'Thanh toan HD-01'], // 6 cols
          ['02/08/2026', 'TX02', '5.000.000', '', '55.000.000', 'Chi phi van phong', 'CTY DICH VU', '123456', 'EXTRA', 'FLAG', 'EXTRA2'], // 11 cols
          ['03/08/2026', 'TX03', '', '25.000.000'], // 4 cols
          ['', '', '', '', ''], // empty row
          ['04/08/2026', 'TX04', '15.000.000', '', '65.000.000', 'Thanh toan tien thue'], // 6 cols
        ];

        const parsed = parseTableRows(raggedRows, 'ragged_statement.csv', 'VCB');
        expect(parsed.transactions.length).toBe(4);
        expect(parsed.totalCredit).toBe(35000000); // 10M + 25M
        expect(parsed.totalDebit).toBe(20000000);  // 5M + 15M
        expect(parsed.transactions[0].credit).toBe(10000000);
        expect(parsed.transactions[1].debit).toBe(5000000);
        expect(parsed.transactions[2].credit).toBe(25000000);
        expect(parsed.transactions[3].debit).toBe(15000000);
      });

      it('safely handles dirty tabular rows with empty strings, whitespace, dashes, and N/A markers', () => {
        const dirtyRows: string[][] = [
          ['Ngày giao dịch', 'Mã GD', 'Số tiền ghi nợ', 'Số tiền ghi có', 'Số dư', 'Nội dung'],
          ['01/08/2026', 'TX-01', '', '10,000,000', '10,000,000', 'Giao dich 1'],
          ['02/08/2026', 'TX-02', '   ', '-', '', 'Chi phi them phu luc'],
          ['03/08/2026', 'TX-03', '5,000,000', 'N/A', '5,000,000', 'Rut tien mat'],
        ];

        expect(() => parseTableRows(dirtyRows, 'dirty.csv')).not.toThrow();
        const parsed = parseTableRows(dirtyRows, 'dirty.csv');
        expect(parsed.transactions.length).toBe(2); // TX-01 and TX-03 (TX-02 has 0 amount, stitched to TX-01)
        expect(parsed.transactions[0].narration).toContain('Chi phi them phu luc');
      });
    });

    describe('1.2 Delimiter Sniffing & Mixed Delimiters', () => {
      it('handles pipe (|), tab (\\t), semicolon (;), and comma (,) seamlessly', () => {
        const pipeCsv = [
          'Ngày|Mã GD|Số tiền ghi có|Số dư|Nội dung',
          '01/08/2026|TX_PIPE_1|10000000|10000000|Thu tien ban hang pipe',
          '02/08/2026|TX_PIPE_2|20000000|30000000|Thu tien dot 2 pipe',
        ].join('\n');

        const snifferPipe = sniffDelimiter(pipeCsv);
        expect(snifferPipe.delimiter).toBe('|');
        const parsedPipe = parseCsvOrTsv(pipeCsv, 'test.pipe');
        expect(parsedPipe.transactions.length).toBe(2);
        expect(parsedPipe.transactions[0].credit).toBe(10000000);

        const tabCsv = [
          'Ngày\tMã GD\tSố tiền ghi có\tSố dư\tNội dung',
          '01/08/2026\tTX_TAB_1\t15000000\t15000000\tThu tien tab',
          '02/08/2026\tTX_TAB_2\t25000000\t40000000\tThu tien tab 2',
        ].join('\n');

        const snifferTab = sniffDelimiter(tabCsv);
        expect(snifferTab.delimiter).toBe('\t');
        const parsedTab = parseCsvOrTsv(tabCsv, 'test.tsv');
        expect(parsedTab.transactions.length).toBe(2);
        expect(parsedTab.transactions[1].credit).toBe(25000000);
      });

      it('sniffs semicolon delimiter even when cells contain multiple commas in text and amounts', () => {
        const complexCsv = [
          'Ngày;Mã GD;Số tiền;Số dư;Diễn giải',
          '10/08/2026;TX001;15,500,000;15,500,000;Cty TNHH Nam, Long, và An',
          '11/08/2026;TX002;24,500,000;40,000,000;Thanh toán đợt 1, 2, và 3',
        ].join('\n');

        const sniffer = sniffDelimiter(complexCsv);
        expect(sniffer.delimiter).toBe(';');
        const parsed = parseCsvOrTsv(complexCsv);
        expect(parsed.transactions.length).toBe(2);
        expect(parsed.transactions[0].amount).toBe(15500000);
        expect(parsed.transactions[1].amount).toBe(24500000);
      });
    });

    describe('1.3 UTF-8 BOM, Raw Bytes & UTF-16 Encoding', () => {
      it('strips UTF-8 BOM and parses Vietnamese diacritics perfectly', () => {
        const bomCsv = '\uFEFFNgày;Mã GD;Số tiền ghi có;Số dư;Nội dung chi tiết\n15/08/2026;BOM01;50000000;50000000;Thanh toán hợp đồng số 88/2026';
        const parsed = parseCsvOrTsv(bomCsv, 'bom.csv');
        expect(parsed.transactions.length).toBe(1);
        expect(parsed.transactions[0].credit).toBe(50000000);
        expect(parsed.transactions[0].narration).toContain('Thanh toán hợp đồng số 88/2026');
      });

      it('strips UTF-16 BOM without corrupting header tokens', () => {
        const utf16Csv = '\uFFFENgày\tMã GD\tSố tiền ghi có\tSố dư\tNội dung\n16/08/2026\tU16_01\t75000000\t75000000\tNội dung UTF16';
        const parsed = parseCsvOrTsv(utf16Csv, 'utf16.tsv');
        expect(parsed.transactions.length).toBe(1);
        expect(parsed.transactions[0].credit).toBe(75000000);
      });
    });

    describe('1.4 Multi-Line Narration Stitching', () => {
      it('stitches continuation narrations across multiple rows with zero monetary amounts', () => {
        const multiLineCsv = [
          'Ngày giao dịch;Mã GD;Số tiền ghi nợ;Số tiền ghi có;Số dư;Diễn giải',
          '01/08/2026;TX_MULTI_1;;100000000;100000000;Thanh toan tien hang thang 8 theo',
          ';;;;;hop dong so 88/2026-LIVA ky giua',
          ';;;;;Cong ty A va Cong ty B kem hoa don VAT',
          '02/08/2026;TX_MULTI_2;25000000;;75000000;Chi phi cong tac phi Ha Noi',
        ].join('\n');

        const parsed = parseCsvOrTsv(multiLineCsv, 'multiline.csv');
        expect(parsed.transactions.length).toBe(2);

        // Transaction 1 narration must contain all 3 lines stitched together
        const t1Narration = parsed.transactions[0].narration;
        expect(t1Narration).toContain('Thanh toan tien hang thang 8 theo');
        expect(t1Narration).toContain('hop dong so 88/2026-LIVA ky giua');
        expect(t1Narration).toContain('Cong ty A va Cong ty B kem hoa don VAT');
        expect(parsed.transactions[0].credit).toBe(100000000);

        // Transaction 2 remains unaffected
        expect(parsed.transactions[1].debit).toBe(25000000);
        expect(parsed.transactions[1].narration).toBe('Chi phi cong tac phi Ha Noi');
      });
    });

    describe('1.5 Currency & Parentheses Chaos (parseVietnameseAmount)', () => {
      it('accurately parses negative parenthesized amounts with various trailing and embedded currencies', () => {
        expect(parseVietnameseAmount('(35.000.000) VND')).toBe(-35000000);
        expect(parseVietnameseAmount('(35.000.000) VNĐ')).toBe(-35000000);
        expect(parseVietnameseAmount('(35,000,000.00) VNĐ')).toBe(-35000000);
        expect(parseVietnameseAmount('(15.000.000 VND)')).toBe(-15000000);
        expect(parseVietnameseAmount('( 50.000.000 ) đ')).toBe(-50000000);
        expect(parseVietnameseAmount('(25,000,000)VND')).toBe(-25000000);
        expect(parseVietnameseAmount('-(35.000.000) VND')).toBe(-35000000);
        expect(parseVietnameseAmount('(-35.000.000) VND')).toBe(-35000000);
        expect(parseVietnameseAmount('(500.000.000.000) VND')).toBe(-500000000000);
      });

      it('handles European dot / English comma chaos with 100% mathematical integrity', () => {
        // Vietnamese/European dot thousand, comma decimal
        expect(parseVietnameseAmount('15.000.000,00')).toBe(15000000);
        expect(parseVietnameseAmount('15.000.000,50')).toBe(15000000);
        expect(parseVietnameseAmount('1.234.567')).toBe(1234567);

        // US comma thousand, dot decimal
        expect(parseVietnameseAmount('15,000,000.00')).toBe(15000000);
        expect(parseVietnameseAmount('15,000,000.50')).toBe(15000000);
        expect(parseVietnameseAmount('1,234,567')).toBe(1234567);

        // Ambiguous single separator formats
        expect(parseVietnameseAmount('15000.50')).toBe(15000);
        expect(parseVietnameseAmount('15000,50')).toBe(15000);
        expect(parseVietnameseAmount('15000.00')).toBe(15000);
        expect(parseVietnameseAmount('15000,00')).toBe(15000);
      });

      it('rejects dates, timestamps, file paths, and noise without producing corrupt numbers', () => {
        expect(parseVietnameseAmount('15/08/2026')).toBe(0);
        expect(parseVietnameseAmount('2026-08-15')).toBe(0);
        expect(parseVietnameseAmount('15-08-2026')).toBe(0);
        expect(parseVietnameseAmount('C:\\banking\\data')).toBe(0);
        expect(parseVietnameseAmount('14:30:00')).toBe(0);
        expect(parseVietnameseAmount('Chua xac dinh')).toBe(0);
        expect(parseVietnameseAmount(null)).toBe(0);
        expect(parseVietnameseAmount(undefined)).toBe(0);
        expect(parseVietnameseAmount('')).toBe(0);
      });
    });

    describe('1.6 Reference Datasets & >= 99.8% Match Rate Benchmark', () => {
      it('verifies all reference files in data/demo_ready/ parse and satisfy balance invariants', () => {
        const vcbXlsxPath = path.join(demoDir, '01_VCB_SaoKe_Thang8_Chuan.xlsx');
        const tcbCsvPath = path.join(demoDir, '02_TCB_SaoKe_Thang8_Chuan.csv');
        const bidvCsvPath = path.join(demoDir, '03_BIDV_SaoKe_NgoaiGio_BatThuong.csv');
        const ledgerPath = path.join(demoDir, '04_SoNhatKy_Chung_Thang8.csv');
        const pasteTxtPath = path.join(demoDir, '05_DuLieu_Dan_GoogleSheets.txt');

        // 1. VCB Excel
        const vcbBuffer = fs.readFileSync(vcbXlsxPath);
        const vcbParsed = parseExcelFile(vcbBuffer, '01_VCB_SaoKe_Thang8_Chuan.xlsx');
        expect(vcbParsed.bankCode).toBe('VCB');
        expect(vcbParsed.balanceInvariantPassed).toBe(true);
        expect(vcbParsed.transactions.length).toBe(8);

        // 2. TCB CSV
        const tcbContent = fs.readFileSync(tcbCsvPath, 'utf-8');
        const tcbParsed = parseCsvOrTsv(tcbContent, '02_TCB_SaoKe_Thang8_Chuan.csv');
        expect(tcbParsed.bankCode).toBe('TCB');
        expect(tcbParsed.balanceInvariantPassed).toBe(true);
        expect(tcbParsed.transactions.length).toBe(5);

        // 3. BIDV CSV
        const bidvContent = fs.readFileSync(bidvCsvPath, 'utf-8');
        const bidvParsed = parseCsvOrTsv(bidvContent, '03_BIDV_SaoKe_NgoaiGio_BatThuong.csv');
        expect(bidvParsed.bankCode).toBe('BIDV');
        expect(bidvParsed.balanceInvariantPassed).toBe(true);
        expect(bidvParsed.transactions.length).toBe(4);

        // 4. Ledger CSV
        const ledgerContent = fs.readFileSync(ledgerPath, 'utf-8');
        const ledgerEntries = parseLedgerCsv(ledgerContent);
        expect(ledgerEntries.length).toBe(7);

        // 5. Pasted Text
        const pasteContent = fs.readFileSync(pasteTxtPath, 'utf-8');
        const pasteParsed = parsePastedTable(pasteContent, 'VCB');
        expect(pasteParsed.transactions.length).toBe(8);
      });

      it('empirically demonstrates >= 99.8% match rate on 2,500-transaction stress reconciliation', () => {
        // Construct 2,500 transactions:
        // 2,495 matched (combination of exact 1:1, fuzzy with fee, and composite split)
        // 5 unallocated odd items (0.2% fail-closed quarantine) -> 99.8% match rate!
        const ledger: LedgerEntry[] = [];
        const bankTxs: RawStatementRow[] = [];
        const baseEpoch = 1785540000;

        for (let i = 1; i <= 2495; i++) {
          const docNo = `HD-2026-${(2000 + i).toString()}`;
          const amount = 5000000 + (i * 25000);
          const isCredit = i % 2 === 0;
          const txType: 'DEBIT' | 'CREDIT' = isCredit ? 'CREDIT' : 'DEBIT';
          const partnerName = `DOI TAC SO ${i}`;

          ledger.push({
            id: `ledg-stress-${i}`,
            docNo,
            entryDate: '2026-08-15',
            entryTimestamp: baseEpoch + i,
            amount,
            entryType: txType,
            partnerName,
            description: `Thanh toan hop dong ${docNo}`,
            status: 'UNMATCHED',
          });

          // 80% exact, 20% fuzzy with wire fee deduction
          const isFuzzy = i % 5 === 0;
          const fee = isFuzzy ? 2200 : 0;
          const bankAmount = amount - fee;
          const narration = isFuzzy
            ? `Napas ck tien ${partnerName} phi ${fee}d`
            : `TT HD ${docNo} tu ${partnerName}`;

          bankTxs.push({
            id: `tx-stress-${i}`,
            date: '2026-08-15',
            time: '10:30:00',
            txDate: baseEpoch + i,
            valueDate: baseEpoch + i,
            txCode: `TXS${i.toString().padStart(7, '0')}`,
            docRef: isFuzzy ? undefined : docNo,
            debit: !isCredit ? bankAmount : 0,
            credit: isCredit ? bankAmount : 0,
            netAmount: isCredit ? bankAmount : -bankAmount,
            amount: bankAmount,
            txType,
            balance: 500000000,
            narration,
            counterparty: partnerName,
            bankCode: 'VCB',
          });
        }

        // Add 5 odd items (0.2% quarantine)
        for (let j = 1; j <= 5; j++) {
          bankTxs.push({
            id: `tx-odd-${j}`,
            date: '2026-08-16',
            txDate: baseEpoch + 3000 + j,
            txCode: `ODD${j}`,
            debit: 0,
            credit: 888888 * j,
            netAmount: 888888 * j,
            amount: 888888 * j,
            txType: 'CREDIT',
            balance: 500000000,
            narration: `Khoan tien chua xac dinh khong ro nguon goc ${j}`,
            bankCode: 'TCB',
          });
        }

        const tStart = performance.now();
        const summary = runReconciliationEngine(bankTxs, ledger);
        const duration = performance.now() - tStart;

        expect(summary.totalBankTransactions).toBe(2500);
        expect(summary.matchedCount).toBe(2495);
        expect(summary.matchRate).toBeGreaterThanOrEqual(99.8);
        expect(summary.hitlQuarantineCount).toBe(5);
        expect(duration).toBeLessThan(150); // High throughput: < 150ms for 2,500 transactions
      });
    });
  });

  // ==========================================================================
  // SECTION 2: NLP ADVERSARIAL STRINGS & ENTITY EXTRACTION STRESS
  // ==========================================================================
  describe('2. NLP Adversarial Strings & Entity Extraction Stress', () => {

    describe('2.1 Mixed Upper/Lowercase Vietnamese Slang & Abbreviations', () => {
      it('normalizes chaotic mixed-case banking slang into formal banking terms', () => {
        const testCases: Array<{ memo: string; expectedToken: string; expectedIntent: string }> = [
          { memo: 'cK tIeN hAng cho cty may bom', expectedToken: 'chuyển khoản', expectedIntent: 'PAYMENT' },
          { memo: 'tToaN hDon may tinh tien', expectedToken: 'thanh toán', expectedIntent: 'PAYMENT' },
          { memo: 'uNg LuOnG dOt 1 thang 8', expectedToken: 'tạm ứng', expectedIntent: 'ADVANCE' },
          { memo: 'vIeTqR tT tien thiet bi', expectedToken: 'vietqr', expectedIntent: 'PAYMENT' },
          { memo: 'phi duy tri tai khoan TCB', expectedToken: 'phí dịch vụ', expectedIntent: 'BANK_FEE' },
          { memo: 'rut tien mat cay atm', expectedToken: 'rut tien', expectedIntent: 'WITHDRAWAL' },
          { memo: 'chuyen tien noi bo cty con', expectedToken: 'noi bo', expectedIntent: 'INTERNAL_TRANSFER' },
        ];

        for (const tc of testCases) {
          const result = normalizeVietnameseIntent(tc.memo);
          expect(result.intent).toBe(tc.expectedIntent);
          expect(result.confidence).toBeGreaterThanOrEqual(0.85);
          expect(result.normalized.toLowerCase()).toContain(tc.expectedToken.toLowerCase());
        }
      });

      it('observes tokenizer behavior on compound abbreviations with slashes or punctuation', () => {
        // When abbreviations contain slashes like "cH/kHoAn", tokenizer splits on non-alphanumeric chars
        const result = normalizeVietnameseIntent('cH/kHoAn nPs 247 chuyen tien');
        expect(result.intent).toBe('PAYMENT'); // Classified as PAYMENT via "chuyen tien" or "nps"
        expect(result.confidence).toBeGreaterThanOrEqual(0.85);
      });
    });

    describe('2.2 Punctuation-Dense & Typo-Ridden Narrations', () => {
      it('extracts invoice entities even with extreme punctuation and punctuation-dense delimiters', () => {
        // Dense punctuation: CK...TIEN,,MAY--BOM//HD::88
        const r1 = normalizeVietnameseIntent('CK...TIEN,,MAY--BOM//HD::88');
        expect(r1.invoiceNumbers).toContain('88');
        expect(r1.intent).toBe('PAYMENT');

        // Hyphenated invoice format: HD-2026-88
        const r2 = normalizeVietnameseIntent('Thanh toan tien vat tu theo HD-2026-88');
        expect(r2.invoiceNumbers).toContain('2026-88');

        // Slash invoice with year: HD/99-VAT
        const r3 = normalizeVietnameseIntent('Chuyen khoan HD/99-VAT ky ngay 10/08');
        expect(r3.invoiceNumbers.some((inv) => inv.includes('99'))).toBe(true);

        // Special characters in invoice numbers: HD#99/2026-VAT
        const rSpecial = normalizeVietnameseIntent('Thanh toan HD#99/2026-VAT');
        expect(rSpecial.invoiceNumbers.some((inv) => inv.includes('99'))).toBe(true);

        // Tax code in messy narration
        const r4 = normalizeVietnameseIntent('Thanh toan hoa don mst 0102345678 cty Liva');
        expect(r4.taxCodes).toContain('0102345678');

        // Napas trace code
        const r5 = normalizeVietnameseIntent('Napas NPS20260815123456 thanh toan');
        expect(r5.traceCodes).toContain('NPS20260815123456');
      });

      it('links typo-ridden and abbreviated remarks to ledger invoices via matchRemarksToLedger', () => {
        const candidateLedgers: LedgerEntry[] = [
          {
            id: 'l1',
            docNo: 'HD-2026-88',
            entryDate: '2026-08-10',
            entryTimestamp: 1785540000,
            amount: 145000000,
            entryType: 'CREDIT',
            partnerName: 'CONG TY MAY BOM HOA BINH',
            description: 'Ban may bom cong nghiep',
            status: 'UNMATCHED',
          },
          {
            id: 'l2',
            docNo: 'INV-101',
            entryDate: '2026-08-11',
            entryTimestamp: 1785540060,
            amount: 85000000,
            entryType: 'CREDIT',
            partnerName: 'TAP DOAN MASAN',
            description: 'Cung cap bao bi thuc pham',
            status: 'UNMATCHED',
          },
        ];

        // Direct invoice number match with typo/slang: "ck tien may bom hd 88 hoa binh"
        const match1 = matchRemarksToLedger('ck tien may bom hd 88 hoa binh', candidateLedgers);
        expect(match1.matchedLedger?.id).toBe('l1');
        expect(match1.confidence).toBeGreaterThanOrEqual(0.85);

        // Invoice match for second ledger: "masan tt inv 101 tien hang"
        const match2 = matchRemarksToLedger('masan tt inv 101 tien hang', candidateLedgers);
        expect(match2.matchedLedger?.id).toBe('l2');
        expect(match2.confidence).toBeGreaterThanOrEqual(0.85);
      });
    });

    describe('2.3 Complex Bank Wire Fee Disentanglement', () => {
      it('identifies pure bank account management fees and allocates to TK 6425', () => {
        const pure1 = disentangleWireFee(22000, 'Phi duy tri tai khoan thang 8/2026');
        expect(pure1.isPureFee).toBe(true);
        expect(pure1.feeAmount).toBe(22000);
        expect(pure1.principal).toBe(0);
        expect(pure1.accountCode).toBe('TK 6425');
        expect(pure1.sumCheckPassed).toBe(true);

        const pure2 = disentangleWireFee(11000, 'Phí quản lý tài khoản định kỳ');
        expect(pure2.isPureFee).toBe(true);
        expect(pure2.feeAmount).toBe(11000);
        expect(pure2.accountCode).toBe('TK 6425');
      });

      it('disentangles embedded wire transfer fees from gross transaction amounts', () => {
        // 50,002,200 VND with embedded 2,200 fee
        const grossAmount = 50002200;
        const res = disentangleWireFee(grossAmount, 'ck tien may bom hd 88 phi 2200 vat');
        expect(res.isPureFee).toBe(false);
        expect(res.feeAmount).toBe(2200);
        expect(res.principal).toBe(50000000);
        expect(res.accountCode).toBe('TK 6425');
        expect(res.sumCheckPassed).toBe(true);
        expect(res.principal + res.feeAmount).toBe(grossAmount);
      });

      it('verifies all 9 known Vietnamese interbank wire fee amounts via checkWireFeeDiscrepancy', () => {
        const invoiceAmount = 100000000;

        for (const fee of KNOWN_VIETNAMESE_WIRE_FEES) {
          // Case A: Bank deducted fee from recipient (bankAmount = invoice - fee)
          const deducted = checkWireFeeDiscrepancy(invoiceAmount - fee, invoiceAmount);
          expect(deducted.isFeeDiscrepancy).toBe(true);
          expect(deducted.feeAmount).toBe(fee);
          expect(deducted.direction).toBe('BANK_DEDUCTED_FEE');

          // Case B: Payer covered fee on top (bankAmount = invoice + fee)
          const covered = checkWireFeeDiscrepancy(invoiceAmount + fee, invoiceAmount);
          expect(covered.isFeeDiscrepancy).toBe(true);
          expect(covered.feeAmount).toBe(fee);
          expect(covered.direction).toBe('PAYER_COVERED_FEE');
        }

        // Non-standard discrepancy (e.g. 5,000 VND or 15,000 VND) is NOT a known wire fee
        const nonFee = checkWireFeeDiscrepancy(invoiceAmount - 5000, invoiceAmount);
        expect(nonFee.isFeeDiscrepancy).toBe(false);
        expect(nonFee.direction).toBe('NONE');
      });
    });
  });

  // ==========================================================================
  // SECTION 3: AML BOUNDARY STRESS TESTING
  // ==========================================================================
  describe('3. AML Boundary Conditions Stress Testing', () => {

    describe('3.1 High-Value Transaction Threshold (Decision 11/2023/QĐ-TTg)', () => {
      it('stress-tests 399,999,999 vs 400,000,000 vs 400,000,001 VND', () => {
        const txBelow = [{ id: 'tx-below', amount: 399_999_999 }];
        const txExact = [{ id: 'tx-exact', amount: 400_000_000 }];
        const txAbove = [{ id: 'tx-above', amount: 400_000_001 }];

        // 399,999,999 must NOT trigger High-Value alert
        expect(detectAmlHighValue(txBelow).length).toBe(0);

        // 400,000,000 MUST trigger High-Value alert with HIGH severity
        const alertsExact = detectAmlHighValue(txExact);
        expect(alertsExact.length).toBe(1);
        expect(alertsExact[0].anomalyType).toBe('HIGH_VALUE');
        expect(alertsExact[0].severity).toBe('HIGH');
        expect(alertsExact[0].totalAmount).toBe(400_000_000);
        expect(alertsExact[0].statutoryRuleRef).toContain('11/2023/QĐ-TTg');

        // 400,000,001 MUST trigger High-Value alert
        const alertsAbove = detectAmlHighValue(txAbove);
        expect(alertsAbove.length).toBe(1);
        expect(alertsAbove[0].severity).toBe('HIGH');
      });

      it('escalates severity to CRITICAL at 1 Billion VND boundary', () => {
        const tx999M = [{ id: 'tx-999M', amount: 999_999_999 }];
        const tx1B = [{ id: 'tx-1B', amount: 1_000_000_000 }];

        const alert999M = detectAmlHighValue(tx999M);
        expect(alert999M[0].severity).toBe('HIGH');

        const alert1B = detectAmlHighValue(tx1B);
        expect(alert1B[0].severity).toBe('CRITICAL');
      });
    });

    describe('3.2 Structuring / Smurfing Threshold (Circular 09/2023/TT-NHNN)', () => {
      it('tests 3 transactions totaling 399,999,999 vs 400,000,000 vs 401,000,000 VND within 24h', () => {
        // 3 sub-400M transactions summing to 399,999,999 VND (< 400M)
        const txsBelow = [
          { id: 'smurf-b1', date: '2026-08-15', amount: 133_333_333 },
          { id: 'smurf-b2', date: '2026-08-15', amount: 133_333_333 },
          { id: 'smurf-b3', date: '2026-08-15', amount: 133_333_333 },
        ];
        expect(detectAmlStructuring(txsBelow).length).toBe(0);

        // 3 sub-400M transactions summing exactly to 400,000,000 VND
        const txsExact = [
          { id: 'smurf-e1', date: '2026-08-15', amount: 133_333_333 },
          { id: 'smurf-e2', date: '2026-08-15', amount: 133_333_333 },
          { id: 'smurf-e3', date: '2026-08-15', amount: 133_333_334 },
        ];
        const alertsExact = detectAmlStructuring(txsExact);
        expect(alertsExact.length).toBe(1);
        expect(alertsExact[0].anomalyType).toBe('STRUCTURING_SMURFING');
        expect(alertsExact[0].totalAmount).toBe(400_000_000);
        expect(alertsExact[0].involvedTransactionIds.length).toBe(3);

        // 3 sub-400M transactions summing to 401,000,000 VND
        const txsAbove = [
          { id: 'smurf-a1', date: '2026-08-15', amount: 134_000_000 },
          { id: 'smurf-a2', date: '2026-08-15', amount: 134_000_000 },
          { id: 'smurf-a3', date: '2026-08-15', amount: 133_000_000 },
        ];
        const alertsAbove = detectAmlStructuring(txsAbove);
        expect(alertsAbove.length).toBe(1);
        expect(alertsAbove[0].totalAmount).toBe(401_000_000);
      });

      it('verifies structuring requires count >= 3 (2 transactions totaling 401M do NOT trigger smurfing)', () => {
        const twoTxs = [
          { id: 'two-1', date: '2026-08-15', amount: 200_500_000 },
          { id: 'two-2', date: '2026-08-15', amount: 200_500_000 },
        ];
        // Only 2 transactions: does NOT satisfy structuring definition
        expect(detectAmlStructuring(twoTxs).length).toBe(0);
      });

      it('excludes single transactions >= 400M from structuring pool (they are high-value, not smurfs)', () => {
        const highValTxs = [
          { id: 'hv-1', date: '2026-08-15', amount: 400_000_000 },
          { id: 'hv-2', date: '2026-08-15', amount: 400_000_000 },
          { id: 'hv-3', date: '2026-08-15', amount: 400_000_000 },
        ];
        expect(detectAmlStructuring(highValTxs).length).toBe(0);
      });
    });

    describe('3.3 Night-Time Velocity Anomaly (23:00:00 - 05:00:00, >= 50M VND)', () => {
      it('evaluates exact second boundaries: 22:59:59 vs 23:00:00 vs 04:59:59 vs 05:00:00 vs 05:00:01', () => {
        const txAmount = 75_000_000; // >= 50M

        // 22:59:59 -> 1 second before night window -> PASS (NO ALERT)
        const t2259 = [{ id: 'tx-2259', time: '22:59:59', amount: txAmount }];
        expect(detectAmlNightVelocity(t2259).length).toBe(0);

        // 23:00:00 -> Exact start of night window -> TRIGGER ALERT
        const t2300 = [{ id: 'tx-2300', time: '23:00:00', amount: txAmount }];
        const alert2300 = detectAmlNightVelocity(t2300);
        expect(alert2300.length).toBe(1);
        expect(alert2300[0].anomalyType).toBe('NIGHT_VELOCITY');

        // 04:59:59 -> Last second inside night window -> TRIGGER ALERT
        const t0459 = [{ id: 'tx-0459', time: '04:59:59', amount: txAmount }];
        const alert0459 = detectAmlNightVelocity(t0459);
        expect(alert0459.length).toBe(1);

        // 05:00:00 -> Exact end of night window (morning start) -> PASS (NO ALERT)
        const t0500 = [{ id: 'tx-0500', time: '05:00:00', amount: txAmount }];
        expect(detectAmlNightVelocity(t0500).length).toBe(0);

        // 05:00:01 -> 1 second after night window -> PASS (NO ALERT)
        const t0501 = [{ id: 'tx-0501', time: '05:00:01', amount: txAmount }];
        expect(detectAmlNightVelocity(t0501).length).toBe(0);
      });

      it('evaluates amount boundary for night transactions: 49,999,999 vs 50,000,000 VND', () => {
        const txNightBelow = [{ id: 'tx-nb', time: '23:30:00', amount: 49_999_999 }];
        const txNightExact = [{ id: 'tx-ne', time: '23:30:00', amount: 50_000_000 }];

        expect(detectAmlNightVelocity(txNightBelow).length).toBe(0);
        const alertsExact = detectAmlNightVelocity(txNightExact);
        expect(alertsExact.length).toBe(1);
        expect(alertsExact[0].totalAmount).toBe(50_000_000);
      });
    });

    describe('3.4 Rapid Pass-Through / Churn Mule Drain Rate (89.9% vs 90.0%)', () => {
      it('evaluates inflow and drain thresholds: < 100M inflow; 89.9% drain vs 90.0% drain', () => {
        // Case A: Inflow is only 99,999,999 (< 100M minimum inflow)
        const txInflowBelow = [
          { id: 'in-99m', credit: 99_999_999, txType: 'CREDIT' },
          { id: 'out-99m', debit: 99_000_000, txType: 'DEBIT' }, // 99% drain but inflow < 100M
        ];
        expect(detectAmlRapidPassThrough(txInflowBelow).length).toBe(0);

        // Case B: Inflow 100,000,000, Outflow 89,900,000 (89.9% drain, strictly < 90.0%)
        const txDrainBelow = [
          { id: 'in-100m-b', credit: 100_000_000, txType: 'CREDIT' },
          { id: 'out-89m', debit: 89_900_000, txType: 'DEBIT' },
        ];
        expect(detectAmlRapidPassThrough(txDrainBelow).length).toBe(0);

        // Case C: Inflow 100,000,000, Outflow 90,000,000 (90.0% drain -> MUST TRIGGER)
        const txDrainExact = [
          { id: 'in-100m-e', credit: 100_000_000, txType: 'CREDIT' },
          { id: 'out-90m', debit: 90_000_000, txType: 'DEBIT' },
        ];
        const alertsExact = detectAmlRapidPassThrough(txDrainExact);
        expect(alertsExact.length).toBe(1);
        expect(alertsExact[0].anomalyType).toBe('RAPID_PASS_THROUGH');
        expect(alertsExact[0].severity).toBe('CRITICAL');

        // Case D: Inflow 100,000,000, Outflow 100,000,000 (100.0% drain -> MUST TRIGGER)
        const txDrain100 = [
          { id: 'in-100m-full', credit: 100_000_000, txType: 'CREDIT' },
          { id: 'out-100m-full', debit: 100_000_000, txType: 'DEBIT' },
        ];
        const alertsFull = detectAmlRapidPassThrough(txDrain100);
        expect(alertsFull.length).toBe(1);
      });
    });

    describe('3.5 Full Multi-Algorithm Surveillance & System Prompt Inspector', () => {
      it('executes full surveillance aggregating high-value, structuring, night, and watchlist alerts', () => {
        const mixedTransactions = [
          { id: 't1', date: '2026-08-15', time: '10:00:00', amount: 450_000_000, credit: 450_000_000, narration: 'Thanh toan' },
          { id: 't2', date: '2026-08-15', time: '23:30:00', amount: 65_000_000, credit: 65_000_000, narration: 'Chuyen tien' },
          { id: 't3', date: '2026-08-16', time: '14:00:00', amount: 20_000_000, credit: 20_000_000, narration: 'Nap tien san Binance P2P USDT' },
        ];

        const allAlerts = runFullAmlSurveillance(mixedTransactions);
        expect(allAlerts.length).toBe(3);
        const types = allAlerts.map((a) => a.anomalyType);
        expect(types).toContain('HIGH_VALUE');
        expect(types).toContain('NIGHT_VELOCITY');
        expect(types).toContain('WATCHLIST_HIT');
      });

      it('inspects active System Prompt text and executes evaluation sandbox', () => {
        const inspected = inspectSystemPrompt();
        expect(inspected.systemPrompt).toContain('BẠN LÀ TÁC TỬ PHÒNG CHỐNG RỬA TIỀN');
        expect(inspected.systemPrompt).toContain('Thông tư 09/2023/TT-NHNN');
        expect(inspected.systemPrompt).toContain('400.000.000');
        expect(inspected.parameters.highValueThreshold).toBe(400_000_000);

        // Sandbox execution
        const sandboxAlerts = inspected.evalSandbox([
          { id: 'sb-1', amount: 420_000_000 },
          { id: 'sb-2', amount: 150_000_000 },
        ]);
        expect(sandboxAlerts.length).toBe(1);
        expect(sandboxAlerts[0].totalAmount).toBe(420_000_000);
      });
    });
  });

  // ==========================================================================
  // SECTION 4: STATUTORY FORM STR PHỤ LỤC II SCHEMA & INTEGRITY
  // ==========================================================================
  describe('4. Statutory Form STR Phụ lục II Schema & Data Integrity', () => {
    it('verifies Form STR strictly adheres to Phụ lục II Thông tư 09/2023/TT-NHNN without undefined or NaN', () => {
      const mockAlert: AmlAlert = {
        alertId: 'aml-test-alert-01',
        anomalyType: 'STRUCTURING_SMURFING',
        severity: 'CRITICAL',
        involvedTransactionIds: ['tx-1', 'tx-2', 'tx-3'],
        totalAmount: 420_000_000,
        detectedAt: '2026-08-15T10:00:00.000Z',
        reasoning: 'Dấu hiệu chia nhỏ giao dịch (Smurfing): 3 giao dịch dưới 400M trong 24h tổng cộng 420.000.000 VND',
        statutoryRuleRef: 'Thông tư 09/2023/TT-NHNN Điều 3',
        suggestedStrReport: true,
      };

      const mockTxs = [
        { id: 'tx-1', counterpartyAccount: '0071009988776', counterparty: 'NGUYEN VAN PHONG', amount: 140_000_000 },
        { id: 'tx-2', counterpartyAccount: '0071009988776', counterparty: 'NGUYEN VAN PHONG', amount: 140_000_000 },
        { id: 'tx-3', counterpartyAccount: '0071009988776', counterparty: 'NGUYEN VAN PHONG', amount: 140_000_000 },
      ];

      const form = generateFormStr(mockAlert, mockTxs, 'Yeu cau CCO trinh cuc PCRT.');

      // Validate all required fields exist and have proper types
      expect(form.reportingEntity).toBe('LIVA SOLUTIONS CO., LTD');
      expect(typeof form.reportDate).toBe('string');
      expect(/^\d{4}-\d{2}-\d{2}$/.test(form.reportDate)).toBe(true);
      expect(form.alertType).toBe('STRUCTURING_SMURFING');
      expect(form.severity).toBe('CRITICAL');
      expect(form.suspectAccount).toBe('0071009988776');
      expect(form.suspectName).toBe('NGUYEN VAN PHONG');
      expect(form.transactionCount).toBe(3);
      expect(form.totalVndAmount).toBe(420_000_000);
      expect(form.formTemplate).toBe('Phụ lục II Thông tư 09/2023/TT-NHNN');
      expect(form.statutoryRuleRef).toContain('Thông tư 09/2023/TT-NHNN');

      // Strict integrity check: no field should be undefined, null, NaN, or string 'undefined' / 'NaN'
      for (const [, v] of Object.entries(form)) {
        expect(v).toBeDefined();
        expect(v).not.toBeNull();
        if (typeof v === 'number') {
          expect(isNaN(v)).toBe(false);
        }
        if (typeof v === 'string') {
          expect(v).not.toContain('undefined');
          expect(v).not.toContain('NaN');
          expect(v.trim().length).toBeGreaterThan(0);
        }
      }
    });

    it('renders pristine official text document meeting Cục Phòng, chống rửa tiền formatting standards', () => {
      const mockAlert: AmlAlert = {
        alertId: 'aml-test-high-val',
        anomalyType: 'HIGH_VALUE',
        severity: 'HIGH',
        involvedTransactionIds: ['tx-hv-1'],
        totalAmount: 500_000_000,
        detectedAt: '2026-08-15T12:00:00.000Z',
        reasoning: 'Giao dịch giá trị lớn: 500.000.000 VND >= 400.000.000 VND',
        statutoryRuleRef: 'Quyết định 11/2023/QĐ-TTg',
        suggestedStrReport: true,
      };

      const form = generateFormStr(mockAlert, []);
      const doc = formatOfficialStrDocument(form);

      // Verify header and legal marks
      expect(doc).toContain('NGÂN HÀNG NHÀ NƯỚC VIỆT NAM');
      expect(doc).toContain('CỤC PHÒNG, CHỐNG RỬA TIỀN');
      expect(doc).toContain('BÁO CÁO GIAO DỊCH ĐÁNG NGỜ (FORM STR)');
      expect(doc).toContain('Phụ lục II Thông tư số 09/2023/TT-NHNN');
      expect(doc).toContain('500.000.000 VND');
      expect(doc).toContain('Quyết định 11/2023/QĐ-TTg');

      // Strictly zero occurrences of undefined, NaN, or null
      expect(doc.includes('undefined')).toBe(false);
      expect(doc.includes('NaN')).toBe(false);
      expect(doc.includes('null')).toBe(false);
    });

    it('provides safe regulatory fallbacks when transactions array is empty or sparse', () => {
      const bareAlert: AmlAlert = {
        alertId: 'aml-bare',
        anomalyType: 'NIGHT_VELOCITY',
        severity: 'HIGH',
        involvedTransactionIds: [],
        totalAmount: 75_000_000,
        detectedAt: new Date().toISOString(),
        reasoning: 'Giao dịch bất thường ngoài giờ 23:30',
        statutoryRuleRef: 'Circular 09/2023/TT-NHNN',
        suggestedStrReport: true,
      };

      const form = generateFormStr(bareAlert, []);
      expect(form.suspectAccount).toBeDefined();
      expect(form.suspectAccount.length).toBeGreaterThan(0);
      expect(form.suspectName).toBeDefined();
      expect(form.suspectName.length).toBeGreaterThan(0);
      expect(form.transactionCount).toBe(1);
      expect(form.totalVndAmount).toBe(75_000_000);

      const doc = formatOfficialStrDocument(form);
      expect(doc.includes('undefined')).toBe(false);
      expect(doc.includes('NaN')).toBe(false);
    });
  });
});
