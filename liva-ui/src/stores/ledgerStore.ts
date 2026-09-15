import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '../utils/logger';
import { useWorkbenchStore } from './workbenchStore';

export type ErpSystem = 'MISA' | 'FAST' | 'BRAVO' | 'SAP';
export type VoucherPostingStatus = 'DRAFT' | 'PENDING_POST' | 'POSTED_TO_ERP' | 'SYNC_FAILED';

export interface JournalVoucherLine {
  id: string;
  accountCode: string;
  accountName: string;
  postingType: 'DEBIT' | 'CREDIT';
  amount: number; // Integer VND
  partnerName?: string;
  memo?: string;
}

export interface JournalVoucher {
  id: string;
  voucherNumber: string;
  voucherDate: string;
  voucherGuid: string; // Idempotency key (UUIDv4)
  batchId: string;
  matchId?: string;
  sourceType: 'RECONCILIATION_MATCH' | 'QUARANTINE_RELEASE' | 'MANUAL';
  targetErp: ErpSystem;
  partnerName: string;
  partnerTaxId?: string;
  description: string;
  totalDebit: number;
  totalCredit: number;
  difference: number; // Must always be 0
  status: VoucherPostingStatus;
  lines: JournalVoucherLine[];
  idempotencyHash: string;
  erpReference?: string;
  postedAt?: string;
  errorMessage?: string;
  retryCount: number;
}

export interface ReconciliationCertificateData {
  certificateId: string;
  bankAccount: string;
  periodStart: string;
  periodEnd: string;
  bankOpeningBalance: number;
  bankTotalCredits: number;
  bankTotalDebits: number;
  bankClosingBalance: number;
  glOpeningBalance: number;
  glTotalDebits: number;
  glTotalCredits: number;
  glClosingBalance: number;
  inTransitDeposits: number;
  inTransitWithdrawals: number;
  adjustedBankBalance: number;
  variance: number; // Must be 0
  isCertified: boolean;
  merkleRootHash: string;
  certifiedByChecker?: string;
  certifiedAt?: string;
}

export const useLedgerStore = defineStore('ledger', () => {
  const workbenchStore = useWorkbenchStore();

  const vouchers = ref<JournalVoucher[]>([]);
  const selectedVoucherId = ref<string | null>(null);
  const activePayloadVoucher = ref<JournalVoucher | null>(null);
  const isPayloadModalOpen = ref(false);

  // Filters
  const filterStatus = ref<string>('ALL');
  const filterErp = ref<string>('ALL');
  const searchQuery = ref('');

  // Active ERP target selection for posting
  const defaultErp = ref<ErpSystem>('MISA');

  // P53 Certificate state
  const certificate = ref<ReconciliationCertificateData>({
    certificateId: 'CERT-20260915-VCB-01',
    bankAccount: 'VCB-00710009821',
    periodStart: '2026-09-01',
    periodEnd: '2026-09-15',
    bankOpeningBalance: 300000000,
    bankTotalCredits: 88250000,
    bankTotalDebits: 9988000,
    bankClosingBalance: 378262000,
    glOpeningBalance: 300000000,
    glTotalDebits: 88250000,
    glTotalCredits: 9988000,
    glClosingBalance: 378262000,
    inTransitDeposits: 0,
    inTransitWithdrawals: 0,
    adjustedBankBalance: 378262000,
    variance: 0,
    isCertified: true,
    merkleRootHash: '0x8f3c1a89b4e056d7289b4f2c0199e82a17cb09f3e451b6817cd89320e6a4b123',
    certifiedByChecker: 'USR-CHECKER-KTT-01',
    certifiedAt: '2026-09-15T18:30:00Z',
  });

  // --- Computed ---
  const filteredVouchers = computed(() => {
    return vouchers.value.filter((v) => {
      if (filterStatus.value !== 'ALL' && v.status !== filterStatus.value) return false;
      if (filterErp.value !== 'ALL' && v.targetErp !== filterErp.value) return false;
      if (searchQuery.value.trim()) {
        const q = searchQuery.value.toLowerCase();
        return (
          v.voucherNumber.toLowerCase().includes(q) ||
          v.partnerName.toLowerCase().includes(q) ||
          v.description.toLowerCase().includes(q) ||
          v.voucherGuid.toLowerCase().includes(q)
        );
      }
      return true;
    });
  });

  const selectedVoucher = computed(() => {
    if (!selectedVoucherId.value) return null;
    return vouchers.value.find((v) => v.id === selectedVoucherId.value) || null;
  });

  const voucherStats = computed(() => {
    const total = vouchers.value.length;
    const posted = vouchers.value.filter((v) => v.status === 'POSTED_TO_ERP').length;
    const pending = vouchers.value.filter((v) => v.status === 'DRAFT' || v.status === 'PENDING_POST').length;
    const failed = vouchers.value.filter((v) => v.status === 'SYNC_FAILED').length;
    const totalAmount = vouchers.value.reduce((acc, v) => acc + v.totalDebit, 0);

    return {
      total,
      posted,
      pending,
      failed,
      totalAmount,
      syncRate: total > 0 ? Math.round((posted / total) * 100) : 0,
    };
  });

  // --- Actions ---

  function selectVoucher(id: string | null) {
    selectedVoucherId.value = id;
  }

  function openPayloadModal(voucher: JournalVoucher) {
    activePayloadVoucher.value = voucher;
    isPayloadModalOpen.value = true;
  }

  function closePayloadModal() {
    isPayloadModalOpen.value = false;
    activePayloadVoucher.value = null;
  }

  /**
   * Sinh mã băm chống trùng lặp (Deterministic Idempotency Hash)
   */
  function generateIdempotencyHash(guid: string, batch: string, amount: number): string {
    let hash = 0;
    const str = `${guid}:${batch}:${amount}`;
    for (let i = 0; i < str.length; i++) {
      hash = (hash << 5) - hash + str.charCodeAt(i);
      hash |= 0;
    }
    const hex = Math.abs(hash).toString(16).padStart(16, '0');
    return `0x${hex}${hex}`;
  }

  /**
   * P50: Sinh các đề xuất bút toán đối ứng từ kết quả đối soát P42 & P44
   */
  function generateVouchersFromWorkbench() {
    const newVouchers: JournalVoucher[] = [];
    const date = new Date().toISOString().split('T')[0];

    // 1. Sinh bút toán cho các cặp matches đã CONFIRMED trong workbench
    workbenchStore.matches.forEach((m, idx) => {
      const bLine = m.bankLines[0];
      const gLine = m.glRecords[0];
      if (!bLine) return;

      const guid = typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID() : `guid-match-${idx}-${Date.now()}`;
      const voucherNo = `PKT-${bLine.bankCode}-${bLine.txCode.slice(-4)}`;
      const amount = Math.abs(bLine.amount);

      let lines: JournalVoucherLine[] = [];

      if (m.feeAllocated > 0) {
        // Có tách phí ngân hàng (TK 6425)
        lines = [
          {
            id: `line-${guid}-1`,
            accountCode: '1121',
            accountName: 'Tiền gửi ngân hàng',
            postingType: 'DEBIT',
            amount: amount,
            memo: bLine.memo,
          },
          {
            id: `line-${guid}-2`,
            accountCode: '6425',
            accountName: 'Chi phí dịch vụ ngân hàng',
            postingType: 'DEBIT',
            amount: m.feeAllocated,
            memo: 'Phí dịch vụ chuyển tiền trừ tự động',
          },
          {
            id: `line-${guid}-3`,
            accountCode: gLine?.accountCode || '131',
            accountName: gLine?.accountCode === '331' ? 'Phải trả người bán' : 'Phải thu khách hàng',
            postingType: 'CREDIT',
            amount: amount + m.feeAllocated,
            partnerName: gLine?.partnerName,
          },
        ];
      } else if (m.tier === 'TIER_3_SPLIT') {
        // Gộp 1:N hóa đơn
        lines.push({
          id: `line-${guid}-1`,
          accountCode: '1121',
          accountName: 'Tiền gửi ngân hàng',
          postingType: 'DEBIT',
          amount: amount,
          memo: bLine.memo,
        });
        m.glRecords.forEach((g, gIdx) => {
          lines.push({
            id: `line-${guid}-${gIdx + 2}`,
            accountCode: g.accountCode || '131',
            accountName: 'Phải thu khách hàng',
            postingType: 'CREDIT',
            amount: Math.abs(g.amount),
            partnerName: g.partnerName,
            memo: g.description,
          });
        });
      } else {
        // 1:1 Chuẩn
        const isReceipt = bLine.amount > 0;
        if (isReceipt) {
          lines = [
            {
              id: `line-${guid}-1`,
              accountCode: '1121',
              accountName: 'Tiền gửi ngân hàng',
              postingType: 'DEBIT',
              amount: amount,
              memo: bLine.memo,
            },
            {
              id: `line-${guid}-2`,
              accountCode: gLine?.accountCode || '131',
              accountName: 'Phải thu khách hàng',
              postingType: 'CREDIT',
              amount: amount,
              partnerName: gLine?.partnerName,
              memo: gLine?.description,
            },
          ];
        } else {
          lines = [
            {
              id: `line-${guid}-1`,
              accountCode: gLine?.accountCode || '331',
              accountName: 'Phải trả người bán',
              postingType: 'DEBIT',
              amount: amount,
              partnerName: gLine?.partnerName,
              memo: gLine?.description,
            },
            {
              id: `line-${guid}-2`,
              accountCode: '1121',
              accountName: 'Tiền gửi ngân hàng',
              postingType: 'CREDIT',
              amount: amount,
              memo: bLine.memo,
            },
          ];
        }
      }

      const totalDebit = lines.filter((l) => l.postingType === 'DEBIT').reduce((acc, l) => acc + l.amount, 0);
      const totalCredit = lines.filter((l) => l.postingType === 'CREDIT').reduce((acc, l) => acc + l.amount, 0);

      newVouchers.push({
        id: `v-${idx + 1}`,
        voucherNumber: voucherNo,
        voucherDate: date,
        voucherGuid: guid,
        batchId: workbenchStore.activeBatchId,
        matchId: m.id,
        sourceType: 'RECONCILIATION_MATCH',
        targetErp: defaultErp.value,
        partnerName: gLine?.partnerName || 'Đối tác hạch toán đối soát',
        partnerTaxId: gLine?.taxId,
        description: `Hạch toán bù trừ đối soát ${m.tierLabel} - ${bLine.memo}`,
        totalDebit,
        totalCredit,
        difference: totalDebit - totalCredit,
        status: 'DRAFT',
        lines,
        idempotencyHash: generateIdempotencyHash(guid, workbenchStore.activeBatchId, totalDebit),
        retryCount: 0,
      });
    });

    // 2. Sinh bút toán cho các item Quarantine đã APPROVED
    workbenchStore.quarantineItems
      .filter((q) => q.status === 'APPROVED')
      .forEach((q, qIdx) => {
        const guid = typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID() : `guid-q-${qIdx}-${Date.now()}`;
        const voucherNo = `PKT-C-LY-${q.txCode.slice(-4)}`;
        const amount = Math.abs(q.amount);
        const glAcc = q.proposedGlAccount || '6425';

        const lines: JournalVoucherLine[] = [
          {
            id: `line-${guid}-1`,
            accountCode: q.amount > 0 ? '1121' : glAcc,
            accountName: q.amount > 0 ? 'Tiền gửi ngân hàng' : `Tài khoản đối ứng (${glAcc})`,
            postingType: 'DEBIT',
            amount: amount,
            memo: q.memo,
          },
          {
            id: `line-${guid}-2`,
            accountCode: q.amount > 0 ? glAcc : '1121',
            accountName: q.amount > 0 ? `Tài khoản đối ứng (${glAcc})` : 'Tiền gửi ngân hàng',
            postingType: 'CREDIT',
            amount: amount,
            memo: q.checkerDecisionNote || q.makerProposal,
          },
        ];

        const totalDebit = lines.filter((l) => l.postingType === 'DEBIT').reduce((acc, l) => acc + l.amount, 0);
        const totalCredit = lines.filter((l) => l.postingType === 'CREDIT').reduce((acc, l) => acc + l.amount, 0);

        newVouchers.push({
          id: `v-q-${qIdx + 1}`,
          voucherNumber: voucherNo,
          voucherDate: date,
          voucherGuid: guid,
          batchId: workbenchStore.activeBatchId,
          sourceType: 'QUARANTINE_RELEASE',
          targetErp: defaultErp.value,
          partnerName: 'Giải phóng hàng đợi cách ly P44',
          description: `Bút toán xử lý cách ly ${q.exceptionType}: ${q.checkerDecisionNote || q.exceptionReason}`,
          totalDebit,
          totalCredit,
          difference: totalDebit - totalCredit,
          status: 'DRAFT',
          lines,
          idempotencyHash: generateIdempotencyHash(guid, workbenchStore.activeBatchId, totalDebit),
          retryCount: 0,
        });
      });

    vouchers.value = newVouchers;
    logger.info(`Đã tổng hợp ${newVouchers.length} chứng từ bút toán tự động từ đối soát`);
  }

  /**
   * P51/P52: Đẩy 1 chứng từ sang hệ thống ERP với Idempotency Key
   */
  async function postVoucherToErp(voucherId: string, erp?: ErpSystem) {
    const v = vouchers.value.find((item) => item.id === voucherId);
    if (!v) throw new Error(`Không tìm thấy chứng từ ${voucherId}`);

    if (v.difference !== 0) {
      throw new Error(`Chứng từ ${v.voucherNumber} chưa cân Nợ/Có (${v.difference} ₫), không thể đồng bộ ERP!`);
    }

    v.targetErp = erp || v.targetErp;
    v.status = 'PENDING_POST';

    // Simulate API posting with deterministic success
    await new Promise((res) => setTimeout(res, 300));

    v.status = 'POSTED_TO_ERP';
    v.erpReference = `${v.targetErp}-VOUCHER-${Date.now().toString().slice(-6)}`;
    v.postedAt = new Date().toISOString();
    v.errorMessage = undefined;

    logger.info(`Chứng từ ${v.voucherNumber} đã ghi sổ thành công sang ${v.targetErp} với mã tham chiếu ${v.erpReference}`);
  }

  /**
   * P52: Đồng bộ hàng loạt sang ERP
   */
  async function batchPostToErp(voucherIds?: string[], erp?: ErpSystem) {
    const targets = voucherIds
      ? vouchers.value.filter((v) => voucherIds.includes(v.id))
      : vouchers.value.filter((v) => v.status === 'DRAFT' || v.status === 'SYNC_FAILED');

    if (targets.length === 0) {
      logger.warn('Không có chứng từ nào cần đồng bộ');
      return 0;
    }

    let successCount = 0;
    for (const v of targets) {
      try {
        await postVoucherToErp(v.id, erp);
        successCount++;
      } catch (err: any) {
        v.status = 'SYNC_FAILED';
        v.errorMessage = err.message;
        v.retryCount++;
      }
    }

    logger.info(`Đã hoàn tất đồng bộ ERP: ${successCount}/${targets.length} chứng từ thành công`);
    return successCount;
  }

  /**
   * P51: Sinh chuỗi JSON chuẩn REST API MISA AMIS
   */
  function getMisaJson(v: JournalVoucher): string {
    const payload = {
      app_id: 'LIVA_BANKING_CORE_V2',
      org_company_code: 'LIVA_ENTERPRISE_VN',
      voucher_guid: v.voucherGuid,
      voucher_no: v.voucherNumber,
      voucher_date: v.voucherDate,
      posted_date: v.voucherDate,
      currency_id: 'VND',
      exchange_rate: 1.0,
      partner_name: v.partnerName,
      partner_tax_id: v.partnerTaxId || null,
      description: v.description,
      total_amount: v.totalDebit,
      idempotency_hash: v.idempotencyHash,
      journal_lines: v.lines.map((l) => ({
        account_code: l.accountCode,
        account_name: l.accountName,
        posting_type: l.postingType,
        amount: l.amount,
        partner_name: l.partnerName || v.partnerName,
        memo: l.memo || v.description,
      })),
    };
    return JSON.stringify(payload, null, 2);
  }

  /**
   * P51: Sinh chuỗi XML chuẩn FAST Business Online
   */
  function getFastXml(v: JournalVoucher): string {
    const linesXml = v.lines
      .map(
        (l) => `    <Line>
      <AccountCode>${l.accountCode}</AccountCode>
      <PostingType>${l.postingType === 'DEBIT' ? 'Nợ' : 'Có'}</PostingType>
      <Amount>${l.amount}</Amount>
      <PartnerName>${l.partnerName || v.partnerName}</PartnerName>
    </Line>`
      )
      .join('\n');

    return `<?xml version="1.0" encoding="UTF-8"?>
<VoucherData xmlns="urn:fast:business:online">
  <Header>
    <VoucherGUID>${v.voucherGuid}</VoucherGUID>
    <VoucherNo>${v.voucherNumber}</VoucherNo>
    <VoucherDate>${v.voucherDate}</VoucherDate>
    <PartnerName>${v.partnerName}</PartnerName>
    <TotalAmount>${v.totalDebit}</TotalAmount>
    <IdempotencyHash>${v.idempotencyHash}</IdempotencyHash>
  </Header>
  <Details>
${linesXml}
  </Details>
</VoucherData>`;
  }

  /**
   * P53: Ký duyệt Biên Bản Đối Chiếu Số Dư Sổ Cái & Sao Kê (Checker Sign-off)
   */
  function certifyCertificate(checkerId: string) {
    if (certificate.value.variance !== 0) {
      throw new Error(`Chênh lệch đối chiếu khác 0 (${certificate.value.variance} ₫), không thể cấp chứng thư số!`);
    }

    certificate.value.isCertified = true;
    certificate.value.certifiedByChecker = checkerId;
    certificate.value.certifiedAt = new Date().toISOString();
    logger.info(`Chứng thư đối chiếu số dư ${certificate.value.certificateId} đã được Kế toán trưởng ký duyệt`);
  }

  // Khởi tạo các bút toán mẫu ban đầu
  generateVouchersFromWorkbench();

  return {
    vouchers,
    selectedVoucherId,
    activePayloadVoucher,
    isPayloadModalOpen,
    filterStatus,
    filterErp,
    searchQuery,
    defaultErp,
    certificate,
    filteredVouchers,
    selectedVoucher,
    voucherStats,
    selectVoucher,
    openPayloadModal,
    closePayloadModal,
    generateVouchersFromWorkbench,
    postVoucherToErp,
    batchPostToErp,
    getMisaJson,
    getFastXml,
    certifyCertificate,
  };
});
