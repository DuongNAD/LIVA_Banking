/**
 * Pinia Store for Treasury Management, Maker-Checker Dual Control & Merkle Audit
 * Adhering to Circular 09/2020/TT-NHNN Articles 16 & 18 & Decree 13/2023/ND-CP
 */

import { defineStore } from 'pinia';
import type {
  PaymentVoucher,
  VoucherStatus,
  AuthMethod,
  AuditLogEntry,
  TourStep,
} from '../types/treasury';
import {
  createPaymentVoucher,
  submitVoucherForApproval,
  approveVoucher as executeApproveVoucher,
  rejectVoucher as executeRejectVoucher,
  settleVoucher as executeSettleVoucher,
  checkVoucherExpiration,
  generateUuidV4,
} from '../engine/treasury/makerChecker';
import {
  ForwardAuditLedger,
  buildMerkleTree,
  computeMerkleLeaf,
} from '../engine/treasury/merkleAudit';
import {
  queryFinancialCopilot,
  queryFinancialCopilotAsync,
  STANDARDIZED_TOUR_STEPS,
} from '../engine/treasury/copilotEngine';
import { executeBankingToolDispatcher } from '../engine/intelligence/bankingAiTools';
import { apiRequest, getApiBaseUrl } from '../services/apiClient';
import { localAiClient, type AiConnectionStatus } from '../services/LocalAiClient';
import { useBankingStore } from './bankingStore';
import { useReconciliationStore } from './reconciliationStore';
import { useAmlStore } from './amlStore';

export interface CopilotMessage {
  id: string;
  sender: 'user' | 'assistant';
  text: string;
  intent?: string;
  metrics?: Record<string, any>;
  timestamp: string;
}

// Initial realistic commercial bank vouchers
function createInitialDemoVouchers(): PaymentVoucher[] {
  const vchApproved: PaymentVoucher = {
    voucherId: 'vch-citad-01',
    makerId: 'maker_accountant_01',
    checkerId: 'checker_cfo_01',
    beneficiaryAccount: '0011000123456',
    beneficiaryBank: 'CITAD_SBV',
    beneficiaryName: 'CỤC QUẢN LÝ DỰ TRỮ NGOẠI HỐI NHNN',
    amountVnd: 500_000_000,
    purpose: 'Nộp bổ sung dự trữ bắt buộc ngoại tệ kỳ II/2026',
    status: 'APPROVED',
    createdAt: new Date(Date.now() - 15 * 60 * 1000).toISOString(),
    submittedAt: new Date(Date.now() - 12 * 60 * 1000).toISOString(),
    approvedAt: new Date(Date.now() - 2 * 60 * 1000).toISOString(),
    authMethod: 'BIOMETRIC_SIM',
    signatureHmac: 'a8b7c9d0e1f234567890abcdef1234567890abcdef1234567890abcdef12345678',
    hitlToken: null,
  };
  vchApproved.merkleLeafHash = computeMerkleLeaf(vchApproved);

  const vchPending: PaymentVoucher = {
    voucherId: 'vch-napas-02',
    makerId: 'maker_accountant_01',
    checkerId: null,
    beneficiaryAccount: 'NAPAS-ACH-247-SETTLE',
    beneficiaryBank: 'NAPAS',
    beneficiaryName: 'CÔNG TY CP THANH TOÁN QUỐC GIA VIỆT NAM (NAPAS)',
    amountVnd: 550_000_000,
    purpose: 'Nạp bổ sung hạn mức ký quỹ thanh toán bù trừ đa phương NAPAS 24/7',
    status: 'PENDING_APPROVAL',
    createdAt: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
    submittedAt: new Date(Date.now() - 4 * 60 * 1000).toISOString(),
    hitlToken: generateUuidV4(),
    tokenExpiresAt: new Date(Date.now() + 11 * 60 * 1000).toISOString(),
  };

  const vchDraft: PaymentVoucher = {
    voucherId: 'vch-nostro-03',
    makerId: 'maker_accountant_02',
    checkerId: null,
    beneficiaryAccount: '1201000888999',
    beneficiaryBank: 'NOSTRO_BILATERAL',
    beneficiaryName: 'TÀI KHOẢN THANH TOÁN SONG PHƯƠNG LIÊN NGÂN HÀNG',
    amountVnd: 145_000_000,
    purpose: 'Quyết toán tra soát bù trừ điện tử liên ngân hàng phiên sáng',
    status: 'DRAFT',
    createdAt: new Date().toISOString(),
  };

  return [vchApproved, vchPending, vchDraft];
}

export const useTreasuryStore = defineStore('treasury', {
  state: () => {
    const auditLedger = new ForwardAuditLedger();
    const initialVouchers = createInitialDemoVouchers();

    // Seed audit ledger entries
    auditLedger.append('SYSTEM_INIT', 'LEDGER_INITIALIZED', { node: 'liva-bank-node-01', version: '2.0.0' });
    auditLedger.append('maker_accountant_01', 'VOUCHER_CREATED', { voucherId: 'vch-citad-01', amountVnd: 500_000_000 });
    auditLedger.append('checker_cfo_01', 'VOUCHER_APPROVED', {
      voucherId: 'vch-citad-01',
      amountVnd: 500_000_000,
      authMethod: 'BIOMETRIC_SIM',
      leafHash: initialVouchers[0].merkleLeafHash,
    });
    auditLedger.append('maker_accountant_01', 'VOUCHER_SUBMITTED', { voucherId: 'vch-napas-02', amountVnd: 550_000_000 });

    return {
      vouchers: initialVouchers as PaymentVoucher[],
      activeVoucherId: null as string | null,
      isMakerCheckerModalOpen: false as boolean,
      auditLedger: auditLedger as ForwardAuditLedger,
      currentUserId: 'checker_cfo_01' as string,
      currentUserName: 'Trần Thị Kiểm Soát (Kiểm Soát Viên Trưởng / Checker)' as string,
      currentUserRole: 'CHECKER' as 'MAKER' | 'CHECKER' | 'ADMIN',

      // Copilot 2D State
      isCopilotOpen: false as boolean,
      copilotMessages: [
        {
          id: 'msg-init',
          sender: 'assistant',
          text: 'Xin chào! Tôi là LIVA Financial Copilot. Tôi có thể hỗ trợ tra cứu thanh khoản ngân quỹ, tiến độ đối soát 3 tầng, hoặc kiểm tra các lệnh chi chờ duyệt.',
          timestamp: new Date().toISOString(),
        },
      ] as CopilotMessage[],

      // Guided Tour State
      isTourActive: false as boolean,
      tourStepIndex: 0 as number,
      tourSecondsRemaining: 60 as number,
      tourSteps: STANDARDIZED_TOUR_STEPS as TourStep[],
      tourTimerId: null as any,

      // Real-time Sync & Server State (Milestone M3)
      isOnlineSyncActive: false as boolean,
      sseConnectionStatus: 'DISCONNECTED' as 'CONNECTED' | 'DISCONNECTED' | 'CONNECTING',
      sseEventSource: null as any,

      // Local AI Connection Status (Milestone M2)
      aiConnectionStatus: 'offline' as AiConnectionStatus,
      isCopilotThinking: false as boolean,
    };
  },

  getters: {
    isOnlineSync: (state) => state.isOnlineSyncActive,
    pendingVouchers: (state) => state.vouchers.filter((v) => v.status === 'PENDING_APPROVAL'),
    approvedVouchers: (state) => state.vouchers.filter((v) => v.status === 'APPROVED'),
    rejectedVouchers: (state) => state.vouchers.filter((v) => v.status === 'REJECTED'),
    draftVouchers: (state) => state.vouchers.filter((v) => v.status === 'DRAFT'),
    settledVouchers: (state) => state.vouchers.filter((v) => v.status === 'SETTLED'),

    totalDisbursedVnd: (state) =>
      state.vouchers
        .filter((v) => v.status === 'APPROVED' || v.status === 'SETTLED')
        .reduce((sum, v) => sum + v.amountVnd, 0),

    totalPendingVnd: (state) =>
      state.vouchers
        .filter((v) => v.status === 'PENDING_APPROVAL')
        .reduce((sum, v) => sum + v.amountVnd, 0),

    activeVoucher: (state) =>
      state.vouchers.find((v) => v.voucherId === state.activeVoucherId) || null,

    merkleRoot: (state) => {
      const approvedLeaves = state.vouchers
        .filter((v) => v.status === 'APPROVED' || v.status === 'SETTLED')
        .map((v) => v.merkleLeafHash || computeMerkleLeaf(v));
      return buildMerkleTree(approvedLeaves).root;
    },

    auditEntries: (state): AuditLogEntry[] => state.auditLedger.getEntries(),

    currentTourStep: (state): TourStep =>
      state.tourSteps[state.tourStepIndex] || state.tourSteps[0],
  },

  actions: {
    setCurrentUser(userId: string, role: 'MAKER' | 'CHECKER' | 'ADMIN', name?: string) {
      this.currentUserId = userId;
      this.currentUserRole = role;
      if (name) this.currentUserName = name;
    },

    createVoucher(
      makerId: string,
      beneficiaryAccount: string,
      beneficiaryBank: string,
      amountVnd: number,
      purpose: string,
      beneficiaryName?: string
    ): PaymentVoucher {
      const voucher = createPaymentVoucher(
        makerId,
        beneficiaryAccount,
        beneficiaryBank,
        amountVnd,
        purpose,
        beneficiaryName
      );
      this.vouchers.push(voucher);
      this.auditLedger.append(makerId, 'VOUCHER_CREATED', {
        voucherId: voucher.voucherId,
        amountVnd: voucher.amountVnd,
        beneficiaryBank,
      });

      // Asynchronously dispatch to server; never throws in offline mode (Decree 13 Zero Data Egress)
      this.dispatchCreateVoucherToServer(voucher).catch(() => {});

      return voucher;
    },

    submitVoucher(voucherId: string): PaymentVoucher {
      const idx = this.vouchers.findIndex((v) => v.voucherId === voucherId);
      if (idx === -1) {
        throw new Error(`Voucher ${voucherId} not found`);
      }
      const updated = submitVoucherForApproval(this.vouchers[idx]);
      this.vouchers[idx] = updated;
      this.auditLedger.append(updated.makerId, 'VOUCHER_SUBMITTED', {
        voucherId: updated.voucherId,
        hitlToken: updated.hitlToken,
        tokenExpiresAt: updated.tokenExpiresAt,
      });

      // Asynchronously dispatch to server
      this.dispatchCreateVoucherToServer(updated).catch(() => {});

      return updated;
    },

    approveVoucher(
      voucherId: string,
      checkerId: string,
      authMethod: AuthMethod = 'BIOMETRIC_SIM',
      hitlToken?: string
    ): PaymentVoucher {
      const idx = this.vouchers.findIndex((v) => v.voucherId === voucherId);
      if (idx === -1) {
        throw new Error(`Voucher ${voucherId} not found`);
      }
      const updated = executeApproveVoucher(this.vouchers[idx], checkerId, authMethod, hitlToken);
      this.vouchers[idx] = updated;
      this.auditLedger.append(checkerId, 'VOUCHER_APPROVED', {
        voucherId: updated.voucherId,
        makerId: updated.makerId,
        checkerId,
        amountVnd: updated.amountVnd,
        authMethod,
        merkleLeafHash: updated.merkleLeafHash,
      });

      // Asynchronously dispatch approval to server
      this.dispatchApproveVoucherToServer(voucherId, checkerId).catch(() => {});

      return updated;
    },

    rejectVoucher(
      voucherId: string,
      checkerId: string,
      reason: string = 'Rejected by checker',
      hitlToken?: string
    ): PaymentVoucher {
      const idx = this.vouchers.findIndex((v) => v.voucherId === voucherId);
      if (idx === -1) {
        throw new Error(`Voucher ${voucherId} not found`);
      }
      const updated = executeRejectVoucher(this.vouchers[idx], checkerId, reason, hitlToken);
      this.vouchers[idx] = updated;
      this.auditLedger.append(checkerId, 'VOUCHER_REJECTED', {
        voucherId: updated.voucherId,
        checkerId,
        rejectReason: reason,
      });

      // Asynchronously dispatch rejection to server
      this.dispatchRejectVoucherToServer(voucherId, reason).catch(() => {});

      return updated;
    },

    async dispatchCreateVoucherToServer(voucher: PaymentVoucher) {
      try {
        const res = await apiRequest('/treasury/vouchers', {
          method: 'POST',
          body: JSON.stringify({
            id: voucher.voucherId,
            voucherId: voucher.voucherId,
            targetAccount: voucher.beneficiaryAccount,
            beneficiaryAccount: voucher.beneficiaryAccount,
            targetBeneficiary: voucher.beneficiaryName,
            beneficiaryName: voucher.beneficiaryName,
            targetBank: voucher.beneficiaryBank,
            beneficiaryBank: voucher.beneficiaryBank,
            amount: voucher.amountVnd,
            amountVnd: voucher.amountVnd,
            description: voucher.purpose,
            purpose: voucher.purpose,
            hitlToken: voucher.hitlToken,
          }),
        });
        if (res.success) {
          this.isOnlineSyncActive = true;
        }
        return res;
      } catch {
        this.isOnlineSyncActive = false;
        return { success: false, isOnline: false };
      }
    },

    async dispatchApproveVoucherToServer(voucherId: string, checkerId?: string) {
      try {
        const res = await apiRequest(`/treasury/vouchers/${voucherId}/approve`, {
          method: 'POST',
          body: JSON.stringify({ checkerId }),
        });
        if (res.success) {
          this.isOnlineSyncActive = true;
        }
        return res;
      } catch {
        this.isOnlineSyncActive = false;
        return { success: false, isOnline: false };
      }
    },

    async dispatchRejectVoucherToServer(voucherId: string, reason?: string) {
      try {
        const res = await apiRequest(`/treasury/vouchers/${voucherId}/reject`, {
          method: 'POST',
          body: JSON.stringify({ reason }),
        });
        if (res.success) {
          this.isOnlineSyncActive = true;
        }
        return res;
      } catch {
        this.isOnlineSyncActive = false;
        return { success: false, isOnline: false };
      }
    },

    async fetchVouchersFromServer(): Promise<boolean> {
      try {
        const res = await apiRequest('/treasury/vouchers', { method: 'GET' });
        if (res.success && res.data?.vouchers && Array.isArray(res.data.vouchers)) {
          this.isOnlineSyncActive = true;
          for (const sVch of res.data.vouchers) {
            const id = sVch.voucherId || sVch.id;
            const existingIdx = this.vouchers.findIndex((v) => v.voucherId === id);
            const normalized: PaymentVoucher = {
              voucherId: id,
              makerId: sVch.makerId || 'usr_maker_01',
              checkerId: sVch.checkerId || null,
              beneficiaryAccount: sVch.beneficiaryAccount || sVch.targetAccount || '',
              beneficiaryBank: sVch.beneficiaryBank || sVch.targetBank || 'CITAD_SBV',
              beneficiaryName: sVch.beneficiaryName || sVch.targetBeneficiary || 'Đơn vị thụ hưởng',
              amountVnd: Number(sVch.amountVnd || sVch.amount || 0),
              purpose: sVch.purpose || sVch.description || '',
              status: (sVch.status as VoucherStatus) || 'PENDING_APPROVAL',
              createdAt: sVch.createdAt || new Date().toISOString(),
              approvedAt: sVch.approvedAt || undefined,
              rejectedAt: sVch.rejectedAt || undefined,
              rejectReason: sVch.rejectionReason || sVch.rejectReason || undefined,
              merkleLeafHash: sVch.merkleLeafHash || undefined,
              hitlToken: sVch.hitlToken || null,
              tokenExpiresAt: sVch.tokenExpiresAt || null,
            };
            if (existingIdx >= 0) {
              this.vouchers[existingIdx] = { ...this.vouchers[existingIdx], ...normalized };
            } else {
              this.vouchers.push(normalized);
            }
          }
          return true;
        }
        return false;
      } catch {
        this.isOnlineSyncActive = false;
        return false;
      }
    },

    initRealtimeSync() {
      if (typeof window === 'undefined' || typeof (window as any).EventSource === 'undefined') {
        return;
      }
      if (this.sseEventSource) {
        try {
          this.sseEventSource.close();
        } catch {}
        this.sseEventSource = null;
      }

      try {
        const apiBase = getApiBaseUrl().replace(/\/api$/, '');
        const sseUrl = `${apiBase}/api/sync/events`;
        const es = new (window as any).EventSource(sseUrl);
        this.sseEventSource = es;
        this.sseConnectionStatus = 'CONNECTING';

        es.onopen = () => {
          this.isOnlineSyncActive = true;
          this.sseConnectionStatus = 'CONNECTED';
          this.fetchVouchersFromServer().catch(() => {});
        };

        es.onerror = () => {
          this.isOnlineSyncActive = false;
          this.sseConnectionStatus = 'DISCONNECTED';
        };

        es.addEventListener('voucher:created', (evt: any) => {
          try {
            const data = JSON.parse(evt.data);
            this.processSseEvent('voucher:created', data);
          } catch (err) {
            console.warn('[TreasuryStore] Failed to parse voucher:created event', err);
          }
        });

        es.addEventListener('voucher:approved', (evt: any) => {
          try {
            const data = JSON.parse(evt.data);
            this.processSseEvent('voucher:approved', data);
          } catch (err) {
            console.warn('[TreasuryStore] Failed to parse voucher:approved event', err);
          }
        });

        es.addEventListener('voucher:rejected', (evt: any) => {
          try {
            const data = JSON.parse(evt.data);
            this.processSseEvent('voucher:rejected', data);
          } catch (err) {
            console.warn('[TreasuryStore] Failed to parse voucher:rejected event', err);
          }
        });

        es.addEventListener('state:snapshot', (evt: any) => {
          try {
            const data = JSON.parse(evt.data);
            this.processSseEvent('state:snapshot', data);
          } catch (err) {
            console.warn('[TreasuryStore] Failed to parse state:snapshot event', err);
          }
        });

        es.addEventListener('account:updated', (evt: any) => {
          try {
            const data = JSON.parse(evt.data);
            this.processSseEvent('account:updated', data);
          } catch (err) {
            console.warn('[TreasuryStore] Failed to parse account:updated event', err);
          }
        });

        es.addEventListener('reconciliation:summary', (evt: any) => {
          try {
            const data = JSON.parse(evt.data);
            this.processSseEvent('reconciliation:summary', data);
          } catch (err) {
            console.warn('[TreasuryStore] Failed to parse reconciliation:summary event', err);
          }
        });

        es.addEventListener('aml:alert', (evt: any) => {
          try {
            const data = JSON.parse(evt.data);
            this.processSseEvent('aml:alert', data);
          } catch (err) {
            console.warn('[TreasuryStore] Failed to parse aml:alert event', err);
          }
        });
      } catch (err) {
        this.isOnlineSyncActive = false;
        this.sseConnectionStatus = 'DISCONNECTED';
      }
    },

    stopRealtimeSync() {
      if (this.sseEventSource) {
        try {
          this.sseEventSource.close();
        } catch {}
        this.sseEventSource = null;
      }
      this.isOnlineSyncActive = false;
      this.sseConnectionStatus = 'DISCONNECTED';
    },

    processSseEvent(event: string, data: any) {
      if (event === 'voucher:created') {
        const raw = data.voucher || data;
        const vId = raw.voucherId || raw.id;
        if (!vId) return;
        const existing = this.vouchers.find((v) => v.voucherId === vId);
        if (!existing) {
          const vch: PaymentVoucher = {
            voucherId: vId,
            makerId: raw.makerId || 'usr_maker_01',
            checkerId: raw.checkerId || null,
            beneficiaryAccount: raw.beneficiaryAccount || raw.targetAccount || '',
            beneficiaryBank: raw.beneficiaryBank || raw.targetBank || 'CITAD_SBV',
            beneficiaryName: raw.beneficiaryName || raw.targetBeneficiary || 'Đơn vị thụ hưởng',
            amountVnd: Number(raw.amountVnd || raw.amount || 0),
            purpose: raw.purpose || raw.description || '',
            status: (raw.status as VoucherStatus) || 'PENDING_APPROVAL',
            createdAt: raw.createdAt || new Date().toISOString(),
            approvedAt: raw.approvedAt || undefined,
            rejectedAt: raw.rejectedAt || undefined,
            rejectReason: raw.rejectionReason || raw.rejectReason || undefined,
            merkleLeafHash: raw.merkleLeafHash || undefined,
            hitlToken: raw.hitlToken || null,
            tokenExpiresAt: raw.tokenExpiresAt || null,
          };
          this.vouchers.unshift(vch);
          this.auditLedger.append(raw.makerId || 'SERVER_SYNC', 'VOUCHER_CREATED', {
            voucherId: vId,
            source: 'SSE_SYNC',
            amountVnd: vch.amountVnd,
          });
        } else {
          if (raw.status) existing.status = raw.status;
          if (raw.approvedAt) existing.approvedAt = raw.approvedAt;
          if (raw.checkerId) existing.checkerId = raw.checkerId;
          if (raw.merkleLeafHash) existing.merkleLeafHash = raw.merkleLeafHash;
        }
      } else if (event === 'voucher:approved') {
        const vId = data.voucherId || data.voucher?.voucherId || data.voucher?.id;
        if (!vId) return;
        const existing = this.vouchers.find((v) => v.voucherId === vId);
        if (existing) {
          existing.status = 'APPROVED';
          existing.approvedAt = data.voucher?.approvedAt || new Date().toISOString();
          existing.checkerId = data.checkerId || data.voucher?.checkerId || 'usr_checker_01';
          existing.merkleLeafHash = data.merkleHash || data.voucher?.merkleLeafHash || computeMerkleLeaf(existing);
          this.auditLedger.append(existing.checkerId || 'usr_checker_01', 'VOUCHER_APPROVED', {
            voucherId: vId,
            source: 'SSE_SYNC',
            merkleHash: existing.merkleLeafHash,
          });
        }
      } else if (event === 'voucher:rejected') {
        const vId = data.voucherId || data.voucher?.voucherId || data.voucher?.id;
        if (!vId) return;
        const existing = this.vouchers.find((v) => v.voucherId === vId);
        if (existing) {
          existing.status = 'REJECTED';
          existing.rejectedAt = data.voucher?.approvedAt || new Date().toISOString();
          existing.checkerId = data.checkerId || data.voucher?.checkerId || 'usr_checker_01';
          existing.rejectReason = data.remarks || data.reason || data.voucher?.rejectionReason || 'Từ chối bởi Kiểm Soát Viên';
          this.auditLedger.append(existing.checkerId || 'usr_checker_01', 'VOUCHER_REJECTED', {
            voucherId: vId,
            source: 'SSE_SYNC',
            reason: existing.rejectReason,
          });
        }
      } else if (event === 'state:snapshot') {
        if (data.vouchers && Array.isArray(data.vouchers)) {
          for (const v of data.vouchers) {
            const vId = v.voucherId || v.id;
            if (!vId) continue;
            const existing = this.vouchers.find((ev) => ev.voucherId === vId);
            if (!existing) {
              const vch: PaymentVoucher = {
                voucherId: vId,
                makerId: v.makerId || 'usr_maker_01',
                checkerId: v.checkerId || null,
                beneficiaryAccount: v.beneficiaryAccount || v.targetAccount || '',
                beneficiaryBank: v.beneficiaryBank || v.targetBank || 'CITAD_SBV',
                beneficiaryName: v.beneficiaryName || v.targetBeneficiary || 'Đơn vị thụ hưởng',
                amountVnd: Number(v.amountVnd || v.amount || 0),
                purpose: v.purpose || v.description || '',
                status: (v.status as VoucherStatus) || 'PENDING_APPROVAL',
                createdAt: v.createdAt || new Date().toISOString(),
                approvedAt: v.approvedAt || undefined,
                rejectedAt: v.rejectedAt || undefined,
                rejectReason: v.rejectionReason || v.rejectReason || undefined,
                merkleLeafHash: v.merkleLeafHash || undefined,
                hitlToken: v.hitlToken || null,
                tokenExpiresAt: v.tokenExpiresAt || null,
              };
              this.vouchers.push(vch);
            } else {
              if (v.status) existing.status = v.status;
              if (v.checkerId) existing.checkerId = v.checkerId;
              if (v.approvedAt) existing.approvedAt = v.approvedAt;
              if (v.merkleLeafHash) existing.merkleLeafHash = v.merkleLeafHash;
            }
          }
        }
        if (data.accounts) {
          const bankingStore = useBankingStore();
          bankingStore.applyStateSnapshot(data);
        }
        if (data.reconciliationSummary) {
          const reconStore = useReconciliationStore();
          reconStore.applyReconciliationSummary(data.reconciliationSummary);
        }
      } else if (event === 'account:updated') {
        const bankingStore = useBankingStore();
        bankingStore.applyAccountUpdated(data);
      } else if (event === 'reconciliation:summary') {
        const reconStore = useReconciliationStore();
        reconStore.applyReconciliationSummary(data);
      } else if (event === 'aml:alert') {
        const amlStore = useAmlStore();
        amlStore.applySseAlert(data);
      }
    },

    settleVoucher(voucherId: string): PaymentVoucher {
      const idx = this.vouchers.findIndex((v) => v.voucherId === voucherId);
      if (idx === -1) {
        throw new Error(`Voucher ${voucherId} not found`);
      }
      const updated = executeSettleVoucher(this.vouchers[idx]);
      this.vouchers[idx] = updated;
      this.auditLedger.append(this.currentUserId, 'VOUCHER_SETTLED', {
        voucherId: updated.voucherId,
        settledAt: updated.settledAt,
      });
      return updated;
    },

    checkAllExpirations() {
      const now = Date.now();
      for (let i = 0; i < this.vouchers.length; i++) {
        if (this.vouchers[i].status === 'PENDING_APPROVAL') {
          const updated = checkVoucherExpiration(this.vouchers[i], now);
          if (updated.status === 'EXPIRED' && this.vouchers[i].status !== 'EXPIRED') {
            this.vouchers[i] = updated;
            this.auditLedger.append('SYSTEM_WATCHDOG', 'TOKEN_EXPIRED', {
              voucherId: updated.voucherId,
            });
          }
        }
      }
    },

    openMakerCheckerModal(voucherId: string) {
      this.activeVoucherId = voucherId;
      this.isMakerCheckerModalOpen = true;
    },

    closeMakerCheckerModal() {
      this.isMakerCheckerModalOpen = false;
      this.activeVoucherId = null;
    },

    toggleCopilot() {
      this.isCopilotOpen = !this.isCopilotOpen;
    },

    async checkAiHealth(force = false): Promise<AiConnectionStatus> {
      const isHealthy = await localAiClient.checkHealth(force);
      this.aiConnectionStatus = isHealthy ? 'connected' : 'offline';
      return this.aiConnectionStatus;
    },

    sendCopilotMessage(query: string) {
      if (!query || query.trim() === '') return;
      const userText = query.trim();

      this.copilotMessages.push({
        id: `msg-${Date.now()}-u`,
        sender: 'user',
        text: userText,
        timestamp: new Date().toISOString(),
      });

      // Pull live dynamic values from sibling Pinia stores
      const bankingStore = useBankingStore();
      const reconStore = useReconciliationStore();
      const amlStore = useAmlStore();

      const liveLiquidVnd = bankingStore.totalLiquidVnd;
      const liveBurnRate = bankingStore.dailyAverageBurnVnd || 15_000_000;
      const liveMatchRate = reconStore.matchRate;
      const liveAlertCount = amlStore.totalAlertsCount;

      const dynamicContext = {
        closingBalance: liveLiquidVnd,
        liquidVnd: liveLiquidVnd,
        dailyBurn: liveBurnRate,
        burnRateVnd: liveBurnRate,
        matchRate: liveMatchRate,
        alertCount: liveAlertCount,
        pendingCount: this.pendingVouchers.length,
      };

      const copilotRes = queryFinancialCopilot(userText, dynamicContext);

      const assistantMsg: CopilotMessage = {
        id: `msg-${Date.now()}-a`,
        sender: 'assistant',
        text: copilotRes.answer,
        intent: copilotRes.intent,
        metrics: copilotRes.metrics,
        timestamp: new Date().toISOString(),
      };

      this.copilotMessages.push(assistantMsg);

      // If not in standalone test runner, asynchronously query local AI (Qwen 14B)
      const isTestEnv = typeof process !== 'undefined' && (process.env?.NODE_ENV === 'test' || process.env?.VITEST);
      if (!isTestEnv) {
        this.isCopilotThinking = true;
        queryFinancialCopilotAsync(userText, dynamicContext)
          .then((aiRes) => {
            if (aiRes && aiRes.metrics?.localAiGenerated) {
              const idx = this.copilotMessages.findIndex((m) => m.id === assistantMsg.id);
              if (idx !== -1) {
                this.copilotMessages[idx] = {
                  ...this.copilotMessages[idx],
                  text: aiRes.answer,
                  intent: aiRes.intent,
                  metrics: { ...aiRes.metrics },
                };
              } else {
                assistantMsg.text = aiRes.answer;
                assistantMsg.intent = aiRes.intent;
                assistantMsg.metrics = { ...aiRes.metrics };
              }
              this.aiConnectionStatus = 'connected';
            }
          })
          .catch(() => {
            this.aiConnectionStatus = 'fallback';
          })
          .finally(() => {
            this.isCopilotThinking = false;
          });
      }
    },

    async sendCopilotToolMessage(query: string) {
      if (!query || query.trim() === '') return;
      const userText = query.trim();

      this.copilotMessages.push({
        id: `msg-${Date.now()}-u`,
        sender: 'user',
        text: userText,
        timestamp: new Date().toISOString(),
      });

      this.isCopilotThinking = true;
      try {
        // Execute with Banking MCP Tools & Security Safeguard Layer (Local Model Qwen 14B)
        const toolRes = await executeBankingToolDispatcher(userText);
        this.aiConnectionStatus = localAiClient.getStatus();

        this.copilotMessages.push({
          id: `msg-${Date.now()}-a`,
          sender: 'assistant',
          text: toolRes.text,
          intent: toolRes.toolUsed,
          metrics: toolRes.data,
          timestamp: new Date().toISOString(),
        });
      } finally {
        this.isCopilotThinking = false;
      }
    },

    // Guided Tour Actions
    startGuidedTour() {
      this.isTourActive = true;
      this.tourStepIndex = 0;
      this.tourSecondsRemaining = 60;
      this.clearTourTimer();

      this.tourTimerId = setInterval(() => {
        if (this.tourSecondsRemaining > 1) {
          this.tourSecondsRemaining--;
        } else {
          this.nextTourStep();
        }
      }, 1000);
    },

    pauseGuidedTour() {
      this.clearTourTimer();
    },

    resumeGuidedTour() {
      if (!this.isTourActive) return;
      this.clearTourTimer();
      this.tourTimerId = setInterval(() => {
        if (this.tourSecondsRemaining > 1) {
          this.tourSecondsRemaining--;
        } else {
          this.nextTourStep();
        }
      }, 1000);
    },

    nextTourStep() {
      if (this.tourStepIndex < this.tourSteps.length - 1) {
        this.tourStepIndex++;
        this.tourSecondsRemaining = 60;
      } else {
        this.stopGuidedTour();
      }
    },

    prevTourStep() {
      if (this.tourStepIndex > 0) {
        this.tourStepIndex--;
        this.tourSecondsRemaining = 60;
      }
    },

    stopGuidedTour() {
      this.isTourActive = false;
      this.clearTourTimer();
      this.tourStepIndex = 0;
      this.tourSecondsRemaining = 60;
    },

    clearTourTimer() {
      if (this.tourTimerId) {
        clearInterval(this.tourTimerId);
        this.tourTimerId = null;
      }
    },

    verifyAuditIntegrity() {
      return this.auditLedger.verifyIntegrity();
    },
  },
});
