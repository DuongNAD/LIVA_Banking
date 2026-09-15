/**
 * BankingDashboardComponents.test.ts (formerly WidgetApp.test.ts)
 * Comprehensive unit and integration tests for the 2D Banking & Treasury UI Dashboard.
 * Replaces legacy 3D avatar & phonemeLipSync test suites.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import BankingApp from '../../src/BankingApp.vue';
import BankingWindowBar from '../../src/components/banking/BankingWindowBar.vue';
import BankingSidebar from '../../src/components/banking/BankingSidebar.vue';
import BankingHeader from '../../src/components/banking/BankingHeader.vue';
import BankCard from '../../src/components/banking/BankCard.vue';
import AutoReconciliationGauge from '../../src/components/banking/AutoReconciliationGauge.vue';
import ReconciliationTrendChart from '../../src/components/banking/ReconciliationTrendChart.vue';
import StatusAllocationDonut from '../../src/components/banking/StatusAllocationDonut.vue';
import TransactionLedger from '../../src/components/banking/TransactionLedger.vue';
import HitlResolutionModal from '../../src/components/banking/HitlResolutionModal.vue';
import FinancialAssistantDrawer from '../../src/components/banking/FinancialAssistantDrawer.vue';
import { useBankingStore } from '../../src/stores/bankingStore';
import { useReconciliationStore } from '../../src/stores/reconciliationStore';

describe('2D Banking & Treasury UI Dashboard Suite', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  // 1. Shell & Window Bar
  describe('BankingWindowBar & App Shell', () => {
    it('renders the window bar with exact URL and MVP badge', () => {
      const wrapper = mount(BankingWindowBar, {
        props: {
          url: 'reconciliation.local',
          badgeText: 'MVP THỰC TẾ',
        },
      });

      expect(wrapper.find('.url-protocol').text()).toBe('liva://');
      expect(wrapper.find('.url-host').text()).toBe('reconciliation.local');
      expect(wrapper.find('.mvp-badge').text()).toBe('MVP THỰC TẾ');
      expect(wrapper.findAll('.dot')).toHaveLength(3);
    });

    it('mounts the complete BankingApp shell with all layout components', () => {
      const wrapper = mount(BankingApp, {
        global: {
          plugins: [createPinia()],
        },
      });

      expect(wrapper.find('.banking-app-shell').exists()).toBe(true);
      expect(wrapper.findComponent(BankingWindowBar).exists()).toBe(true);
      expect(wrapper.findComponent(BankingSidebar).exists()).toBe(true);
      expect(wrapper.findComponent(BankingHeader).exists()).toBe(true);
    });
  });

  // 2. Sidebar Navigation
  describe('BankingSidebar Navigation (7 Items)', () => {
    it('renders Brand LIVA Reconciliation and 7 distinct navigation items', async () => {
      const wrapper = mount(BankingSidebar, {
        props: { activeItem: 'overview' },
      });

      expect(wrapper.find('.brand-title').text()).toBe('LIVA');
      expect(wrapper.find('.brand-sub').text()).toBe('Reconciliation');

      const items = wrapper.findAll('.nav-item');
      expect(items).toHaveLength(7);

      const labels = items.map(i => i.find('.nav-item-label').text());
      expect(labels).toEqual([
        'TỔNG QUAN',
        'ĐỐI SOÁT',
        'GIAO DỊCH',
        'THU CHI',
        'NGÂN QUỸ',
        'BÁO CÁO',
        'CÀI ĐẶT',
      ]);

      // Verify active class on first item by default
      expect(items[0].classes()).toContain('active');

      // Verify green active status dot on 'ĐỐI SOÁT'
      const reconcileItem = items[1];
      expect(reconcileItem.find('.status-dot').exists()).toBe(true);

      // Trigger navigation event
      await items[1].trigger('click');
      expect(wrapper.emitted('navigate')).toBeTruthy();
      expect(wrapper.emitted('navigate')?.[0]).toEqual(['reconcile']);
    });
  });

  // 3. Header Bar
  describe('BankingHeader', () => {
    it('renders dashboard title, user profile, notification bell, and search input', async () => {
      const wrapper = mount(BankingHeader, {
        props: {
          userName: 'Nguyễn Minh Trí',
          hasUnreadNotifications: true,
        },
      });

      expect(wrapper.find('.header-title').text()).toBe('LIVA Reconciliation Dashboard');
      expect(wrapper.find('.user-name').text()).toBe('Nguyễn Minh Trí');
      expect(wrapper.find('.bell-badge').exists()).toBe(true);

      // Search input emit
      const searchInput = wrapper.find('.search-input');
      await searchInput.setValue('VCB2026');
      expect(wrapper.emitted('search')).toBeTruthy();
      expect(wrapper.emitted('search')?.[0]).toEqual(['VCB2026']);

      // Assistant toggle button
      const assistantBtn = wrapper.find('.assistant-btn');
      await assistantBtn.trigger('click');
      expect(wrapper.emitted('toggleAssistant')).toBeTruthy();
    });
  });

  // 4. Row 1: Bank Cards & Auto-Reconciliation Gauge
  describe('Row 1: Bank Cards & Auto-Reconciliation Gauge', () => {
    it('renders Vietcombank Card with formatted balances and sync timestamp', () => {
      const wrapper = mount(BankCard, {
        props: {
          bankCode: 'VCB',
          bankName: 'Ngân hàng Vietcombank',
          balance: 1450230000,
          reconciled: 1449850000,
          unreconciled: 380000,
          lastSync: '10:30 AM',
        },
      });

      expect(wrapper.find('.bank-name-text').text()).toBe('Ngân hàng Vietcombank');
      expect(wrapper.find('.balance-value').text()).toBe('1,450,230,000 VND');
      expect(wrapper.find('.reconciled-val').text()).toBe('1,449,850,000 VND');
      expect(wrapper.find('.unreconciled-val').text()).toBe('380,000 VND');
      expect(wrapper.find('.last-sync-text').text()).toBe('Last sync: 10:30 AM');
      expect(wrapper.find('.bank-icon-vcb').exists()).toBe(true);
    });

    it('renders Techcombank Card with formatted balances and sync timestamp', () => {
      const wrapper = mount(BankCard, {
        props: {
          bankCode: 'TCB',
          bankName: 'Techcombank',
          balance: 785600000,
          reconciled: 785100000,
          unreconciled: 500000,
          lastSync: '10:30 AM',
        },
      });

      expect(wrapper.find('.bank-name-text').text()).toBe('Techcombank');
      expect(wrapper.find('.balance-value').text()).toBe('785,600,000 VND');
      expect(wrapper.find('.reconciled-val').text()).toBe('785,100,000 VND');
      expect(wrapper.find('.unreconciled-val').text()).toBe('500,000 VND');
      expect(wrapper.find('.bank-icon-tcb').exists()).toBe(true);
    });

    it('renders AutoReconciliationGauge with 99.8% semi-circular meter and breakdown stats', () => {
      const wrapper = mount(AutoReconciliationGauge, {
        props: {
          reconciledRate: 99.8,
          reconciledCount: 1842,
          totalCount: 1845,
          autoRate: 99.2,
          manualRate: 0.6,
          unmatchedCount: 3,
          unmatchedRate: 0.2,
        },
      });

      expect(wrapper.find('.gauge-percentage').text()).toBe('99.8%');
      expect(wrapper.find('.gauge-subtitle').text()).toBe('Tỷ lệ khớp');
      expect(wrapper.find('.matched-text-vn').text()).toBe('Đã khớp: 1,842/1,845 GD');
      expect(wrapper.find('.matched-text-en').text()).toBe('Matches: 1,842/1,845 Trx');

      const breakdownRows = wrapper.findAll('.breakdown-row');
      expect(breakdownRows[0].text()).toContain('Tự động:99.2%');
      expect(breakdownRows[1].text()).toContain('Bằng tay:0.6%');
      expect(breakdownRows[2].text()).toContain('Chưa khớp:3 (0.2%)');
    });
  });

  // 5. Row 2: Charts (Trend Line & Status Donut)
  describe('Row 2: Charts', () => {
    it('renders 30-day SVG line chart with legend and filter', () => {
      const wrapper = mount(ReconciliationTrendChart, {
        props: {
          filterLabel: 'Last lướt 30 ngày',
        },
      });

      expect(wrapper.find('.trend-title').text()).toBe('XU HƯỚNG ĐỐI SOÁT');
      expect(wrapper.find('.filter-pill-btn').text()).toContain('Last lướt 30 ngày');
      expect(wrapper.findAll('.legend-item')).toHaveLength(2);
      expect(wrapper.find('.line-svg').exists()).toBe(true);
      expect(wrapper.find('.line-svg').findAll('polyline')).toHaveLength(2); // Matched and Unmatched lines
    });

    it('renders Status Allocation Donut chart with Khớp and Chưa khớp callouts', () => {
      const wrapper = mount(StatusAllocationDonut, {
        props: {
          matchedRate: 99.8,
          unmatchedRate: 0.2,
        },
      });

      expect(wrapper.find('.card-title').text()).toBe('PHÂN BỔ TRẠNG THÁI');
      expect(wrapper.find('.callout-left').text()).toContain('Chưa khớp0.2%');
      expect(wrapper.find('.callout-right').text()).toContain('Khớp99.8%');
    });
  });

  // 6. Row 3: Real-time Transaction Ledger & Hot-Folder Dropzone
  describe('Row 3: Transaction Ledger & Hot-Folder', () => {
    it('renders transaction table and statement dropzone with format chips', () => {
      const pinia = createPinia();
      setActivePinia(pinia);
      useReconciliationStore().seedBenchmarkData();
      const wrapper = mount(TransactionLedger, {
        global: {
          plugins: [pinia],
        },
      });

      expect(wrapper.find('.ledger-title').text()).toContain('SỔ GIAO DỊCH CHÍNH TÌM THỰC');
      expect(wrapper.find('.statement-dropzone').exists()).toBe(true);
      expect(wrapper.find('.dropzone-badges').text()).toContain('VCB .xlsx');
      expect(wrapper.find('.dropzone-badges').text()).toContain('TCB .csv');
      expect(wrapper.find('.dropzone-badges').text()).toContain('BIDV .pdf');

      const rows = wrapper.findAll('.tx-row');
      expect(rows.length).toBeGreaterThanOrEqual(4);
    });

    it('filters transactions when clicking status tabs', async () => {
      const pinia = createPinia();
      setActivePinia(pinia);
      useReconciliationStore().seedBenchmarkData();
      const wrapper = mount(TransactionLedger, {
        global: {
          plugins: [pinia],
        },
      });

      const tabs = wrapper.findAll('.tab-btn');
      expect(tabs.length).toBe(4);

      // Click 'Đã khớp' tab
      await tabs[1].trigger('click');
      const matchedRows = wrapper.findAll('.tx-row');
      for (const r of matchedRows) {
        expect(r.find('.status-matched').exists()).toBe(true);
      }
    });
  });

  // 7. HITL Two-Phase Confirmation Modal
  describe('HitlResolutionModal (Two-Phase Confirmation)', () => {
    it('renders audit diff comparison and resolves transaction with single-use token', async () => {
      const pinia = createPinia();
      setActivePinia(pinia);
      const reconcileStore = useReconciliationStore();
      reconcileStore.seedBenchmarkData();
      const targetTx = reconcileStore.transactions[2]; // VCB2026103003
      targetTx.tokenUuid = 'token-uuid-test-998';

      const wrapper = mount(HitlResolutionModal, {
        props: {
          transaction: targetTx,
          isOpen: true,
        },
        global: {
          plugins: [pinia],
        },
      });

      expect(wrapper.find('.two-phase-badge').text()).toBe('Two-Phase Confirmation');
      expect(wrapper.find('.token-code').text()).toBe('token-uuid-test-998');
      expect(wrapper.find('.bank-col').text()).toContain(targetTx.txCode);
      expect(wrapper.find('.erp-col').text()).toContain('Chênh lệch (Variance)');

      // Confirm resolution
      const confirmBtn = wrapper.find('.btn-confirm');
      await confirmBtn.trigger('click');
      await flushPromises();

      expect(wrapper.emitted('resolved')).toBeTruthy();
      expect(wrapper.emitted('resolved')?.[0]).toEqual([targetTx.txCode]);
    });
  });

  // 8. 2D Conversational Financial Assistant
  describe('FinancialAssistantDrawer (Pure 2D with 12-Bar SVG Soundwave)', () => {
    it('renders 12-bar reactive soundwave, suggestion chips, and handles chat queries', async () => {
      const wrapper = mount(FinancialAssistantDrawer, {
        props: { isOpen: true },
      });

      expect(wrapper.find('.drawer-title').text()).toBe('Trợ Lý Tài Chính LIVA');
      expect(wrapper.find('.compliance-tag').text()).toContain('Zero Cloud Leakage');

      // Verify 12-bar SVG soundwave
      const bars = wrapper.findAll('.soundwave-bar');
      expect(bars).toHaveLength(12);

      // Test suggestion chips
      const chips = wrapper.findAll('.chip-btn');
      expect(chips.length).toBeGreaterThanOrEqual(3);

      // Test voice toggle
      const micBtn = wrapper.find('.mic-toggle-btn');
      await micBtn.trigger('click');
      expect(micBtn.classes()).toContain('is-active');

      // Test text input query
      const input = wrapper.find('.chat-input');
      await input.setValue('Vị thế tiền mặt VCB hôm nay');
      const sendBtn = wrapper.find('.send-btn');
      await sendBtn.trigger('click');

      const messages = wrapper.findAll('.message-bubble-wrapper');
      expect(messages.length).toBeGreaterThanOrEqual(3);
    });
  });
});
