<script setup lang="ts">
/**
 * SolvencyRatioGrid.vue — P71 Định Lượng Khả Năng Trả Nợ & Tỷ Số Thanh Khoản
 * =========================================================================
 * Hiển thị thẻ chỉ số DSCR, Quick Ratio, Current Ratio, Cash Ratio, ICR
 * với ngưỡng cảnh báo tiêu chuẩn ngân hàng Việt Nam (Basel II & TT 41/2016).
 */
import { useRiskStore } from '../../../stores/riskStore';

const store = useRiskStore();

function formatVnd(val: number): string {
  return `${val.toLocaleString('vi-VN')} ₫`;
}
</script>

<template>
  <div class="solvency-grid">
    <!-- 1. DSCR (Debt Service Coverage Ratio) Card -->
    <div class="ratio-card highlight" :class="store.dscrData.category.toLowerCase()">
      <div class="card-header">
        <div class="title-wrap">
          <span class="ratio-tag">DSCR</span>
          <span class="ratio-title">Hệ Số Phủ Nợ Vay</span>
        </div>
        <span class="badge" :class="store.dscrData.category.toLowerCase()">
          {{ store.dscrData.category }}
        </span>
      </div>

      <div class="ratio-value-row">
        <span class="ratio-number">{{ store.dscrData.ratio.toFixed(2) }}x</span>
        <span class="ratio-benchmark">Chuẩn: ≥ 1.30x</span>
      </div>

      <div class="buffer-indicator">
        <span class="buffer-label">Đệm dòng tiền an toàn (NOI - Nợ):</span>
        <strong
          class="buffer-val"
          :class="store.dscrData.buffer >= 0 ? 'text-green' : 'text-red'"
        >
          {{ store.dscrData.buffer >= 0 ? '+' : '' }}{{ formatVnd(store.dscrData.buffer) }}
        </strong>
      </div>

      <div class="ratio-details">
        <div class="detail-row">
          <span>Dòng tiền thuần (NOI):</span>
          <span>{{ formatVnd(store.netOperatingIncome) }}</span>
        </div>
        <div class="detail-row">
          <span>Tổng nghĩa vụ nợ (Gốc + Lãi):</span>
          <span>{{ formatVnd(store.totalDebtService) }}</span>
        </div>
      </div>
    </div>

    <!-- 2. Quick Ratio (Acid-Test) Card -->
    <div class="ratio-card" :class="store.quickRatioData.status.toLowerCase()">
      <div class="card-header">
        <div class="title-wrap">
          <span class="ratio-tag">QUICK</span>
          <span class="ratio-title">Thanh Toán Nhanh</span>
        </div>
        <span class="badge" :class="store.quickRatioData.status.toLowerCase()">
          {{ store.quickRatioData.status }}
        </span>
      </div>

      <div class="ratio-value-row">
        <span class="ratio-number">{{ store.quickRatioData.ratio.toFixed(2) }}x</span>
        <span class="ratio-benchmark">Chuẩn: ≥ 1.00x</span>
      </div>

      <div class="ratio-details mt-auto">
        <div class="detail-row">
          <span>Tài sản thanh khoản cao:</span>
          <span>{{ formatVnd(store.liquidAssets) }}</span>
        </div>
        <div class="detail-row">
          <span>Nợ ngắn hạn:</span>
          <span>{{ formatVnd(store.financialInput.currentLiabilities) }}</span>
        </div>
      </div>
    </div>

    <!-- 3. Current Ratio Card -->
    <div class="ratio-card" :class="store.currentRatioData.status.toLowerCase()">
      <div class="card-header">
        <div class="title-wrap">
          <span class="ratio-tag">CURRENT</span>
          <span class="ratio-title">Thanh Toán Hiện Hành</span>
        </div>
        <span class="badge" :class="store.currentRatioData.status.toLowerCase()">
          {{ store.currentRatioData.status }}
        </span>
      </div>

      <div class="ratio-value-row">
        <span class="ratio-number">{{ store.currentRatioData.ratio.toFixed(2) }}x</span>
        <span class="ratio-benchmark">Chuẩn: ≥ 1.50x</span>
      </div>

      <div class="ratio-details mt-auto">
        <div class="detail-row">
          <span>Tài sản ngắn hạn (gồm tồn kho):</span>
          <span>{{ formatVnd(store.liquidAssets + store.financialInput.inventory) }}</span>
        </div>
        <div class="detail-row">
          <span>Tồn kho:</span>
          <span>{{ formatVnd(store.financialInput.inventory) }}</span>
        </div>
      </div>
    </div>

    <!-- 4. Cash Ratio Card -->
    <div class="ratio-card" :class="store.cashRatioData.status.toLowerCase()">
      <div class="card-header">
        <div class="title-wrap">
          <span class="ratio-tag">CASH</span>
          <span class="ratio-title">Thanh Toán Tức Thời</span>
        </div>
        <span class="badge" :class="store.cashRatioData.status.toLowerCase()">
          {{ store.cashRatioData.status }}
        </span>
      </div>

      <div class="ratio-value-row">
        <span class="ratio-number">{{ store.cashRatioData.ratio.toFixed(2) }}x</span>
        <span class="ratio-benchmark">Chuẩn: ≥ 0.20x</span>
      </div>

      <div class="ratio-details mt-auto">
        <div class="detail-row">
          <span>Tiền mặt & tương đương:</span>
          <span>{{ formatVnd(store.financialInput.cashAndEquivalents + store.financialInput.marketableSecurities) }}</span>
        </div>
        <div class="detail-row">
          <span>Nợ ngắn hạn đến hạn:</span>
          <span>{{ formatVnd(store.financialInput.currentLiabilities) }}</span>
        </div>
      </div>
    </div>

    <!-- 5. ICR (Interest Coverage Ratio) Card -->
    <div class="ratio-card" :class="store.icrData.status.toLowerCase()">
      <div class="card-header">
        <div class="title-wrap">
          <span class="ratio-tag">ICR</span>
          <span class="ratio-title">Chi Trả Lãi Vay</span>
        </div>
        <span class="badge" :class="store.icrData.status.toLowerCase()">
          {{ store.icrData.status }}
        </span>
      </div>

      <div class="ratio-value-row">
        <span class="ratio-number">{{ store.icrData.ratio.toFixed(2) }}x</span>
        <span class="ratio-benchmark">Chuẩn: ≥ 2.00x</span>
      </div>

      <div class="ratio-details mt-auto">
        <div class="detail-row">
          <span>Lợi nhuận trước lãi & thuế (EBIT):</span>
          <span>{{ formatVnd(store.financialInput.ebit) }}</span>
        </div>
        <div class="detail-row">
          <span>Chi phí lãi vay:</span>
          <span>{{ formatVnd(store.financialInput.interestExpense) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.solvency-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 16px;
  width: 100%;
}

.ratio-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.ratio-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.06);
}

.ratio-card.highlight {
  border-left: 4px solid #2563eb;
}

.ratio-card.healthy,
.ratio-card.strong {
  border-left: 4px solid #10b981;
}

.ratio-card.watchlist,
.ratio-card.adequate {
  border-left: 4px solid #f59e0b;
}

.ratio-card.distressed,
.ratio-card.critical {
  border-left: 4px solid #ef4444;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.title-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ratio-tag {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 800;
  background: #f1f5f9;
  color: #334155;
  padding: 2px 6px;
  border-radius: 4px;
}

.ratio-title {
  font-size: 13px;
  font-weight: 700;
  color: #0f172a;
}

.badge {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 12px;
  text-transform: uppercase;
}

.badge.healthy,
.badge.strong {
  background: #dcfce7;
  color: #166534;
}

.badge.watchlist,
.badge.adequate {
  background: #fef3c7;
  color: #92400e;
}

.badge.distressed,
.badge.critical {
  background: #fee2e2;
  color: #991b1b;
}

.badge.debt_free {
  background: #e0e7ff;
  color: #3730a3;
}

.ratio-value-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
}

.ratio-number {
  font-family: 'JetBrains Mono', monospace;
  font-size: 26px;
  font-weight: 800;
  color: #0f172a;
}

.ratio-benchmark {
  font-size: 11px;
  color: #64748b;
}

.buffer-indicator {
  background: #f8fafc;
  border: 1px dashed #cbd5e1;
  border-radius: 6px;
  padding: 8px 10px;
  font-size: 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.buffer-label {
  color: #64748b;
  font-size: 11px;
}

.buffer-val {
  font-family: 'JetBrains Mono', monospace;
  font-weight: 700;
}

.text-green {
  color: #10b981;
}

.text-red {
  color: #ef4444;
}

.ratio-details {
  display: flex;
  flex-direction: column;
  gap: 6px;
  border-top: 1px solid #f1f5f9;
  padding-top: 10px;
  font-size: 11px;
  color: #64748b;
}

.detail-row {
  display: flex;
  justify-content: space-between;
}

.mt-auto {
  margin-top: auto;
}
</style>
