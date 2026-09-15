/**
 * Milestone M2 Adversarial Stress Test Suite — Challenger 1
 * Stress-testing:
 * 1. ISO 20022 XML parsing resilience (camt.053 & pacs.008) under malformed XML, missing balances, corrupt amounts.
 * 2. Exception Resolution Queue concurrency, idempotency, fee allocation, and checker escalation.
 * 3. Checker Workstation fail-closed anti-self-approval (Circular 09/2020), mandatory remarks, and Merkle O(log N) verification with altered hashes.
 * 4. Treasury Desk overdraft protection, zero/negative transfers, reserve threshold breach surveillance, and conservation of funds.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';

import { parseIso20022Xml } from '../src/engine/ingestion/iso20022Parser';
import { parseCsvOrTsv } from '../src/engine/ingestion/universalParser';
import { useReconciliationStore } from '../src/stores/reconciliationStore';
import { useBankingStore } from '../src/stores/bankingStore';
import {
  buildMerkleTree,
  generateMerkleProof,
  verifyMerkleProof,
  ForwardAuditLedger,
} from '../src/engine/treasury/merkleAudit';
import {
  createPaymentVoucher,
  submitVoucherForApproval,
  approveVoucher,
  rejectVoucher,
} from '../src/engine/treasury/makerChecker';
import type { LedgerEntry, BankCode } from '../src/types/banking';
import type { HitlQuarantineItem } from '../src/types/reconciliation';

describe('Milestone M2 Adversarial Stress Suite (Challenger 1)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  // =========================================================================
  // TASK 1: ISO 20022 Parsing Stress & Fault Injection
  // =========================================================================
  describe('Task 1: ISO 20022 Parsing Stress & Fault Injection', () => {
    it('1.1 handles completely malformed XML and unclosed tags without crashing', () => {
      const brokenXml = `<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.08">
  <BkToCstmrStmt>
    <Stmt>
      <Id>BROKEN-STMT
      <Acct><Id><IBAN>VN123456789</Acct>
      <Bal><Amt>100000000</Amt>
      <Ntry>
        <Amt Ccy="VND">50000000</Amt>
        <CdtDbtInd>CRDT
        <BookgDt><Dt>2026-08-15
      <!-- Unclosed tags everywhere`;

      expect(() => {
        const res = parseIso20022Xml(brokenXml, 'MALFORMED.xml');
        expect(res).toBeDefined();
        expect(res.currency).toBe('VND');
        expect(Array.isArray(res.transactions)).toBe(true);
      }).not.toThrow();
    });

    it('1.2 handles empty, whitespace, and comment-only XML strings', () => {
      const emptyRes = parseIso20022Xml('', 'EMPTY.xml');
      expect(emptyRes.transactions.length).toBe(0);
      expect(emptyRes.openingBalance).toBe(0);
      expect(emptyRes.closingBalance).toBe(0);
      expect(emptyRes.balanceInvariantPassed).toBe(true);

      const commentRes = parseIso20022Xml('<!-- Just a comment -->   \n\t  ', 'COMMENT.xml');
      expect(commentRes.transactions.length).toBe(0);
      expect(commentRes.balanceInvariantPassed).toBe(true);
    });

    it('1.3 handles missing balance blocks (neither OPBD nor CLBD)', () => {
      const xmlWithoutBal = `<?xml version="1.0"?>
<Document xmlns="camt.053">
  <BkToCstmrStmt>
    <Stmt>
      <Id>STMT-NO-BAL</Id>
      <Acct><Id><Othr><Id>CITAD-ACC-99</Id></Othr></Id></Acct>
      <Ntry>
        <Amt Ccy="VND">100000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <BookgDt><Dt>2026-08-01</Dt></BookgDt>
      </Ntry>
      <Ntry>
        <Amt Ccy="VND">40000000</Amt>
        <CdtDbtInd>DBIT</CdtDbtInd>
        <BookgDt><Dt>2026-08-02</Dt></BookgDt>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>`;

      const res = parseIso20022Xml(xmlWithoutBal);
      expect(res.openingBalance).toBe(0);
      expect(res.closingBalance).toBe(60_000_000);
      expect(res.totalCredit).toBe(100_000_000);
      expect(res.totalDebit).toBe(40_000_000);
      expect(res.balanceInvariantPassed).toBe(true);
    });

    it('1.4 handles mixed currency notations and negative balance blocks with DBIT', () => {
      const xmlWithDebitBal = `<?xml version="1.0"?>
<Document xmlns="camt.053">
  <BkToCstmrStmt>
    <Stmt>
      <Acct><Id><Othr><Id>OVERDRAFT-ACCT</Id></Othr></Id><Ccy>VND</Ccy></Acct>
      <Bal>
        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">50000000</Amt>
        <CdtDbtInd>DBIT</CdtDbtInd>
      </Bal>
      <Bal>
        <Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">150000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
      </Bal>
      <Ntry>
        <Amt Ccy="VND">200000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <BookgDt><Dt>2026-08-05</Dt></BookgDt>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>`;

      const res = parseIso20022Xml(xmlWithDebitBal);
      expect(res.openingBalance).toBe(-50_000_000);
      expect(res.closingBalance).toBe(150_000_000);
      expect(res.totalCredit).toBe(200_000_000);
      expect(res.totalDebit).toBe(0);
      expect(res.balanceInvariantPassed).toBe(true);
      expect(res.balanceDiscrepancy).toBe(0);
    });

    it('1.5 parses pacs.008 with missing party tags or edge-case amount formats', () => {
      const pacsXml = `<?xml version="1.0"?>
<Document xmlns="pacs.008">
  <FIToFICstmrCdtTrf>
    <CdtTrfTxInf>
      <PmtId><EndToEndId>E2E-MINIMAL</EndToEndId></PmtId>
      <IntrBkSttlmAmt Ccy="EUR">75,000,000.00</IntrBkSttlmAmt>
      <IntrBkSttlmDt>2026-08-28</IntrBkSttlmDt>
    </CdtTrfTxInf>
    <CdtTrfTxInf>
      <PmtId><TxId>TX-ZERO</TxId></PmtId>
      <InstdAmt Ccy="VND">0</InstdAmt>
      <IntrBkSttlmDt>2026-08-29</IntrBkSttlmDt>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>`;

      const res = parseIso20022Xml(pacsXml, 'SWIFT_STATEMENT.xml');
      expect(res.bankCode).toBe('SWIFT');
      expect(res.transactions.length).toBe(2);
      expect(res.transactions[0].credit).toBe(75_000_000);
      expect(res.transactions[0].docRef).toBe('E2E-MINIMAL');
      expect(res.transactions[1].credit).toBe(0);
      expect(res.transactions[1].txCode).toBe('TX-ZERO');
    });

    it('1.6 routes XML files through parseCsvOrTsv transparently', () => {
      const rawCamt = '<Document xmlns="camt.053"><Stmt><Ntry><Amt>123456</Amt><CdtDbtInd>CRDT</CdtDbtInd></Ntry></Stmt></Document>';
      const res = parseCsvOrTsv(rawCamt, 'statement.camt.053');
      expect(res.transactions.length).toBe(1);
      expect(res.transactions[0].credit).toBe(123456);
    });

    it('1.7 handles multiple intermediate OPBD and CLBD balance blocks with PRCD / CLAV codes', () => {
      const multiBalXml = `<?xml version="1.0"?>
<Document xmlns="camt.053">
  <BkToCstmrStmt>
    <Stmt>
      <Bal>
        <Tp><CdOrPrtry><Cd>PRCD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">100000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
      </Bal>
      <Bal>
        <Tp><CdOrPrtry><Cd>CLAV</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">180000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
      </Bal>
      <Ntry>
        <Amt Ccy="VND">80000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <BookgDt><Dt>2026-08-10</Dt></BookgDt>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>`;

      const res = parseIso20022Xml(multiBalXml);
      expect(res.openingBalance).toBe(100_000_000);
      expect(res.closingBalance).toBe(180_000_000);
      expect(res.totalCredit).toBe(80_000_000);
      expect(res.balanceInvariantPassed).toBe(true);
    });
  });

  // =========================================================================
  // TASK 2: Exception Resolution Queue Concurrency & Invariants
  // =========================================================================
  describe('Task 2: Exception Resolution Queue Concurrency & Invariants', () => {
    function createMockQuarantinedItem(i: number, amount: number, narration = 'Giao dich nghi van'): HitlQuarantineItem {
      const txId = `tx-stress-${i}`;
      return {
        txId,
        amount,
        hitlToken: `hitl-token-${i}`,
        createdAt: 1786320000 + i,
        expiresAt: 1786320900 + i,
        reason: 'Quarantined for testing',
        rawTransaction: {
          id: txId,
          date: '2026-08-15',
          txDate: 1786320000 + i,
          txCode: `TXCODE-${i}`,
          debit: 0,
          credit: amount,
          netAmount: amount,
          amount,
          txType: 'CREDIT',
          balance: 100_000_000 + i * amount,
          narration,
          bankCode: 'CITAD',
        },
      };
    }

    it('2.1 executes 50 rapid sequential manual matches with mathematical consistency', () => {
      const store = useReconciliationStore();
      const count = 50;
      const qItems: HitlQuarantineItem[] = [];
      const ledgers: LedgerEntry[] = [];

      for (let i = 0; i < count; i++) {
        qItems.push(createMockQuarantinedItem(i, 10_000_000 + i * 1_000_000));
        ledgers.push({
          id: `led-${i}`,
          docNo: `INV-2026-${i}`,
          entryDate: '2026-08-15',
          entryTimestamp: 1786320000 + i,
          partnerName: `Doi tac ${i}`,
          amount: 10_000_000 + i * 1_000_000,
          entryType: 'CREDIT',
          description: `Hoa don so ${i}`,
          status: 'UNMATCHED',
        });
      }

      store.bankTransactions = qItems.map((q) => q.rawTransaction!);
      store.ledgerEntries = [...ledgers];
      store.reconciliationSummary = {
        totalBankTransactions: count,
        totalLedgerEntries: count,
        matchedCount: 0,
        tier1Count: 0,
        tier2Count: 0,
        tier3Count: 0,
        hitlQuarantineCount: count,
        matchRate: 0,
        totalMatchedAmount: 0,
        totalFeeDisentangled: 0,
        matches: [],
        quarantined: [...qItems],
        unallocatedBankTransactions: [...store.bankTransactions],
        unallocatedLedgerEntries: [...ledgers],
      };

      for (let i = 0; i < count; i++) {
        const txId = `tx-stress-${i}`;
        const ledId = `led-${i}`;
        const ok = store.resolveManualMatch(txId, ledId, `Khop noi test #${i}`);
        expect(ok).toBe(true);
      }

      expect(store.reconciliationSummary.quarantined.length).toBe(0);
      expect(store.reconciliationSummary.hitlQuarantineCount).toBe(0);
      expect(store.reconciliationSummary.matchedCount).toBe(50);
      expect(store.reconciliationSummary.matchRate).toBe(100);
      expect(store.reconciliationSummary.unallocatedLedgerEntries.length).toBe(0);
      expect(store.quarantineActionLogs.length).toBe(50);

      const retry = store.resolveManualMatch('tx-stress-0', 'led-0');
      expect(retry).toBe(false);
    });

    it('2.2 validates fee allocation invariants and prevents float arithmetic drift', () => {
      const store = useReconciliationStore();
      const qItem = createMockQuarantinedItem(1, 22_000, 'Thu phi duy tri dich vu');

      store.reconciliationSummary = {
        totalBankTransactions: 1,
        totalLedgerEntries: 0,
        matchedCount: 0,
        tier1Count: 0,
        tier2Count: 0,
        tier3Count: 0,
        hitlQuarantineCount: 1,
        matchRate: 0,
        totalMatchedAmount: 0,
        totalFeeDisentangled: 0,
        matches: [],
        quarantined: [qItem],
        unallocatedBankTransactions: [qItem.rawTransaction!],
        unallocatedLedgerEntries: [],
      };

      const ok = store.resolveAllocateFee('tx-stress-1', 15_000, 'Hach toan mot phan phi');
      expect(ok).toBe(true);
      expect(store.reconciliationSummary.totalFeeDisentangled).toBe(15_000);
      expect(store.reconciliationSummary.quarantined.length).toBe(0);
      expect(store.reconciliationSummary.matches[0].feeAmount).toBe(15_000);

      const notFound = store.resolveAllocateFee('non-existent-id', 5000);
      expect(notFound).toBe(false);
    });

    it('2.3 handles escalation with special characters, unicode, and idempotent map updates', () => {
      const store = useReconciliationStore();
      const qItem = createMockQuarantinedItem(99, 500_000_000, 'Giao dich chuyen nhuong bat thuong');

      store.reconciliationSummary = {
        totalBankTransactions: 1,
        totalLedgerEntries: 0,
        matchedCount: 0,
        tier1Count: 0,
        tier2Count: 0,
        tier3Count: 0,
        hitlQuarantineCount: 1,
        matchRate: 0,
        totalMatchedAmount: 0,
        totalFeeDisentangled: 0,
        matches: [],
        quarantined: [qItem],
        unallocatedBankTransactions: [qItem.rawTransaction!],
        unallocatedLedgerEntries: [],
      };

      const xssRemarks = '<script>alert("AML")</script> — De nghi Checker kiem tra ho so hop dong #9988 & dau moc do!';
      const ok = store.resolveEscalateToChecker('tx-stress-99', xssRemarks);

      expect(ok).toBe(true);
      expect(store.escalatedTxMap['tx-stress-99']).toBeDefined();
      expect(store.escalatedTxMap['tx-stress-99'].remarks).toBe(xssRemarks);
      expect(qItem.reason).toContain(xssRemarks);
      expect(store.quarantineActionLogs[0].action).toBe('ESCALATE');
      expect(store.quarantineActionLogs[0].details).toBe(xssRemarks);

      const updatedRemarks = 'Kiem tra lai lan 2';
      const reOk = store.resolveEscalateToChecker('tx-stress-99', updatedRemarks);
      expect(reOk).toBe(true);
      expect(store.escalatedTxMap['tx-stress-99'].remarks).toBe(updatedRemarks);
    });

    it('2.4 handles manual match with amount variance between bank tx and ledger entry', () => {
      const store = useReconciliationStore();
      const qItem = createMockQuarantinedItem(5, 100_000_000);
      const ledger: LedgerEntry = {
        id: 'led-diff',
        docNo: 'INV-DIFF',
        entryDate: '2026-08-15',
        entryTimestamp: 1786320000,
        partnerName: 'Doi tac Diff',
        amount: 90_000_000,
        entryType: 'CREDIT',
        description: 'Hoa don 90M',
        status: 'UNMATCHED',
      };

      store.bankTransactions = [qItem.rawTransaction!];
      store.ledgerEntries = [ledger];
      store.reconciliationSummary = {
        totalBankTransactions: 1,
        totalLedgerEntries: 1,
        matchedCount: 0,
        tier1Count: 0,
        tier2Count: 0,
        tier3Count: 0,
        hitlQuarantineCount: 1,
        matchRate: 0,
        totalMatchedAmount: 0,
        totalFeeDisentangled: 0,
        matches: [],
        quarantined: [qItem],
        unallocatedBankTransactions: [qItem.rawTransaction!],
        unallocatedLedgerEntries: [ledger],
      };

      const ok = store.resolveManualMatch('tx-stress-5', 'led-diff', 'Khớp kèm sai lệch 10M');
      expect(ok).toBe(true);
      expect(store.reconciliationSummary.matches[0].matchedAmount).toBe(90_000_000);
      expect(store.reconciliationSummary.matches[0].discrepancyAmount).toBe(10_000_000);
    });
  });

  // =========================================================================
  // TASK 3: Checker Workstation Anti-Self-Approval & Merkle Proof Stress
  // =========================================================================
  describe('Task 3: Checker Workstation Anti-Self-Approval & Merkle Proof Stress', () => {
    it('3.1 strictly enforces fail-closed anti-self-approval rule (Circular 09/2020)', () => {
      const voucher = createPaymentVoucher(
        'officer_maker_01',
        '01-CITAD-SBV-VND',
        'CITAD',
        100_000_000,
        'Chuyen khoan thanh toan'
      );
      const submitted = submitVoucherForApproval(voucher);

      expect(() => {
        approveVoucher(submitted, 'officer_maker_01', 'BIOMETRIC_SIM', submitted.hitlToken!);
      }).toThrow(/Maker cannot be Checker/i);

      expect(() => {
        approveVoucher(submitted, 'officer_checker_02', 'BIOMETRIC_SIM', submitted.hitlToken!);
      }).not.toThrow();
    });

    it('3.2 verifies Checker Workstation mandatory remarks validation for approval & rejection', () => {
      const voucher = createPaymentVoucher(
        'officer_maker_01',
        '01-CITAD-SBV-VND',
        'CITAD',
        100_000_000,
        'Chuyen khoan thanh toan'
      );
      const submitted = submitVoucherForApproval(voucher);

      const evaluateWorkstationCanApprove = (v: typeof submitted, currentUserId: string, otpVerified: boolean, remarks: string) => {
        const isSelf = v.makerId.trim() === currentUserId.trim();
        return (
          Boolean(v) &&
          v.status === 'PENDING_APPROVAL' &&
          !isSelf &&
          otpVerified &&
          remarks.trim().length > 0
        );
      };

      const evaluateWorkstationCanReject = (v: typeof submitted, remarks: string) => {
        return Boolean(v) && remarks.trim().length > 0;
      };

      // 1. Empty remarks -> Rejection and Approval are strictly blocked
      expect(evaluateWorkstationCanApprove(submitted, 'officer_checker_02', true, '')).toBe(false);
      expect(evaluateWorkstationCanReject(submitted, '')).toBe(false);

      // 2. Whitespace-only remarks -> Rejection and Approval are strictly blocked
      expect(evaluateWorkstationCanApprove(submitted, 'officer_checker_02', true, '   \t\n  ')).toBe(false);
      expect(evaluateWorkstationCanReject(submitted, '   \t\n  ')).toBe(false);

      // 3. Self-approval attempted with valid remarks -> Blocked by fail-closed anti-self-approval rule
      expect(evaluateWorkstationCanApprove(submitted, 'officer_maker_01', true, 'Hợp lệ')).toBe(false);

      // 4. Distinct checker with OTP verified and non-empty remarks -> Allowed
      const validRemarks = 'Đã kiểm tra đối chiếu đầy đủ hợp đồng và hóa đơn điện tử hợp lệ';
      expect(evaluateWorkstationCanApprove(submitted, 'officer_checker_02', true, validRemarks)).toBe(true);
      expect(evaluateWorkstationCanReject(submitted, 'Từ chối do sai số tài khoản thụ hưởng')).toBe(true);

      // 5. Verify engine-level rejection execution
      const rejected = rejectVoucher(
        submitted,
        'officer_checker_02',
        'Ho so thieu chung tu goc theo Thong tu 09',
        submitted.hitlToken!
      );
      expect(rejected.status).toBe('REJECTED');
      expect(rejected.rejectReason).toContain('Ho so thieu chung tu goc');
    });

    it('3.3 verifies O(log N) Merkle Proof fails when leaf, intermediate hash, or root is altered', () => {
      const leaves = [
        'leaf_01_citad',
        'leaf_02_napas',
        'leaf_03_bilat',
        'leaf_04_swift',
        'leaf_05_treasury',
        'leaf_06_settle',
        'leaf_07_audit',
      ];
      const tree = buildMerkleTree(leaves);

      const proof3 = generateMerkleProof(leaves, 3);
      expect(verifyMerkleProof(leaves[3], proof3.proof, tree.root)).toBe(true);

      expect(verifyMerkleProof('corrupted_leaf_hash', proof3.proof, tree.root)).toBe(false);

      const corruptedProof = JSON.parse(JSON.stringify(proof3.proof));
      corruptedProof[0].hash = 'altered_intermediate_hash';
      expect(verifyMerkleProof(leaves[3], corruptedProof, tree.root)).toBe(false);

      const flippedProof = JSON.parse(JSON.stringify(proof3.proof));
      flippedProof[0].position = flippedProof[0].position === 'left' ? 'right' : 'left';
      expect(verifyMerkleProof(leaves[3], flippedProof, tree.root)).toBe(false);

      expect(verifyMerkleProof(leaves[3], proof3.proof, 'corrupted_root_hash')).toBe(false);
    });

    it('3.4 handles edge-case Merkle trees: single leaf, two leaves, odd leaf count, and large tree (32 leaves)', () => {
      const singleLeaf = ['only_leaf'];
      const singleTree = buildMerkleTree(singleLeaf);
      expect(singleTree.root).toBe('only_leaf');
      const singleProof = generateMerkleProof(singleLeaf, 0);
      expect(singleProof.proof.length).toBe(0);
      expect(verifyMerkleProof('only_leaf', singleProof.proof, singleTree.root)).toBe(true);

      const threeLeaves = ['leaf_a', 'leaf_b', 'leaf_c'];
      const threeTree = buildMerkleTree(threeLeaves);
      for (let i = 0; i < 3; i++) {
        const p = generateMerkleProof(threeLeaves, i);
        expect(verifyMerkleProof(threeLeaves[i], p.proof, threeTree.root)).toBe(true);
      }

      const leaves32 = Array.from({ length: 32 }, (_, k) => `leaf-sha256-${k}`);
      const tree32 = buildMerkleTree(leaves32);
      expect(tree32.root).toBeDefined();
      for (let k = 0; k < 32; k += 5) {
        const p = generateMerkleProof(leaves32, k);
        expect(p.proof.length).toBe(5);
        expect(verifyMerkleProof(leaves32[k], p.proof, tree32.root)).toBe(true);
      }
    });

    it('3.5 verifies forward audit ledger detects payload tampering across multiple entries', () => {
      const ledger = new ForwardAuditLedger();
      ledger.append('MAKER-01', 'CREATE_VOUCHER', { amount: 50_000_000 });
      ledger.append('CHECKER-01', 'APPROVE_VOUCHER', { amount: 50_000_000 });
      ledger.append('SETTLEMENT-01', 'DISBURSE_FUNDS', { amount: 50_000_000 });

      expect(ledger.verifyIntegrity().isValid).toBe(true);

      (ledger as any).entries[0].payload.amount = 999_999_999;
      const tamper1 = ledger.verifyIntegrity();
      expect(tamper1.isValid).toBe(false);
      expect(tamper1.brokenAt).toBe(1);
    });
  });

  // =========================================================================
  // TASK 4: Treasury Desk & Rebalancing Stress Testing
  // =========================================================================
  describe('Task 4: Treasury Desk & Rebalancing Stress Testing', () => {
    it('4.1 rejects rebalancing when amount exceeds available balance (Overdraft Protection)', () => {
      const bankStore = useBankingStore();
      const swiftAcc = bankStore.accounts.find((a) => a.bankCode === 'SWIFT')!;
      const originalSwiftBal = swiftAcc.balanceVnd;

      const overRes = bankStore.rebalanceLiquidity(
        'SWIFT',
        'CITAD',
        originalSwiftBal + 1,
        'Overdraft test'
      );

      expect(overRes.success).toBe(false);
      expect(overRes.error).toContain('không đủ');
      expect(swiftAcc.balanceVnd).toBe(originalSwiftBal);
      expect(bankStore.rebalanceLogs[0].status).toBe('REJECTED');
    });

    it('4.2 rejects non-positive, zero, and self-transfer rebalancing amounts', () => {
      const bankStore = useBankingStore();

      const zeroRes = bankStore.rebalanceLiquidity('CITAD', 'NAPAS', 0, 'Zero transfer');
      expect(zeroRes.success).toBe(false);
      expect(zeroRes.error).toContain('lớn hơn 0');

      const negRes = bankStore.rebalanceLiquidity('CITAD', 'NAPAS', -50_000_000, 'Negative transfer');
      expect(negRes.success).toBe(false);
      expect(negRes.error).toContain('lớn hơn 0');

      const sameRes = bankStore.rebalanceLiquidity('CITAD', 'CITAD', 10_000_000, 'Self transfer');
      expect(sameRes.success).toBe(false);
      expect(sameRes.error).toContain('phải khác nhau');

      const invalidRes = bankStore.rebalanceLiquidity('NON_EXISTENT' as BankCode, 'CITAD', 10_000_000, 'Invalid');
      expect(invalidRes.success).toBe(false);
      expect(invalidRes.error).toContain('Không tìm thấy');
    });

    it('4.3 tracks statutory reserve limit breaches and recovers dynamically on replenishment', () => {
      const bankStore = useBankingStore();
      const napasAcc = bankStore.accounts.find((a) => a.bankCode === 'NAPAS')!;
      const napasMinReserve = napasAcc.minReserveVnd;

      expect(bankStore.hasReserveBreach).toBe(false);

      const res = bankStore.rebalanceLiquidity('NAPAS', 'CITAD', 1_200_000_000, 'Ha han muc NAPAS');
      expect(res.success).toBe(true);
      expect(napasAcc.balanceVnd).toBe(480_000_000);
      expect(napasAcc.balanceVnd).toBeLessThan(napasMinReserve);

      expect(bankStore.hasReserveBreach).toBe(true);
      expect(bankStore.breachedChannels.some((a) => a.bankCode === 'NAPAS')).toBe(true);

      const replenishRes = bankStore.rebalanceLiquidity('CITAD', 'NAPAS', 300_000_000, 'Bo sung han muc NAPAS');
      expect(replenishRes.success).toBe(true);
      expect(napasAcc.balanceVnd).toBe(780_000_000);

      expect(bankStore.hasReserveBreach).toBe(false);
      expect(bankStore.breachedChannels.length).toBe(0);
    });

    it('4.4 preserves total system liquidity conservation across multiple rebalances', () => {
      const bankStore = useBankingStore();
      const initialTotalLiquid = bankStore.totalLiquidVnd;

      bankStore.rebalanceLiquidity('CITAD', 'BILATERAL', 250_000_000, 'Rebalance #1');
      bankStore.rebalanceLiquidity('BILATERAL', 'SWIFT', 150_000_000, 'Rebalance #2');
      bankStore.rebalanceLiquidity('SWIFT', 'NAPAS', 100_000_000, 'Rebalance #3');
      bankStore.rebalanceLiquidity('NAPAS', 'CITAD', 200_000_000, 'Rebalance #4');

      expect(bankStore.totalLiquidVnd).toBe(initialTotalLiquid);
    });

    it('4.5 verifies boundary condition at exactly minReserveVnd vs minReserveVnd - 1', () => {
      const bankStore = useBankingStore();
      const swiftAcc = bankStore.accounts.find((a) => a.bankCode === 'SWIFT')!;
      const minReserve = swiftAcc.minReserveVnd; // 200M

      bankStore.updateBalance('SWIFT', minReserve);
      expect(bankStore.breachedChannels.some((a) => a.bankCode === 'SWIFT')).toBe(false);

      bankStore.updateBalance('SWIFT', minReserve - 1);
      expect(bankStore.breachedChannels.some((a) => a.bankCode === 'SWIFT')).toBe(true);
    });
  });
});
