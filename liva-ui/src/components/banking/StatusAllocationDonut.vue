<script setup lang="ts">
/**
 * StatusAllocationDonut.vue
 * Status Allocation Pie/Donut Chart matching user mockup 1:1.
 */
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    matchedRate?: number;
    unmatchedRate?: number;
  }>(),
  {
    matchedRate: 99.8,
    unmatchedRate: 0.2,
  }
);

// SVG Circle parameters
// Center (100, 100), Radius 55, Circumference = 2 * PI * 55 ≈ 345.575
const radius = 55;
const circumference = 2 * Math.PI * radius;

const matchedDashOffset = computed(() => {
  return circumference * (1 - props.matchedRate / 100);
});
</script>

<template>
  <div class="status-allocation-card">
    <div class="card-header">
      <h3 class="card-title">PHÂN BỔ TRẠNG THÁI</h3>
    </div>

    <!-- SVG Donut Chart Canvas with Callouts -->
    <div class="donut-chart-container">
      <svg viewBox="0 0 240 180" class="donut-svg">
        <!-- Callout Left: Chưa khớp 0.2% -->
        <g class="callout-left">
          <text x="35" y="45" class="callout-label" text-anchor="middle">Chưa khớp</text>
          <text x="35" y="60" class="callout-percent text-red" text-anchor="middle">{{ unmatchedRate.toFixed(1) }}%</text>
          <!-- Pointer line -->
          <polyline points="55,50 85,50 98,72" fill="none" stroke="#cbd5e1" stroke-width="1.2" />
        </g>

        <!-- Donut Ring / Pie -->
        <g transform="rotate(-90 120 95)">
          <!-- Background / Unmatched Red Ring -->
          <circle
            cx="120"
            cy="95"
            :r="radius"
            fill="none"
            stroke="#ef4444"
            stroke-width="38"
          />

          <!-- Matched Green Arc -->
          <circle
            cx="120"
            cy="95"
            :r="radius"
            fill="none"
            stroke="#10b981"
            stroke-width="38"
            :stroke-dasharray="circumference"
            :stroke-dashoffset="matchedDashOffset"
            class="donut-progress-arc"
          />
        </g>

        <!-- Center Hole for Donut appearance -->
        <circle cx="120" cy="95" r="32" fill="#ffffff" />

        <!-- Callout Right: Khớp 99.8% -->
        <g class="callout-right">
          <polyline points="142,120 160,140 185,140" fill="none" stroke="#cbd5e1" stroke-width="1.2" />
          <text x="205" y="137" class="callout-label" text-anchor="start">Khớp</text>
          <text x="205" y="152" class="callout-percent text-green" text-anchor="start">{{ matchedRate.toFixed(1) }}%</text>
        </g>
      </svg>
    </div>
  </div>
</template>

<style scoped>
.status-allocation-card {
  background: #ffffff;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  padding: 18px 20px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 260px;
}

.card-header {
  margin-bottom: 8px;
}

.card-title {
  font-size: 14px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.donut-chart-container {
  width: 100%;
  height: 180px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.donut-svg {
  width: 100%;
  height: 100%;
  max-width: 260px;
  overflow: visible;
}

.donut-progress-arc {
  transition: stroke-dashoffset 0.8s cubic-bezier(0.4, 0, 0.2, 1);
}

.callout-label {
  font-size: 11px;
  font-weight: 600;
  fill: #475569;
}

.callout-percent {
  font-size: 12px;
  font-weight: 800;
}

.text-red {
  fill: #ef4444;
}

.text-green {
  fill: #10b981;
}
</style>
