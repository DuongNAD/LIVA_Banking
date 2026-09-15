import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '../utils/logger';

export type MatchTier = 'TIER_1_EXACT' | 'TIER_2_FUZZY' | 'TIER_3_SPLIT' | 'MANUAL';
export type MatchStatus = 'PROPOSED' | 'CONFIRMED' | 'QUARANTINED';
export type UnmatchReasonCode = 'SAI_DOI_TAC' | 'LECH_THOI_GIAN' | 'LECH_PHI' | 'HOA_DON_HUY' | 'TRUNG_LAP' | 'KHAC';

export type QuarantineExceptionType = 'AML_HIGH_VALUE' | 'FEE_OVER_LIMIT' | 'UNKNOWN_PARTNER' | 'DUPLICATE_REF' | 'REVERSAL_SUSPECT';
export type QuarantineStatus = 'PENDING_MAKER' | 'SUBMITTED_TO_CHECKER' | 'APPROVED' | 'REJECTED' | 'EXPIRED';

export interface QuarantineItem {
  id: string;
  txCode: string;
  date: string;
  time: string;
  bankCode: 'VCB' | 'TCB' | 'BIDV' | 'MBB';
  accountNumber: string;
  memo: string;
  amount: number;
  exceptionType: QuarantineExceptionType;
  exceptionReason: string;
  severity: 'CRITICAL' | 'HIGH' | 'MEDIUM';
  status: QuarantineStatus;
  makerId?: string;
  makerProposal?: string;
  proposedGlAccount?: string;
  submittedAt?: string;
  checkerId?: string;
  checkerDecisionNote?: string;
  resolvedAt?: string;
  tokenUuid: string;
  createdAt: number;
  ttlSeconds: number;
  hashProof?: string;
}

export interface BankStatementLine {
  id: string;
  txCode: string;
  date: string;
  time: string;
  bankCode: 'VCB' | 'TCB' | 'BIDV' | 'MBB' | 'CTG';
  accountNumber: string;
  memo: string;
  amount: number; // Integer VND: positive = credit, negative = debit
  runningBalance: number;
  isMatched: boolean;
  matchId?: string;
  isQuarantined?: boolean;
}

export interface GlRecord {
  id: string;
  voucherNumber: string;
  date: string;
  erpSource: 'MISA' | 'FAST' | 'BRAVO' | 'SAP';
  partnerName: string;
  taxId?: string;
  description: string;
  amount: number; // Integer VND: positive = debit/receipt, negative = credit/payment
  accountCode: string; // e.g. "1121", "131", "331"
  costCenter?: string;
  isMatched: boolean;
  matchId?: string;
}

export interface MatchCard {
  id: string;
  tier: MatchTier;
  tierLabel: string;
  confidenceScore: number; // 0.0 to 1.0
  bankLines: BankStatementLine[];
  glRecords: GlRecord[];
  netBankAmount: number;
  netGlAmount: number;
  difference: number; // netBankAmount - netGlAmount
  feeAllocated: number; // Amount diverted to TK 6425
  status: MatchStatus;
  notes?: string;
  unmatchHistory?: {
    unmatchedAt: string;
    operatorId: string;
    reasonCode: UnmatchReasonCode;
    reasonNote: string;
    hashProof: string;
  }[];
}

export interface SplitSolverResult {
  targetAmount: number;
  combinations: {
    glRecords: GlRecord[];
    totalAmount: number;
    difference: number;
    confidenceScore: number;
  }[];
}

export const useWorkbenchStore = defineStore('workbench', () => {
  const activeBatchId = ref('BATCH-20260915-VCB-01');
  const batchStatus = ref<'IN_PROGRESS' | 'SUBMITTED_TO_CHECKER' | 'APPROVED' | 'CLOSED'>('IN_PROGRESS');
  const makerUserId = ref('USR-MAKER-KETOAN-01');
  
  // Columns data
  const bankLines = ref<BankStatementLine[]>([]);
  const glRecords = ref<GlRecord[]>([]);
  const matches = ref<MatchCard[]>([]);

  // Selection states for manual pairing
  const selectedBankLineIds = ref<string[]>([]);
  const selectedGlRecordIds = ref<string[]>([]);

  // Filter & Search states
  const bankSearchQuery = ref('');
  const bankFilterBank = ref<string>('ALL');
  const bankFilterStatus = ref<'ALL' | 'UNMATCHED' | 'MATCHED'>('UNMATCHED');

  const glSearchQuery = ref('');
  const glFilterSource = ref<string>('ALL');
  const glFilterStatus = ref<'ALL' | 'UNMATCHED' | 'MATCHED'>('UNMATCHED');

  const matchFilterTier = ref<string>('ALL');
  const matchFilterDiffOnly = ref(false);

  // Operator Role state (Circular 09 4-Eyes switching simulation)
  const currentOperatorRole = ref<'MAKER' | 'CHECKER'>('MAKER');
  const currentOperatorId = computed(() =>
    currentOperatorRole.value === 'MAKER' ? 'USR-MAKER-KETOAN-01' : 'USR-CHECKER-KTT-01'
  );

  // Quarantine state
  const quarantineItems = ref<QuarantineItem[]>([]);
  const selectedQuarantineId = ref<string | null>(null);
  const quarantineFilterStatus = ref<string>('ALL');
  const quarantineFilterSeverity = ref<string>('ALL');

  // Active modal targets
  const splitSolverTarget = ref<BankStatementLine | null>(null);
  const unmatchTarget = ref<MatchCard | null>(null);

  // --- Computed Views ---
  const filteredQuarantineItems = computed(() => {
    return quarantineItems.value.filter((item) => {
      if (quarantineFilterStatus.value !== 'ALL' && item.status !== quarantineFilterStatus.value) return false;
      if (quarantineFilterSeverity.value !== 'ALL' && item.severity !== quarantineFilterSeverity.value) return false;
      return true;
    });
  });

  const activeQuarantineItem = computed(() => {
    if (!selectedQuarantineId.value) return null;
    return quarantineItems.value.find((item) => item.id === selectedQuarantineId.value) || null;
  });
  const filteredBankLines = computed(() => {
    return bankLines.value.filter((b) => {
      if (bankFilterStatus.value === 'UNMATCHED' && b.isMatched) return false;
      if (bankFilterStatus.value === 'MATCHED' && !b.isMatched) return false;
      if (bankFilterBank.value !== 'ALL' && b.bankCode !== bankFilterBank.value) return false;
      if (bankSearchQuery.value.trim()) {
        const q = bankSearchQuery.value.toLowerCase();
        return (
          b.txCode.toLowerCase().includes(q) ||
          b.memo.toLowerCase().includes(q) ||
          b.amount.toString().includes(q)
        );
      }
      return true;
    });
  });

  const filteredGlRecords = computed(() => {
    return glRecords.value.filter((g) => {
      if (glFilterStatus.value === 'UNMATCHED' && g.isMatched) return false;
      if (glFilterStatus.value === 'MATCHED' && !g.isMatched) return false;
      if (glFilterSource.value !== 'ALL' && g.erpSource !== glFilterSource.value) return false;
      if (glSearchQuery.value.trim()) {
        const q = glSearchQuery.value.toLowerCase();
        return (
          g.voucherNumber.toLowerCase().includes(q) ||
          g.partnerName.toLowerCase().includes(q) ||
          g.description.toLowerCase().includes(q) ||
          g.amount.toString().includes(q)
        );
      }
      return true;
    });
  });

  const filteredMatches = computed(() => {
    return matches.value.filter((m) => {
      if (matchFilterTier.value !== 'ALL' && m.tier !== matchFilterTier.value) return false;
      if (matchFilterDiffOnly.value && m.difference === 0) return false;
      return true;
    });
  });

  const summaryStats = computed(() => {
    const totalBank = bankLines.value.reduce((acc, b) => acc + b.amount, 0);
    const totalGl = glRecords.value.reduce((acc, g) => acc + g.amount, 0);
    const matchedBankCount = bankLines.value.filter((b) => b.isMatched).length;
    const unmatchedBankCount = bankLines.value.length - matchedBankCount;
    const matchRate = bankLines.value.length > 0 ? (matchedBankCount / bankLines.value.length) * 100 : 0;
    const pendingDiscrepancyCount = matches.value.filter((m) => m.difference !== 0 && m.feeAllocated === 0).length;

    return {
      totalBank,
      totalGl,
      matchedBankCount,
      unmatchedBankCount,
      matchRate: Math.round(matchRate * 10) / 10,
      pendingDiscrepancyCount,
      totalMatches: matches.value.length,
      balanceInvariantValid: totalBank === totalGl,
    };
  });

  // --- Actions ---

  function toggleBankSelection(id: string) {
    const idx = selectedBankLineIds.value.indexOf(id);
    if (idx === -1) selectedBankLineIds.value.push(id);
    else selectedBankLineIds.value.splice(idx, 1);
  }

  function toggleGlSelection(id: string) {
    const idx = selectedGlRecordIds.value.indexOf(id);
    if (idx === -1) selectedGlRecordIds.value.push(id);
    else selectedGlRecordIds.value.splice(idx, 1);
  }

  function clearSelections() {
    selectedBankLineIds.value = [];
    selectedGlRecordIds.value = [];
  }

  /**
   * Tạo match thủ công giữa các dòng đã chọn
   */
  function createManualMatch(customBankIds?: string[], customGlIds?: string[]) {
    const bIds = customBankIds || selectedBankLineIds.value;
    const gIds = customGlIds || selectedGlRecordIds.value;

    if (bIds.length === 0 || gIds.length === 0) {
      logger.warn('Cần chọn ít nhất 1 dòng sao kê và 1 chứng từ sổ cái để ghép');
      return null;
    }

    const selectedBanks = bankLines.value.filter((b) => bIds.includes(b.id));
    const selectedGls = glRecords.value.filter((g) => gIds.includes(g.id));

    const netBank = selectedBanks.reduce((acc, b) => acc + b.amount, 0);
    const netGl = selectedGls.reduce((acc, g) => acc + g.amount, 0);
    const diff = netBank - netGl;

    const matchId = `MATCH-MANUAL-${Date.now()}`;
    const newMatch: MatchCard = {
      id: matchId,
      tier: 'MANUAL',
      tierLabel: 'Ghép thủ công (Maker)',
      confidenceScore: 1.0,
      bankLines: selectedBanks,
      glRecords: selectedGls,
      netBankAmount: netBank,
      netGlAmount: netGl,
      difference: diff,
      feeAllocated: 0,
      status: 'CONFIRMED',
      notes: `Người dùng ${makerUserId.value} ghép thủ công ${bIds.length}:${gIds.length}`,
    };

    // Đánh dấu đã match
    selectedBanks.forEach((b) => {
      b.isMatched = true;
      b.matchId = matchId;
    });
    selectedGls.forEach((g) => {
      g.isMatched = true;
      g.matchId = matchId;
    });

    matches.value.unshift(newMatch);
    clearSelections();
    logger.info(`Đã tạo match thủ công ${matchId}`);
    return newMatch;
  }

  /**
   * Tách phí ngân hàng tự động (P46: 1.100đ - 22.000đ sang TK 6425)
   */
  function autoSplitFee(matchId: string) {
    const target = matches.value.find((m) => m.id === matchId);
    if (!target) return;

    const absDiff = Math.abs(target.difference);
    if (absDiff >= 1100 && absDiff <= 22000) {
      target.feeAllocated = absDiff;
      target.difference = 0;
      target.notes = (target.notes ? target.notes + ' | ' : '') + `Đã tách phí ngân hàng ${absDiff.toLocaleString('vi-VN')} VND sang TK 6425`;
      logger.info(`Đã tách phí ngân hàng cho match ${matchId}: ${absDiff} VND`);
    } else {
      logger.warn(`Chênh lệch ${absDiff} VND nằm ngoài dải phí quy định (1.100 - 22.000 VND)`);
    }
  }

  /**
   * Hủy match có lý do bắt buộc và ghi nhận hash audit
   */
  function unmatchWithReason(matchId: string, reasonCode: UnmatchReasonCode, reasonNote: string) {
    const idx = matches.value.findIndex((m) => m.id === matchId);
    if (idx === -1) return;

    const target = matches.value[idx];
    
    // Tạo hash proof mô phỏng chuỗi hash-chain
    const timestamp = new Date().toISOString();
    const hashProof = `hash-${Date.now().toString(16)}-${Math.random().toString(16).slice(2, 10)}`;

    // Trả lại trạng thái chưa match
    target.bankLines.forEach((b) => {
      const found = bankLines.value.find((item) => item.id === b.id);
      if (found) {
        found.isMatched = false;
        found.matchId = undefined;
      }
    });

    target.glRecords.forEach((g) => {
      const found = glRecords.value.find((item) => item.id === g.id);
      if (found) {
        found.isMatched = false;
        found.matchId = undefined;
      }
    });

    // Ghi nhận lịch sử audit
    if (!target.unmatchHistory) target.unmatchHistory = [];
    target.unmatchHistory.push({
      unmatchedAt: timestamp,
      operatorId: makerUserId.value,
      reasonCode,
      reasonNote,
      hashProof,
    });

    matches.value.splice(idx, 1);
    logger.info(`Đã hủy match ${matchId} với lý do: ${reasonCode} - ${reasonNote}`);
  }

  /**
   * P45 Split Solver: Tìm tập con GL records (k <= 8) có tổng bằng dòng sao kê
   */
  function solveSubsetSum(targetBank: BankStatementLine, maxK: number = 8): SplitSolverResult {
    const target = Math.abs(targetBank.amount);
    const candidates = glRecords.value.filter((g) => !g.isMatched && Math.abs(g.amount) <= target);

    const combinations: SplitSolverResult['combinations'] = [];

    // Tìm kiếm tổ hợp đơn giản (backtracking bounded k <= 8)
    function backtrack(startIndex: number, currentCombo: GlRecord[], currentSum: number) {
      if (currentCombo.length > maxK) return;

      if (currentSum === target && currentCombo.length > 0) {
        combinations.push({
          glRecords: [...currentCombo],
          totalAmount: currentSum,
          difference: 0,
          confidenceScore: 0.95 - currentCombo.length * 0.02,
        });
        return;
      }

      if (currentSum > target) return;

      for (let i = startIndex; i < candidates.length; i++) {
        if (combinations.length >= 5) break; // Giới hạn 5 phương án tốt nhất
        const item = candidates[i];
        currentCombo.push(item);
        backtrack(i + 1, currentCombo, currentSum + Math.abs(item.amount));
        currentCombo.pop();
      }
    }

    backtrack(0, [], 0);

    return {
      targetAmount: target,
      combinations,
    };
  }

  /**
   * Chấp nhận phương án Split từ Solver
   */
  function applySplitSolution(targetBank: BankStatementLine, gls: GlRecord[]) {
    const matchId = `MATCH-SPLIT-${Date.now()}`;
    const netBank = targetBank.amount;
    const netGl = gls.reduce((acc, g) => acc + g.amount, 0);

    const newMatch: MatchCard = {
      id: matchId,
      tier: 'TIER_3_SPLIT',
      tierLabel: `Split 1:${gls.length} Solver (P45)`,
      confidenceScore: 0.92,
      bankLines: [targetBank],
      glRecords: gls,
      netBankAmount: netBank,
      netGlAmount: netGl,
      difference: netBank - netGl,
      feeAllocated: 0,
      status: 'CONFIRMED',
      notes: `Split Solver tự động khớp 1 GD ngân hàng với ${gls.length} hóa đơn ERP`,
    };

    targetBank.isMatched = true;
    targetBank.matchId = matchId;

    gls.forEach((g) => {
      const found = glRecords.value.find((item) => item.id === g.id);
      if (found) {
        found.isMatched = true;
        found.matchId = matchId;
      }
    });

    matches.value.unshift(newMatch);
    logger.info(`Đã áp dụng Split Solver cho match ${matchId}`);
  }

  /**
   * Trình duyệt batch cho Kế toán trưởng (R04)
   */
  function submitBatchToChecker() {
    if (summaryStats.value.pendingDiscrepancyCount > 0) {
      throw new Error(`Còn ${summaryStats.value.pendingDiscrepancyCount} cặp chênh lệch chưa giải trình hoặc tách phí!`);
    }

    batchStatus.value = 'SUBMITTED_TO_CHECKER';
    logger.info(`Batch ${activeBatchId.value} đã được trình Kế toán trưởng duyệt`);
  }

  // --- P44 Quarantine Queue Actions (Circular 09 Dual Control) ---

  function switchOperatorRole(role: 'MAKER' | 'CHECKER') {
    currentOperatorRole.value = role;
    logger.info(`Đã chuyển đổi vai trò thao tác sang: ${role}`);
  }

  function selectQuarantineItem(id: string | null) {
    selectedQuarantineId.value = id;
  }

  function proposeQuarantineResolution(itemId: string, proposedGlAccount: string, note: string) {
    const item = quarantineItems.value.find((i) => i.id === itemId);
    if (!item) {
      throw new Error(`Không tìm thấy item cách ly ${itemId}`);
    }
    if (currentOperatorRole.value !== 'MAKER') {
      throw new Error('Chỉ người có vai trò Kế toán viên (Maker) mới được đề xuất giải trình cách ly');
    }
    if (!note || note.trim().length < 10) {
      throw new Error('Ghi chú giải trình đề xuất phải có độ dài tối thiểu 10 ký tự');
    }

    const tokenUuid = typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID() : `tok-${Date.now()}`;
    item.status = 'SUBMITTED_TO_CHECKER';
    item.makerId = currentOperatorId.value;
    item.makerProposal = note.trim();
    item.proposedGlAccount = proposedGlAccount;
    item.submittedAt = new Date().toISOString();
    item.tokenUuid = tokenUuid;
    item.createdAt = Date.now();
    item.ttlSeconds = 900;
    // Hash chain proof
    item.hashProof = `0x${Array.from({ length: 64 }, () => Math.floor(Math.random() * 16).toString(16)).join('')}`;

    logger.info(`Item ${itemId} đã được Maker đề xuất giải trình và chuyển Checker duyệt với Token ${tokenUuid}`);
  }

  function approveQuarantineItem(itemId: string, decisionNote: string) {
    const item = quarantineItems.value.find((i) => i.id === itemId);
    if (!item) {
      throw new Error(`Không tìm thấy item cách ly ${itemId}`);
    }
    if (currentOperatorRole.value !== 'CHECKER') {
      throw new Error('Chỉ Kế toán trưởng (Checker) mới có quyền phê duyệt item cách ly');
    }
    // Strict Segregation of Duties (SoD) Invariant: maker_user_id != checker_user_id
    if (item.makerId && item.makerId === currentOperatorId.value) {
      throw new Error(`Vi phạm nguyên tắc 4 mắt (SoD): Người phê duyệt (${currentOperatorId.value}) không được trùng với người đề xuất (${item.makerId})!`);
    }
    if (!decisionNote || decisionNote.trim().length < 10) {
      throw new Error('Lý do phê duyệt phải có độ dài tối thiểu 10 ký tự');
    }

    item.status = 'APPROVED';
    item.checkerId = currentOperatorId.value;
    item.checkerDecisionNote = decisionNote.trim();
    item.resolvedAt = new Date().toISOString();
    item.hashProof = `0x${Array.from({ length: 64 }, () => Math.floor(Math.random() * 16).toString(16)).join('')}`;

    logger.info(`Item ${itemId} đã được Checker (${currentOperatorId.value}) phê duyệt hợp lệ`);
  }

  function rejectQuarantineItem(itemId: string, decisionNote: string) {
    const item = quarantineItems.value.find((i) => i.id === itemId);
    if (!item) {
      throw new Error(`Không tìm thấy item cách ly ${itemId}`);
    }
    if (currentOperatorRole.value !== 'CHECKER') {
      throw new Error('Chỉ Kế toán trưởng (Checker) mới có quyền từ chối item cách ly');
    }
    if (item.makerId && item.makerId === currentOperatorId.value) {
      throw new Error(`Vi phạm nguyên tắc 4 mắt (SoD): Người từ chối (${currentOperatorId.value}) không được trùng với người đề xuất (${item.makerId})!`);
    }
    if (!decisionNote || decisionNote.trim().length < 10) {
      throw new Error('Lý do bác bỏ/từ chối phải có độ dài tối thiểu 10 ký tự');
    }

    item.status = 'REJECTED';
    item.checkerId = currentOperatorId.value;
    item.checkerDecisionNote = decisionNote.trim();
    item.resolvedAt = new Date().toISOString();
    item.hashProof = `0x${Array.from({ length: 64 }, () => Math.floor(Math.random() * 16).toString(16)).join('')}`;

    logger.info(`Item ${itemId} đã bị Checker bác bỏ với lý do: ${decisionNote}`);
  }

  /**
   * Nạp dữ liệu mẫu benchmark cho P42 Workbench & P44 Quarantine
   */
  function seedBenchmarkData() {
    bankLines.value = [
      {
        id: 'bk-01',
        txCode: 'VCB26258912',
        date: '2026-09-15',
        time: '09:15:20',
        bankCode: 'VCB',
        accountNumber: 'VCB-00710009821',
        memo: 'CTY CP AN PHAT THANH TOAN HD 8821 VIETQR NAPAS',
        amount: 50000000,
        runningBalance: 350000000,
        isMatched: false,
      },
      {
        id: 'bk-02',
        txCode: 'VCB26259001',
        date: '2026-09-15',
        time: '10:30:12',
        bankCode: 'VCB',
        accountNumber: 'VCB-00710009821',
        memo: 'CTY CO KHI MINH TAM TT TIEN HANG (TRU PHI 12K)',
        amount: -9988000,
        runningBalance: 340012000,
        isMatched: false,
      },
      {
        id: 'bk-03',
        txCode: 'VCB26259110',
        date: '2026-09-15',
        time: '11:00:45',
        bankCode: 'VCB',
        accountNumber: 'VCB-00710009821',
        memo: 'TAP DOAN THUONG MAI DUONG DONG TT DOT 1 BATCH',
        amount: 30000000,
        runningBalance: 370012000,
        isMatched: false,
      },
      {
        id: 'bk-04',
        txCode: 'TCB26259200',
        date: '2026-09-15',
        time: '11:45:00',
        bankCode: 'TCB',
        accountNumber: 'TCB-1903456789',
        memo: 'BAO HIEM XA HOI QUAN 1 CHI TIEN CHE DO THAI SAN',
        amount: 14500000,
        runningBalance: 124500000,
        isMatched: false,
      },
      {
        id: 'bk-05',
        txCode: 'BIDV26259300',
        date: '2026-09-15',
        time: '14:20:10',
        bankCode: 'BIDV',
        accountNumber: 'BIDV-1201000456',
        memo: 'TIEN LAI TIEN GUI CO KY HAN THANG 09/2026',
        amount: 8250000,
        runningBalance: 408250000,
        isMatched: false,
      },
    ];

    glRecords.value = [
      {
        id: 'gl-01',
        voucherNumber: 'INV-8821',
        date: '2026-09-15',
        erpSource: 'MISA',
        partnerName: 'Công ty Cổ phần An Phát',
        taxId: '0102030405',
        description: 'Thu tiền bán linh kiện điện tử theo hóa đơn 8821',
        amount: 50000000,
        accountCode: '131',
        isMatched: false,
      },
      {
        id: 'gl-02',
        voucherNumber: 'INV-8829',
        date: '2026-09-14',
        erpSource: 'MISA',
        partnerName: 'Công ty CP Cơ Khí Minh Tâm',
        taxId: '0301020304',
        description: 'Thanh toán tiền gia công khuôn đúc cơ khí đợt 2',
        amount: -10000000,
        accountCode: '331',
        isMatched: false,
      },
      {
        id: 'gl-03',
        voucherNumber: 'INV-8835',
        date: '2026-09-15',
        erpSource: 'FAST',
        partnerName: 'Tập đoàn TM Dương Đông',
        taxId: '0405060708',
        description: 'Hóa đơn dịch vụ kho bãi tháng 08',
        amount: 12000000,
        accountCode: '131',
        isMatched: false,
      },
      {
        id: 'gl-04',
        voucherNumber: 'INV-8836',
        date: '2026-09-15',
        erpSource: 'FAST',
        partnerName: 'Tập đoàn TM Dương Đông',
        taxId: '0405060708',
        description: 'Hóa đơn vận chuyển logistics đường biển',
        amount: 18000000,
        accountCode: '131',
        isMatched: false,
      },
      {
        id: 'gl-05',
        voucherNumber: 'PKT-9901',
        date: '2026-09-15',
        erpSource: 'BRAVO',
        partnerName: 'Bảo hiểm Xã hội TP.HCM',
        description: 'Thu tiền trợ cấp BHXH chế độ người lao động',
        amount: 14500000,
        accountCode: '1388',
        isMatched: false,
      },
    ];

    matches.value = [];
    clearSelections();

    // Tự động sinh Tier 1 proposal ban đầu cho bk-01 và gl-01 (Exact 50,000,000)
    const b01 = bankLines.value[0];
    const g01 = glRecords.value[0];
    const tier1MatchId = 'MATCH-AUTO-TIER1-01';

    b01.isMatched = true;
    b01.matchId = tier1MatchId;
    g01.isMatched = true;
    g01.matchId = tier1MatchId;

    matches.value.push({
      id: tier1MatchId,
      tier: 'TIER_1_EXACT',
      tierLabel: 'Tier 1 (Exact 100%)',
      confidenceScore: 1.0,
      bankLines: [b01],
      glRecords: [g01],
      netBankAmount: b01.amount,
      netGlAmount: g01.amount,
      difference: 0,
      feeAllocated: 0,
      status: 'CONFIRMED',
      notes: 'Khớp chính xác số tiền, số hóa đơn 8821 và đối tác',
    });
    // Dữ liệu cách ly mẫu (P44 Quarantine Benchmark)
    quarantineItems.value = [
      {
        id: 'q-01',
        txCode: 'VCB26259901',
        date: '2026-09-15',
        time: '08:45:10',
        bankCode: 'VCB',
        accountNumber: 'VCB-00710009821',
        memo: 'CTY TNHH DAU TU VA XAY DUNG THINH PHAT CHUYEN TIEN TAM UNG HD GOI THAU XL-05',
        amount: 550000000,
        exceptionType: 'AML_HIGH_VALUE',
        exceptionReason: 'Giao dịch giá trị lớn vượt ngưỡng 400.000.000 VND theo Quyết định 11/2023/QĐ-TTg; yêu cầu kiểm soát AML và phê duyệt kép trước khi ghi sổ cái.',
        severity: 'CRITICAL',
        status: 'PENDING_MAKER',
        tokenUuid: 'tok-aml-20260915-01',
        createdAt: Date.now() - 3600000,
        ttlSeconds: 900,
        hashProof: '0x8f3c1a89b4e056d7289b4f2c0199e82a17cb09f3e451b6817cd89320e6a4b123',
      },
      {
        id: 'q-02',
        txCode: 'TCB26259902',
        date: '2026-09-15',
        time: '09:12:00',
        bankCode: 'TCB',
        accountNumber: 'TCB-1903456789',
        memo: 'THU PHI DICH VU CHUYEN TIEN QUOC TE VA DIEN PHI SWIFT INTERMEDIARY CHARGE',
        amount: -45000,
        exceptionType: 'FEE_OVER_LIMIT',
        exceptionReason: 'Lệch phí ngân hàng trung gian 45.000 VND vượt hạn mức tự động dung sai 22.000 VND.',
        severity: 'MEDIUM',
        status: 'SUBMITTED_TO_CHECKER',
        makerId: 'USR-MAKER-KETOAN-01',
        makerProposal: 'Phí điện báo phát sinh thêm do điện chuyển ngoại tệ đi nước ngoài. Đề nghị hạch toán vào TK 6425 - Chi phí tài chính ngân hàng.',
        proposedGlAccount: '6425',
        submittedAt: new Date(Date.now() - 180000).toISOString(),
        tokenUuid: 'tok-fee-20260915-02',
        createdAt: Date.now() - 180000,
        ttlSeconds: 900,
        hashProof: '0x4a9b8c7d6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a9b',
      },
      {
        id: 'q-03',
        txCode: 'BIDV26259903',
        date: '2026-09-15',
        time: '10:05:30',
        bankCode: 'BIDV',
        accountNumber: 'BIDV-1201000456',
        memo: 'CHUYEN TIEN THANH TOAN TIEN MUA HANG HOA KHONG RO NGUON',
        amount: 125000000,
        exceptionType: 'UNKNOWN_PARTNER',
        exceptionReason: 'Nội dung sao kê không chứa MST, số hợp đồng hay tên đối tác khớp danh mục ERP.',
        severity: 'HIGH',
        status: 'PENDING_MAKER',
        tokenUuid: 'tok-unk-20260915-03',
        createdAt: Date.now() - 7200000,
        ttlSeconds: 900,
        hashProof: '0x7b6a5c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8b7c6d5e4f3a2b1c0d9e8f7a6b',
      },
      {
        id: 'q-04',
        txCode: 'MBB26259904',
        date: '2026-09-15',
        time: '11:30:15',
        bankCode: 'MBB',
        accountNumber: 'MBB-0680100123',
        memo: 'FT26091599812 - TRA TIEN HANG THANG 9 CTY PHAT DAT',
        amount: 18500000,
        exceptionType: 'DUPLICATE_REF',
        exceptionReason: 'Mã tham chiếu FT26091599812 trùng lặp với giao dịch đã đối soát trong kỳ trước.',
        severity: 'HIGH',
        status: 'PENDING_MAKER',
        tokenUuid: 'tok-dup-20260915-04',
        createdAt: Date.now() - 5400000,
        ttlSeconds: 900,
        hashProof: '0x1f2e3d4c5b6a708990a1b2c3d4e5f60718293a4b5c6d7e8f9a0b1c2d3e4f5a6b',
      },
    ];
    selectedQuarantineId.value = 'q-01';
  }

  // Khởi tạo dữ liệu benchmark khi nạp store
  seedBenchmarkData();

  return {
    activeBatchId,
    batchStatus,
    makerUserId,
    currentOperatorRole,
    currentOperatorId,
    bankLines,
    glRecords,
    matches,
    quarantineItems,
    selectedQuarantineId,
    quarantineFilterStatus,
    quarantineFilterSeverity,
    selectedBankLineIds,
    selectedGlRecordIds,
    bankSearchQuery,
    bankFilterBank,
    bankFilterStatus,
    glSearchQuery,
    glFilterSource,
    glFilterStatus,
    matchFilterTier,
    matchFilterDiffOnly,
    splitSolverTarget,
    unmatchTarget,
    filteredBankLines,
    filteredGlRecords,
    filteredMatches,
    filteredQuarantineItems,
    activeQuarantineItem,
    summaryStats,
    toggleBankSelection,
    toggleGlSelection,
    clearSelections,
    createManualMatch,
    autoSplitFee,
    unmatchWithReason,
    solveSubsetSum,
    applySplitSolution,
    submitBatchToChecker,
    switchOperatorRole,
    selectQuarantineItem,
    proposeQuarantineResolution,
    approveQuarantineItem,
    rejectQuarantineItem,
    seedBenchmarkData,
  };
});

