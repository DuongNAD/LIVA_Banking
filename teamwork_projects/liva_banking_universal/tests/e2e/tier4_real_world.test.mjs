/**
 * Tier 4: Real-World Multi-Bank Application Scenarios E2E Test Suite
 * Covers 5 end-to-end multi-bank application workflows utilizing authoritative
 * datasets from data/demo_ready/ (VCB, TCB, BIDV, Ledger, Google Sheets).
 */

import {
  assert,
  assertEqual,
  assertTrue,
  assertFalse,
  assertThrows,
  loadDemoFile,
  loadDemoText,
  parseStatementCsvOrText,
  parseExcelFile,
  parseLedgerCsv,
  verifyBalanceInvariants,
  matchTier1Exact,
  matchTier2Fuzzy,
  matchTier3Split,
  normalizeVietnameseIntent,
  disentangleWireFee,
  detectAmlHighValue,
  detectAmlStructuring,
  detectAmlNightVelocity,
  detectAmlRapidPassThrough,
  runFullAmlSurveillance,
  generateFormStr,
  createPaymentVoucher,
  submitVoucherForApproval,
  approveVoucher,
  computeMerkleLeaf,
  buildMerkleTree,
  queryFinancialCopilot,
  createGuidedTourState,
  verifyWebDeploymentConfig,
} from './harness_helpers.mjs';

export async function runTier4Tests() {
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
  // SCENARIO S01: Real Multi-Bank Month-End Reconciliation Cycle
  // =========================================================================
  test('S01', 'S01: Full multi-bank month-end reconciliation cycle (VCB, TCB, BIDV vs Ledger)', () => {
    // 1. Ingest VCB Excel
    const vcbBuf = loadDemoFile('01_VCB_SaoKe_Thang8_Chuan.xlsx');
    const vcbParsed = parseExcelFile(vcbBuf, '01_VCB_SaoKe_Thang8_Chuan.xlsx');
    assertTrue(vcbParsed.transactions.length > 0, 'VCB must parse transactions');

    // 2. Ingest TCB CSV
    const tcbText = loadDemoText('02_TCB_SaoKe_Thang8_Chuan.csv');
    const tcbParsed = parseStatementCsvOrText(tcbText, '02_TCB_SaoKe_Thang8_Chuan.csv');
    assertTrue(tcbParsed.balanceInvariantPassed, 'TCB must pass balance invariant');

    // 3. Ingest BIDV CSV
    const bidvText = loadDemoText('03_BIDV_SaoKe_NgoaiGio_BatThuong.csv');
    const bidvParsed = parseStatementCsvOrText(bidvText, '03_BIDV_SaoKe_NgoaiGio_BatThuong.csv');
    assertTrue(bidvParsed.transactions.length >= 4, 'BIDV must have at least 4 transactions');

    // 4. Ingest General Ledger CSV
    const ledgerText = loadDemoText('04_SoCai_KeToan_HoaDon.csv');
    const ledgers = parseLedgerCsv(ledgerText);
    assertTrue(ledgers.length >= 6, 'Ledger must have invoices');

    // 5. Aggregate all bank transactions
    const allBankTxs = [...vcbParsed.transactions, ...tcbParsed.transactions, ...bidvParsed.transactions];

    // 6. Execute 3-Tier Reconciliation Cascade
    // Tier 1 Exact
    const t1Res = matchTier1Exact(allBankTxs, ledgers);
    // Tier 2 Fuzzy Heuristic
    const t2Res = matchTier2Fuzzy(t1Res.unallocatedBank, t1Res.unallocatedLedger);
    // Tier 3 Split Solver
    const t3Res = matchTier3Split(t2Res.unallocatedBank, t2Res.unallocatedLedger);

    const totalMatches = t1Res.matches.length + t2Res.matches.length + t3Res.matches.length;
    assertTrue(totalMatches >= 3, 'Must match real transactions across 3 tiers');

    // Verify match tiers are appropriately tagged
    assertTrue(t2Res.matches.some((m) => m.tier === 'TIER2_FUZZY'), 'Must have Tier 2 matches');
  });

  // =========================================================================
  // SCENARIO S02: Real AML Fraud Surveillance & Statutory STR Filing Lifecycle
  // =========================================================================
  test('S02', 'S02: End-to-end AML surveillance detecting 4 anomaly types and generating statutory Form STR', () => {
    // 1. Ingest BIDV abnormal statement + Google Sheets pasted transactions
    const bidvText = loadDemoText('03_BIDV_SaoKe_NgoaiGio_BatThuong.csv');
    const bidvParsed = parseStatementCsvOrText(bidvText, 'bidv.csv');

    const sheetsText = loadDemoText('05_DuLieu_Dan_GoogleSheets.txt');
    const sheetsParsed = parseStatementCsvOrText(sheetsText, 'sheets.txt');

    const combinedTxs = [...bidvParsed.transactions, ...sheetsParsed.transactions];

    // 2. Run AML detectors
    const highValAlerts = detectAmlHighValue(combinedTxs);
    const structuringAlerts = detectAmlStructuring(combinedTxs);
    const nightAlerts = detectAmlNightVelocity(combinedTxs);
    const passThroughAlerts = detectAmlRapidPassThrough(combinedTxs);

    // Verify all 4 anomaly types detected
    assertTrue(highValAlerts.length >= 1, 'Must detect high value transaction (420M BIDV)');
    assertTrue(structuringAlerts.length >= 1, 'Must detect structuring smurfing (3x sub-400M on 10/08)');
    assertTrue(nightAlerts.length >= 1, 'Must detect night velocity (02:15:20)');
    assertTrue(passThroughAlerts.length >= 1, 'Must detect rapid pass-through churn (420M in, 419.5M out in 7m)');

    // 3. Generate Form STR for smurfing
    const strStructuring = generateFormStr(structuringAlerts[0], combinedTxs, 'Phát hiện 3 giao dịch chia nhỏ ngày 10/08');
    assertEqual(strStructuring.alertType, 'STRUCTURING_SMURFING');
    assertEqual(strStructuring.reportingEntity, 'LIVA SOLUTIONS CO., LTD');
    assertTrue(strStructuring.formTemplate.includes('Thông tư 09/2023/TT-NHNN'));

    // 4. Generate Form STR for rapid pass-through
    const strPassThrough = generateFormStr(passThroughAlerts[0], combinedTxs, 'Chuyển tiền ra ví điện tử trong 7 phút');
    assertEqual(strPassThrough.alertType, 'RAPID_PASS_THROUGH');
  });

  // =========================================================================
  // SCENARIO S03: High-Value Payment Maker-Checker & Merkle Chain Lifecycle
  // =========================================================================
  test('S03', 'S03: Circular 09/2020 Maker-Checker dual control pipeline with cryptographic Merkle proof', () => {
    // 1. Maker drafts 550M payment voucher for Masan equipment
    const voucher = createPaymentVoucher(
      'maker_accountant_01',
      '19034567890123',
      'TCB',
      550_000_000,
      'Cung ung lo xe nang chuyen dung Masan HD-91'
    );
    assertEqual(voucher.status, 'DRAFT');

    // 2. System flags AML Decision 11/2023 High-Value condition
    const amlAlerts = detectAmlHighValue([{ id: voucher.voucherId, amount: voucher.amountVnd }]);
    assertEqual(amlAlerts.length, 1);
    assertTrue(voucher.amountVnd >= 400_000_000);

    // 3. Maker submits voucher
    const pending = submitVoucherForApproval(voucher);
    assertEqual(pending.status, 'PENDING_APPROVAL');

    // 4. Maker self-approval strictly blocked (Fail-Closed)
    assertThrows(() => {
      approveVoucher(pending, 'maker_accountant_01');
    }, 'Maker cannot be Checker');

    // 5. Checker approves voucher with Biometric confirmation
    const approved = approveVoucher(pending, 'checker_cfo_01', 'BIOMETRIC_SIM');
    assertEqual(approved.status, 'APPROVED');
    assertEqual(approved.authMethod, 'BIOMETRIC_SIM');
    assertTrue(Boolean(approved.merkleLeafHash));

    // 6. Incorporate into Merkle Tree Audit Root
    const tree = buildMerkleTree([approved.merkleLeafHash]);
    assertEqual(tree.root, approved.merkleLeafHash);

    // 7. Verify tamper resistance
    const tamperedLeaf = computeMerkleLeaf({ ...approved, amountVnd: 550_000_001 });
    const tamperedTree = buildMerkleTree([tamperedLeaf]);
    assertTrue(tree.root !== tamperedTree.root, 'Merkle root must diverge upon data tampering');
  });

  // =========================================================================
  // SCENARIO S04: Unstructured Memo Ingestion & Wire Fee Disentanglement Pipeline
  // =========================================================================
  test('S04', 'S04: Unstructured memo abbreviation expansion and zero-hallucination wire fee booking (TK 6425)', () => {
    // 1. Ingest Google Sheets pasted text
    const sheetsText = loadDemoText('05_DuLieu_Dan_GoogleSheets.txt');
    const parsed = parseStatementCsvOrText(sheetsText, 'clipboard.txt');

    // 2. Scan memos and expand colloquial Vietnamese abbreviations
    for (const tx of parsed.transactions) {
      const intentResult = normalizeVietnameseIntent(tx.narration);
      assertTrue(typeof intentResult.normalized === 'string');
      assertTrue(Array.isArray(intentResult.invoiceNumbers));
    }

    // 3. Disentangle wire fee from sample transaction
    const grossAmount = 85_000_000;
    const memo = 'Thanh toan dot 2 hop dong FPT phi 2200';
    const feeRes = disentangleWireFee(grossAmount, memo);

    assertEqual(feeRes.feeAmount, 2200);
    assertEqual(feeRes.principal, 84_997_800);
    assertEqual(feeRes.accountCode, 'TK 6425');
    assertEqual(feeRes.accountName, 'Chi phí dịch vụ ngân hàng');
    assertTrue(feeRes.sumCheckPassed);
    assertEqual(feeRes.principal + feeRes.feeAmount, grossAmount);
  });

  // =========================================================================
  // SCENARIO S05: 5-Minute Guided Presentation Tour & Conversational Copilot
  // =========================================================================
  test('S05', 'S05: Full 5-minute automated walkthrough sequence and conversational 2D Financial Copilot', () => {
    // 1. Initialize guided presentation tour
    const tour = createGuidedTourState();
    assertEqual(tour.steps.length, 5);

    // 2. Step 0: Ingestion
    tour.play();
    assertEqual(tour.getCurrentStep().id, 'INGESTION');

    // 3. Step 1: Reconciliation
    const s1 = tour.nextStep();
    assertEqual(s1.id, 'RECONCILIATION');

    // 4. Step 2: AML Surveillance
    const s2 = tour.nextStep();
    assertEqual(s2.id, 'AML_SURVEILLANCE');

    // 5. Step 3: Maker-Checker Dual Control
    const s3 = tour.nextStep();
    assertEqual(s3.id, 'MAKER_CHECKER');

    // 6. Step 4: Overview & Copilot Drawer
    const s4 = tour.nextStep();
    assertEqual(s4.id, 'OVERVIEW_COPILOT');
    assertTrue(tour.isCompleted());

    // 7. Query Copilot on liquidity runway
    const runwayRes = queryFinancialCopilot('thanh khoản hiện tại', { closingBalance: 1_380_600_000 });
    assertEqual(runwayRes.intent, 'LIQUIDITY_RUNWAY');
    assertTrue(runwayRes.answer.includes('1.380.600.000 VND'));

    // 8. Query Copilot on reconciliation rate
    const reconRes = queryFinancialCopilot('tỷ lệ đối soát tháng 8');
    assertEqual(reconRes.intent, 'RECONCILIATION_RATE');
    assertTrue(reconRes.answer.includes('99.8%'));

    // 9. Query Copilot on highest expense
    const expenseRes = queryFinancialCopilot('khoản chi phí lớn nhất');
    assertEqual(expenseRes.intent, 'TOP_EXPENSE');
    assertTrue(expenseRes.answer.includes('550.000.000 VND'));

    // 10. Verify zero-backend deployment configuration
    const deployment = verifyWebDeploymentConfig();
    assertTrue(deployment.hasPackageJson);
    assertTrue(deployment.zeroEgressGuarantee);
  });

  return results;
}
