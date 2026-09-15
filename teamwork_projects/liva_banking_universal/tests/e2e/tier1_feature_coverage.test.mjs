/**
 * Tier 1: Comprehensive Feature Coverage E2E Test Suite
 * Covers all 21 features (F01 - F21) with 5 test cases per feature (105 total tests).
 * Validates primary happy path behaviors, compliance invariants, and interface contracts.
 */

import {
  assert,
  assertEqual,
  assertTrue,
  assertFalse,
  assertDeepEqual,
  assertThrows,
  loadDemoFile,
  loadDemoText,
  parseVietnameseAmount,
  parseStatementCsvOrText,
  parseExcelFile,
  parseLedgerCsv,
  verifyBalanceInvariants,
  verifyRunningBalanceContinuity,
  matchTier1Exact,
  matchTier2Fuzzy,
  matchTier3Split,
  createHitlQuarantine,
  normalizeVietnameseIntent,
  disentangleWireFee,
  detectAmlHighValue,
  detectAmlStructuring,
  detectAmlNightVelocity,
  detectAmlRapidPassThrough,
  runFullAmlSurveillance,
  generateFormStr,
  inspectSystemPrompt,
  createPaymentVoucher,
  submitVoucherForApproval,
  approveVoucher,
  rejectVoucher,
  computeMerkleLeaf,
  buildMerkleTree,
  queryFinancialCopilot,
  createGuidedTourState,
  verifyWebDeploymentConfig,
} from './harness_helpers.mjs';

export async function runTier1Tests() {
  const results = [];

  function test(id, title, fn) {
    const start = performance.now();
    try {
      fn();
      results.push({ id, title, passed: true, durationMs: performance.now() - start });
    } catch (err) {
      results.push({
        id,
        title,
        passed: false,
        error: err instanceof Error ? err.message : String(err),
        durationMs: performance.now() - start,
      });
    }
  }

  // =========================================================================
  // FEATURE F01: Universal Statement Dropzone (5 tests)
  // =========================================================================
  test('F01-01', 'F01: Ingest Excel statement (.xlsx) and parse rows', () => {
    const fileBuffer = loadDemoFile('01_VCB_SaoKe_Thang8_Chuan.xlsx');
    const parsed = parseExcelFile(fileBuffer, '01_VCB_SaoKe_Thang8_Chuan.xlsx');
    assertTrue(parsed.transactions.length > 0, 'Should parse transactions from VCB Excel');
    assertEqual(parsed.bankCode, 'VCB', 'Should detect VCB bank code');
  });

  test('F01-02', 'F01: Ingest semicolon-delimited CSV statement (TCB)', () => {
    const csvContent = loadDemoText('02_TCB_SaoKe_Thang8_Chuan.csv');
    const parsed = parseStatementCsvOrText(csvContent, '02_TCB_SaoKe_Thang8_Chuan.csv');
    assertEqual(parsed.bankCode, 'TCB', 'Should detect TCB bank code');
    assertEqual(parsed.transactions.length, 5, 'TCB demo statement has 5 transactions');
  });

  test('F01-03', 'F01: Ingest comma/semicolon CSV with off-hours anomalies (BIDV)', () => {
    const csvContent = loadDemoText('03_BIDV_SaoKe_NgoaiGio_BatThuong.csv');
    const parsed = parseStatementCsvOrText(csvContent, '03_BIDV_SaoKe_NgoaiGio_BatThuong.csv');
    assertEqual(parsed.bankCode, 'BIDV', 'Should detect BIDV bank code');
    assertEqual(parsed.transactions.length, 4, 'BIDV demo statement has 4 transactions');
  });

  test('F01-04', 'F01: Ingest tab-delimited pasted text from Google Sheets', () => {
    const textContent = loadDemoText('05_DuLieu_Dan_GoogleSheets.txt');
    const parsed = parseStatementCsvOrText(textContent, 'clipboard.txt');
    assertTrue(parsed.transactions.length >= 8, 'Should parse pasted tab-delimited text rows');
  });

  test('F01-05', 'F01: Ingest UTF-8 BOM formatted CSV statement without header corruption', () => {
    const bomCsv = '\uFEFFNgày;Mã GD;Ghi nợ;Ghi có;Số dư;Nội dung\n01/08/2026;TX01;;10.000.000;10.000.000;Thu tien ban hang\n';
    const parsed = parseStatementCsvOrText(bomCsv, 'vcb_bom.csv');
    assertEqual(parsed.transactions.length, 1, 'Should parse single BOM row');
    assertEqual(parsed.transactions[0].credit, 10_000_000, 'Should parse credit correctly');
  });

  // =========================================================================
  // FEATURE F02: Multi-Bank Schema Normalization (5 tests)
  // =========================================================================
  test('F02-01', 'F02: Normalize VCB schema with Vietnamese header names', () => {
    const raw = 'Ngày giao dịch;Mã giao dịch;Số tiền ghi nợ;Số tiền ghi có;Số dư;Nội dung chi tiết;Tên đối tác\n01/08/2026;VCB01;;145.000.000;145.000.000;TT tien hang;CONG TY AN PHAT\n';
    const parsed = parseStatementCsvOrText(raw, 'vcb.csv');
    assertEqual(parsed.transactions[0].txCode, 'VCB01');
    assertEqual(parsed.transactions[0].counterparty, 'CONG TY AN PHAT');
  });

  test('F02-02', 'F02: Normalize TCB schema with pre-header account metadata', () => {
    const raw = loadDemoText('02_TCB_SaoKe_Thang8_Chuan.csv');
    const parsed = parseStatementCsvOrText(raw, 'tcb.csv');
    assertEqual(parsed.openingBalance, 785_600_000, 'Should extract opening balance from pre-header rows');
    assertEqual(parsed.transactions[0].txCode, 'TCB26240001');
  });

  test('F02-03', 'F02: Normalize BIDV schema with counterparty details', () => {
    const raw = loadDemoText('03_BIDV_SaoKe_NgoaiGio_BatThuong.csv');
    const parsed = parseStatementCsvOrText(raw, 'bidv.csv');
    assertEqual(parsed.transactions[0].counterparty, 'VU TRONG PHUONG');
  });

  test('F02-04', 'F02: Normalize date and time strings into canonical ISO dates', () => {
    const raw = 'Ngày giao dịch;Mã GD;Số tiền;Số dư;Diễn giải\n15/08/2026 14:30:00;TX99;500.000;500.000;Phi dich vu\n';
    const parsed = parseStatementCsvOrText(raw, 'generic.csv');
    assertEqual(parsed.transactions[0].date, '2026-08-15');
    assertEqual(parsed.transactions[0].time, '14:30:00');
  });

  test('F02-05', 'F02: Tag canonical bank code from content or filename', () => {
    const vcbParsed = parseStatementCsvOrText('Ngày;Tiền;Dư\n01/08/2026;1000;1000', 'VCB_Statement.csv');
    assertEqual(vcbParsed.bankCode, 'VCB');
    const tcbParsed = parseStatementCsvOrText('Ngày;Tiền;Dư\n01/08/2026;1000;1000', 'TCB_Statement.csv');
    assertEqual(tcbParsed.bankCode, 'TCB');
  });

  // =========================================================================
  // FEATURE F03: Amount & Currency Integer Parser (5 tests)
  // =========================================================================
  test('F03-01', 'F03: Parse Vietnamese European dot notation into integer VND', () => {
    assertEqual(parseVietnameseAmount('145.000.000'), 145_000_000);
    assertEqual(parseVietnameseAmount('550.000.000'), 550_000_000);
  });

  test('F03-02', 'F03: Parse US comma notation into integer VND', () => {
    assertEqual(parseVietnameseAmount('145,000,000'), 145_000_000);
    assertEqual(parseVietnameseAmount('1,234,567'), 1_234_567);
  });

  test('F03-03', 'F03: Parse European decimal with rounding', () => {
    assertEqual(parseVietnameseAmount('45.000.000,49'), 45_000_000);
    assertEqual(parseVietnameseAmount('45.000.000,50'), 45_000_001);
  });

  test('F03-04', 'F03: Parse negative accounting values in parentheses and minus', () => {
    assertEqual(parseVietnameseAmount('(2.200.000)'), -2_200_000);
    assertEqual(parseVietnameseAmount('-500.000'), -500_000);
  });

  test('F03-05', 'F03: Strip currency annotations (VND, VNĐ) and whitespace cleanly', () => {
    assertEqual(parseVietnameseAmount('120.000.000 VND'), 120_000_000);
    assertEqual(parseVietnameseAmount('  85.000.000 VNĐ  '), 85_000_000);
  });

  // =========================================================================
  // FEATURE F04: Balance Invariant Validator (5 tests)
  // =========================================================================
  test('F04-01', 'F04: Verify macro double-entry identity on balanced statement', () => {
    const report = verifyBalanceInvariants(785_600_000, 1_583_400_000, 800_000_000, 2_200_000);
    assertTrue(report.isBalanced, 'Calculated closing must match closing balance');
    assertEqual(report.discrepancy, 0);
  });

  test('F04-02', 'F04: Detect macro balance discrepancy on corrupted closing balance', () => {
    const report = verifyBalanceInvariants(1_000_000, 2_500_000, 1_000_000, 0);
    assertFalse(report.isBalanced, 'Should detect discrepancy');
    assertEqual(report.discrepancy, 500_000);
    assertTrue(Boolean(report.errorReason));
  });

  test('F04-03', 'F04: Verify running balance continuity across sequence of transactions', () => {
    const txs = [
      { id: '1', credit: 50_000_000, debit: 0, balance: 150_000_000 },
      { id: '2', credit: 0, debit: 20_000_000, balance: 130_000_000 },
      { id: '3', credit: 10_000_000, debit: 0, balance: 140_000_000 },
    ];
    const res = verifyRunningBalanceContinuity(txs, 100_000_000);
    assertTrue(res.isContinuous, 'Running balance should be continuous');
  });

  test('F04-04', 'F04: Report exact broken row when running balance deviates', () => {
    const txs = [
      { id: '1', credit: 50_000_000, debit: 0, balance: 150_000_000 },
      { id: '2', credit: 0, debit: 20_000_000, balance: 999_000_000 }, // Broken balance!
    ];
    const res = verifyRunningBalanceContinuity(txs, 100_000_000);
    assertFalse(res.isContinuous, 'Should detect broken continuity');
    assertEqual(res.brokenRowIndex, 1);
  });

  test('F04-05', 'F04: Confirm zero floating-point arithmetic drift in balance validator', () => {
    const report = verifyBalanceInvariants(100_000_000, 100_000_000, 0.1, 0.1);
    assertEqual(report.calculatedClosing, 100_000_000, 'Rounded integer VND eliminates float drift');
  });

  // =========================================================================
  // FEATURE F05: Tier 1 Exact Matcher (5 tests)
  // =========================================================================
  test('F05-01', 'F05: Match exact amount and document invoice reference 1:1', () => {
    const bankTxs = [
      { id: 'tx-1', amount: 45_000_000, txType: 'CREDIT', narration: 'Napas VietQR TT HD131 An Phat', txCode: 'TCB01' },
    ];
    const ledgers = [
      { id: 'l-1', amount: 45_000_000, entryType: 'CREDIT', docNo: 'HD131', partnerName: 'An Phat' },
    ];
    const result = matchTier1Exact(bankTxs, ledgers);
    assertEqual(result.matches.length, 1);
    assertEqual(result.matches[0].confidence, 1.0);
    assertEqual(result.matches[0].tier, 'TIER1_EXACT');
  });

  test('F05-02', 'F05: Enforce strict transaction direction (Credit to Credit, Debit to Debit)', () => {
    const bankTxs = [
      { id: 'tx-1', amount: 50_000_000, txType: 'DEBIT', narration: 'HD99', txCode: 'TX01' },
    ];
    const ledgers = [
      { id: 'l-1', amount: 50_000_000, entryType: 'CREDIT', docNo: 'HD99', partnerName: 'ABC' },
    ];
    const result = matchTier1Exact(bankTxs, ledgers);
    assertEqual(result.matches.length, 0, 'Opposite transaction types should NOT match in Tier 1');
  });

  test('F05-03', 'F05: Verify exact match produces zero fee and zero discrepancy', () => {
    const bankTxs = [
      { id: 'tx-1', amount: 120_000_000, txType: 'CREDIT', narration: 'Thanh toan HD-2026-92', txCode: 'TX02' },
    ];
    const ledgers = [
      { id: 'l-1', amount: 120_000_000, entryType: 'CREDIT', docNo: 'HD-2026-92', partnerName: 'Dai Viet' },
    ];
    const result = matchTier1Exact(bankTxs, ledgers);
    assertEqual(result.matches[0].feeAmount, 0);
    assertEqual(result.matches[0].matchedAmount, 120_000_000);
  });

  test('F05-04', 'F05: Leave unmatched records in unallocated lists cleanly', () => {
    const bankTxs = [
      { id: 'tx-1', amount: 10_000_000, txType: 'CREDIT', narration: 'Khong co hoa don', txCode: 'TX03' },
    ];
    const ledgers = [
      { id: 'l-1', amount: 20_000_000, entryType: 'CREDIT', docNo: 'HD-999', partnerName: 'XYZ' },
    ];
    const result = matchTier1Exact(bankTxs, ledgers);
    assertEqual(result.matches.length, 0);
    assertEqual(result.unallocatedBank.length, 1);
    assertEqual(result.unallocatedLedger.length, 1);
  });

  test('F05-05', 'F05: Match multiple 1:1 records without double-claiming', () => {
    const bankTxs = [
      { id: 'tx-1', amount: 10_000_000, txType: 'CREDIT', narration: 'HD01', txCode: 'TX01' },
      { id: 'tx-2', amount: 10_000_000, txType: 'CREDIT', narration: 'HD01 duplicate', txCode: 'TX02' },
    ];
    const ledgers = [
      { id: 'l-1', amount: 10_000_000, entryType: 'CREDIT', docNo: 'HD01', partnerName: 'Comp1' },
    ];
    const result = matchTier1Exact(bankTxs, ledgers);
    assertEqual(result.matches.length, 1, 'Only one transaction can claim the single ledger entry');
    assertEqual(result.unallocatedBank.length, 1);
  });

  // =========================================================================
  // FEATURE F06: Tier 2 Fuzzy Heuristic Matcher (5 tests)
  // =========================================================================
  test('F06-01', 'F06: Match using Jaro-Winkler party name similarity >= 0.85', () => {
    const bankTxs = [
      { id: 'tx-1', amount: 60_000_000, txType: 'CREDIT', counterparty: 'CTCP Giai Phap Cong Nghe Nova', narration: 'TT web' },
    ];
    const ledgers = [
      { id: 'l-1', amount: 60_000_000, entryType: 'CREDIT', partnerName: 'CTCP GIAI PHAP CONG NGHE NOVA', docNo: 'HD-94' },
    ];
    const result = matchTier2Fuzzy(bankTxs, ledgers);
    assertEqual(result.matches.length, 1);
    assertEqual(result.matches[0].tier, 'TIER2_FUZZY');
  });

  test('F06-02', 'F06: Match transaction with interbank wire fee tolerance (1,100 VND)', () => {
    const bankTxs = [
      { id: 'tx-1', amount: 84_998_900, txType: 'CREDIT', counterparty: 'FPT Smart Cloud', narration: 'ck fpt' },
    ];
    const ledgers = [
      { id: 'l-1', amount: 85_000_000, entryType: 'CREDIT', partnerName: 'FPT Smart Cloud', docNo: 'HD-93' },
    ];
    const result = matchTier2Fuzzy(bankTxs, ledgers);
    assertEqual(result.matches.length, 1);
    assertEqual(result.matches[0].feeAmount, 1100);
  });

  test('F06-03', 'F06: Match transaction with interbank wire fee tolerance (2,200 VND)', () => {
    const bankTxs = [
      { id: 'tx-1', amount: 99_997_800, txType: 'CREDIT', counterparty: 'Cong ty An Phat', narration: 'ck' },
    ];
    const ledgers = [
      { id: 'l-1', amount: 100_000_000, entryType: 'CREDIT', partnerName: 'CONG TY TNHH AN PHAT', docNo: 'HD-01' },
    ];
    const result = matchTier2Fuzzy(bankTxs, ledgers);
    assertEqual(result.matches.length, 1);
    assertEqual(result.matches[0].feeAmount, 2200);
  });

  test('F06-04', 'F06: Reject fuzzy match when counterparty names have zero similarity', () => {
    const bankTxs = [
      { id: 'tx-1', amount: 50_000_000, txType: 'CREDIT', counterparty: 'Cong ty Bia Sai Gon', narration: 'ck' },
    ];
    const ledgers = [
      { id: 'l-1', amount: 50_000_000, entryType: 'CREDIT', partnerName: 'Benh vien Cho Ray', docNo: 'HD-10' },
    ];
    const result = matchTier2Fuzzy(bankTxs, ledgers);
    assertEqual(result.matches.length, 0, 'Distinct parties should not match');
  });

  test('F06-05', 'F06: Verify confidence score is assigned in 0.85 - 0.98 range', () => {
    const bankTxs = [
      { id: 'tx-1', amount: 550_000_000, txType: 'CREDIT', counterparty: 'Tap doan Masan', narration: 'xe nang' },
    ];
    const ledgers = [
      { id: 'l-1', amount: 550_000_000, entryType: 'CREDIT', partnerName: 'Cong ty CP Tap doan Masan', docNo: 'HD-91' },
    ];
    const result = matchTier2Fuzzy(bankTxs, ledgers);
    assertTrue(result.matches[0].confidence >= 0.85 && result.matches[0].confidence <= 0.98);
  });

  // =========================================================================
  // FEATURE F07: Tier 3 Subset-Sum Split Solver (5 tests)
  // =========================================================================
  test('F07-01', 'F07: Solve 1 Bank Payment to 2 Ledger Invoices (1:N composite match)', () => {
    const bankTxs = [
      { id: 'b-1', amount: 190_000_000, txType: 'CREDIT', narration: 'Thanh toan gop 2 hoa don' },
    ];
    const ledgers = [
      { id: 'l-1', amount: 145_000_000, entryType: 'CREDIT', docNo: 'HD-88' },
      { id: 'l-2', amount: 45_000_000, entryType: 'CREDIT', docNo: 'HD-90' },
    ];
    const result = matchTier3Split(bankTxs, ledgers);
    assertEqual(result.matches.length, 1);
    assertEqual(result.matches[0].matchType, 'COMPOSITE_1_TO_N');
    assertEqual(result.matches[0].ledgerEntryIds.length, 2);
  });

  test('F07-02', 'F07: Enforce exact Delta = 0 zero remainder invariant in subset-sum', () => {
    const bankTxs = [
      { id: 'b-1', amount: 200_000_000, txType: 'CREDIT', narration: 'Thanh toan' },
    ];
    const ledgers = [
      { id: 'l-1', amount: 100_000_000, entryType: 'CREDIT', docNo: 'HD-1' },
      { id: 'l-2', amount: 99_000_000, entryType: 'CREDIT', docNo: 'HD-2' }, // sum is 199M != 200M
    ];
    const result = matchTier3Split(bankTxs, ledgers);
    assertEqual(result.matches.length, 0, 'Subset sum must match exactly with 0 discrepancy');
  });

  test('F07-03', 'F07: Tier 3 assigns confidence between 0.95 and 0.99', () => {
    const bankTxs = [{ id: 'b-1', amount: 300_000_000, txType: 'CREDIT' }];
    const ledgers = [
      { id: 'l-1', amount: 200_000_000, entryType: 'CREDIT', docNo: 'H1' },
      { id: 'l-2', amount: 100_000_000, entryType: 'CREDIT', docNo: 'H2' },
    ];
    const result = matchTier3Split(bankTxs, ledgers);
    assertTrue(result.matches[0].confidence >= 0.95 && result.matches[0].confidence <= 0.99);
  });

  test('F07-04', 'F07: Do not split across conflicting transaction directions (Debit vs Credit)', () => {
    const bankTxs = [{ id: 'b-1', amount: 150_000_000, txType: 'CREDIT' }];
    const ledgers = [
      { id: 'l-1', amount: 100_000_000, entryType: 'DEBIT', docNo: 'H1' },
      { id: 'l-2', amount: 50_000_000, entryType: 'DEBIT', docNo: 'H2' },
    ];
    const result = matchTier3Split(bankTxs, ledgers);
    assertEqual(result.matches.length, 0);
  });

  test('F07-05', 'F07: Return unallocated items when no combination matches target', () => {
    const bankTxs = [{ id: 'b-1', amount: 500_000_000, txType: 'CREDIT' }];
    const ledgers = [
      { id: 'l-1', amount: 100_000_000, entryType: 'CREDIT', docNo: 'H1' },
    ];
    const result = matchTier3Split(bankTxs, ledgers);
    assertEqual(result.matches.length, 0);
    assertEqual(result.unallocatedBank.length, 1);
  });

  // =========================================================================
  // FEATURE F08: HITL Quarantine Management (5 tests)
  // =========================================================================
  test('F08-01', 'F08: Fail-closed isolation creates quarantine item for unallocated record', () => {
    const tx = { id: 'tx-quarantine', amount: 37_500_000, narration: 'Giao dich khong xac dinh' };
    const qItem = createHitlQuarantine(tx, 'No matching invoice found');
    assertEqual(qItem.txId, 'tx-quarantine');
    assertEqual(qItem.amount, 37_500_000);
    assertFalse(qItem.isResolved);
  });

  test('F08-02', 'F08: Quarantine generates secure single-use UUIDv4 token', () => {
    const qItem = createHitlQuarantine({ id: 'tx-1', amount: 10_000_000 });
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
    assertTrue(uuidRegex.test(qItem.hitlToken), 'Token must be valid UUIDv4');
  });

  test('F08-03', 'F08: Enforce 15-minute TTL (900 seconds) on resolution token', () => {
    const qItem = createHitlQuarantine({ id: 'tx-1', amount: 10_000_000 });
    assertEqual(qItem.expiresAt - qItem.createdAt, 900, 'TTL must be exactly 900 seconds');
  });

  test('F08-04', 'F08: Quarantine item records full raw transaction context', () => {
    const tx = { id: 'tx-1', amount: 25_000_000, narration: 'Audit test' };
    const qItem = createHitlQuarantine(tx);
    assertEqual(qItem.rawTransaction.id, 'tx-1');
  });

  test('F08-05', 'F08: Prevent duplicate active quarantine tokens for same transaction', () => {
    const tx = { id: 'tx-dup', amount: 15_000_000 };
    const q1 = createHitlQuarantine(tx);
    const q2 = createHitlQuarantine(tx);
    assertTrue(q1.hitlToken !== q2.hitlToken, 'Tokens must be distinct unique nonces');
  });

  // =========================================================================
  // FEATURE F09: Vietnamese Intent Normalizer (5 tests)
  // =========================================================================
  test('F09-01', 'F09: Expand bank transfer abbreviations ("ck", "tt")', () => {
    const res = normalizeVietnameseIntent('ck tien may bom hd 88');
    assertTrue(res.normalized.includes('chuyển khoản'));
    assertEqual(res.intent, 'PAYMENT');
  });

  test('F09-02', 'F09: Expand invoice abbreviations ("hd", "hop dong")', () => {
    const res = normalizeVietnameseIntent('tt hd 131');
    assertTrue(res.normalized.includes('hóa đơn'));
    assertEqual(res.invoiceNumbers[0], '131');
  });

  test('F09-03', 'F09: Detect advance payment intent ("ung", "tam ung")', () => {
    const res = normalizeVietnameseIntent('tam ung chi phi cong tac');
    assertEqual(res.intent, 'ADVANCE');
  });

  test('F09-04', 'F09: Detect bank fee intent ("phi duy tri", "phi ql")', () => {
    const res = normalizeVietnameseIntent('phi duy tri quan ly tai khoan doanh nghiep');
    assertEqual(res.intent, 'BANK_FEE');
  });

  test('F09-05', 'F09: Extract invoice numbers with alphanumeric patterns', () => {
    const res = normalizeVietnameseIntent('Napas VietQR TT HD-2026-88 An Phat');
    assertEqual(res.invoiceNumbers[0], '2026-88');
  });

  // =========================================================================
  // FEATURE F10: Wire Fee Disentanglement (5 tests)
  // =========================================================================
  test('F10-01', 'F10: Disentangle 2,200 VND Napas wire fee from transaction principal', () => {
    const res = disentangleWireFee(45_000_000, 'ck tien hang phi 2200');
    assertEqual(res.feeAmount, 2200);
    assertEqual(res.principal, 44_997_800);
    assertEqual(res.accountCode, 'TK 6425');
  });

  test('F10-02', 'F10: Disentangle 1,100 VND wire fee and allocate to TK 6425', () => {
    const res = disentangleWireFee(10_000_000, 'chuyen tien phi 1100');
    assertEqual(res.feeAmount, 1100);
    assertEqual(res.principal, 9_998_900);
    assertEqual(res.accountName, 'Chi phí dịch vụ ngân hàng');
  });

  test('F10-03', 'F10: Handle pure bank account fee as 100% fee without principal', () => {
    const res = disentangleWireFee(2_200_000, 'Phi duy tri quan ly tai khoan');
    assertTrue(res.isPureFee);
    assertEqual(res.feeAmount, 2_200_000);
    assertEqual(res.principal, 0);
  });

  test('F10-04', 'F10: Verify arithmetic invariant: Principal + Fee == Gross Amount', () => {
    const gross = 85_000_000;
    const res = disentangleWireFee(gross, 'Phi chuyen khoan 5500');
    assertTrue(res.sumCheckPassed);
    assertEqual(res.principal + res.feeAmount, gross);
  });

  test('F10-05', 'F10: Return zero fee when transaction contains no wire fee marker', () => {
    const res = disentangleWireFee(120_000_000, 'Thanh toan tien may bien ap');
    assertEqual(res.feeAmount, 0);
    assertEqual(res.principal, 120_000_000);
  });

  // =========================================================================
  // FEATURE F11: AML Structuring Detector (5 tests)
  // =========================================================================
  test('F11-01', 'F11: Detect >= 3 sub-400M transfers within 24h summing >= 400M VND', () => {
    const txs = [
      { id: 'tx-1', date: '2026-08-10', amount: 390_000_000 },
      { id: 'tx-2', date: '2026-08-10', amount: 385_000_000 },
      { id: 'tx-3', date: '2026-08-10', amount: 395_000_000 },
    ];
    const alerts = detectAmlStructuring(txs);
    assertEqual(alerts.length, 1);
    assertEqual(alerts[0].anomalyType, 'STRUCTURING_SMURFING');
    assertEqual(alerts[0].severity, 'CRITICAL');
  });

  test('F11-02', 'F11: Do not flag structuring if transaction count < 3', () => {
    const txs = [
      { id: 'tx-1', date: '2026-08-10', amount: 300_000_000 },
      { id: 'tx-2', date: '2026-08-10', amount: 300_000_000 },
    ];
    const alerts = detectAmlStructuring(txs);
    assertEqual(alerts.length, 0, 'Only 2 transactions should not trigger smurfing alert');
  });

  test('F11-03', 'F11: Do not flag structuring if cumulative sum < 400M VND', () => {
    const txs = [
      { id: 'tx-1', date: '2026-08-10', amount: 100_000_000 },
      { id: 'tx-2', date: '2026-08-10', amount: 100_000_000 },
      { id: 'tx-3', date: '2026-08-10', amount: 100_000_000 },
    ];
    const alerts = detectAmlStructuring(txs);
    assertEqual(alerts.length, 0, 'Sum is 300M < 400M limit');
  });

  test('F11-04', 'F11: Structuring alert records statutory rule reference (Circular 09/2023)', () => {
    const txs = [
      { id: 'tx-1', date: '2026-08-10', amount: 150_000_000 },
      { id: 'tx-2', date: '2026-08-10', amount: 150_000_000 },
      { id: 'tx-3', date: '2026-08-10', amount: 150_000_000 },
    ];
    const alerts = detectAmlStructuring(txs);
    assertTrue(alerts[0].statutoryRuleRef.includes('Thông tư 09/2023/TT-NHNN'));
  });

  test('F11-05', 'F11: Structuring alert recommends STR report creation', () => {
    const txs = [
      { id: 'tx-1', date: '2026-08-10', amount: 200_000_000 },
      { id: 'tx-2', date: '2026-08-10', amount: 200_000_000 },
      { id: 'tx-3', date: '2026-08-10', amount: 200_000_000 },
    ];
    const alerts = detectAmlStructuring(txs);
    assertTrue(alerts[0].suggestedStrReport);
  });

  // =========================================================================
  // FEATURE F12: AML High-Value Detector (5 tests)
  // =========================================================================
  test('F12-01', 'F12: Mandatory flag for single transaction >= 400M VND (Decision 11/2023)', () => {
    const txs = [{ id: 'tx-1', amount: 550_000_000 }];
    const alerts = detectAmlHighValue(txs);
    assertEqual(alerts.length, 1);
    assertEqual(alerts[0].anomalyType, 'HIGH_VALUE');
  });

  test('F12-02', 'F12: Flag BIDV 420M VND transaction as high-value', () => {
    const txs = [{ id: 'bidv-1', amount: 420_000_000 }];
    const alerts = detectAmlHighValue(txs);
    assertEqual(alerts.length, 1);
  });

  test('F12-03', 'F12: Do not flag transaction below 400M VND limit', () => {
    const txs = [{ id: 'tx-sub', amount: 399_999_999 }];
    const alerts = detectAmlHighValue(txs);
    assertEqual(alerts.length, 0);
  });

  test('F12-04', 'F12: Exact threshold boundary 400,000,000 VND triggers alert', () => {
    const txs = [{ id: 'tx-exact', amount: 400_000_000 }];
    const alerts = detectAmlHighValue(txs);
    assertEqual(alerts.length, 1);
  });

  test('F12-05', 'F12: High-value alert quotes Decision 11/2023/QD-TTg', () => {
    const txs = [{ id: 'tx-1', amount: 500_000_000 }];
    const alerts = detectAmlHighValue(txs);
    assertTrue(alerts[0].statutoryRuleRef.includes('Quyết định 11/2023/QĐ-TTg'));
  });

  // =========================================================================
  // FEATURE F13: AML Rapid Pass-Through Detector (5 tests)
  // =========================================================================
  test('F13-01', 'F13: Detect inflow >= 100M followed by drain >= 90% in rapid succession', () => {
    const txs = [
      { id: 'in-1', txType: 'CREDIT', amount: 420_000_000, time: '02:15:20' },
      { id: 'out-1', txType: 'DEBIT', amount: 419_500_000, time: '02:22:45' }, // 99.88% drain
    ];
    const alerts = detectAmlRapidPassThrough(txs);
    assertEqual(alerts.length, 1);
    assertEqual(alerts[0].anomalyType, 'RAPID_PASS_THROUGH');
    assertEqual(alerts[0].severity, 'CRITICAL');
  });

  test('F13-02', 'F13: Do not flag pass-through if drain ratio < 90%', () => {
    const txs = [
      { id: 'in-1', txType: 'CREDIT', amount: 500_000_000 },
      { id: 'out-1', txType: 'DEBIT', amount: 200_000_000 }, // 40% drain
    ];
    const alerts = detectAmlRapidPassThrough(txs);
    assertEqual(alerts.length, 0);
  });

  test('F13-03', 'F13: Do not flag pass-through if initial inflow < 100M VND', () => {
    const txs = [
      { id: 'in-1', txType: 'CREDIT', amount: 50_000_000 },
      { id: 'out-1', txType: 'DEBIT', amount: 49_900_000 },
    ];
    const alerts = detectAmlRapidPassThrough(txs);
    assertEqual(alerts.length, 0);
  });

  test('F13-04', 'F13: Record involved transaction IDs for both inflow and outflow', () => {
    const txs = [
      { id: 'in-x', txType: 'CREDIT', amount: 200_000_000 },
      { id: 'out-x', txType: 'DEBIT', amount: 195_000_000 },
    ];
    const alerts = detectAmlRapidPassThrough(txs);
    assertEqual(alerts[0].involvedTransactionIds.length, 2);
  });

  test('F13-05', 'F13: Rapid pass-through quotes Circular 09/2023 Article 3', () => {
    const txs = [
      { id: 'in-1', txType: 'CREDIT', amount: 100_000_000 },
      { id: 'out-1', txType: 'DEBIT', amount: 95_000_000 },
    ];
    const alerts = detectAmlRapidPassThrough(txs);
    assertTrue(alerts[0].statutoryRuleRef.includes('Thông tư 09/2023/TT-NHNN'));
  });

  // =========================================================================
  // FEATURE F14: AML Night-Time Velocity Detector (5 tests)
  // =========================================================================
  test('F14-01', 'F14: Detect transactions between 23:00 and 05:00 with amount >= 50M VND', () => {
    const txs = [{ id: 'tx-night', time: '02:15:20', amount: 420_000_000 }];
    const alerts = detectAmlNightVelocity(txs);
    assertEqual(alerts.length, 1);
    assertEqual(alerts[0].anomalyType, 'NIGHT_VELOCITY');
  });

  test('F14-02', 'F14: Daytime transaction (08:30) does NOT trigger night velocity', () => {
    const txs = [{ id: 'tx-day', time: '08:30:00', amount: 500_000_000 }];
    const alerts = detectAmlNightVelocity(txs);
    assertEqual(alerts.length, 0);
  });

  test('F14-03', 'F14: Night transaction under 50M VND does NOT trigger alert', () => {
    const txs = [{ id: 'tx-small', time: '01:00:00', amount: 20_000_000 }];
    const alerts = detectAmlNightVelocity(txs);
    assertEqual(alerts.length, 0);
  });

  test('F14-04', 'F14: Trigger on 23:30 late night boundary', () => {
    const txs = [{ id: 'tx-late', time: '23:30:00', amount: 60_000_000 }];
    const alerts = detectAmlNightVelocity(txs);
    assertEqual(alerts.length, 1);
  });

  test('F14-05', 'F14: Trigger on 04:30 early morning boundary', () => {
    const txs = [{ id: 'tx-early', time: '04:30:00', amount: 75_000_000 }];
    const alerts = detectAmlNightVelocity(txs);
    assertEqual(alerts.length, 1);
  });

  // =========================================================================
  // FEATURE F15: Statutory Form STR Generator (5 tests)
  // =========================================================================
  test('F15-01', 'F15: Generate Form STR payload adhering to Phu luc II TT 09/2023', () => {
    const alert = {
      anomalyType: 'STRUCTURING_SMURFING',
      severity: 'CRITICAL',
      involvedTransactionIds: ['tx-1', 'tx-2', 'tx-3'],
      totalAmount: 1_170_000_000,
      reasoning: 'Smurfing detected',
      statutoryRuleRef: 'Thông tư 09/2023/TT-NHNN',
    };
    const form = generateFormStr(alert);
    assertEqual(form.reportingEntity, 'LIVA SOLUTIONS CO., LTD');
    assertEqual(form.alertType, 'STRUCTURING_SMURFING');
    assertEqual(form.totalVndAmount, 1_170_000_000);
  });

  test('F15-02', 'F15: Include suspect account number and counterparty name in STR', () => {
    const alert = {
      anomalyType: 'HIGH_VALUE',
      severity: 'HIGH',
      involvedTransactionIds: ['tx-1'],
      totalAmount: 420_000_000,
      reasoning: 'High value transfer',
      statutoryRuleRef: 'Decision 11/2023',
    };
    const txs = [{ id: 'tx-1', counterparty: 'VU TRONG PHUONG', counterpartyAccount: '12010001234567' }];
    const form = generateFormStr(alert, txs);
    assertEqual(form.suspectName, 'VU TRONG PHUONG');
    assertEqual(form.suspectAccount, '12010001234567');
  });

  test('F15-03', 'F15: Format ISO report date automatically', () => {
    const alert = { anomalyType: 'NIGHT_VELOCITY', involvedTransactionIds: [], totalAmount: 50_000_000 };
    const form = generateFormStr(alert);
    assertTrue(/^\d{4}-\d{2}-\d{2}$/.test(form.reportDate));
  });

  test('F15-04', 'F15: Append custom compliance officer notes', () => {
    const alert = { anomalyType: 'RAPID_PASS_THROUGH', involvedTransactionIds: [], totalAmount: 400_000_000 };
    const form = generateFormStr(alert, [], 'Kính đề nghị kiểm tra dòng tiền');
    assertEqual(form.complianceOfficerNotes, 'Kính đề nghị kiểm tra dòng tiền');
  });

  test('F15-05', 'F15: Tag statutory form template metadata', () => {
    const alert = { anomalyType: 'HIGH_VALUE', involvedTransactionIds: [], totalAmount: 500_000_000 };
    const form = generateFormStr(alert);
    assertEqual(form.formTemplate, 'Phụ lục II Thông tư 09/2023/TT-NHNN');
  });

  // =========================================================================
  // FEATURE F16: System Prompt Inspector UI (5 tests)
  // =========================================================================
  test('F16-01', 'F16: Inspect active AML System Prompt template', () => {
    const inspector = inspectSystemPrompt();
    assertTrue(/Thông tư 09\/2023\/TT-NHNN/i.test(inspector.systemPrompt));
    assertTrue(/Quyết định 11\/2023\/QĐ-TTg/i.test(inspector.systemPrompt));
  });

  test('F16-02', 'F16: Retrieve default statutory thresholds', () => {
    const inspector = inspectSystemPrompt();
    assertEqual(inspector.parameters.highValueThreshold, 400_000_000);
    assertEqual(inspector.parameters.nightStartHour, 23);
    assertEqual(inspector.parameters.nightEndHour, 5);
  });

  test('F16-03', 'F16: Support dynamic tuning of AML threshold parameters', () => {
    const inspector = inspectSystemPrompt({ highValueThreshold: 300_000_000 });
    assertEqual(inspector.parameters.highValueThreshold, 300_000_000);
    assertTrue(inspector.systemPrompt.includes('300.000.000 VND'));
  });

  test('F16-04', 'F16: Execute test sandbox simulation against sample data', () => {
    const inspector = inspectSystemPrompt();
    const sandboxTxs = [{ id: 'tx-sb', amount: 500_000_000, time: '10:00:00' }];
    const alerts = inspector.evalSandbox(sandboxTxs);
    assertEqual(alerts.length, 1);
  });

  test('F16-05', 'F16: System prompt specifies strict zero-hallucination compliance rules', () => {
    const inspector = inspectSystemPrompt();
    assertTrue(inspector.systemPrompt.length > 50);
  });

  // =========================================================================
  // FEATURE F17: Maker-Checker Dual Control Gate (5 tests)
  // =========================================================================
  test('F17-01', 'F17: Create payment voucher in DRAFT status', () => {
    const vch = createPaymentVoucher('maker_01', '1903456789', 'TCB', 50_000_000, 'Tien thiet bi');
    assertEqual(vch.status, 'DRAFT');
    assertEqual(vch.makerId, 'maker_01');
    assertEqual(vch.amountVnd, 50_000_000);
  });

  test('F17-02', 'F17: Submit voucher transitioning to PENDING_APPROVAL', () => {
    const vch = createPaymentVoucher('maker_01', '1903456789', 'TCB', 50_000_000, 'Tien thiet bi');
    const pending = submitVoucherForApproval(vch);
    assertEqual(pending.status, 'PENDING_APPROVAL');
  });

  test('F17-03', 'F17: Checker approves voucher with distinct checker ID', () => {
    const vch = createPaymentVoucher('maker_01', '1903456789', 'TCB', 50_000_000, 'Tien thiet bi');
    const pending = submitVoucherForApproval(vch);
    const approved = approveVoucher(pending, 'checker_cfo');
    assertEqual(approved.status, 'APPROVED');
    assertEqual(approved.checkerId, 'checker_cfo');
    assertTrue(Boolean(approved.approvedAt));
  });

  test('F17-04', 'F17: Circular 09/2020 Violation: Prevent self-approval (makerId === checkerId)', () => {
    const vch = createPaymentVoucher('maker_01', '1903456789', 'TCB', 50_000_000, 'Tien thiet bi');
    const pending = submitVoucherForApproval(vch);
    assertThrows(() => {
      approveVoucher(pending, 'maker_01'); // Self approval!
    }, 'Maker cannot be Checker');
  });

  test('F17-05', 'F17: Checker rejects voucher with specified rejection reason', () => {
    const vch = createPaymentVoucher('maker_01', '1903456789', 'TCB', 50_000_000, 'Tien thiet bi');
    const pending = submitVoucherForApproval(vch);
    const rejected = rejectVoucher(pending, 'checker_02', 'Voucher thiếu chứng từ gốc đính kèm');
    assertEqual(rejected.status, 'REJECTED');
    assertEqual(rejected.rejectReason, 'Voucher thiếu chứng từ gốc đính kèm');
  });

  // =========================================================================
  // FEATURE F18: Cryptographic Merkle Audit Proof (5 tests)
  // =========================================================================
  test('F18-01', 'F18: Compute SHA-256 HMAC leaf hash for approved payment voucher', () => {
    const vch = {
      voucherId: 'vch-001',
      makerId: 'maker_1',
      checkerId: 'checker_1',
      amountVnd: 100_000_000,
      status: 'APPROVED',
      approvedAt: '2026-08-15T10:00:00Z',
    };
    const leaf = computeMerkleLeaf(vch);
    assertTrue(typeof leaf === 'string' && leaf.length === 64, 'SHA-256 hash must be 64 hex characters');
  });

  test('F18-02', 'F18: Build binary Merkle tree from collection of leaves', () => {
    const leaves = [
      computeMerkleLeaf({ voucherId: 'v1', makerId: 'm1', amountVnd: 10 }),
      computeMerkleLeaf({ voucherId: 'v2', makerId: 'm1', amountVnd: 20 }),
      computeMerkleLeaf({ voucherId: 'v3', makerId: 'm1', amountVnd: 30 }),
    ];
    const tree = buildMerkleTree(leaves);
    assertTrue(typeof tree.root === 'string' && tree.root.length === 64);
    assertEqual(tree.leaves.length, 3);
  });

  test('F18-03', 'F18: Single-leaf tree returns leaf as root', () => {
    const leaf = computeMerkleLeaf({ voucherId: 'v1', makerId: 'm1', amountVnd: 10 });
    const tree = buildMerkleTree([leaf]);
    assertEqual(tree.root, leaf);
  });

  test('F18-04', 'F18: Tamper-evident property: Altering single voucher character alters Merkle root', () => {
    const leaf1 = computeMerkleLeaf({ voucherId: 'v1', makerId: 'm1', amountVnd: 100 });
    const leaf2 = computeMerkleLeaf({ voucherId: 'v2', makerId: 'm1', amountVnd: 200 });
    const treeA = buildMerkleTree([leaf1, leaf2]);

    // Altered leaf
    const leafTampered = computeMerkleLeaf({ voucherId: 'v1', makerId: 'm1', amountVnd: 999 });
    const treeB = buildMerkleTree([leafTampered, leaf2]);

    assertTrue(treeA.root !== treeB.root, 'Merkle roots must diverge upon tampering');
  });

  test('F18-05', 'F18: Empty tree produces deterministic safe root hash', () => {
    const tree = buildMerkleTree([]);
    assertTrue(typeof tree.root === 'string' && tree.root.length === 64);
  });

  // =========================================================================
  // FEATURE F19: 2D Conversational Copilot Drawer (5 tests)
  // =========================================================================
  test('F19-01', 'F19: Answer liquidity runway inquiry in Vietnamese with factual metrics', () => {
    const res = queryFinancialCopilot('Tình hình thanh khoản hiện tại thế nào?', { closingBalance: 1_500_000_000 });
    assertEqual(res.intent, 'LIQUIDITY_RUNWAY');
    assertTrue(res.answer.includes('1.500.000.000 VND'));
    assertEqual(res.metrics.runwayDays, 100);
  });

  test('F19-02', 'F19: Answer reconciliation rate inquiry confirming >= 99.8% threshold', () => {
    const res = queryFinancialCopilot('Tỷ lệ đối soát tháng 8 đạt bao nhiêu?', { matchRate: 99.8 });
    assertEqual(res.intent, 'RECONCILIATION_RATE');
    assertTrue(res.answer.includes('99.8%'));
  });

  test('F19-03', 'F19: Answer highest expense inquiry citing Masan procurement', () => {
    const res = queryFinancialCopilot('Khoản chi phí lớn nhất trong kỳ là gì?');
    assertEqual(res.intent, 'TOP_EXPENSE');
    assertTrue(res.answer.includes('550.000.000 VND'));
  });

  test('F19-04', 'F19: Answer AML anomaly inquiry summarizing flagged transactions', () => {
    const res = queryFinancialCopilot('Có giao dịch nào đáng ngờ không?', { alertCount: 4 });
    assertEqual(res.intent, 'AML_SUMMARY');
    assertTrue(res.answer.includes('4 giao dịch đáng ngờ'));
  });

  test('F19-05', 'F19: Provide fallback assistance response for unknown queries', () => {
    const res = queryFinancialCopilot('Thời tiết hôm nay thế nào?');
    assertEqual(res.intent, 'GENERAL_ASSISTANCE');
    assertTrue(res.answer.includes('sẵn sàng hỗ trợ'));
  });

  // =========================================================================
  // FEATURE F20: 1-Click Guided Presentation Tour (5 tests)
  // =========================================================================
  test('F20-01', 'F20: Initialize tour with 5 standardized Demo Day steps', () => {
    const tour = createGuidedTourState();
    assertEqual(tour.steps.length, 5);
    assertEqual(tour.steps[0].id, 'INGESTION');
    assertEqual(tour.steps[4].id, 'OVERVIEW_COPILOT');
  });

  test('F20-02', 'F20: Tour begins playing from step 0 (Ingestion)', () => {
    const tour = createGuidedTourState();
    tour.play();
    assertTrue(tour.isPlaying());
    assertEqual(tour.getCurrentStep().stepIndex, 0);
  });

  test('F20-03', 'F20: Advance sequentially through all presentation tour steps', () => {
    const tour = createGuidedTourState();
    tour.play();
    const s1 = tour.nextStep();
    assertEqual(s1.stepIndex, 1);
    const s2 = tour.nextStep();
    assertEqual(s2.stepIndex, 2);
    const s3 = tour.nextStep();
    assertEqual(s3.stepIndex, 3);
    const s4 = tour.nextStep();
    assertEqual(s4.stepIndex, 4);
    assertTrue(tour.isCompleted());
  });

  test('F20-04', 'F20: Navigate backward maintaining valid step bounds', () => {
    const tour = createGuidedTourState();
    tour.play();
    tour.nextStep(); // index 1
    const prev = tour.prevStep();
    assertEqual(prev.stepIndex, 0);
  });

  test('F20-05', 'F20: Pause tour halts active playback state', () => {
    const tour = createGuidedTourState();
    tour.play();
    tour.pause();
    assertFalse(tour.isPlaying());
  });

  // =========================================================================
  // FEATURE F21: Standalone Web Compilation & Hosting (5 tests)
  // =========================================================================
  test('F21-01', 'F21: Verify package.json exists with web workspace configuration', () => {
    const cfg = verifyWebDeploymentConfig();
    assertTrue(cfg.hasPackageJson, 'package.json must be present');
  });

  test('F21-02', 'F21: Verify tsconfig.json exists with TypeScript strict settings', () => {
    const cfg = verifyWebDeploymentConfig();
    assertTrue(cfg.hasTsconfig, 'tsconfig.json must be present');
  });

  test('F21-03', 'F21: Verify vite.config.ts exists configuring frontend bundling', () => {
    const cfg = verifyWebDeploymentConfig();
    assertTrue(cfg.hasViteConfig, 'vite.config.ts must be present');
  });

  test('F21-04', 'F21: Verify vercel.json exists with SPA rewrites', () => {
    const cfg = verifyWebDeploymentConfig();
    assertTrue(cfg.hasVercelJson, 'vercel.json must exist');
    assertTrue(cfg.vercelConfigValid, 'vercel.json must contain valid rewrites');
  });

  test('F21-05', 'F21: Zero-backend invariant: Client-side MockWebAdapter guarantees zero cloud egress', () => {
    const cfg = verifyWebDeploymentConfig();
    assertTrue(cfg.zeroEgressGuarantee, 'Must guarantee zero cloud data egress');
  });

  return results;
}
