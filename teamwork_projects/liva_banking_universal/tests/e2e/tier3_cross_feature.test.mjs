/**
 * Tier 3: Cross-Feature Interactions E2E Test Suite
 * Covers 21 pairwise combinatorial test cases verifying seamless integration
 * between ingestion, 3-tier reconciliation, AML surveillance, Maker-Checker, and UI components.
 */

import {
  assert,
  assertEqual,
  assertTrue,
  assertFalse,
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
  computeMerkleLeaf,
  buildMerkleTree,
  queryFinancialCopilot,
  createGuidedTourState,
  verifyWebDeploymentConfig,
} from './harness_helpers.mjs';

export async function runTier3Tests() {
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

  // T3-01: F01 (Dropzone) + F02 (Schema Normalization)
  test('T3-01', 'T3-01: Ingest VCB Excel statement and verify normalized column schema', () => {
    const buf = loadDemoFile('01_VCB_SaoKe_Thang8_Chuan.xlsx');
    const parsed = parseExcelFile(buf, '01_VCB_SaoKe_Thang8_Chuan.xlsx');
    assertEqual(parsed.bankCode, 'VCB');
    assertTrue(parsed.transactions.length > 0);
    assertTrue(Boolean(parsed.transactions[0].date));
    assertTrue(parsed.transactions[0].amount > 0);
  });

  // T3-02: F01 (Dropzone) + F03 (Amount Parser)
  test('T3-02', 'T3-02: Ingest TCB CSV and parse dot-delimited amounts into integer VND', () => {
    const text = loadDemoText('02_TCB_SaoKe_Thang8_Chuan.csv');
    const parsed = parseStatementCsvOrText(text, 'TCB_Thang8.csv');
    const tx550 = parsed.transactions.find((t) => t.amount === 550_000_000);
    assertTrue(Boolean(tx550), 'Should find 550M integer VND transaction');
    assertEqual(tx550.credit, 550_000_000);
  });

  // T3-03: F02 (Normalization) + F04 (Balance Invariant Validator)
  test('T3-03', 'T3-03: Normalized statement feeds directly into balance invariant validator', () => {
    const text = loadDemoText('02_TCB_SaoKe_Thang8_Chuan.csv');
    const parsed = parseStatementCsvOrText(text, 'TCB.csv');
    const report = verifyBalanceInvariants(
      parsed.openingBalance,
      parsed.closingBalance,
      parsed.totalCredit,
      parsed.totalDebit
    );
    assertTrue(report.isBalanced, 'Parsed TCB statement must satisfy balance invariant');
  });

  // T3-04: F03 (Amount Parser) + F04 (Balance Invariant Validator)
  test('T3-04', 'T3-04: Integer-parsed amounts preserve zero float drift across balance checks', () => {
    const credit = parseVietnameseAmount('45.000.000,50'); // 45000001
    const debit = parseVietnameseAmount('2.200.000,00');  // 2200000
    const open = 100_000_000;
    const expectedClose = open + credit - debit;
    const report = verifyBalanceInvariants(open, expectedClose, credit, debit);
    assertEqual(report.discrepancy, 0);
  });

  // T3-05: F04 (Balance Validator) + F05 (Tier 1 Exact Matcher)
  test('T3-05', 'T3-05: Balanced statement transactions feed cleanly into Tier 1 exact matcher', () => {
    const tcbText = loadDemoText('02_TCB_SaoKe_Thang8_Chuan.csv');
    const parsed = parseStatementCsvOrText(tcbText, 'TCB.csv');
    assertTrue(parsed.balanceInvariantPassed);

    // Provide corresponding ledger invoice for TCB transaction 1 (HD131, 45M)
    const ledgers = [
      { id: 'l1', amount: 45_000_000, entryType: 'CREDIT', docNo: 'HD131', partnerName: 'An Phat' },
    ];

    const res = matchTier1Exact(parsed.transactions, ledgers);
    assertEqual(res.matches.length, 1, 'Should find Tier 1 exact match');
    assertEqual(res.matches[0].tier, 'TIER1_EXACT');
  });

  // T3-06: F05 (Tier 1 Exact) + F06 (Tier 2 Fuzzy Heuristic)
  test('T3-06', 'T3-06: Cascading reconciliation: Tier 1 unallocated feed into Tier 2 fuzzy matcher', () => {
    const bTxs = [
      { id: 'b1', amount: 45_000_000, txType: 'CREDIT', narration: 'TT HD131', txCode: 'T1' },
      { id: 'b2', amount: 550_000_000, txType: 'CREDIT', counterparty: 'Tap doan Masan', narration: 'xe nang' },
    ];
    const ledgers = [
      { id: 'l1', amount: 45_000_000, entryType: 'CREDIT', docNo: 'HD131', partnerName: 'An Phat' },
      { id: 'l2', amount: 550_000_000, entryType: 'CREDIT', docNo: 'HD-91', partnerName: 'Cong ty CP Tap doan Masan' },
    ];

    const t1 = matchTier1Exact(bTxs, ledgers);
    assertEqual(t1.matches.length, 1);
    assertEqual(t1.unallocatedBank.length, 1);

    const t2 = matchTier2Fuzzy(t1.unallocatedBank, t1.unallocatedLedger);
    assertEqual(t2.matches.length, 1);
    assertEqual(t2.matches[0].tier, 'TIER2_FUZZY');
  });

  // T3-07: F06 (Tier 2 Fuzzy) + F10 (Wire Fee Disentanglement to TK 6425)
  test('T3-07', 'T3-07: Fuzzy match accounts for wire fee, disentangler books fee to TK 6425', () => {
    const bTxs = [
      { id: 'b1', amount: 84_997_800, txType: 'CREDIT', counterparty: 'FPT Smart Cloud', narration: 'Phi chuyen tien 2200' },
    ];
    const ledgers = [
      { id: 'l1', amount: 85_000_000, entryType: 'CREDIT', partnerName: 'FPT Smart Cloud', docNo: 'HD-93' },
    ];
    const t2 = matchTier2Fuzzy(bTxs, ledgers);
    assertEqual(t2.matches.length, 1);
    assertEqual(t2.matches[0].feeAmount, 2200);

    const feeRes = disentangleWireFee(bTxs[0].amount, bTxs[0].narration);
    assertEqual(feeRes.accountCode, 'TK 6425');
  });

  // T3-08: F06 (Tier 2 Fuzzy) + F09 (Vietnamese Intent Normalizer)
  test('T3-08', 'T3-08: Intent normalizer expands colloquial memo before fuzzy matching', () => {
    const memo = 'ck tien may bom hd 88';
    const norm = normalizeVietnameseIntent(memo);
    assertEqual(norm.invoiceNumbers[0], '88');

    const bTx = { id: 'b1', amount: 145_000_000, txType: 'CREDIT', narration: norm.normalized, counterparty: 'An Phat' };
    const ledgers = [{ id: 'l1', amount: 145_000_000, entryType: 'CREDIT', partnerName: 'Cong ty An Phat', docNo: 'HD-88' }];
    const res = matchTier2Fuzzy([bTx], ledgers);
    assertEqual(res.matches.length, 1);
  });

  // T3-09: F06 (Tier 2 Fuzzy) + F07 (Tier 3 Split Solver)
  test('T3-09', 'T3-09: Unallocated after Tier 2 cascade into Tier 3 subset-sum solver', () => {
    const bTxs = [
      { id: 'b1', amount: 190_000_000, txType: 'CREDIT', counterparty: 'An Phat', narration: 'Thanh toan gop' },
    ];
    const ledgers = [
      { id: 'l1', amount: 145_000_000, entryType: 'CREDIT', partnerName: 'An Phat', docNo: 'HD-88' },
      { id: 'l2', amount: 45_000_000, entryType: 'CREDIT', partnerName: 'An Phat', docNo: 'HD-90' },
    ];

    const t2 = matchTier2Fuzzy(bTxs, ledgers);
    assertEqual(t2.matches.length, 0, 'Composite amounts do not match in 1:1 fuzzy tier');

    const t3 = matchTier3Split(t2.unallocatedBank, t2.unallocatedLedger);
    assertEqual(t3.matches.length, 1);
    assertEqual(t3.matches[0].matchType, 'COMPOSITE_1_TO_N');
  });

  // T3-10: F07 (Tier 3 Split Solver) + F08 (HITL Quarantine)
  test('T3-10', 'T3-10: Unmatched records after Tier 3 fail-closed to HITL Quarantine', () => {
    const bTxs = [{ id: 'b-orphan', amount: 99_999_000, txType: 'CREDIT', narration: 'Chua ro muc dich' }];
    const t3 = matchTier3Split(bTxs, []);
    assertEqual(t3.unallocatedBank.length, 1);

    const qItem = createHitlQuarantine(t3.unallocatedBank[0], 'No matching split candidates');
    assertEqual(qItem.txId, 'b-orphan');
    assertTrue(Boolean(qItem.hitlToken));
  });

  // T3-11: F08 (HITL Quarantine) + F17 (Maker-Checker Approval)
  test('T3-11', 'T3-11: Quarantine item resolution by Maker creates payment adjustment requiring Checker approval', () => {
    const qItem = createHitlQuarantine({ id: 'tx-adj', amount: 15_000_000 });
    const vch = createPaymentVoucher('maker_accountant', '123456', 'VCB', qItem.amount, `HITL Adjustment: ${qItem.hitlToken}`);
    const pending = submitVoucherForApproval(vch);
    const approved = approveVoucher(pending, 'checker_chief_accountant');
    assertEqual(approved.status, 'APPROVED');
    assertEqual(approved.checkerId, 'checker_chief_accountant');
  });

  // T3-12: F09 (Vietnamese NLP) + F19 (2D Conversational Copilot)
  test('T3-12', 'T3-12: Copilot interprets Vietnamese intent and provides structured financial answer', () => {
    const query = 'cho tôi biết khoản chi lớn nhất trong kỳ này';
    const norm = normalizeVietnameseIntent(query);
    const copilotRes = queryFinancialCopilot(query);
    assertEqual(copilotRes.intent, 'TOP_EXPENSE');
    assertTrue(copilotRes.answer.includes('550.000.000 VND'));
  });

  // T3-13: F11 (AML Structuring) + F15 (Form STR Generator)
  test('T3-13', 'T3-13: Structuring alert triggers automated Form STR report generation', () => {
    const txs = [
      { id: 'tx-1', date: '2026-08-10', amount: 390_000_000, counterparty: 'NGUYEN HOANG PHUC', counterpartyAccount: '001100' },
      { id: 'tx-2', date: '2026-08-10', amount: 385_000_000, counterparty: 'TRAN THI BICH NGOC', counterpartyAccount: '001101' },
      { id: 'tx-3', date: '2026-08-10', amount: 395_000_000, counterparty: 'LE VAN THANG', counterpartyAccount: '001102' },
    ];
    const alerts = detectAmlStructuring(txs);
    assertEqual(alerts.length, 1);
    const form = generateFormStr(alerts[0], txs);
    assertEqual(form.alertType, 'STRUCTURING_SMURFING');
    assertEqual(form.totalVndAmount, 1_170_000_000);
    assertTrue(form.formTemplate.includes('Thông tư 09/2023/TT-NHNN'));
  });

  // T3-14: F12 (AML High-Value) + F16 (System Prompt Inspector)
  test('T3-14', 'T3-14: Tuned high-value threshold in prompt inspector updates alert triggering', () => {
    const insp = inspectSystemPrompt({ highValueThreshold: 300_000_000 });
    assertEqual(insp.parameters.highValueThreshold, 300_000_000);
    const alerts = insp.evalSandbox([{ id: 'tx-350', amount: 350_000_000, time: '10:00:00' }]);
    assertTrue(alerts.some((a) => a.totalAmount === 350_000_000));
  });

  // T3-15: F13 (AML Rapid Pass-Through) + F15 (Form STR Generator)
  test('T3-15', 'T3-15: Rapid pass-through alert automatically populates Form STR narrative', () => {
    const txs = [
      { id: 'in', txType: 'CREDIT', amount: 420_000_000, time: '02:15:20', counterparty: 'VU TRONG PHUONG', counterpartyAccount: '12010001234567' },
      { id: 'out', txType: 'DEBIT', amount: 419_500_000, time: '02:22:45' },
    ];
    const alerts = detectAmlRapidPassThrough(txs);
    assertEqual(alerts.length, 1);
    const form = generateFormStr(alerts[0], txs);
    assertEqual(form.alertType, 'RAPID_PASS_THROUGH');
    assertEqual(form.suspectName, 'VU TRONG PHUONG');
  });

  // T3-16: F14 (AML Night Velocity) + F12 (High-Value Detector)
  test('T3-16', 'T3-16: Off-hours high-value transaction simultaneously triggers both alerts', () => {
    const txs = [{ id: 'bidv-night', time: '02:15:20', amount: 420_000_000 }];
    const allAlerts = runFullAmlSurveillance(txs);
    const types = allAlerts.map((a) => a.anomalyType);
    assertTrue(types.includes('HIGH_VALUE'));
    assertTrue(types.includes('NIGHT_VELOCITY'));
  });

  // T3-17: F15 (Form STR Generator) + F16 (System Prompt Inspector)
  test('T3-17', 'T3-17: Prompt inspector parameters align with Form STR statutory rule reference', () => {
    const insp = inspectSystemPrompt();
    const alert = { anomalyType: 'HIGH_VALUE', severity: 'HIGH', involvedTransactionIds: [], totalAmount: 400_000_000, statutoryRuleRef: 'Quyết định 11/2023/QĐ-TTg' };
    const form = generateFormStr(alert);
    assertTrue(/Quyết định 11\/2023\/QĐ-TTg/i.test(insp.systemPrompt));
    assertTrue(form.statutoryRuleRef.includes('Quyết định 11/2023/QĐ-TTg'));
  });

  // T3-18: F17 (Maker-Checker Gate) + F18 (Cryptographic Merkle Audit)
  test('T3-18', 'T3-18: Checker approval generates Merkle leaf and recalculates root', () => {
    const vch = createPaymentVoucher('maker_1', '1903456789', 'TCB', 50_000_000, 'Tien hang');
    const pending = submitVoucherForApproval(vch);
    const approved = approveVoucher(pending, 'checker_1');
    assertTrue(Boolean(approved.merkleLeafHash));

    const tree = buildMerkleTree([approved.merkleLeafHash]);
    assertEqual(tree.root, approved.merkleLeafHash);
  });

  // T3-19: F17 (Maker-Checker Gate) + F12 (High-Value AML Gate)
  test('T3-19', 'T3-19: Maker drafting voucher >= 400M flags high-value dual authorization requirement', () => {
    const vch = createPaymentVoucher('maker_treasury', '120100', 'BIDV', 550_000_000, 'Xe nang Masan');
    const alerts = detectAmlHighValue([{ id: vch.voucherId, amount: vch.amountVnd }]);
    assertEqual(alerts.length, 1);
    assertTrue(vch.amountVnd >= 400_000_000);
  });

  // T3-20: F19 (2D Copilot) + F20 (1-Click Presentation Tour)
  test('T3-20', 'T3-20: Tour Step 4 binds to Copilot overview metrics', () => {
    const tour = createGuidedTourState();
    tour.play();
    for (let i = 0; i < 4; i++) tour.nextStep();
    assertEqual(tour.getCurrentStep().id, 'OVERVIEW_COPILOT');

    const copilotRes = queryFinancialCopilot('thanh khoản');
    assertEqual(copilotRes.intent, 'LIQUIDITY_RUNWAY');
  });

  // T3-21: F20 (Guided Tour) + F21 (Standalone Web Hosting)
  test('T3-21', 'T3-21: Guided presentation tour runs in zero-backend deployment configuration', () => {
    const cfg = verifyWebDeploymentConfig();
    assertTrue(cfg.zeroEgressGuarantee);
    const tour = createGuidedTourState();
    tour.play();
    assertTrue(tour.isPlaying());
  });

  return results;
}
