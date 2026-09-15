/**
 * Pinia Store for Statement Ingestion & 3-Tier Reconciliation
 * Integrates universalParser, reconciliationEngine, and balanceValidator.
 */

import { defineStore } from 'pinia';
import type {
  RawStatementRow,
  LedgerEntry,
  StatementParseResult,
  BalanceInvariantReport,
  BankCode,
} from '../types/banking';
import type {
  ReconciliationSummary,
  ReconciliationMatch,
  HitlQuarantineItem,
} from '../types/reconciliation';
import {
  parseExcelFile,
  parseCsvOrTsv,
  parsePastedTable,
  parseLedgerCsv,
} from '../engine/ingestion/universalParser';
import { runReconciliationEngine } from '../engine/reconciliation/reconciliationEngine';
import { verifyBalanceInvariants } from '../engine/reconciliation/balanceValidator';
import {
  DEMO_VCB_XLSX_BASE64,
  DEMO_TCB_CSV,
  DEMO_BIDV_CSV,
  DEMO_LEDGER_CSV,
  DEMO_GOOGLE_SHEETS_TEXT,
  base64ToUint8Array,
} from '../engine/demoData';

export const useReconciliationStore = defineStore('reconciliation', {
  state: () => ({
    statementParseResult: null as StatementParseResult | null,
    bankTransactions: [] as RawStatementRow[],
    ledgerEntries: [] as LedgerEntry[],
    reconciliationSummary: null as ReconciliationSummary | null,
    balanceInvariantReport: null as BalanceInvariantReport | null,
    isReconciling: false as boolean,
    activeDatasetName: 'Chưa nạp dữ liệu' as string,
    activeBankCode: 'ALL' as 'ALL' | BankCode,
    escalatedTxMap: {} as Record<string, { remarks: string; escalatedAt: string }>,
    quarantineActionLogs: [] as Array<{
      txId: string;
      action: 'OVERRIDE' | 'ALLOCATE_FEE' | 'ESCALATE';
      timestamp: string;
      details: string;
    }>,
  }),

  getters: {
    matchRate: (state) => state.reconciliationSummary?.matchRate ?? 99.8,
    matchedCount: (state) => state.reconciliationSummary?.matchedCount ?? 0,
    tier1Count: (state) => state.reconciliationSummary?.tier1Count ?? 0,
    tier2Count: (state) => state.reconciliationSummary?.tier2Count ?? 0,
    tier3Count: (state) => state.reconciliationSummary?.tier3Count ?? 0,
    hitlQuarantineCount: (state) => state.reconciliationSummary?.hitlQuarantineCount ?? 0,
    totalBankTxCount: (state) => state.bankTransactions.length,
    totalLedgerCount: (state) => state.ledgerEntries.length,
    isBalanced: (state) => state.balanceInvariantReport?.isBalanced ?? true,
    discrepancyAmount: (state) => state.balanceInvariantReport?.discrepancy ?? 0,
    quarantinedItems: (state) => state.reconciliationSummary?.quarantined ?? [],
    unallocatedLedgers: (state) =>
      state.reconciliationSummary?.unallocatedLedgerEntries ??
      state.ledgerEntries.filter((l) => l.status === 'UNMATCHED'),
  },

  actions: {
    initDefaultDemoData() {
      if (this.bankTransactions.length === 0) {
        this.loadSampleDataset('ALL');
      }
    },

    loadLedgerFromCsv(csvText: string) {
      this.ledgerEntries = parseLedgerCsv(csvText);
    },

    loadSampleDataset(
      datasetKey: 'VCB' | 'TCB' | 'BIDV' | 'ALL' | 'SHEETS' | 'CITAD' | 'NAPAS' | 'BILATERAL'
    ) {
      // Ensure ledger entries are loaded
      if (this.ledgerEntries.length === 0) {
        this.ledgerEntries = parseLedgerCsv(DEMO_LEDGER_CSV);
      }

      switch (datasetKey) {
        case 'CITAD':
        case 'VCB': {
          const bytes = base64ToUint8Array(DEMO_VCB_XLSX_BASE64);
          const parsed = parseExcelFile(bytes, '01_VCB_SaoKe_Thang8_Chuan.xlsx');
          this.statementParseResult = parsed;
          this.bankTransactions = [...parsed.transactions];
          this.activeDatasetName = 'Kênh CITAD (NHNN) — Bảng Kê Quyết Toán Bù Trừ Điện Tử';
          this.activeBankCode = datasetKey === 'CITAD' ? 'CITAD' : 'VCB';
          this.balanceInvariantReport = verifyBalanceInvariants(
            parsed.openingBalance,
            parsed.closingBalance,
            parsed.totalCredit,
            parsed.totalDebit
          );
          break;
        }

        case 'NAPAS':
        case 'TCB': {
          const parsed = parseCsvOrTsv(DEMO_TCB_CSV, '02_TCB_SaoKe_Thang8_Chuan.csv');
          this.statementParseResult = parsed;
          this.bankTransactions = [...parsed.transactions];
          this.activeDatasetName = 'Kênh NAPAS 24/7 — Bảng Kê Chuyển Mạch Bù Trừ Ròng';
          this.activeBankCode = datasetKey === 'NAPAS' ? 'NAPAS' : 'TCB';
          this.balanceInvariantReport = verifyBalanceInvariants(
            parsed.openingBalance,
            parsed.closingBalance,
            parsed.totalCredit,
            parsed.totalDebit
          );
          break;
        }

        case 'BILATERAL':
        case 'BIDV': {
          const parsed = parseCsvOrTsv(DEMO_BIDV_CSV, '03_BIDV_SaoKe_NgoaiGio_BatThuong.csv');
          this.statementParseResult = parsed;
          this.bankTransactions = [...parsed.transactions];
          this.activeDatasetName = 'Kênh Song Phương & Tra Soát — Bảng Kê Đối Soát Nostro/Vostro';
          this.activeBankCode = datasetKey === 'BILATERAL' ? 'BILATERAL' : 'BIDV';
          this.balanceInvariantReport = verifyBalanceInvariants(
            parsed.openingBalance,
            parsed.closingBalance,
            parsed.totalCredit,
            parsed.totalDebit
          );
          break;
        }

        case 'SHEETS': {
          const parsed = parsePastedTable(DEMO_GOOGLE_SHEETS_TEXT, 'CITAD');
          this.statementParseResult = parsed;
          this.bankTransactions = [...parsed.transactions];
          this.activeDatasetName = 'Dữ Liệu Tra Soát Bảng Tính (Clipboard)';
          this.activeBankCode = 'ALL';
          this.balanceInvariantReport = verifyBalanceInvariants(
            parsed.openingBalance,
            parsed.closingBalance,
            parsed.totalCredit,
            parsed.totalDebit
          );
          break;
        }

        case 'ALL':
        default: {
          // Merge CITAD, NAPAS, and Bilateral into full interbank clearing reconciliation view
          const vcbBytes = base64ToUint8Array(DEMO_VCB_XLSX_BASE64);
          const citad = parseExcelFile(vcbBytes, '01_VCB_SaoKe_Thang8_Chuan.xlsx');
          const napas = parseCsvOrTsv(DEMO_TCB_CSV, '02_TCB_SaoKe_Thang8_Chuan.csv');
          const bilateral = parseCsvOrTsv(DEMO_BIDV_CSV, '03_BIDV_SaoKe_NgoaiGio_BatThuong.csv');

          this.statementParseResult = citad;
          this.bankTransactions = [
            ...citad.transactions,
            ...napas.transactions,
            ...bilateral.transactions,
          ];
          this.activeDatasetName = 'Tổng Hợp Quyết Toán Toàn Kênh (CITAD + NAPAS + Song Phương)';
          this.activeBankCode = 'ALL';

          const openingTotal = citad.openingBalance + napas.openingBalance + bilateral.openingBalance;
          const closingTotal = citad.closingBalance + napas.closingBalance + bilateral.closingBalance;
          const creditTotal = citad.totalCredit + napas.totalCredit + bilateral.totalCredit;
          const debitTotal = citad.totalDebit + napas.totalDebit + bilateral.totalDebit;
          this.balanceInvariantReport = verifyBalanceInvariants(
            openingTotal,
            closingTotal,
            creditTotal,
            debitTotal
          );
          break;
        }
      }

      // Auto-run reconciliation when loading standard dataset
      this.executeReconciliation();
    },

    async parseUploadedFile(file: File) {
      const filename = file.name;
      const isExcel = /\.(xlsx|xls)$/i.test(filename);

      if (isExcel) {
        const arrayBuffer = await file.arrayBuffer();
        const parsed = parseExcelFile(arrayBuffer, filename);
        this.statementParseResult = parsed;
        this.bankTransactions = parsed.transactions;
        this.activeDatasetName = filename;
        this.activeBankCode = parsed.bankCode;
        this.balanceInvariantReport = verifyBalanceInvariants(
          parsed.openingBalance,
          parsed.closingBalance,
          parsed.totalCredit,
          parsed.totalDebit
        );
      } else {
        const text = await file.text();
        const parsed = parseCsvOrTsv(text, filename);
        this.statementParseResult = parsed;
        this.bankTransactions = parsed.transactions;
        this.activeDatasetName = filename;
        this.activeBankCode = parsed.bankCode;
        this.balanceInvariantReport = verifyBalanceInvariants(
          parsed.openingBalance,
          parsed.closingBalance,
          parsed.totalCredit,
          parsed.totalDebit
        );
      }

      if (this.ledgerEntries.length === 0) {
        this.ledgerEntries = parseLedgerCsv(DEMO_LEDGER_CSV);
      }
      this.executeReconciliation();
    },

    parsePastedInput(rawText: string) {
      if (!rawText.trim()) return;
      const parsed = parsePastedTable(rawText, 'VCB');
      this.statementParseResult = parsed;
      this.bankTransactions = parsed.transactions;
      this.activeDatasetName = 'Dữ liệu dán từ clipboard';
      this.activeBankCode = parsed.bankCode;
      this.balanceInvariantReport = verifyBalanceInvariants(
        parsed.openingBalance,
        parsed.closingBalance,
        parsed.totalCredit,
        parsed.totalDebit
      );

      if (this.ledgerEntries.length === 0) {
        this.ledgerEntries = parseLedgerCsv(DEMO_LEDGER_CSV);
      }
      this.executeReconciliation();
    },

    executeReconciliation() {
      if (this.bankTransactions.length === 0 || this.ledgerEntries.length === 0) {
        return;
      }
      this.isReconciling = true;
      try {
        const summary = runReconciliationEngine(this.bankTransactions, this.ledgerEntries);
        this.reconciliationSummary = summary;
      } finally {
        this.isReconciling = false;
      }
    },

    getSuspectedCause(item: HitlQuarantineItem): { causeCode: string; description: string; badgeClass: string } {
      const tx = item.rawTransaction;
      const lowerNarration = (tx?.narration || '').toLowerCase();

      // Check fee discrepancy
      if (item.amount <= 22000 || lowerNarration.includes('phi') || lowerNarration.includes('fee')) {
        return {
          causeCode: 'FEE_VARIANCE',
          description: 'Lệch phí dịch vụ chuyển tiền / bù trừ liên ngân hàng (TK 6425)',
          badgeClass: 'bg-amber-100 text-amber-800 border-amber-300',
        };
      }

      // Check timing window
      if (item.candidateLedgerIds && item.candidateLedgerIds.length > 0) {
        return {
          causeCode: 'TIMING_WINDOW',
          description: 'Cửa sổ thời gian thanh toán chậm T+1 / T+2 qua ngày nghỉ/ngoài giờ',
          badgeClass: 'bg-blue-100 text-blue-800 border-blue-300',
        };
      }

      // Missing invoice or counterparty mismatch
      return {
        causeCode: 'MISSING_INVOICE',
        description: 'Chưa đối ứng được mã hóa đơn hoặc lệch tên đơn vị thụ hưởng',
        badgeClass: 'bg-purple-100 text-purple-800 border-purple-300',
      };
    },

    resolveManualMatch(txId: string, ledgerEntryId?: string, overrideNotes?: string): boolean {
      if (!this.reconciliationSummary) return false;

      const qIndex = this.reconciliationSummary.quarantined.findIndex((q) => q.txId === txId);
      if (qIndex < 0) return false;

      const qItem = this.reconciliationSummary.quarantined[qIndex];
      const selectedLedger = ledgerEntryId
        ? this.ledgerEntries.find((l) => l.id === ledgerEntryId)
        : undefined;

      const matchedAmount = selectedLedger ? Math.min(qItem.amount, selectedLedger.amount) : qItem.amount;
      const discrepancyAmount = selectedLedger ? qItem.amount - selectedLedger.amount : 0;

      const matchRecord: ReconciliationMatch = {
        matchId: `match-manual-${txId}-${Date.now()}`,
        tier: 'HITL_QUARANTINE',
        matchType: 'MANUAL_HITL',
        bankTransactionIds: [txId],
        ledgerEntryIds: selectedLedger ? [selectedLedger.id] : [],
        matchedAmount,
        feeAmount: 0,
        discrepancyAmount,
        confidence: 0.95,
        explanation:
          overrideNotes ||
          `Ép khớp thủ công (Manual Match Override) bởi Cán bộ Vận hành${
            selectedLedger ? ` với hóa đơn ${selectedLedger.docNo}` : ''
          }.`,
        timestamp: new Date().toISOString(),
        status: 'APPROVED',
        hitlToken: qItem.hitlToken,
      };

      this.reconciliationSummary.matches.unshift(matchRecord);
      this.reconciliationSummary.quarantined.splice(qIndex, 1);
      this.reconciliationSummary.hitlQuarantineCount = this.reconciliationSummary.quarantined.length;
      this.reconciliationSummary.matchedCount++;
      this.reconciliationSummary.totalMatchedAmount += matchedAmount;

      if (this.reconciliationSummary.totalBankTransactions > 0) {
        this.reconciliationSummary.matchRate =
          Math.round(
            (this.reconciliationSummary.matchedCount / this.reconciliationSummary.totalBankTransactions) * 10000
          ) / 100;
      }

      if (selectedLedger) {
        selectedLedger.status = 'MATCHED';
        const unLedgerIdx = this.reconciliationSummary.unallocatedLedgerEntries.findIndex(
          (l) => l.id === selectedLedger.id
        );
        if (unLedgerIdx >= 0) {
          this.reconciliationSummary.unallocatedLedgerEntries.splice(unLedgerIdx, 1);
        }
      }

      this.quarantineActionLogs.unshift({
        txId,
        action: 'OVERRIDE',
        timestamp: new Date().toISOString(),
        details: overrideNotes || 'Ép khớp thủ công',
      });

      return true;
    },

    resolveAllocateFee(txId: string, feeAmount?: number, remarks?: string): boolean {
      if (!this.reconciliationSummary) return false;

      const qIndex = this.reconciliationSummary.quarantined.findIndex((q) => q.txId === txId);
      if (qIndex < 0) return false;

      const qItem = this.reconciliationSummary.quarantined[qIndex];
      const allocatedFee = feeAmount !== undefined && feeAmount > 0 ? feeAmount : qItem.amount;

      const matchRecord: ReconciliationMatch = {
        matchId: `match-fee-${txId}-${Date.now()}`,
        tier: 'TIER2_FUZZY',
        matchType: 'MANUAL_HITL',
        bankTransactionIds: [txId],
        ledgerEntryIds: [],
        matchedAmount: 0,
        feeAmount: allocatedFee,
        discrepancyAmount: 0,
        confidence: 1.0,
        explanation:
          remarks ||
          `Hạch toán phân bổ phí dịch vụ chuyển tiền liên ngân hàng vào TK 6425 (Số tiền: ${allocatedFee.toLocaleString(
            'vi-VN'
          )} VND).`,
        timestamp: new Date().toISOString(),
        status: 'APPROVED',
        hitlToken: qItem.hitlToken,
      };

      this.reconciliationSummary.matches.unshift(matchRecord);
      this.reconciliationSummary.quarantined.splice(qIndex, 1);
      this.reconciliationSummary.hitlQuarantineCount = this.reconciliationSummary.quarantined.length;
      this.reconciliationSummary.matchedCount++;
      this.reconciliationSummary.totalFeeDisentangled += allocatedFee;

      if (this.reconciliationSummary.totalBankTransactions > 0) {
        this.reconciliationSummary.matchRate =
          Math.round(
            (this.reconciliationSummary.matchedCount / this.reconciliationSummary.totalBankTransactions) * 10000
          ) / 100;
      }

      this.quarantineActionLogs.unshift({
        txId,
        action: 'ALLOCATE_FEE',
        timestamp: new Date().toISOString(),
        details: remarks || `Hạch toán phí TK 6425: ${allocatedFee} VND`,
      });

      return true;
    },

    resolveEscalateToChecker(txId: string, escalationRemarks: string): boolean {
      if (!this.reconciliationSummary) return false;

      const qItem = this.reconciliationSummary.quarantined.find((q) => q.txId === txId);
      if (!qItem) return false;

      this.escalatedTxMap[txId] = {
        remarks: escalationRemarks,
        escalatedAt: new Date().toISOString(),
      };

      qItem.reason = `[ĐÃ CHUYỂN KIỂM SOÁT VIÊN XỬ LÝ]: ${escalationRemarks}`;

      this.quarantineActionLogs.unshift({
        txId,
        action: 'ESCALATE',
        timestamp: new Date().toISOString(),
        details: escalationRemarks,
      });

      return true;
    },
  },
});
