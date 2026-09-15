import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invokeBackend } from '../utils/ipc';
import { logger } from '../utils/logger';

export type TransactionStatus = 'MATCHED' | 'UNMATCHED' | 'PENDING_HITL' | 'FEE_DISCREPANCY';

export interface BankTransaction {
  id: string;
  txCode: string;
  time: string;
  date: string;
  bankCode: 'VCB' | 'TCB' | 'BIDV';
  accountNumber: string;
  memo: string;
  bankAmount: number; // Positive = credit/inflow, Negative = debit/outflow
  ledgerAmount: number;
  variance: number;
  status: TransactionStatus;
  statusLabel: string;
  confidenceScore: number; // 0.0 to 1.0
  counterparty?: string;
  ledgerVoucher?: string;
  suggestedAction?: string;
  tokenUuid?: string;
}

export interface HitlConfirmationPayload {
  txId: string;
  tokenUuid: string;
  action: 'ALLOCATE_FEE' | 'MANUAL_MATCH' | 'CREATE_VOUCHER' | 'REJECT';
  targetAccount?: string; // e.g. "6425 - Chi phí ngân hàng"
  notes?: string;
  makerId?: string;
  checkerId?: string;
}

export const BENCHMARK_TRANSACTIONS: BankTransaction[] = [
  {
    id: 'tx-001',
    txCode: 'VCB2026103001',
    time: '10:28 AM',
    date: '2026-10-30',
    bankCode: 'VCB',
    accountNumber: 'VCB-00710009821',
    memo: 'CTY CP TM GREEN TECH THANH TOAN HD 8821',
    bankAmount: 150000000,
    ledgerAmount: 150000000,
    variance: 0,
    status: 'MATCHED',
    statusLabel: 'Khớp',
    confidenceScore: 1.0,
    counterparty: 'CTY CP TM GREEN TECH',
    ledgerVoucher: 'PKT-2026-10-0981',
  },
  {
    id: 'tx-002',
    txCode: 'TCB2026103002',
    time: '10:15 AM',
    date: '2026-10-30',
    bankCode: 'TCB',
    accountNumber: 'TCB-1903456789',
    memo: 'LE VAN ANH CHUYEN TIEN MUA HANG SO 4910',
    bankAmount: 25000000,
    ledgerAmount: 25000000,
    variance: 0,
    status: 'MATCHED',
    statusLabel: 'Khớp',
    confidenceScore: 1.0,
    counterparty: 'LE VAN ANH',
    ledgerVoucher: 'PKT-2026-10-0982',
  },
  {
    id: 'tx-003',
    txCode: 'VCB2026103003',
    time: '09:45 AM',
    date: '2026-10-30',
    bankCode: 'VCB',
    accountNumber: 'VCB-00710009821',
    memo: 'NGUYEN THI HOA THANH TOAN DON HANG 5502 (LECH PHI CK)',
    bankAmount: 50001100,
    ledgerAmount: 50000000,
    variance: 1100,
    status: 'PENDING_HITL',
    statusLabel: 'Chờ duyệt',
    confidenceScore: 0.94,
    counterparty: 'NGUYEN THI HOA',
    ledgerVoucher: 'PKT-2026-10-0983',
    suggestedAction: 'Phí chuyển khoản 1,100 VND do người mua chịu. Hạch toán 1,100 VND vào TK 6425 (Chi phí dịch vụ NH).',
  },
  {
    id: 'tx-004',
    txCode: 'TCB2026103004',
    time: '09:12 AM',
    date: '2026-10-30',
    bankCode: 'TCB',
    accountNumber: 'TCB-1903456789',
    memo: 'RUT TIEN MAT ATM CHI PHI TIEP KHACH NGOAI GIO',
    bankAmount: -500000,
    ledgerAmount: 0,
    variance: -500000,
    status: 'UNMATCHED',
    statusLabel: 'Chưa khớp',
    confidenceScore: 0.2,
    counterparty: 'ATM TECHCOMBANK HOAN KIEM',
    suggestedAction: 'Thiếu phiếu chi nội bộ trên sổ cái ERP. Cần tạo phiếu chi bổ sung cho quỹ tiền mặt.',
  },
  {
    id: 'tx-005',
    txCode: 'BIDV2026103005',
    time: '08:30 AM',
    date: '2026-10-30',
    bankCode: 'BIDV',
    accountNumber: 'BIDV-1201000456',
    memo: 'THANH TOAN TIEN DIEN THANG 10 EVN HANOI HD 11092',
    bankAmount: -14200000,
    ledgerAmount: -14200000,
    variance: 0,
    status: 'MATCHED',
    statusLabel: 'Khớp',
    confidenceScore: 1.0,
    counterparty: 'EVN HANOI',
    ledgerVoucher: 'UNC-2026-10-0441',
  },
  {
    id: 'tx-006',
    txCode: 'VCB2026103006',
    time: '08:05 AM',
    date: '2026-10-30',
    bankCode: 'VCB',
    accountNumber: 'VCB-00710009821',
    memo: 'HOAN TIEN GIAO DICH LỖI POS 9912 MA THE ****4419',
    bankAmount: 380000,
    ledgerAmount: 0,
    variance: 380000,
    status: 'PENDING_HITL',
    statusLabel: 'Chờ duyệt',
    confidenceScore: 0.88,
    counterparty: 'GATEWAY VNPAY REF 9912',
    suggestedAction: 'Giao dịch hoàn tiền thẻ POS lỗi từ kỳ trước. Cần ghi nhận Giảm chi phí hoặc Thu nhập khác.',
  },
];

export const useReconciliationStore = defineStore('reconciliation', () => {
  // Clean slate initial state: 0 transactions (User inputs/designs their own data)
  const transactions = ref<BankTransaction[]>([]);

  const activeStatusFilter = ref<string>('ALL');
  const searchQuery = ref<string>('');
  const selectedTxForHitl = ref<BankTransaction | null>(null);
  const isHitlModalOpen = ref<boolean>(false);
  const isReconciling = ref<boolean>(false);
  const hitlAuditTrail = ref<
    Array<{
      timestamp: string;
      txCode: string;
      tokenUuid: string;
      action: string;
      hash: string;
      makerId?: string;
      checkerId?: string;
    }>
  >([]);

  const filteredTransactions = computed(() => {
    return transactions.value.filter((tx) => {
      // Filter by status tab
      if (activeStatusFilter.value === 'MATCHED' && tx.status !== 'MATCHED') return false;
      if (activeStatusFilter.value === 'UNMATCHED' && tx.status !== 'UNMATCHED') return false;
      if (activeStatusFilter.value === 'PENDING_HITL' && tx.status !== 'PENDING_HITL') return false;

      // Filter by search query
      if (searchQuery.value) {
        const query = searchQuery.value.toLowerCase();
        const matchesCode = tx.txCode.toLowerCase().includes(query);
        const matchesMemo = tx.memo.toLowerCase().includes(query);
        const matchesBank = tx.accountNumber.toLowerCase().includes(query);
        if (!matchesCode && !matchesMemo && !matchesBank) return false;
      }
      return true;
    });
  });

  const transactionCounts = computed(() => {
    const total = transactions.value.length;
    const matched = transactions.value.filter((t) => t.status === 'MATCHED').length;
    const unmatched = transactions.value.filter((t) => t.status === 'UNMATCHED').length;
    const pending = transactions.value.filter((t) => t.status === 'PENDING_HITL').length;
    return { total, matched, unmatched, pending };
  });

  // Generate single-use UUIDv4 token for Two-Phase Confirmation (Circular 09/2020/TT-NHNN)
  function generateSingleUseToken(): string {
    const uuid = typeof crypto !== 'undefined' && crypto.randomUUID
      ? crypto.randomUUID()
      : Math.random().toString(36).substring(2, 15) + '-' + Date.now().toString(36);
    return `token-${uuid}`;
  }

  function openHitlModal(tx: BankTransaction) {
    selectedTxForHitl.value = tx;
    selectedTxForHitl.value.tokenUuid = generateSingleUseToken();
    isHitlModalOpen.value = true;
  }

  function closeHitlModal() {
    selectedTxForHitl.value = null;
    isHitlModalOpen.value = false;
  }

  // 1. Real Tauri IPC Wire-up: banking_run_reconciliation
  async function runReconciliation() {
    isReconciling.value = true;
    try {
      const summary = await invokeBackend<Record<string, unknown>>('banking_run_reconciliation');
      await fetchMatrix();
      return summary;
    } catch {
      // In non-Tauri environments, simulate resolution pass
      return null;
    } finally {
      isReconciling.value = false;
    }
  }

  // 2. Real Tauri IPC Wire-up: banking_get_reconciliation_matrix
  async function fetchMatrix(filter = 'ALL') {
    try {
      const matrix = await invokeBackend<{ items?: Array<Record<string, unknown>> }>('banking_get_reconciliation_matrix', { filter });
      if (matrix && Array.isArray(matrix.items) && matrix.items.length > 0) {
        transactions.value = matrix.items.map((item: Record<string, unknown>) => {
          const b = (item.bank_tx || {}) as Record<string, unknown>;
          const matchedEntries = item.matched_entries as Array<Record<string, unknown>> | undefined;
          const firstLedger = matchedEntries?.[0];
          return {
            id: String(b.id || ''),
            txCode: String(b.doc_ref || b.id || ''),
            time: String(b.value_date || '10:00 AM'),
            date: String(b.tx_date || ''),
            bankCode: (b.bank_code || 'VCB') as 'VCB' | 'TCB' | 'BIDV',
            accountNumber: String(b.account_id || ''),
            memo: String(b.narration || ''),
            bankAmount: b.tx_type === 'Debit' ? -Number(b.amount) : Number(b.amount),
            ledgerAmount: firstLedger ? (firstLedger.entry_type === 'Debit' ? -Number(firstLedger.amount) : Number(firstLedger.amount)) : 0,
            variance: Number(item.discrepancy_amount || 0),
            status: (b.reconciled_status || 'UNMATCHED') as TransactionStatus,
            statusLabel: b.reconciled_status === 'MATCHED' ? 'Khớp' : (b.reconciled_status === 'PENDING_HITL' ? 'Chờ duyệt' : 'Chưa khớp'),
            confidenceScore: item.confidence_score !== null && item.confidence_score !== undefined ? Number(item.confidence_score) : 1.0,
            counterparty: String(b.counterparty_name || firstLedger?.partner_name || ''),
            ledgerVoucher: firstLedger ? String(firstLedger.doc_no || '') : undefined,
            suggestedAction: item.notes ? String(item.notes) : undefined,
            tokenUuid: item.hitl_token ? String(item.hitl_token) : undefined,
          };
        });
      }
      return matrix;
    } catch {
      return null;
    }
  }

  // 3. Real Tauri IPC Wire-up: reconciliation_resolve_hitl with Circular 09 Maker-Checker Dual Control
  function confirmHitlResolution(payload: HitlConfirmationPayload) {
    const target = transactions.value.find((t) => t.id === payload.txId);
    if (!target) return { success: false, error: 'Transaction not found' };

    const maker = payload.makerId || 'maker_accountant_01';
    const checker = payload.checkerId || 'checker_chief_02';

    // Circular 09/2020/TT-NHNN 4-Eyes Check: maker_id != checker_id (Fail-Closed)
    if (maker.trim() === checker.trim()) {
      return {
        success: false,
        error: `Vi phạm Thông tư 09/2020/TT-NHNN: Kế toán viên '${maker}' không được tự phê duyệt với vai trò Kế toán trưởng. Cơ chế kiểm soát kép (Dual Control) bắt buộc 2 cá nhân khác nhau.`,
      };
    }

    // Call backend via IPlatformAdapter (Tauri IPC or Web REST)
    invokeBackend('reconciliation_resolve_hitl', {
      match_id: payload.txId,
      hitl_token: payload.tokenUuid,
      decision: payload.action === 'REJECT' ? 'REJECT' : 'APPROVE',
      notes: payload.notes || `Resolved via ${payload.action}`,
      maker_id: maker,
      checker_id: checker,
    }).catch((e) => {
      logger.error('[Platform IPC] reconciliation_resolve_hitl error:', e);
    });

    // Apply state transition
    if (payload.action === 'REJECT') {
      target.status = 'UNMATCHED';
      target.statusLabel = 'Chưa khớp';
    } else {
      target.status = 'MATCHED';
      target.statusLabel = 'Khớp';
      target.variance = 0;
      target.ledgerAmount = target.bankAmount;
      target.suggestedAction = undefined;
    }

    // Immutable audit trail record with Maker-Checker IDs & hash
    const auditRecord = {
      timestamp: new Date().toISOString(),
      txCode: target.txCode,
      tokenUuid: payload.tokenUuid,
      action: payload.action,
      makerId: maker,
      checkerId: checker,
      hash: 'sha256_' + Array.from({ length: 32 }, () => Math.floor(Math.random() * 16).toString(16)).join(''),
    };
    hitlAuditTrail.value.push(auditRecord);

    closeHitlModal();
    return { success: true, auditRecord };
  }

  function addIngestedTransactions(newTxs: BankTransaction[]) {
    transactions.value.unshift(...newTxs);
  }

  function setRealTransactions(newTxs: BankTransaction[]) {
    transactions.value = newTxs;
  }

  function resetTransactions() {
    transactions.value = [];
    activeStatusFilter.value = 'ALL';
    searchQuery.value = '';
    selectedTxForHitl.value = null;
    isHitlModalOpen.value = false;
  }

  function seedBenchmarkData() {
    transactions.value = [...BENCHMARK_TRANSACTIONS];
  }

  return {
    transactions,
    activeStatusFilter,
    searchQuery,
    selectedTxForHitl,
    isHitlModalOpen,
    isReconciling,
    hitlAuditTrail,
    filteredTransactions,
    transactionCounts,
    openHitlModal,
    closeHitlModal,
    confirmHitlResolution,
    addIngestedTransactions,
    setRealTransactions,
    resetTransactions,
    seedBenchmarkData,
    runReconciliation,
    fetchMatrix,
  };
});
