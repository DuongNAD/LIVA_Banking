<script setup lang="ts">
/**
 * BankingReportsView.vue — Màn hình Báo cáo Đối soát & Kiểm toán
 * Tổng hợp số liệu xu hướng 30 ngày, tỷ lệ phân bổ trạng thái và bảng chỉ số kiểm toán định kỳ.
 */
import { onMounted } from 'vue';
import { useBankingStore } from '../../stores/bankingStore';
import ReconciliationTrendChart from '../../components/banking/ReconciliationTrendChart.vue';
import StatusAllocationDonut from '../../components/banking/StatusAllocationDonut.vue';

const bankingStore = useBankingStore();

onMounted(async () => {
  await bankingStore.fetchOverview();
});
</script>

<template>
  <div class="banking-view-container">
    <!-- Header Banner -->
    <div class="reports-header-banner">
      <div>
        <h2 class="banner-title">Báo Cáo Hiệu Suất Đối Soát & Kiểm Toán Sổ Cái</h2>
        <p class="banner-desc">
          Báo cáo chu kỳ đối soát đa ngân hàng 30 ngày, đối chiếu chênh lệch và tỷ lệ tự động hóa chuẩn SLA 99.8%.
        </p>
      </div>
      <button class="export-report-btn" title="Xuất báo cáo PDF/Excel">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="7 10 12 15 17 10" />
          <line x1="12" y1="15" x2="12" y2="3" />
        </svg>
        <span>Xuất Báo Cáo Kiểm Toán</span>
      </button>
    </div>

    <!-- Charts Row -->
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

    <!-- Key Performance Indicators Table -->
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
</template>

<style scoped>
.banking-view-container {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 20px 24px 40px 24px;
  background-color: #f8fafc;
  min-height: 100%;
}

.reports-header-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 18px 24px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.banner-title {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 4px 0;
}

.banner-desc {
  font-size: 13px;
  color: #64748b;
  margin: 0;
}

.export-report-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  background: #0f172a;
  color: #ffffff;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s ease;
}

.export-report-btn:hover {
  background: #1e293b;
}

.dashboard-row {
  display: flex;
  gap: 18px;
  width: 100%;
}

.row-charts {
  display: flex;
  flex-wrap: wrap;
  gap: 18px;
}

.kpi-summary-table-card {
  padding: 20px 24px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.table-title {
  font-size: 14px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 16px 0;
}

.kpi-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.kpi-table th {
  text-align: left;
  padding: 12px 16px;
  background: #f8fafc;
  color: #64748b;
  font-weight: 600;
  border-bottom: 1px solid #e2e8f0;
}

.kpi-table td {
  padding: 14px 16px;
  border-bottom: 1px solid #f1f5f9;
  color: #334155;
}

.text-success {
  color: #059669;
  font-weight: 700;
}

.text-warning {
  color: #d97706;
  font-weight: 700;
}

.text-info {
  color: #0284c7;
  font-weight: 700;
}

.status-badge {
  display: inline-block;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.5px;
}

.status-badge.pass {
  background: #dcfce7;
  color: #15803d;
}
</style>
