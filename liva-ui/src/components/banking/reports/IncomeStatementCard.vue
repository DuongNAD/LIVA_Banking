<script setup lang="ts">
/**
 * IncomeStatementCard.vue — P92 Báo Cáo Kết Quả Hoạt Động Kinh Doanh (Mẫu B02-DN)
 * =================================================================================
 * Trực quan hóa Báo Cáo Kết Quả Kinh Doanh dạng bậc thang tài chính (Waterfall).
 * Hiển thị chi tiết Doanh thu thuần, Giá vốn, Chi phí hoạt động, Lợi nhuận trước & sau thuế.
 */
import { useReportStore } from '../../../stores/reportStore';

const store = useReportStore();

function formatVnd(val: number): string {
  if (val === 0) return '0 ₫';
  return `${val.toLocaleString('vi-VN')} ₫`;
}
</script>

<template>
  <div class="income-statement-card">
    <div class="card-header">
      <div>
        <div class="header-tags">
          <span class="vas-badge">MẪU B02-DN (TT 200/2014/TT-BTC)</span>
          <span class="kpi-pill">
            Biên Lợi Nhuận Gộp: <strong>{{ (store.grossProfitMarginBps / 100).toFixed(2) }}%</strong>
          </span>
          <span class="kpi-pill net">
            Biên Lợi Nhuận Ròng: <strong>{{ (store.netProfitMarginBps / 100).toFixed(2) }}%</strong>
          </span>
        </div>
        <h3 class="card-title">Báo Cáo Kết Quả Hoạt Động Kinh Doanh (Income Statement)</h3>
        <p class="card-desc">
          Số liệu kế toán lũy kế kỳ báo cáo {{ store.currentPeriod }}. Thuế suất TNDN phổ thông 20%.
        </p>
      </div>
    </div>

    <!-- P&L Breakdown Table -->
    <div class="pnl-table-wrap">
      <table class="pnl-table">
        <thead>
          <tr>
            <th class="col-stt">Mã</th>
            <th class="col-item">Chỉ tiêu tài chính</th>
            <th class="col-val">Số tiền (VND)</th>
            <th class="col-pct">% Doanh thu</th>
          </tr>
        </thead>
        <tbody>
          <!-- Doanh thu -->
          <tr class="row-sub">
            <td class="cell-code">01</td>
            <td class="cell-name">1. Doanh thu bán hàng và cung cấp dịch vụ</td>
            <td class="cell-val">{{ formatVnd(store.incomeStatement.grossRevenue) }}</td>
            <td class="cell-pct">100.0%</td>
          </tr>
          <tr class="row-minor">
            <td class="cell-code">02</td>
            <td class="cell-name indent">2. Các khoản giảm trừ doanh thu (chiết khấu, giảm giá)</td>
            <td class="cell-val">{{ formatVnd(store.incomeStatement.revenueDeductions) }}</td>
            <td class="cell-pct">-</td>
          </tr>
          <tr class="row-highlight">
            <td class="cell-code">10</td>
            <td class="cell-name font-bold">3. Doanh thu thuần về bán hàng và cung cấp dịch vụ (10 = 01 - 02)</td>
            <td class="cell-val font-bold text-accent">{{ formatVnd(store.incomeStatement.netRevenue) }}</td>
            <td class="cell-pct font-bold">100.0%</td>
          </tr>

          <!-- Giá vốn -->
          <tr class="row-sub">
            <td class="cell-code">11</td>
            <td class="cell-name">4. Giá vốn hàng bán</td>
            <td class="cell-val text-neg">- {{ formatVnd(store.incomeStatement.cogs) }}</td>
            <td class="cell-pct text-neg">
              {{ ((store.incomeStatement.cogs * 100) / store.incomeStatement.netRevenue).toFixed(1) }}%
            </td>
          </tr>
          <tr class="row-highlight">
            <td class="cell-code">20</td>
            <td class="cell-name font-bold">5. Lợi nhuận gộp về bán hàng và CCDV (20 = 10 - 11)</td>
            <td class="cell-val font-bold text-success">{{ formatVnd(store.incomeStatement.grossProfit) }}</td>
            <td class="cell-pct font-bold text-success">
              {{ (store.grossProfitMarginBps / 100).toFixed(1) }}%
            </td>
          </tr>

          <!-- Chi phí hoạt động -->
          <tr class="row-minor">
            <td class="cell-code">21</td>
            <td class="cell-name indent">6. Doanh thu hoạt động tài chính (lãi tiền gửi, tỷ giá)</td>
            <td class="cell-val text-pos">+ {{ formatVnd(store.incomeStatement.financialIncome) }}</td>
            <td class="cell-pct">
              {{ ((store.incomeStatement.financialIncome * 100) / store.incomeStatement.netRevenue).toFixed(1) }}%
            </td>
          </tr>
          <tr class="row-minor">
            <td class="cell-code">22</td>
            <td class="cell-name indent">7. Chi phí tài chính</td>
            <td class="cell-val text-neg">- {{ formatVnd(store.incomeStatement.financialExpense) }}</td>
            <td class="cell-pct">
              {{ ((store.incomeStatement.financialExpense * 100) / store.incomeStatement.netRevenue).toFixed(1) }}%
            </td>
          </tr>
          <tr class="row-sub-minor">
            <td class="cell-code">23</td>
            <td class="cell-name sub-indent">— Trong đó: Chi phí lãi vay ngân hàng</td>
            <td class="cell-val text-muted">({{ formatVnd(store.incomeStatement.interestExpense) }})</td>
            <td class="cell-pct text-muted">-</td>
          </tr>
          <tr class="row-minor">
            <td class="cell-code">25</td>
            <td class="cell-name indent">8. Chi phí bán hàng</td>
            <td class="cell-val text-neg">- {{ formatVnd(store.incomeStatement.sellingExpense) }}</td>
            <td class="cell-pct">
              {{ ((store.incomeStatement.sellingExpense * 100) / store.incomeStatement.netRevenue).toFixed(1) }}%
            </td>
          </tr>
          <tr class="row-minor">
            <td class="cell-code">26</td>
            <td class="cell-name indent">9. Chi phí quản lý doanh nghiệp</td>
            <td class="cell-val text-neg">- {{ formatVnd(store.incomeStatement.adminExpense) }}</td>
            <td class="cell-pct">
              {{ ((store.incomeStatement.adminExpense * 100) / store.incomeStatement.netRevenue).toFixed(1) }}%
            </td>
          </tr>

          <!-- Lợi nhuận HĐKD -->
          <tr class="row-highlight">
            <td class="cell-code">30</td>
            <td class="cell-name font-bold">10. Lợi nhuận thuần từ hoạt động kinh doanh (30 = 20 + 21 - 22 - 25 - 26)</td>
            <td class="cell-val font-bold text-success">{{ formatVnd(store.incomeStatement.operatingProfit) }}</td>
            <td class="cell-pct font-bold">
              {{ ((store.incomeStatement.operatingProfit * 100) / store.incomeStatement.netRevenue).toFixed(1) }}%
            </td>
          </tr>

          <!-- Khác -->
          <tr class="row-minor">
            <td class="cell-code">40</td>
            <td class="cell-name indent">11. Lợi nhuận khác (Mã 31 Thu nhập khác - Mã 32 Chi phí khác)</td>
            <td class="cell-val text-pos">+ {{ formatVnd(store.incomeStatement.otherProfit) }}</td>
            <td class="cell-pct">-</td>
          </tr>

          <!-- PBT -->
          <tr class="row-highlight">
            <td class="cell-code">50</td>
            <td class="cell-name font-bold">12. Tổng lợi nhuận kế toán trước thuế (50 = 30 + 40)</td>
            <td class="cell-val font-bold text-accent">{{ formatVnd(store.incomeStatement.profitBeforeTax) }}</td>
            <td class="cell-pct font-bold">
              {{ ((store.incomeStatement.profitBeforeTax * 100) / store.incomeStatement.netRevenue).toFixed(1) }}%
            </td>
          </tr>

          <!-- Thuế TNDN -->
          <tr class="row-sub">
            <td class="cell-code">51</td>
            <td class="cell-name indent">13. Chi phí thuế thu nhập doanh nghiệp hiện hành (20%)</td>
            <td class="cell-val text-neg">- {{ formatVnd(store.incomeStatement.currentCitExpense) }}</td>
            <td class="cell-pct text-neg">
              {{ ((store.incomeStatement.currentCitExpense * 100) / store.incomeStatement.netRevenue).toFixed(1) }}%
            </td>
          </tr>

          <!-- PAT -->
          <tr class="row-final">
            <td class="cell-code font-bold">60</td>
            <td class="cell-name font-bold text-gold">14. LỢI NHUẬN SAU THUẾ THU NHẬP DOANH NGHIỆP (60 = 50 - 51)</td>
            <td class="cell-val font-bold text-gold font-mono">{{ formatVnd(store.incomeStatement.netProfitAfterTax) }}</td>
            <td class="cell-pct font-bold text-gold">
              {{ (store.netProfitMarginBps / 100).toFixed(1) }}%
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.income-statement-card {
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
  flex-wrap: wrap;
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

.kpi-pill {
  font-size: 11px;
  color: #94a3b8;
  background: rgba(255, 255, 255, 0.05);
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.kpi-pill.net {
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.1);
  border-color: rgba(245, 158, 11, 0.3);
}

.kpi-pill strong {
  color: #f8fafc;
}

.kpi-pill.net strong {
  color: #f59e0b;
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

.pnl-table-wrap {
  overflow-x: auto;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
}

.pnl-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
  color: #e2e8f0;
}

.pnl-table th,
.pnl-table td {
  padding: 8px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.pnl-table th {
  background: #0f172a;
  color: #94a3b8;
  font-weight: 600;
  text-align: left;
}

.col-stt {
  width: 50px;
  text-align: center !important;
}

.col-item {
  width: auto;
}

.col-val {
  width: 180px;
  text-align: right !important;
}

.col-pct {
  width: 100px;
  text-align: right !important;
}

.cell-code {
  color: #64748b;
  text-align: center;
  font-family: monospace;
}

.cell-name {
  color: #cbd5e1;
}

.cell-name.indent {
  padding-left: 24px;
}

.cell-name.sub-indent {
  padding-left: 40px;
  font-size: 11px;
  color: #94a3b8;
}

.cell-val {
  text-align: right;
  font-family: monospace;
}

.cell-pct {
  text-align: right;
  color: #94a3b8;
}

.row-highlight {
  background: rgba(56, 189, 248, 0.04);
}

.row-final {
  background: rgba(245, 158, 11, 0.08);
  border-top: 2px solid rgba(245, 158, 11, 0.3);
}

.text-accent {
  color: #38bdf8;
}

.text-success {
  color: #10b981;
}

.text-gold {
  color: #f59e0b;
}

.text-neg {
  color: #f87171;
}

.text-pos {
  color: #34d399;
}

.text-muted {
  color: #64748b;
}

.font-bold {
  font-weight: 700;
}
</style>
