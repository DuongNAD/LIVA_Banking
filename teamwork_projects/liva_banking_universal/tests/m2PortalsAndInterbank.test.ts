/**
 * Milestone M2 Comprehensive Test Suite — Specialized Banking Staff Portals & Interbank Ingestion
 * Covers Features F05, F06, F07, F08, F09, F10:
 * - F05: Interbank clearing statement ingestion (ISO 20022 camt.053 & pacs.008 XML)
 * - F06: Exception & Unmatched Resolution Queue UI & Store Actions (manual match, fee allocation 6425, checker escalation)
 * - F07 & F08: Operations Supervisor / Checker Workstation, Anti-Self-Approval, Merkle O(log N) verification
 * - F09: Compliance & AML/CFT Portal (5 surveillance triggers & statutory Form STR export)
 * - F10: Treasury & Liquidity Desk (multi-channel surveillance, net flow, reserve limits, cash rebalancing)
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';

import { parseIso20022Xml } from '../src/engine/ingestion/iso20022Parser';
import { detectBankCode, parseCsvOrTsv } from '../src/engine/ingestion/universalParser';
import { useReconciliationStore } from '../src/stores/reconciliationStore';
import { useBankingStore } from '../src/stores/bankingStore';
import {
  detectAmlHighValue,
  detectAmlStructuring,
  detectAmlNightVelocity,
  detectAmlRapidPassThrough,
  detectAmlWatchlistKeywords,
} from '../src/engine/intelligence/amlSurveillance';
import { generateFormStr, formatOfficialStrDocument } from '../src/engine/intelligence/strGenerator';
import {
  buildMerkleTree,
  computeMerkleLeaf,
  generateMerkleProof,
  verifyMerkleProof,
  ForwardAuditLedger,
} from '../src/engine/treasury/merkleAudit';
import {
  createPaymentVoucher,
  submitVoucherForApproval,
  approveVoucher,
} from '../src/engine/treasury/makerChecker';
import type { RawStatementRow, LedgerEntry } from '../src/types/banking';
import type { HitlQuarantineItem } from '../src/types/reconciliation';

describe('Milestone M2: Specialized Banking Staff Portals & Interbank Settlement', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  // =========================================================================
  // 1. Feature F05: Interbank Clearing Statement Ingestion (ISO 20022 XML)
  // =========================================================================
  describe('Feature F05: Interbank Clearing Statement Ingestion (ISO 20022 XML)', () => {
    const sampleCamt053Xml = `<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.08">
  <BkToCstmrStmt>
    <GrpHdr>
      <MsgId>MSG-CITAD-20260831-001</MsgId>
      <CreDtTm>2026-08-31T17:00:00Z</CreDtTm>
    </GrpHdr>
    <Stmt>
      <Id>STMT-2026-08-31</Id>
      <Acct>
        <Id>
          <Othr>
            <Id>01-CITAD-SBV-VND</Id>
          </Othr>
        </Id>
        <Nm>TÀI KHOẢN TIỀN GỬI THANH TOÁN TẠI SỞ GIAO DỊCH NHNN</Nm>
        <Ccy>VND</Ccy>
      </Acct>
      <Bal>
        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">2000000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Dt><Dt>2026-08-01</Dt></Dt>
      </Bal>
      <Bal>
        <Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="VND">2450000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Dt><Dt>2026-08-31</Dt></Dt>
      </Bal>
      <Ntry>
        <Amt Ccy="VND">500000000</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <BookgDt><Dt>2026-08-15</Dt></BookgDt>
        <ValDt><Dt>2026-08-15</Dt></ValDt>
        <AcctSvcrRef>CITAD-TX-001</AcctSvcrRef>
        <NtryDtls>
          <TxDtls>
            <Refs>
              <EndToEndId>E2E-CITAD-991</EndToEndId>
              <InstrId>INS-001</InstrId>
            </Refs>
            <RltdPties>
              <Dbtr><Nm>KHO BẠC NHÀ NƯỚC HÀ NỘI</Nm></Dbtr>
              <DbtrAcct><Id><Othr><Id>KB-001-VND</Id></Othr></Id></DbtrAcct>
            </RltdPties>
            <RmtInf>
              <Ustrd>Quyet toan von ngan sach nha nuoc ky 08/2026</Ustrd>
            </RmtInf>
          </TxDtls>
        </NtryDtls>
      </Ntry>
      <Ntry>
        <Amt Ccy="VND">50000000</Amt>
        <CdtDbtInd>DBIT</CdtDbtInd>
        <BookgDt><Dt>2026-08-20</Dt></BookgDt>
        <AcctSvcrRef>CITAD-FEE-002</AcctSvcrRef>
        <NtryDtls>
          <TxDtls>
            <Refs>
              <EndToEndId>E2E-CITAD-992</EndToEndId>
            </Refs>
            <RltdPties>
              <Cdtr><Nm>SỞ GIAO DỊCH NHNN PHÍ DỊCH VỤ</Nm></Cdtr>
            </RltdPties>
            <RmtInf>
              <Ustrd>Phi duy tri ket noi he thong IBPS/CITAD thang 08/2026</Ustrd>
            </RmtInf>
          </TxDtls>
        </NtryDtls>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>`;

    it('parses ISO 20022 camt.053 XML statement accurately', () => {
      const result = parseIso20022Xml(sampleCamt053Xml);
      expect(result.bankCode).toBe('CITAD');
      expect(result.accountNumber).toBe('01-CITAD-SBV-VND');
      expect(result.accountName).toContain('SỞ GIAO DỊCH NHNN');
      expect(result.currency).toBe('VND');
      expect(result.openingBalance).toBe(2_000_000_000);
      expect(result.closingBalance).toBe(2_450_000_000);
      expect(result.totalCredit).toBe(500_000_000);
      expect(result.totalDebit).toBe(50_000_000);
      expect(result.balanceInvariantPassed).toBe(true);
      expect(result.balanceDiscrepancy).toBe(0);
      expect(result.transactions.length).toBe(2);

      const tx1 = result.transactions[0];
      expect(tx1.credit).toBe(500_000_000);
      expect(tx1.txType).toBe('CREDIT');
      expect(tx1.counterparty).toBe('KHO BẠC NHÀ NƯỚC HÀ NỘI');
      expect(tx1.narration).toContain('Quyet toan von ngan sach nha nuoc');

      const tx2 = result.transactions[1];
      expect(tx2.debit).toBe(50_000_000);
      expect(tx2.txType).toBe('DEBIT');
      expect(tx2.counterparty).toBe('SỞ GIAO DỊCH NHNN PHÍ DỊCH VỤ');
    });

    it('detects ISO 20022 XML formats and routes transparently via parseCsvOrTsv', () => {
      const detected = detectBankCode(sampleCamt053Xml);
      expect(detected).toBe('CITAD');

      const parsed = parseCsvOrTsv(sampleCamt053Xml);
      expect(parsed.bankCode).toBe('CITAD');
      expect(parsed.balanceInvariantPassed).toBe(true);
      expect(parsed.transactions.length).toBe(2);
    });

    it('parses pacs.008 XML customer credit transfer format', () => {
      const samplePacsXml = `<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>SWIFT-PACS008-20260825</MsgId>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <EndToEndId>SWIFT-E2E-10029</EndToEndId>
        <TxId>TX-SWIFT-001</TxId>
      </PmtId>
      <IntrBkSttlmAmt Ccy="VND">350000000</IntrBkSttlmAmt>
      <IntrBkSttlmDt>2026-08-25</IntrBkSttlmDt>
      <Dbtr>
        <Nm>DEUTSCHE BANK FRANKFURT</Nm>
      </Dbtr>
      <DbtrAcct>
        <Id><Othr><Id>DEUT-EUR-01</Id></Othr></Id>
      </DbtrAcct>
      <RmtInf>
        <Ustrd>Thanh toan hop dong nhap khau linh kien LC-9988</Ustrd>
      </RmtInf>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>`;

      const detected = detectBankCode(samplePacsXml);
      expect(detected).toBe('SWIFT');

      const parsed = parseIso20022Xml(samplePacsXml);
      expect(parsed.bankCode).toBe('SWIFT');
      expect(parsed.transactions.length).toBe(1);
      expect(parsed.transactions[0].credit).toBe(350_000_000);
      expect(parsed.transactions[0].counterparty).toBe('DEUTSCHE BANK FRANKFURT');
    });
  });

  // =========================================================================
  // 2. Feature F06: Exception & Unmatched Resolution Queue UI & Store Actions
  // =========================================================================
  describe('Feature F06: Exception & Unmatched Resolution Queue', () => {
    it('diagnoses suspected causes accurately with getSuspectedCause', () => {
      const store = useReconciliationStore();

      // 1. Fee variance (narration mentions phi)
      const qFee: HitlQuarantineItem = {
        txId: 'tx-fee',
        amount: 22_000,
        hitlToken: 'hitl-token-1',
        createdAt: 1786320000,
        expiresAt: 1786320900,
        reason: 'Possible fee mismatch',
        rawTransaction: {
          id: 'tx-fee',
          date: '2026-08-10',
          txDate: 1786320000,
          txCode: 'FEE',
          debit: 22_000,
          credit: 0,
          netAmount: -22_000,
          amount: 22_000,
          txType: 'DEBIT',
          balance: 100_000_000,
          narration: 'Thu phi chuyen tien lien ngan hang Napas 24/7',
          bankCode: 'TCB',
        },
      };
      const causeFee = store.getSuspectedCause(qFee);
      expect(causeFee.causeCode).toBe('FEE_VARIANCE');

      // 2. Timing window (has candidate ledger IDs)
      const qTiming: HitlQuarantineItem = {
        txId: 'tx-timing',
        amount: 50_000_000,
        hitlToken: 'hitl-token-2',
        createdAt: 1787184000,
        expiresAt: 1787184900,
        reason: 'Potential candidate outside strict date range',
        candidateLedgerIds: ['led-timing-1'],
        rawTransaction: {
          id: 'tx-timing',
          date: '2026-08-20',
          txDate: 1787184000,
          txCode: 'TX02',
          debit: 0,
          credit: 50_000_000,
          netAmount: 50_000_000,
          amount: 50_000_000,
          txType: 'CREDIT',
          balance: 50_000_000,
          narration: 'Thanh toan tien hang tre han',
          bankCode: 'VCB',
        },
      };
      const causeTiming = store.getSuspectedCause(qTiming);
      expect(causeTiming.causeCode).toBe('TIMING_WINDOW');

      // 3. Missing invoice / name mismatch
      const qMissing: HitlQuarantineItem = {
        txId: 'tx-missing',
        amount: 100_000_000,
        hitlToken: 'hitl-token-3',
        createdAt: 1787184000,
        expiresAt: 1787184900,
        reason: 'No invoice matched',
        rawTransaction: {
          id: 'tx-missing',
          date: '2026-08-20',
          txDate: 1787184000,
          txCode: 'TX03',
          debit: 0,
          credit: 100_000_000,
          netAmount: 100_000_000,
          amount: 100_000_000,
          txType: 'CREDIT',
          balance: 100_000_000,
          narration: 'Chuyen tien khong ghi ro noi dung',
          bankCode: 'BIDV',
        },
      };
      const causeMissing = store.getSuspectedCause(qMissing);
      expect(causeMissing.causeCode).toBe('MISSING_INVOICE');
    });

    it('executes resolveManualMatch and updates unmatched queue', () => {
      const store = useReconciliationStore();
      const tx: RawStatementRow = {
        id: 'tx-manual-1',
        date: '2026-08-10',
        txDate: 1786320000,
        txCode: 'TX01',
        debit: 0,
        credit: 50_000_000,
        netAmount: 50_000_000,
        amount: 50_000_000,
        txType: 'CREDIT',
        balance: 50_000_000,
        narration: 'Thanh toan hop dong A',
        bankCode: 'VCB',
      };
      const ledger: LedgerEntry = {
        id: 'led-manual-1',
        docNo: 'HD-MANUAL',
        entryDate: '2026-08-10',
        entryTimestamp: 1786320000,
        partnerName: 'CONG TY A',
        amount: 50_000_000,
        entryType: 'CREDIT',
        description: 'Thanh toan HD-MANUAL',
        status: 'UNMATCHED',
      };

      store.bankTransactions = [tx];
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
        quarantined: [
          {
            txId: 'tx-manual-1',
            amount: 50_000_000,
            hitlToken: 'hitl-man-1',
            createdAt: 1786320000,
            expiresAt: 1786320900,
            reason: 'Requires human inspection',
            rawTransaction: tx,
          },
        ],
        unallocatedBankTransactions: [tx],
        unallocatedLedgerEntries: [ledger],
      };

      const success = store.resolveManualMatch(
        'tx-manual-1',
        'led-manual-1',
        'Khớp nối thủ công xác minh qua hóa đơn điện tử'
      );

      expect(success).toBe(true);
      expect(store.reconciliationSummary!.quarantined.length).toBe(0);
      expect(store.reconciliationSummary!.matches.length).toBe(1);
      expect(store.reconciliationSummary!.matches[0].matchType).toBe('MANUAL_HITL');
      expect(store.quarantineActionLogs.some((a) => a.action === 'OVERRIDE')).toBe(true);
    });

    it('executes resolveAllocateFee to accounting account 6425 / 811', () => {
      const store = useReconciliationStore();
      const tx: RawStatementRow = {
        id: 'tx-fee-alloc',
        date: '2026-08-11',
        txDate: 1786406400,
        txCode: 'TXFEE',
        debit: 22_000,
        credit: 0,
        netAmount: -22_000,
        amount: 22_000,
        txType: 'DEBIT',
        balance: 99_978_000,
        narration: 'Thu phi chuyen tien lien ngan hang',
        bankCode: 'TCB',
      };

      store.bankTransactions = [tx];
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
        quarantined: [
          {
            txId: 'tx-fee-alloc',
            amount: 22_000,
            hitlToken: 'hitl-fee-1',
            createdAt: 1786406400,
            expiresAt: 1786407300,
            reason: 'Bank service fee',
            rawTransaction: tx,
          },
        ],
        unallocatedBankTransactions: [tx],
        unallocatedLedgerEntries: [],
      };

      const success = store.resolveAllocateFee(
        'tx-fee-alloc',
        22_000,
        'Hạch toán chi phí dịch vụ ngân hàng chuyển khoản TCB vào TK 6425'
      );

      expect(success).toBe(true);
      expect(store.reconciliationSummary!.quarantined.length).toBe(0);
      expect(store.reconciliationSummary!.matches[0].feeAmount).toBe(22_000);
      expect(store.quarantineActionLogs.some((a) => a.action === 'ALLOCATE_FEE')).toBe(true);
    });

    it('executes resolveEscalateToChecker and tracks supervisor escalation record', () => {
      const store = useReconciliationStore();
      const tx: RawStatementRow = {
        id: 'tx-escalate',
        date: '2026-08-12',
        txDate: 1786492800,
        txCode: 'TX-ESC',
        debit: 0,
        credit: 300_000_000,
        netAmount: 300_000_000,
        amount: 300_000_000,
        txType: 'CREDIT',
        balance: 300_000_000,
        narration: 'Khoan tien nghi van can lanh dao xac minh nguon goc',
        bankCode: 'BIDV',
      };

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
        quarantined: [
          {
            txId: 'tx-escalate',
            amount: 300_000_000,
            hitlToken: 'hitl-esc-1',
            createdAt: 1786492800,
            expiresAt: 1786493700,
            reason: 'Requires supervisor verification',
            rawTransaction: tx,
          },
        ],
        unallocatedBankTransactions: [tx],
        unallocatedLedgerEntries: [],
      };

      const success = store.resolveEscalateToChecker(
        'tx-escalate',
        'Nghi vấn giao dịch chênh lệch cần Giám đốc Vận hành liên hệ ngân hàng đối tác'
      );

      expect(success).toBe(true);
      expect(store.escalatedTxMap['tx-escalate']).toBeDefined();
      expect(store.escalatedTxMap['tx-escalate'].remarks).toContain('Giám đốc Vận hành');
      expect(store.quarantineActionLogs.some((a) => a.action === 'ESCALATE')).toBe(true);
    });
  });

  // =========================================================================
  // 3. Features F07 & F08: Operations Supervisor / Checker Workstation
  // =========================================================================
  describe('Features F07 & F08: Checker Workstation & Fail-Closed Anti-Self-Approval', () => {
    it('enforces fail-closed anti-self-approval when makerId === checkerId', () => {
      const voucher = createPaymentVoucher(
        'maker_officer_01',
        '01-CITAD-SBV-VND',
        'CITAD',
        500_000_000,
        'Dieu chuyen von thanh khoan'
      );
      const submitted = submitVoucherForApproval(voucher);

      // Attempting self-approval must throw an error adhering to Circular 09/2020/TT-NHNN
      expect(() => {
        approveVoucher(submitted, 'maker_officer_01', 'BIOMETRIC_SIM', submitted.hitlToken!);
      }).toThrow(/Maker cannot be Checker/i);
    });

    it('allows distinct checker approval and computes verifiable Merkle leaf hash', () => {
      const voucher = createPaymentVoucher(
        'maker_officer_01',
        '01-CITAD-SBV-VND',
        'CITAD',
        500_000_000,
        'Dieu chuyen von thanh khoan'
      );
      const submitted = submitVoucherForApproval(voucher);

      const approved = approveVoucher(
        submitted,
        'checker_supervisor_02',
        'BIOMETRIC_SIM',
        submitted.hitlToken!
      );

      expect(approved.status).toBe('APPROVED');
      expect(approved.checkerId).toBe('checker_supervisor_02');
      expect(approved.merkleLeafHash).toBeDefined();

      // Verify computed leaf matches
      const expectedLeaf = computeMerkleLeaf(approved);
      expect(approved.merkleLeafHash).toBe(expectedLeaf);
    });

    it('verifies O(log N) Merkle Proof inclusion and detects tampering', () => {
      const leaves = [
        'leaf-hash-citad-1',
        'leaf-hash-napas-2',
        'leaf-hash-bilat-3',
        'leaf-hash-swift-4',
      ];
      const tree = buildMerkleTree(leaves);

      // Generate and verify legitimate leaf at index 1
      const proofObj = generateMerkleProof(leaves, 1);
      const isValid = verifyMerkleProof(leaves[1], proofObj.proof, tree.root);
      expect(isValid).toBe(true);

      // Tampered leaf must fail verification
      const isTamperedValid = verifyMerkleProof('tampered-hash-xxx', proofObj.proof, tree.root);
      expect(isTamperedValid).toBe(false);
    });

    it('verifies tamper-evident forward hash chain integrity in ForwardAuditLedger', () => {
      const ledger = new ForwardAuditLedger();
      ledger.append('MAKER-01', 'CREATE_VOUCHER', { amount: 100_000_000 });
      ledger.append('CHECKER-02', 'APPROVE_VOUCHER', { amount: 100_000_000 });
      ledger.append('TREASURY-01', 'SETTLE_FUNDS', { amount: 100_000_000 });

      const integrity = ledger.verifyIntegrity();
      expect(integrity.isValid).toBe(true);

      // Simulate tampering with an entry's payload
      (ledger as any).entries[1].payload.amount = 999_999_999;
      const tamperedIntegrity = ledger.verifyIntegrity();
      expect(tamperedIntegrity.isValid).toBe(false);
      expect(tamperedIntegrity.brokenAt).toBe(2);
    });
  });

  // =========================================================================
  // 4. Feature F09: Compliance & AML/CFT Surveillance Portal
  // =========================================================================
  describe('Feature F09: Compliance & AML/CFT Portal', () => {
    it('detects high-value transaction (>= 400M VND) per Decision 11/2023/QĐ-TTg', () => {
      const txs = [
        { id: 'tx-high', amount: 450_000_000, credit: 450_000_000, narration: 'Chuyen tien mua BĐS' },
        { id: 'tx-low', amount: 50_000_000, credit: 50_000_000, narration: 'Tien tieu dung' },
      ];
      const alerts = detectAmlHighValue(txs);
      expect(alerts.length).toBe(1);
      expect(alerts[0].anomalyType).toBe('HIGH_VALUE');
      expect(alerts[0].totalAmount).toBe(450_000_000);
      expect(alerts[0].statutoryRuleRef).toContain('11/2023/QĐ-TTg');
    });

    it('detects structuring/smurfing (< 400M split in 24h) per Circular 09/2023/TT-NHNN', () => {
      const txs = [
        { id: 'tx-s1', date: '2026-08-15', amount: 390_000_000, credit: 390_000_000 },
        { id: 'tx-s2', date: '2026-08-15', amount: 380_000_000, credit: 380_000_000 },
        { id: 'tx-s3', date: '2026-08-15', amount: 395_000_000, credit: 395_000_000 },
      ];
      const alerts = detectAmlStructuring(txs);
      expect(alerts.length).toBe(1);
      expect(alerts[0].anomalyType).toBe('STRUCTURING_SMURFING');
      expect(alerts[0].severity).toBe('CRITICAL');
      expect(alerts[0].totalAmount).toBe(1_165_000_000);
    });

    it('detects night-time velocity anomaly (23:00 - 05:00 >= 50M)', () => {
      const txs = [
        { id: 'tx-night-1', time: '02:30:15', amount: 150_000_000, credit: 150_000_000 },
        { id: 'tx-day-1', time: '14:30:00', amount: 150_000_000, credit: 150_000_000 },
      ];
      const alerts = detectAmlNightVelocity(txs);
      expect(alerts.length).toBe(1);
      expect(alerts[0].anomalyType).toBe('NIGHT_VELOCITY');
      expect(alerts[0].involvedTransactionIds).toContain('tx-night-1');
    });

    it('detects rapid pass-through churn mule (>= 100M in -> >= 90% out in minutes)', () => {
      const txs = [
        { id: 'inflow-1', credit: 500_000_000, amount: 500_000_000, txType: 'CREDIT' },
        { id: 'outflow-1', debit: 495_000_000, amount: 495_000_000, txType: 'DEBIT' },
      ];
      const alerts = detectAmlRapidPassThrough(txs);
      expect(alerts.length).toBe(1);
      expect(alerts[0].anomalyType).toBe('RAPID_PASS_THROUGH');
      expect(alerts[0].severity).toBe('CRITICAL');
    });

    it('detects watchlist keyword hit', () => {
      const txs = [
        { id: 'wl-1', narration: 'Chuyen tien nap san bet88 ca do online', credit: 20_000_000 },
      ];
      const alerts = detectAmlWatchlistKeywords(txs);
      expect(alerts.length).toBe(1);
      expect(alerts[0].anomalyType).toBe('WATCHLIST_HIT');
    });

    it('generates Form STR Phụ lục II with prefilled statutory fields and document export', () => {
      const alert = {
        alertId: 'aml-test-01',
        anomalyType: 'STRUCTURING_SMURFING' as const,
        severity: 'CRITICAL' as const,
        involvedTransactionIds: ['tx-1', 'tx-2'],
        totalAmount: 780_000_000,
        detectedAt: '2026-08-15T14:00:00Z',
        reasoning: 'Nghi van chia nho tien gui de ne tranh nguong 400M',
        statutoryRuleRef: 'Thong tu 09/2023/TT-NHNN',
        suggestedStrReport: true,
      };
      const txs = [
        { id: 'tx-1', counterparty: 'NGUYEN HOANG PHUC', counterpartyAccount: '001100234567', amount: 390_000_000 },
        { id: 'tx-2', counterparty: 'NGUYEN HOANG PHUC', counterpartyAccount: '001100234567', amount: 390_000_000 },
      ];

      const form = generateFormStr(alert, txs, 'Canh bao khach hang chuyen tien lien tuc');
      expect(form.reportingEntity).toContain('LIVA SOLUTIONS CO., LTD');
      expect(form.suspectName).toBe('NGUYEN HOANG PHUC');
      expect(form.suspectAccount).toBe('001100234567');
      expect(form.totalVndAmount).toBe(780_000_000);

      const docText = formatOfficialStrDocument(form);
      expect(docText).toContain('CỤC PHÒNG, CHỐNG RỬA TIỀN');
      expect(docText).toContain('Phụ lục II ban hành kèm theo Thông tư số 09/2023/TT-NHNN');
      expect(docText).toContain('780.000.000');
    });
  });

  // =========================================================================
  // 5. Feature F10: Treasury & Liquidity Desk
  // =========================================================================
  describe('Feature F10: Treasury & Liquidity Desk', () => {
    it('initializes interbank accounts with statutory minimum reserve limits', () => {
      const bankStore = useBankingStore();
      const citad = bankStore.accounts.find((a) => a.bankCode === 'CITAD');
      const napas = bankStore.accounts.find((a) => a.bankCode === 'NAPAS');
      const bilat = bankStore.accounts.find((a) => a.bankCode === 'BILATERAL');
      const swift = bankStore.accounts.find((a) => a.bankCode === 'SWIFT');

      expect(citad?.minReserveVnd).toBe(1_000_000_000);
      expect(napas?.minReserveVnd).toBe(500_000_000);
      expect(bilat?.minReserveVnd).toBe(300_000_000);
      expect(swift?.minReserveVnd).toBe(200_000_000);
    });

    it('computes aggregated net cash flow (inflow - outflow) correctly', () => {
      const bankStore = useBankingStore();
      const expectedInflow = bankStore.accounts.reduce((s, a) => s + a.inflowMonthVnd, 0);
      const expectedOutflow = bankStore.accounts.reduce((s, a) => s + a.outflowMonthVnd, 0);
      const expectedNet = expectedInflow - expectedOutflow;

      expect(bankStore.totalInflowMonthVnd).toBe(expectedInflow);
      expect(bankStore.totalOutflowMonthVnd).toBe(expectedOutflow);
      expect(bankStore.netFlowMonthVnd).toBe(expectedNet);
    });

    it('detects liquidity risk limits breach when channel balance drops below reserve', () => {
      const bankStore = useBankingStore();

      // Initially no active channel is below reserve
      expect(bankStore.hasReserveBreach).toBe(false);

      // Simulate heavy withdrawal on NAPAS below 500M minimum reserve
      bankStore.updateBalance('NAPAS', 400_000_000);
      expect(bankStore.hasReserveBreach).toBe(true);
      expect(bankStore.breachedChannels.some((a) => a.bankCode === 'NAPAS')).toBe(true);
    });

    it('executes intraday cash rebalancing between channels and records audit logs', () => {
      const bankStore = useBankingStore();
      const citadBefore = bankStore.accounts.find((a) => a.bankCode === 'CITAD')!.balanceVnd;
      const napasBefore = bankStore.accounts.find((a) => a.bankCode === 'NAPAS')!.balanceVnd;

      const transferAmount = 300_000_000;
      const res = bankStore.rebalanceLiquidity(
        'CITAD',
        'NAPAS',
        transferAmount,
        'Cân đối thanh khoản bù trừ cuối ngày',
        'CHECKER-01'
      );

      expect(res.success).toBe(true);
      expect(res.logId).toBeDefined();

      const citadAfter = bankStore.accounts.find((a) => a.bankCode === 'CITAD')!.balanceVnd;
      const napasAfter = bankStore.accounts.find((a) => a.bankCode === 'NAPAS')!.balanceVnd;

      expect(citadAfter).toBe(citadBefore - transferAmount);
      expect(napasAfter).toBe(napasBefore + transferAmount);

      const log = bankStore.rebalanceLogs[0];
      expect(log.fromChannel).toBe('CITAD');
      expect(log.toChannel).toBe('NAPAS');
      expect(log.amountVnd).toBe(transferAmount);
      expect(log.status).toBe('COMPLETED');
    });

    it('fails closed when attempting rebalancing with insufficient funds or invalid params', () => {
      const bankStore = useBankingStore();

      // Same channel rejection
      const resSame = bankStore.rebalanceLiquidity('CITAD', 'CITAD', 100_000_000, 'Test');
      expect(resSame.success).toBe(false);
      expect(resSame.error).toContain('phải khác nhau');

      // Zero or negative amount rejection
      const resZero = bankStore.rebalanceLiquidity('CITAD', 'NAPAS', 0, 'Test');
      expect(resZero.success).toBe(false);

      // Insufficient funds rejection
      const resOver = bankStore.rebalanceLiquidity(
        'SWIFT',
        'CITAD',
        999_999_999_999,
        'Qua han muc'
      );
      expect(resOver.success).toBe(false);
      expect(resOver.error).toContain('không đủ');

      const rejLog = bankStore.rebalanceLogs[0];
      expect(rejLog.status).toBe('REJECTED');
    });
  });
});
