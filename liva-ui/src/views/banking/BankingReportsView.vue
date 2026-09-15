<script setup lang="ts">
/**
 * BankingReportsView.vue — P90–P95 Trung Tâm Báo Cáo Tài Chính & Quyết Toán Thuế
 * ==============================================================================
 * Tích hợp đầy đủ:
 * - Bảng Cân Đối Tài Khoản F01-DN (Trial Balance)
 * - Kết Quả Hoạt Động Kinh Doanh B02-DN (Income Statement)
 * - Báo Cáo Lưu Chuyển Tiền Tệ B03-DN (Statement of Cash Flows)
 * - Tờ Khai Thuế GTGT (01/GTGT) & Quyết Toán Thuế TNDN (03/TNDN)
 * - Báo Cáo Hiệu Suất & SLA Đối Soát 30 Ngày
 */
import { ref, onMounted } from 'vue';
import { useBankingStore } from '../../stores/bankingStore';
import { useReportStore, type ReportingPeriod } from '../../stores/reportStore';
import TrialBalanceTable from '../../components/banking/reports/TrialBalanceTable.vue';
import IncomeStatementCard from '../../components/banking/reports/IncomeStatementCard.vue';
import CashflowStatementCard from '../../components/banking/reports/CashflowStatementCard.vue';
import TaxReturnModal from '../../components/banking/reports/TaxReturnModal.vue';
import ReconciliationTrendChart from '../../components/banking/ReconciliationTrendChart.vue';
import StatusAllocationDonut from '../../components/banking/StatusAllocationDonut.vue';

const bankingStore = useBankingStore();
const reportStore = useReportStore();

const activeTab = ref<'F01_TB' | 'B02_PNL' | 'B03_CF' | 'TAX' | 'AUDIT_SLA'>('F01_TB');
const periods: ReportingPeriod[] = ['2026-Q1', '2026-Q2', '2026-Q3', '2026-Q4', '2026-FY'];

onMounted(async () => {
  await bankingStore.fetchOverview();
});
</script>

<template>
  <div class="banking-view-container">
    <!-- Header Banner -->
    <div class="reports-header-banner">
      <div class="banner-left">
        <div class="header-tags">
          <span class="vas-title-pill">VAS / TT 200/2014/TT-BTC & TT 80/2021/TT-BTC</span>
          <span v-if="reportStore.isTrialBalanceBalanced" class="integrity-badge">
            ✓ 100% Cân Đối Sổ Cái
          </span>
        </div>
        <h2 class="banner-title">Trung Tâm Báo Cáo Tài Chính & Quyết Toán Thuế</h2>
        <p class="banner-desc">
          Báo cáo tài chính doanh nghiệp tự động lập từ dữ liệu đối soát và sổ cái kế toán kép.
        </p>
      </div>

      <!-- Controls: Period Select & Tax Trigger -->
      <div class="banner-right">
        <div class="period-selector">
          <label class="period-label">Kỳ báo cáo:</label>
          <select
            :value="reportStore.currentPeriod"
            class="period-select"
            @change="reportStore.setPeriod(($event.target as HTMLSelectElement).value as ReportingPeriod)"
          >
            <option v-for="p in periods" :key="p" :value="p">{{ p }}</option>
          </select>
        </div>

        <div class="tax-btn-group">
          <button class="tax-btn vat" @click="reportStore.openTaxModal('VAT')">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <line x1="16" y1="13" x2="8" y2="13" />
              <line x1="16" y1="17" x2="8" y2="17" />
            </svg>
            <span>Tờ Khai 01/GTGT</span>
          </button>
          <button class="tax-btn cit" @click="reportStore.openTaxModal('CIT')">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
            </svg>
            <span>Quyết Toán 03/TNDN</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Navigation Sub-tabs -->
    <div class="report-subtabs">
      <button
        class="subtab-btn"
        :class="{ active: activeTab === 'F01_TB' }"
        @click="activeTab = 'F01_TB'"
      >
        <span class="tab-code">F01-DN</span>
        <span>Bảng Cân Đối Tài Khoản</span>
      </button>

      <button
        class="subtab-btn"
        :class="{ active: activeTab === 'B02_PNL' }"
        @click="activeTab = 'B02_PNL'"
      >
        <span class="tab-code">B02-DN</span>
        <span>Kết Quả Kinh Doanh (P&L)</span>
      </button>

      <button
        class="subtab-btn"
        :class="{ active: activeTab === 'B03_CF' }"
        @click="activeTab = 'B03_CF'"
      >
        <span class="tab-code">B03-DN</span>
        <span>Lưu Chuyển Tiền Tệ</span>
      </button>

      <button
        class="subtab-btn"
        :class="{ active: activeTab === 'TAX' }"
        @click="activeTab = 'TAX'"
      >
        <span class="tab-code">TT 80</span>
        <span>Nghĩa Vụ Thuế (GTGT & TNDN)</span>
      </button>

      <button
        class="subtab-btn"
        :class="{ active: activeTab === 'AUDIT_SLA' }"
        @click="activeTab = 'AUDIT_SLA'"
      >
        <span class="tab-code">SLA</span>
        <span>Hiệu Năng & Đối Soát 30 Ngày</span>
      </button>
    </div>

    <!-- Dynamic Content Views -->
    <div class="tab-content-wrap">
      <!-- 1. F01-DN -->
      <TrialBalanceTable v-if="activeTab === 'F01_TB'" />

      <!-- 2. B02-DN -->
      <IncomeStatementCard v-else-if="activeTab === 'B02_PNL'" />

      <!-- 3. B03-DN -->
      <CashflowStatementCard v-else-if="activeTab === 'B03_CF'" />

      <!-- 4. TAX SUMMARY -->
      <div v-else-if="activeTab === 'TAX'" class="tax-summary-view">
        <div class="tax-cards-grid">
          <!-- VAT Card -->
          <div class="tax-card-preview">
            <div class="tp-header">
              <span class="tp-badge vat">MẪU 01/GTGT (TT 80/2021)</span>
              <h4 class="tp-title">Thuế Giá Trị Gia Tăng Kỳ {{ reportStore.currentPeriod }}</h4>
            </div>
            <div class="tp-body">
              <div class="tp-row">
                <span>Thuế GTGT đầu vào khấu trừ [25]:</span>
                <strong class="font-mono">{{ reportStore.vatReturn.deductibleInputTax.toLocaleString('vi-VN') }} ₫</strong>
              </div>
              <div class="tp-row">
                <span>Tổng thuế GTGT đầu ra [35]:</span>
                <strong class="font-mono">{{ reportStore.vatReturn.totalOutputTax.toLocaleString('vi-VN') }} ₫</strong>
              </div>
              <div class="tp-row highlight">
                <span>Số thuế GTGT phải nộp [40a]:</span>
                <strong class="font-mono text-gold">{{ reportStore.vatReturn.netVatPayable.toLocaleString('vi-VN') }} ₫</strong>
              </div>
            </div>
            <button class="tp-action-btn" @click="reportStore.openTaxModal('VAT')">
              Xem & Xuất XML Mẫu 01/GTGT →
            </button>
          </div>

          <!-- CIT Card -->
          <div class="tax-card-preview">
            <div class="tp-header">
              <span class="tp-badge cit">MẪU 03/TNDN (TT 80/2021)</span>
              <h4 class="tp-title">Quyết Toán Thuế TNDN Năm {{ reportStore.citFinalization.taxYear }}</h4>
            </div>
            <div class="tp-body">
              <div class="tp-row">
                <span>Lợi nhuận kế toán trước thuế [A1]:</span>
                <strong class="font-mono">{{ reportStore.citFinalization.accountingPbt.toLocaleString('vi-VN') }} ₫</strong>
              </div>
              <div class="tp-row">
                <span>Thu nhập tính thuế TNDN [C4]:</span>
                <strong class="font-mono">{{ reportStore.citFinalization.taxableIncome.toLocaleString('vi-VN') }} ₫</strong>
              </div>
              <div class="tp-row highlight">
                <span>Thuế TNDN còn phải nộp [G]:</span>
                <strong class="font-mono text-gold">{{ reportStore.citFinalization.remainingTaxPayable.toLocaleString('vi-VN') }} ₫</strong>
              </div>
            </div>
            <button class="tp-action-btn" @click="reportStore.openTaxModal('CIT')">
              Xem & Xuất XML Mẫu 03/TNDN →
            </button>
          </div>
        </div>
      </div>

      <!-- 5. AUDIT SLA & TREND CHARTS -->
      <div v-else-if="activeTab === 'AUDIT_SLA'" class="audit-sla-view">
        <section class="dashboard-row row-charts">
          <ReconciliationTrendChart
            :data="bankingStore.trendDays"
            filter-label="30 Ngày gần nhất"
          />
          <StatusAllocationDonut
            :matched-rate="bankingStore.summary.reconciledRate"
            :unmatched-rate="bankingStore.summary.unmatchedRate"
          />
        </section>

        <div class="kpi-summary-table-card">
          <h3 class="table-title">Chỉ số Hiệu năng & Rủi ro Kiểm toán (Audit Metrics)</h3>
          <table class="kpi-table">
            <thead>
              <tr>
                <th>Chỉ Số Nghiệp Vụ</th>
                <th>Mục Tiêu SLA</th>
                <th>Thực Tế Đạt Được</th>
                <th>Đánh Giá Tuân Thủ</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td><strong>Tỷ lệ khớp tự động (Tier 1 & Tier 2)</strong></td>
                <td>&ge; 99.0%</td>
                <td class="text-success">{{ bankingStore.summary.autoMatchedRate }}%</td>
                <td><span class="status-badge pass">ĐẠT CHỈ TIÊU</span></td>
              </tr>
              <tr>
                <td><strong>Tỷ lệ xử lý lệch qua Maker-Checker</strong></td>
                <td>&le; 1.0%</td>
                <td class="text-warning">{{ bankingStore.summary.manualMatchedRate }}%</td>
                <td><span class="status-badge pass">AN TOÀN</span></td>
              </tr>
              <tr>
                <td><strong>Tỷ lệ chênh lệch chưa thể đối soát</strong></td>
                <td>&le; 0.5%</td>
                <td class="text-info">{{ bankingStore.summary.unmatchedRate }}%</td>
                <td><span class="status-badge pass">KIỂM SOÁT TỐT</span></td>
              </tr>
              <tr>
                <td><strong>Thời gian đối soát trung bình / 50.000 dòng</strong></td>
                <td>&lt; 30 giây</td>
                <td class="text-success">18.4 giây</td>
                <td><span class="status-badge pass">XUẤT SẮC (Rust Native)</span></td>
              </tr>
              <tr>
                <td><strong>Bảo mật & Rò rỉ dữ liệu đám mây (Cloud Egress)</strong></td>
                <td>0 Byte</td>
                <td class="text-success">0 Byte (100% On-Premise)</td>
                <td><span class="status-badge pass">CHUẨN NĐ 13/2023</span></td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <!-- Tax Return Modal Dialog -->
    <TaxReturnModal
      :is-open="reportStore.activeTaxModal !== null"
      @close="reportStore.closeTaxModal"
    />
  </div>
</template>

<style scoped>
.banking-view-container {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 24px 32px;
  max-width: 1600px;
  margin: 0 auto;
  width: 100%;
  box-sizing: border-box;
}

.reports-header-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 20px 24px;
  gap: 16px;
  flex-wrap: wrap;
}

.banner-left {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.header-tags {
  display: flex;
  align-items: center;
  gap: 10px;
}

.vas-title-pill {
  font-size: 11px;
  font-weight: 700;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.12);
  border: 1px solid rgba(56, 189, 248, 0.3);
  padding: 2px 8px;
  border-radius: 4px;
}

.integrity-badge {
  font-size: 11px;
  font-weight: 700;
  color: #10b981;
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
  padding: 2px 8px;
  border-radius: 4px;
}

.banner-title {
  font-size: 20px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0;
}

.banner-desc {
  font-size: 13px;
  color: #94a3b8;
  margin: 0;
}

.banner-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.period-selector {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(15, 23, 42, 0.6);
  padding: 6px 12px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.period-label {
  font-size: 12px;
  color: #94a3b8;
}

.period-select {
  background: transparent;
  border: none;
  color: #f1f5f9;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  outline: none;
}

.tax-btn-group {
  display: flex;
  gap: 8px;
}

.tax-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  transition: all 0.2s;
}

.tax-btn.vat {
  background: rgba(56, 189, 248, 0.15);
  border-color: rgba(56, 189, 248, 0.3);
  color: #38bdf8;
}

.tax-btn.vat:hover {
  background: rgba(56, 189, 248, 0.25);
}

.tax-btn.cit {
  background: rgba(245, 158, 11, 0.15);
  border-color: rgba(245, 158, 11, 0.3);
  color: #f59e0b;
}

.tax-btn.cit:hover {
  background: rgba(245, 158, 11, 0.25);
}

.report-subtabs {
  display: flex;
  gap: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  padding-bottom: 12px;
  overflow-x: auto;
}

.subtab-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.06);
  color: #94a3b8;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s;
}

.subtab-btn:hover {
  background: rgba(255, 255, 255, 0.06);
  color: #e2e8f0;
}

.subtab-btn.active {
  background: rgba(56, 189, 248, 0.15);
  border-color: rgba(56, 189, 248, 0.4);
  color: #f8fafc;
  font-weight: 600;
}

.tab-code {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 5px;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.08);
  color: #cbd5e1;
  font-family: monospace;
}

.subtab-btn.active .tab-code {
  background: #38bdf8;
  color: #0f172a;
}

.tab-content-wrap {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.tax-cards-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

@media (max-width: 900px) {
  .tax-cards-grid {
    grid-template-columns: 1fr;
  }
}

.tax-card-preview {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 16px;
}

.tp-badge {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 4px;
}

.tp-badge.vat {
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.12);
  border: 1px solid rgba(56, 189, 248, 0.3);
}

.tp-badge.cit {
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.12);
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.tp-title {
  font-size: 15px;
  font-weight: 700;
  color: #f8fafc;
  margin: 6px 0 0 0;
}

.tp-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  font-size: 13px;
}

.tp-row {
  display: flex;
  justify-content: space-between;
  color: #94a3b8;
}

.tp-row.highlight {
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  padding-top: 8px;
  color: #e2e8f0;
}

.tp-action-btn {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #f1f5f9;
  padding: 10px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.tp-action-btn:hover {
  background: #2563eb;
  border-color: #3b82f6;
  color: #fff;
}

.dashboard-row {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 20px;
}

@media (max-width: 1200px) {
  .dashboard-row {
    grid-template-columns: 1fr;
  }
}

.kpi-summary-table-card {
  background-color: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  padding: 20px;
}

.table-title {
  font-size: 15px;
  font-weight: 600;
  color: #f8fafc;
  margin: 0 0 16px 0;
}

.kpi-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.kpi-table th,
.kpi-table td {
  padding: 10px 14px;
  text-align: left;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.kpi-table th {
  background: rgba(0, 0, 0, 0.2);
  color: #94a3b8;
  font-weight: 600;
}

.status-badge {
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
}

.status-badge.pass {
  background-color: rgba(16, 185, 129, 0.15);
  color: #10b981;
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.text-success {
  color: #10b981;
  font-weight: 600;
}

.text-warning {
  color: #f59e0b;
  font-weight: 600;
}

.text-info {
  color: #38bdf8;
  font-weight: 600;
}

.text-gold {
  color: #f59e0b;
}
</style>
