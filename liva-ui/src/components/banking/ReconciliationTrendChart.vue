<script setup lang="ts">
/**
 * ReconciliationTrendChart.vue
 * 30-Day SVG Line Chart matching user mockup 1:1.
 */
import { ref, computed } from 'vue';
import type { TrendDataPoint } from '../../stores/bankingStore';

const props = withDefaults(
  defineProps<{
    data?: TrendDataPoint[];
    filterLabel?: string;
  }>(),
  {
    filterLabel: 'Last lướt 30 ngày',
    data: () => [
      { date: '10/1', matched: 130, unmatched: 15 },
      { date: '10/3', matched: 60, unmatched: 35 },
      { date: '10/6', matched: 75, unmatched: 22 },
      { date: '10/9', matched: 125, unmatched: 18 },
      { date: '10/11', matched: 68, unmatched: 25 },
      { date: '10/13', matched: 140, unmatched: 12 },
      { date: '10/15', matched: 80, unmatched: 30 },
      { date: '10/17', matched: 175, unmatched: 8 },
      { date: '10/20', matched: 100, unmatched: 16 },
      { date: '10/23', matched: 128, unmatched: 20 },
      { date: '10/26', matched: 95, unmatched: 15 },
      { date: '10/29', matched: 148, unmatched: 10 },
      { date: '10/30', matched: 120, unmatched: 5 },
    ],
  }
);

// Chart Dimensions
const width = 500;
const height = 180;
const padding = { top: 20, right: 20, bottom: 30, left: 35 };

const chartWidth = width - padding.left - padding.right;
const chartHeight = height - padding.top - padding.bottom;

const maxY = 200;
const yTicks = [0, 50, 100, 150, 200];

// Compute coordinate points
const xStep = computed(() => {
  const count = props.data.length;
  return count > 1 ? chartWidth / (count - 1) : chartWidth;
});

function getY(val: number): number {
  const clamped = Math.min(maxY, Math.max(0, val));
  return padding.top + chartHeight - (clamped / maxY) * chartHeight;
}

function getX(index: number): number {
  return padding.left + index * xStep.value;
}

const matchedPoints = computed(() => {
  return props.data.map((d, i) => `${getX(i)},${getY(d.matched)}`).join(' ');
});

const unmatchedPoints = computed(() => {
  return props.data.map((d, i) => `${getX(i)},${getY(d.unmatched)}`).join(' ');
});

const matchedAreaPath = computed(() => {
  if (!props.data.length) return '';
  const firstX = getX(0);
  const lastX = getX(props.data.length - 1);
  const baseline = padding.top + chartHeight;
  return `M ${firstX},${baseline} L ${props.data.map((d, i) => `${getX(i)},${getY(d.matched)}`).join(' L ')} L ${lastX},${baseline} Z`;
});

// Interactive hover
const hoveredIndex = ref<number | null>(null);

function onMouseMove(e: MouseEvent) {
  const target = e.currentTarget as HTMLElement;
  const rect = target.getBoundingClientRect();
  const mouseX = e.clientX - rect.left - padding.left;
  const rawIdx = Math.round(mouseX / xStep.value);
  if (rawIdx >= 0 && rawIdx < props.data.length) {
    hoveredIndex.value = rawIdx;
  }
}

function onMouseLeave() {
  hoveredIndex.value = null;
}
</script>

<template>
  <div class="trend-chart-card">
    <div class="trend-header">
      <h3 class="trend-title">XU HƯỚNG ĐỐI SOÁT</h3>

      <div class="trend-controls">
        <button class="filter-pill-btn">
          <span>{{ filterLabel }}</span>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Chart Legend -->
    <div class="chart-legend">
      <div class="legend-item">
        <span class="legend-line line-green" />
        <span class="legend-text">Matched</span>
      </div>
      <div class="legend-item">
        <span class="legend-line line-gray" />
        <span class="legend-text">Unmatched</span>
      </div>
    </div>

    <!-- SVG Line Chart Canvas -->
    <div class="svg-container" @mousemove="onMouseMove" @mouseleave="onMouseLeave">
      <svg :viewBox="`0 0 ${width} ${height}`" preserveAspectRatio="none" class="line-svg">
        <defs>
          <linearGradient id="matchedGrad" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#10b981" stop-opacity="0.18" />
            <stop offset="100%" stop-color="#10b981" stop-opacity="0.0" />
          </linearGradient>
        </defs>

        <!-- Horizontal Grid Lines & Y-labels -->
        <g class="grid-lines">
          <template v-for="tick in yTicks" :key="tick">
            <line
              :x1="padding.left"
              :y1="getY(tick)"
              :x2="width - padding.right"
              :y2="getY(tick)"
              stroke="#f1f5f9"
              stroke-width="1"
            />
            <text
              :x="padding.left - 8"
              :y="getY(tick) + 4"
              text-anchor="end"
              class="axis-text"
            >
              {{ tick }}
            </text>
          </template>
        </g>

        <!-- Area under matched line -->
        <path :d="matchedAreaPath" fill="url(#matchedGrad)" />

        <!-- Unmatched Polyline (Grey) -->
        <polyline
          :points="unmatchedPoints"
          fill="none"
          stroke="#64748b"
          stroke-width="2.2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />

        <!-- Matched Polyline (Green) -->
        <polyline
          :points="matchedPoints"
          fill="none"
          stroke="#10b981"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />

        <!-- X-axis Date Labels -->
        <g class="x-labels">
          <template v-for="(item, idx) in data" :key="item.date">
            <text
              v-if="idx % 2 === 0 || idx === data.length - 1"
              :x="getX(idx)"
              :y="height - 8"
              text-anchor="middle"
              class="axis-text"
            >
              {{ item.date }}
            </text>
          </template>
        </g>

        <!-- Hover Indicator Guideline & Dots -->
        <g v-if="hoveredIndex !== null">
          <line
            :x1="getX(hoveredIndex)"
            :y1="padding.top"
            :x2="getX(hoveredIndex)"
            :y2="padding.top + chartHeight"
            stroke="#94a3b8"
            stroke-dasharray="3,3"
            stroke-width="1"
          />
          <circle
            :cx="getX(hoveredIndex)"
            :cy="getY(data[hoveredIndex].matched)"
            r="4.5"
            fill="#10b981"
            stroke="#ffffff"
            stroke-width="2"
          />
          <circle
            :cx="getX(hoveredIndex)"
            :cy="getY(data[hoveredIndex].unmatched)"
            r="4.5"
            fill="#64748b"
            stroke="#ffffff"
            stroke-width="2"
          />
        </g>
      </svg>

      <!-- Hover Tooltip Overlay -->
      <div
        v-if="hoveredIndex !== null"
        class="chart-tooltip"
        :style="{
          left: `${(getX(hoveredIndex) / width) * 100}%`,
          top: `${(getY(data[hoveredIndex].matched) / height) * 100 - 45}%`,
        }"
      >
        <span class="tooltip-date">{{ data[hoveredIndex].date }}</span>
        <span class="tooltip-val text-green">Khớp: {{ data[hoveredIndex].matched }}</span>
        <span class="tooltip-val text-gray">Lệch: {{ data[hoveredIndex].unmatched }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.trend-chart-card {
  background: #ffffff;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  padding: 18px 20px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
  display: flex;
  flex-direction: column;
  flex: 1.6;
  min-width: 320px;
}

.trend-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}

.trend-title {
  font-size: 14px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.filter-pill-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #ffffff;
  border: 1px solid #cbd5e1;
  border-radius: 16px;
  padding: 4px 10px;
  font-size: 12px;
  color: #475569;
  cursor: pointer;
  transition: all 0.15s ease;
}

.filter-pill-btn:hover {
  background: #f8fafc;
  border-color: #94a3b8;
}

.chart-legend {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 8px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.legend-line {
  width: 14px;
  height: 3px;
  border-radius: 2px;
}

.line-green {
  background-color: #10b981;
}

.line-gray {
  background-color: #64748b;
}

.legend-text {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
}

.svg-container {
  position: relative;
  width: 100%;
  height: 180px;
}

.line-svg {
  width: 100%;
  height: 100%;
}

.axis-text {
  font-size: 10px;
  fill: #94a3b8;
  font-family: inherit;
  user-select: none;
}

.chart-tooltip {
  position: absolute;
  transform: translate(-50%, -100%);
  background: rgba(15, 23, 42, 0.9);
  color: #ffffff;
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 11px;
  display: flex;
  flex-direction: column;
  pointer-events: none;
  white-space: nowrap;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.2);
  z-index: 10;
}

.tooltip-date {
  font-weight: 700;
  color: #cbd5e1;
  margin-bottom: 2px;
}

.text-green {
  color: #34d399;
}

.text-gray {
  color: #cbd5e1;
}
</style>
