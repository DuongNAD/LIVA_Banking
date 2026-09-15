<script setup lang="ts">
/**
 * RollingCashflowSentinelCard.vue — P61 Rolling 30/90-Day Cashflow Sentinel
 * =========================================================================
 * Visualizes projected liquidity trajectory, runway countdown,
 * and early warnings against minimum operating reserve breaches.
 */
import { computed } from 'vue';
import { useTreasuryStore } from '../../../stores/treasuryStore';

const store = useTreasuryStore();

function formatVnd(val: number): string {
  return `${val.toLocaleString('vi-VN')} ₫`;
}

// Compute SVG Chart coordinates
const chartPoints = computed(() => {
  const data = store.projectedCashflowData;
  const width = 700;
  const height = 160;
  const padding = 20;

  if (data.length === 0) return { path: '', points: [], width, height, reserveY: 80 };

  const maxVal = Math.max(...data.map((d) => d.projectedBalance), store.minOperatingReserve * 1.5);
  const minVal = Math.min(...data.map((d) => d.projectedBalance), 0);
  const range = maxVal - minVal || 1;

  const pts = data.map((d, i) => {
    const x = padding + (i / (data.length - 1)) * (width - 2 * padding);
    const y = height - padding - ((d.projectedBalance - minVal) / range) * (height - 2 * padding);
    return { x, y, data: d };
  });

  const path = pts.reduce((acc, p, i) => `${acc} ${i === 0 ? 'M' : 'L'} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`, '');

  // Calculate y position for 500M minimum reserve line
  const reserveY = height - padding - ((store.minOperatingReserve - minVal) / range) * (height - 2 * padding);

  return { path, points: pts, width, height, reserveY };
});
</script>

<template>
  <div class="sentinel-card">
    <!-- Sentinel Header & Horizon Switcher -->
    <div class="sentinel-header">
      <div class="header-left">
        <span class="sentinel-tag">P61 SENTINEL</span>
        <h3 class="sentinel-title">Tháp Canh Dự Báo Dòng Tiền & Cảnh Báo Sớm Thanh Khoản</h3>
      </div>

      <div class="horizon-switch-group">
        <span class="switch-lbl">Kỳ dự báo:</span>
        <button
          class="horizon-btn"
          :class="{ active: store.projectionHorizonDays === 30 }"
          @click="store.projectionHorizonDays = 30"
        >
          30 Ngày
        </button>
        <button
          class="horizon-btn"
          :class="{ active: store.projectionHorizonDays === 60 }"
          @click="store.projectionHorizonDays = 60"
        >
          60 Ngày
        </button>
        <button
          class="horizon-btn"
          :class="{ active: store.projectionHorizonDays === 90 }"
          @click="store.projectionHorizonDays = 90"
        >
          90 Ngày
        </button>
      </div>
    </div>

    <!-- Health Status Banner -->
    <div class="health-banner" :class="store.liquidityHealthStatus.level.toLowerCase()">
      <div class="banner-status-icon">
        {{ store.liquidityHealthStatus.level === 'SAFE' ? '🛡️' : '⚠️' }}
      </div>
      <div class="banner-text">
        <div class="status-title">{{ store.liquidityHealthStatus.title }}</div>
        <div class="status-desc">{{ store.liquidityHealthStatus.desc }}</div>
      </div>
      <div class="runway-stat">
        <span class="runway-lbl">Runway Thanh Khoản:</span>
        <span class="runway-days">
          {{ store.runwayDays > 90 ? '> 90 ngày' : `${store.runwayDays} ngày` }}
        </span>
      </div>
    </div>

    <!-- SVG Trajectory Chart -->
    <div class="chart-box">
      <div class="chart-meta-row">
        <span class="legend-item trajectory">
          <span class="dot line-dot" /> Quỹ đạo số dư dự báo (Projected Cash)
        </span>
        <span class="legend-item reserve">
          <span class="dot red-dot" /> Ngưỡng đệm thanh khoản an toàn (500M VND)
        </span>
      </div>

      <div class="svg-wrapper">
        <svg :viewBox="`0 0 ${chartPoints.width} ${chartPoints.height}`" class="sentinel-svg">
          <!-- Horizontal Grid Line for 500M Reserve -->
          <line
            x1="20"
            :y1="chartPoints.reserveY"
            :x2="chartPoints.width - 20"
            :y2="chartPoints.reserveY"
            stroke="#ef4444"
            stroke-dasharray="4 4"
            stroke-width="1.5"
          />
          <text
            :x="chartPoints.width - 160"
            :y="chartPoints.reserveY - 6"
            fill="#ef4444"
            font-size="10"
            font-weight="700"
          >
            Đệm an toàn: 500M ₫
          </text>

          <!-- Projected Cash Curve -->
          <path
            :d="chartPoints.path"
            fill="none"
            stroke="#38bdf8"
            stroke-width="2.5"
            stroke-linecap="round"
          />

          <!-- Interactive Points -->
          <circle
            v-for="(p, idx) in chartPoints.points.filter((_, i) => i % Math.ceil(chartPoints.points.length / 10) === 0)"
            :key="idx"
            :cx="p.x"
            :cy="p.y"
            r="4"
            :fill="p.data.isBreached ? '#ef4444' : '#38bdf8'"
            stroke="#0f172a"
            stroke-width="2"
          />
        </svg>
      </div>

      <!-- Footer Stats -->
      <div class="chart-summary-bar">
        <div v-if="store.lowestProjectedPoint" class="summary-item">
          <span class="s-lbl">Điểm đáy dự báo:</span>
          <span class="s-val text-amber">
            {{ formatVnd(store.lowestProjectedPoint.projectedBalance) }} (Ngày {{ store.lowestProjectedPoint.date }})
          </span>
        </div>
        <div class="summary-item">
          <span class="s-lbl">Tổng số dư khả dụng hiện tại:</span>
          <span class="s-val text-green">{{ formatVnd(store.totalCashPosition) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sentinel-card {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.sentinel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.sentinel-tag {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 800;
  background: rgba(56, 189, 248, 0.2);
  color: #38bdf8;
  padding: 2px 6px;
  border-radius: 4px;
}

.sentinel-title {
  font-size: 15px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0;
}

.horizon-switch-group {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  padding: 3px;
}

.switch-lbl {
  font-size: 11px;
  color: #64748b;
  padding-left: 6px;
}

.horizon-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 11px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.horizon-btn.active {
  background: #3b82f6;
  color: #ffffff;
}

.health-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-radius: 8px;
  padding: 12px 18px;
  gap: 14px;
}

.health-banner.safe {
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: #34d399;
}

.health-banner.warning {
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.3);
  color: #fbbf24;
}

.health-banner.critical {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #f87171;
}

.banner-status-icon {
  font-size: 24px;
}

.banner-text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.status-title {
  font-size: 13px;
  font-weight: 700;
}

.status-desc {
  font-size: 12px;
  color: #cbd5e1;
}

.runway-stat {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
}

.runway-lbl {
  font-size: 10px;
  color: #64748b;
  text-transform: uppercase;
  font-weight: 700;
}

.runway-days {
  font-family: 'JetBrains Mono', monospace;
  font-size: 18px;
  font-weight: 800;
}

.chart-box {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.chart-meta-row {
  display: flex;
  align-items: center;
  gap: 18px;
  font-size: 11px;
  color: #94a3b8;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.line-dot {
  background: #38bdf8;
}

.red-dot {
  background: #ef4444;
}

.svg-wrapper {
  width: 100%;
  height: 160px;
}

.sentinel-svg {
  width: 100%;
  height: 100%;
}

.chart-summary-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  padding-top: 10px;
  font-size: 12px;
}

.summary-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.s-lbl {
  color: #64748b;
}

.s-val {
  font-family: 'JetBrains Mono', monospace;
  font-weight: 700;
}

.s-val.text-amber {
  color: #fbbf24;
}

.s-val.text-green {
  color: #34d399;
}
</style>
