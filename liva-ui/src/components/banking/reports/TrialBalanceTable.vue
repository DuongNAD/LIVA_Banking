<script setup lang="ts">
/**
 * TrialBalanceTable.vue — P91 Bảng Cân Đối Số Phát Sinh (Mẫu F01-DN)
 * ====================================================================
 * Trực quan hóa Bảng Cân Đối Tài Khoản 6 cột theo Thông tư 200/2014/TT-BTC.
 * Tự động kiểm tra 3 bất biến toán học: Dư đầu kỳ, Phát sinh trong kỳ, Dư cuối kỳ.
 */
import { useReportStore } from '../../../stores/reportStore';

const store = useReportStore();

function formatVnd(val: number): string {
  if (val === 0) return '-';
  return val.toLocaleString('vi-VN');
}
</script>

<template>
  <div class="trial-balance-card">
    <div class="card-header">
      <div>
        <div class="header-tags">
          <span class="vas-badge">MẪU F01-DN (TT 200/2014/TT-BTC)</span>
          <span v-if="store.isTrialBalanceBalanced" class="status-pill balanced">
            ✓ ĐẲNG THỨC KÉP CÂN BẰNG TUYỆT ĐỐI (100% BALANCED)
          </span>
          <span v-else class="status-pill unbalanced">
            ⚠ CẢNH BÁO: LỆCH CÂN ĐỐI TÀI KHOẢN
          </span>
        </div>
        <h3 class="card-title">Bảng Cân Đối Số Phát Sinh Tài Khoản (Trial Balance)</h3>
        <p class="card-desc">
          Kiểm toán tính toàn vẹn 3 đẳng thức: Tổng Nợ = Tổng Có (Đầu kỳ, Phát sinh và Cuối kỳ).
        </p>
      </div>

      <div class="period-pill">
        Kỳ: <strong>{{ store.currentPeriod }}</strong>
      </div>
    </div>

    <!-- 6-Column Balance Table -->
    <div class="table-container">
      <table class="balance-table">
        <thead>
          <tr>
            <th rowspan="2" class="col-code">Số hiệu</th>
            <th rowspan="2" class="col-name">Tên tài khoản</th>
            <th colspan="2" class="col-group">Số dư đầu kỳ</th>
            <th colspan="2" class="col-group">Số phát sinh trong kỳ</th>
            <th colspan="2" class="col-group">Số dư cuối kỳ</th>
          </tr>
          <tr class="sub-head">
            <th class="col-num">Nợ (VND)</th>
            <th class="col-num">Có (VND)</th>
            <th class="col-num">Nợ (VND)</th>
            <th class="col-num">Có (VND)</th>
            <th class="col-num">Nợ (VND)</th>
            <th class="col-num">Có (VND)</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="line in store.trialBalanceLines"
            :key="line.accountCode"
            class="data-row"
          >
            <td class="code-cell font-mono">{{ line.accountCode }}</td>
            <td class="name-cell">{{ line.accountName }}</td>
            <td class="num-cell">{{ formatVnd(line.openingDebit) }}</td>
            <td class="num-cell">{{ formatVnd(line.openingCredit) }}</td>
            <td class="num-cell">{{ formatVnd(line.periodDebit) }}</td>
            <td class="num-cell">{{ formatVnd(line.periodCredit) }}</td>
            <td class="num-cell">{{ formatVnd(line.closingDebit) }}</td>
            <td class="num-cell">{{ formatVnd(line.closingCredit) }}</td>
          </tr>
        </tbody>
        <tfoot>
          <tr class="total-row">
            <td colspan="2" class="total-label">TỔNG CỘNG (VAS VALIDATION)</td>
            <td class="num-cell font-mono highlight">{{ formatVnd(store.totalOpeningDebit) }}</td>
            <td class="num-cell font-mono highlight">{{ formatVnd(store.totalOpeningCredit) }}</td>
            <td class="num-cell font-mono highlight">{{ formatVnd(store.totalPeriodDebit) }}</td>
            <td class="num-cell font-mono highlight">{{ formatVnd(store.totalPeriodCredit) }}</td>
            <td class="num-cell font-mono highlight">{{ formatVnd(store.totalClosingDebit) }}</td>
            <td class="num-cell font-mono highlight">{{ formatVnd(store.totalClosingCredit) }}</td>
          </tr>
        </tfoot>
      </table>
    </div>

    <div class="card-footer">
      <div class="footer-check">
        <span class="check-dot" :class="{ ok: store.isTrialBalanceBalanced }" />
        <span class="check-text">
          Đẳng thức 1: {{ store.totalOpeningDebit === store.totalOpeningCredit ? '✓ Đạt' : '✗ Lệch' }} |
          Đẳng thức 2: {{ store.totalPeriodDebit === store.totalPeriodCredit ? '✓ Đạt' : '✗ Lệch' }} |
          Đẳng thức 3: {{ store.totalClosingDebit === store.totalClosingCredit ? '✓ Đạt' : '✗ Lệch' }}
        </span>
      </div>
      <div class="zero-float-note">
        Đơn vị tính: Đồng Việt Nam (VND) · Zero Float Drift · Deterministic Integer Math
      </div>
    </div>
  </div>
</template>

<style scoped>
.trial-balance-card {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.header-tags {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.vas-badge {
  font-size: 11px;
  font-weight: 700;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.12);
  border: 1px solid rgba(56, 189, 248, 0.3);
  padding: 2px 8px;
  border-radius: 4px;
}

.status-pill {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 4px;
}

.status-pill.balanced {
  color: #10b981;
  background: rgba(16, 185, 129, 0.15);
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.status-pill.unbalanced {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.card-title {
  font-size: 16px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0 0 4px 0;
}

.card-desc {
  font-size: 12px;
  color: #94a3b8;
  margin: 0;
}

.period-pill {
  font-size: 12px;
  color: #cbd5e1;
  background: rgba(255, 255, 255, 0.06);
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.table-container {
  overflow-x: auto;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
}

.balance-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
  color: #e2e8f0;
}

.balance-table th,
.balance-table td {
  padding: 8px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  border-right: 1px solid rgba(255, 255, 255, 0.04);
}

.balance-table th {
  background: #0f172a;
  color: #94a3b8;
  font-weight: 600;
  text-align: center;
}

.sub-head th {
  background: #131d31;
  font-size: 11px;
}

.col-code {
  width: 70px;
}

.col-name {
  width: 220px;
  text-align: left !important;
}

.col-num {
  width: 120px;
  text-align: right !important;
}

.data-row:hover {
  background: rgba(255, 255, 255, 0.03);
}

.code-cell {
  color: #38bdf8;
  font-weight: 600;
  text-align: center;
}

.name-cell {
  text-align: left;
  color: #f1f5f9;
}

.num-cell {
  text-align: right;
}

.total-row {
  background: #0f172a;
  font-weight: 700;
}

.total-label {
  text-align: right;
  color: #38bdf8;
  letter-spacing: 0.5px;
}

.highlight {
  color: #10b981;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  padding-top: 4px;
}

.footer-check {
  display: flex;
  align-items: center;
  gap: 8px;
}

.check-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #ef4444;
}

.check-dot.ok {
  background: #10b981;
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.6);
}

.check-text {
  color: #cbd5e1;
  font-family: monospace;
}

.zero-float-note {
  color: #64748b;
  font-size: 11px;
  font-style: italic;
}
</style>
