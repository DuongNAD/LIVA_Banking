<script setup lang="ts">
/**
 * AutoReconciliationGauge.vue
 * Semi-circular gauge meter card matching user mockup 1:1.
 * Real-time match rate gauge targeting 99.8% with SLA indicator.
 */
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    reconciledCount?: number;
    totalCount?: number;
    reconciledRate?: number;
    autoRate?: number;
    manualRate?: number;
    unmatchedCount?: number;
    unmatchedRate?: number;
    targetRate?: number;
  }>(),
  {
    reconciledCount: 1842,
    totalCount: 1845,
    reconciledRate: 99.8,
    autoRate: 99.2,
    manualRate: 0.6,
    unmatchedCount: 3,
    unmatchedRate: 0.2,
    targetRate: 99.8,
  }
);

// Semi-circle arc geometry
// Radius = 75, Center = (100, 95)
// Semi-circumference = Math.PI * 75 ≈ 235.62
const arcCircumference = Math.PI * 75;
const strokeDashoffset = computed(() => {
  const fraction = Math.min(1, Math.max(0, props.reconciledRate / 100));
  return arcCircumference * (1 - fraction);
});

const isTargetAchieved = computed(() => {
  return props.reconciledRate >= props.targetRate;
});
</script>

<template>
  <div class="auto-reconciliation-card">
    <div class="card-header">
      <div class="header-titles">
        <h3 class="card-title">KẾT QUẢ ĐỐI SOÁT TỰ ĐỘNG</h3>
        <span class="target-subtitle">Mục tiêu: {{ targetRate.toFixed(1) }}% (Auto-Reconcile Target)</span>
      </div>
      <span
        class="target-badge"
        :class="isTargetAchieved ? 'badge-success' : 'badge-warning'"
      >
        {{ isTargetAchieved ? '✓ ĐẠT CHỈ TIÊU' : '⚠ CẦN TỐI ƯU' }}
      </span>
    </div>

    <!-- Semi-circular SVG Gauge -->
    <div class="gauge-wrapper">
      <svg class="gauge-svg" viewBox="0 0 200 115" width="200" height="115">
        <!-- Background Track -->
        <path
          d="M 25 100 A 75 75 0 0 1 175 100"
          fill="none"
          stroke="#e2e8f0"
          stroke-width="16"
          stroke-linecap="round"
        />

        <!-- Active Green Progress Arc -->
        <path
          d="M 25 100 A 75 75 0 0 1 175 100"
          fill="none"
          stroke="#10b981"
          stroke-width="16"
          stroke-linecap="round"
          :stroke-dasharray="arcCircumference"
          :stroke-dashoffset="strokeDashoffset"
          class="gauge-progress-path"
        />
      </svg>

      <!-- Center Typography inside gauge -->
      <div class="gauge-center-text">
        <span class="gauge-percentage">{{ reconciledRate.toFixed(1) }}%</span>
        <span class="gauge-subtitle">Tỷ lệ khớp</span>
      </div>
    </div>

    <!-- Summary Details Under Gauge -->
    <div class="gauge-details">
      <div class="matched-summary-line">
        <span class="matched-text-vn">Đã khớp: {{ reconciledCount.toLocaleString() }}/{{ totalCount.toLocaleString() }} GD</span>
        <span class="matched-text-en">Matches: {{ reconciledCount.toLocaleString() }}/{{ totalCount.toLocaleString() }} Trx</span>
      </div>

      <div class="breakdown-table">
        <div class="breakdown-row">
          <span class="breakdown-label">Tự động:</span>
          <span class="breakdown-val text-green font-bold">{{ autoRate.toFixed(1) }}%</span>
        </div>
        <div class="breakdown-row">
          <span class="breakdown-label">Bằng tay:</span>
          <span class="breakdown-val text-amber">{{ manualRate.toFixed(1) }}%</span>
        </div>
        <div class="breakdown-row">
          <span class="breakdown-label">Chưa khớp:</span>
          <span class="breakdown-val text-red font-bold">{{ unmatchedCount }} ({{ unmatchedRate.toFixed(1) }}%)</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.auto-reconciliation-card {
  background: #ffffff;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  padding: 18px 20px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 8px;
}

.header-titles {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.card-title {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: #1e293b;
  margin: 0;
}

.target-subtitle {
  font-size: 11px;
  color: #64748b;
}

.target-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 3px 8px;
  border-radius: 4px;
  letter-spacing: 0.3px;
}

.badge-success {
  background: #ecfdf5;
  color: #059669;
  border: 1px solid #a7f3d0;
}

.badge-warning {
  background: #fffbeb;
  color: #d97706;
  border: 1px solid #fde68a;
}

.gauge-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 8px 0 4px 0;
}

.gauge-svg {
  overflow: visible;
}

.gauge-progress-path {
  transition: stroke-dashoffset 0.6s cubic-bezier(0.4, 0, 0.2, 1);
}

.gauge-center-text {
  position: absolute;
  bottom: 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.gauge-percentage {
  font-size: 32px;
  font-weight: 800;
  color: #0f172a;
  line-height: 1;
  font-feature-settings: 'tnum';
}

.gauge-subtitle {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  margin-top: 2px;
}

.gauge-details {
  border-top: 1px solid #f1f5f9;
  padding-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.matched-summary-line {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  font-weight: 600;
}

.matched-text-vn {
  color: #1e293b;
}

.matched-text-en {
  color: #94a3b8;
  font-size: 11px;
}

.breakdown-table {
  display: flex;
  justify-content: space-between;
  background: #f8fafc;
  border-radius: 8px;
  padding: 8px 12px;
  font-size: 12px;
}

.breakdown-row {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.breakdown-label {
  color: #64748b;
  font-size: 11px;
}

.breakdown-val {
  font-weight: 700;
  font-feature-settings: 'tnum';
}

.text-green {
  color: #10b981;
}

.text-amber {
  color: #f59e0b;
}

.text-red {
  color: #ef4444;
}
</style>
