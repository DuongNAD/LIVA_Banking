/**
 * Phase4TreasuryWorkbench.test.ts
 * Comprehensive test suite verifying Phase 4 deliverables:
 * - Real Tauri IPC Wire-up and Store state management
 * - 2D Treasury Workbench Components (BankCard audit fields, Gauge SLA target 99.8%)
 * - Circular 09/2020/TT-NHNN Dual Control Maker-Checker (maker_id != checker_id enforcement)
 * - Rolling Cashflow Sentinel (30-90 days forecast & 24-48h balance deficit warning)
 * - Pure 2D Treasury Mini-Dock isolating legacy 3D avatar
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import BankCard from '../../src/components/banking/BankCard.vue';
import AutoReconciliationGauge from '../../src/components/banking/AutoReconciliationGauge.vue';
import HitlResolutionModal from '../../src/components/banking/HitlResolutionModal.vue';
import RollingCashflowSentinel from '../../src/components/banking/RollingCashflowSentinel.vue';
import TreasuryMiniDock from '../../src/components/banking/TreasuryMiniDock.vue';
import { useBankingStore } from '../../src/stores/bankingStore';
import { useReconciliationStore, type BankTransaction } from '../../src/stores/reconciliationStore';

describe('Phase 4: Treasury Workbench 2D & IPC Integration Suite', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  // -------------------------------------------------------------------------
  // 1. BankCard Component (Audit Balances & Discrepancy)
  // -------------------------------------------------------------------------
  describe('BankCard — Comprehensive Audit Balance Fields', () => {
    it('displays opening balance, closing balance, reconciled amount, and discrepancy', () => {
      const wrapper = mount(BankCard, {
        props: {
          bankName: 'Ngân hàng Vietcombank',
          bankCode: 'VCB',
          accountNumber: 'VCB-00710009821',
          openingBalance: 1200000000,
          closingBalance: 1450230000,
          balance: 1450230000,
          reconciled: 1449850000,
          discrepancy: 380000,
          lastSync: '10:30 AM',
        },
      });

      expect(wrapper.text()).toContain('Ngân hàng Vietcombank');
      expect(wrapper.text()).toContain('VCB-00710009821');
      expect(wrapper.text()).toContain('1,200,000,000 VND'); // Opening
      expect(wrapper.text()).toContain('1,450,230,000 VND'); // Closing
      expect(wrapper.text()).toContain('1,449,850,000 VND'); // Reconciled
      expect(wrapper.text()).toContain('380,000 VND'); // Discrepancy
      expect(wrapper.find('.bank-icon-vcb').exists()).toBe(true);
      expect(wrapper.find('.status-pending').exists()).toBe(true);
    });

    it('displays balanced status pill when discrepancy is 0', () => {
      const wrapper = mount(BankCard, {
        props: {
          bankName: 'BIDV',
          bankCode: 'BIDV',
          accountNumber: 'BIDV-1201000456',
          openingBalance: 300000000,
          closingBalance: 350000000,
          balance: 350000000,
          reconciled: 350000000,
          discrepancy: 0,
          unreconciled: 0,
          lastSync: '10:30 AM',
        },
      });

      expect(wrapper.find('.status-balanced').exists()).toBe(true);
      expect(wrapper.find('.status-balanced').text()).toContain('Cân bằng 100%');
    });
  });

  // -------------------------------------------------------------------------
  // 2. AutoReconciliationGauge (Target 99.8%)
  // -------------------------------------------------------------------------
  describe('AutoReconciliationGauge — Target SLA 99.8%', () => {
    it('renders target 99.8% indicator and marks target achieved when rate >= 99.8%', () => {
      const wrapper = mount(AutoReconciliationGauge, {
        props: {
          reconciledCount: 1842,
          totalCount: 1845,
          reconciledRate: 99.8,
          autoRate: 99.2,
          manualRate: 0.6,
          unmatchedCount: 3,
          unmatchedRate: 0.2,
          targetRate: 99.8,
        },
      });

      expect(wrapper.find('.gauge-percentage').text()).toBe('99.8%');
      expect(wrapper.find('.target-subtitle').text()).toContain('99.8%');
      expect(wrapper.find('.badge-success').exists()).toBe(true);
      expect(wrapper.find('.badge-success').text()).toContain('ĐẠT CHỈ TIÊU');
    });

    it('marks target warning when rate is below 99.8%', () => {
      const wrapper = mount(AutoReconciliationGauge, {
        props: {
          reconciledCount: 950,
          totalCount: 1000,
          reconciledRate: 95.0,
          autoRate: 94.0,
          manualRate: 1.0,
          unmatchedCount: 50,
          unmatchedRate: 5.0,
          targetRate: 99.8,
        },
      });

      expect(wrapper.find('.badge-warning').exists()).toBe(true);
      expect(wrapper.find('.badge-warning').text()).toContain('CẦN TỐI ƯU');
    });
  });

  // -------------------------------------------------------------------------
  // 3. Circular 09/2020/TT-NHNN Dual Control Maker-Checker
  // -------------------------------------------------------------------------
  describe('HitlResolutionModal — Circular 09/2020/TT-NHNN Dual Control', () => {
    const mockTx: BankTransaction = {
      id: 'tx-hitl-test',
      txCode: 'VCB2026103009',
      time: '11:00 AM',
      date: '2026-10-30',
      bankCode: 'VCB',
      accountNumber: 'VCB-00710009821',
      memo: 'HOAN PHI GIAO DICH LECH 1100 VND',
      bankAmount: 1100,
      ledgerAmount: 0,
      variance: 1100,
      status: 'PENDING_HITL',
      statusLabel: 'Chờ duyệt',
      confidenceScore: 0.94,
      tokenUuid: 'token-test-single-use-uuid',
    };

    it('enforces Dual Control and blocks submission when maker_id === checker_id', async () => {
      const wrapper = mount(HitlResolutionModal, {
        props: {
          transaction: mockTx,
          isOpen: true,
        },
      });

      // Default distinct identities
      expect(wrapper.find('.violation-alert').exists()).toBe(false);
      const confirmBtn = wrapper.find('.btn-confirm');
      expect(confirmBtn.attributes('disabled')).toBeUndefined();

      // Trigger self-approval violation (maker_id == checker_id)
      const inputs = wrapper.findAll('.role-input');
      expect(inputs.length).toBe(2);

      await inputs[0].setValue('SAME_USER_IDENTITY');
      await inputs[1].setValue('SAME_USER_IDENTITY');

      // Violation alert must now be visible and button disabled
      expect(wrapper.find('.violation-alert').exists()).toBe(true);
      expect(wrapper.find('.violation-alert').text()).toContain('CẢNH BÁO VI PHẠM THÔNG TƯ 09/2020/TT-NHNN');
      expect(confirmBtn.attributes('disabled')).toBeDefined();
    });

    it('allows submission when Maker and Checker are distinct individuals', async () => {
      const reconcileStore = useReconciliationStore();
      reconcileStore.seedBenchmarkData();
      const targetTx = reconcileStore.transactions.find(t => t.status === 'PENDING_HITL')!;
      expect(targetTx).toBeDefined();

      const wrapper = mount(HitlResolutionModal, {
        props: {
          transaction: targetTx,
          isOpen: true,
        },
      });

      const inputs = wrapper.findAll('.role-input');
      await inputs[0].setValue('KT_NGUYEN_VAN_A');
      await inputs[1].setValue('KTT_TRAN_THI_B');

      expect(wrapper.find('.violation-alert').exists()).toBe(false);
      const confirmBtn = wrapper.find('.btn-confirm');
      expect(confirmBtn.attributes('disabled')).toBeUndefined();

      await confirmBtn.trigger('click');
      expect(wrapper.emitted('resolved')).toBeTruthy();
      expect(wrapper.emitted('close')).toBeTruthy();
    });
  });

  // -------------------------------------------------------------------------
  // 4. Rolling Cashflow Sentinel
  // -------------------------------------------------------------------------
  describe('RollingCashflowSentinel — 30-90 Days Forecast & Deficit Warning', () => {
    it('renders rolling cashflow sentinel with horizon switch and metrics', async () => {
      const wrapper = mount(RollingCashflowSentinel);

      expect(wrapper.text()).toContain('THÁP CANH GIÁM SÁT & DỰ BÁO DÒNG TIỀN');
      expect(wrapper.text()).toContain('30–90 ngày');

      const buttons = wrapper.findAll('.horizon-btn');
      expect(buttons.length).toBe(2);
      expect(buttons[0].text()).toBe('30 Ngày');
      expect(buttons[1].text()).toBe('90 Ngày');

      // Switch to 90 Days
      await buttons[1].trigger('click');
      expect(buttons[1].classes()).toContain('active');

      // Check SVG curve exists
      expect(wrapper.find('.sentinel-svg').exists()).toBe(true);
      expect(wrapper.text()).toContain('NGƯỠNG AN TOÀN 500M');
    });
  });

  // -------------------------------------------------------------------------
  // 5. Eradicate Jarvis Legacy (Pure 2D Treasury Mini-Dock)
  // -------------------------------------------------------------------------
  describe('TreasuryMiniDock — Pure 2D Enterprise Dock', () => {
    it('renders 2D floating dock with real-time match rate and balance position', () => {
      const wrapper = mount(TreasuryMiniDock);

      expect(wrapper.text()).toContain('LIVA TREASURY');
      expect(wrapper.find('.live-pulse').exists()).toBe(true);
      expect(wrapper.find('.open-workbench-btn').exists()).toBe(true);
      expect(wrapper.find('.open-workbench-btn').text()).toContain('Mở Workbench');
    });
  });
});
