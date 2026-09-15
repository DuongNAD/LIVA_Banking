<script setup lang="ts">
/**
 * TreasuryMiniDock.vue
 * Lightweight 2D Enterprise Treasury Mini-Dock (Replaces legacy 3D avatar / ghost mode widget).
 * Pure 2D UI with zero WebGL / zero 3D canvas overhead.
 */
import { onMounted } from 'vue';
import { useBankingStore } from '../../stores/bankingStore';
import { invokeBackend } from '../../utils/ipc';

const bankingStore = useBankingStore();

function formatVnd(val: number): string {
  return new Intl.NumberFormat('en-US').format(val) + ' VND';
}

async function openDashboard() {
  try {
    await invokeBackend('open_dashboard');
  } catch {
    // In web mode, redirect or open new window
    window.location.href = '/dashboard.html';
  }
}

onMounted(async () => {
  await bankingStore.fetchOverview();
});
</script>

<template>
  <div class="mini-dock-container">
    <div class="mini-dock-pill">
      <!-- Status Indicator -->
      <div class="status-indicator">
        <span class="live-pulse" />
        <span class="dock-title">LIVA TREASURY</span>
      </div>

      <!-- Quick Metrics -->
      <div class="dock-metrics">
        <div class="metric-item">
          <span class="metric-label">Khớp:</span>
          <span class="metric-val text-green">{{ bankingStore.summary.reconciledRate }}%</span>
        </div>
        <div class="metric-item">
          <span class="metric-label">Vị thế:</span>
          <span class="metric-val font-mono">{{ formatVnd(bankingStore.totalBalanceAll) }}</span>
        </div>
      </div>

      <!-- Action: Open Main Workbench -->
      <button class="open-workbench-btn" @click="openDashboard">
        Mở Workbench ↗
      </button>
    </div>
  </div>
</template>

<style scoped>
.mini-dock-container {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100vw;
  height: 100vh;
  background: transparent;
  padding: 12px;
  box-sizing: border-box;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.mini-dock-pill {
  background: rgba(15, 23, 42, 0.92);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 9999px;
  padding: 8px 18px;
  display: flex;
  align-items: center;
  gap: 16px;
  box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.3);
  color: #ffffff;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
}

.live-pulse {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: #10b981;
  box-shadow: 0 0 8px #10b981;
}

.dock-title {
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.8px;
  color: #94a3b8;
}

.dock-metrics {
  display: flex;
  align-items: center;
  gap: 14px;
  border-left: 1px solid rgba(255, 255, 255, 0.1);
  border-right: 1px solid rgba(255, 255, 255, 0.1);
  padding: 0 14px;
}

.metric-item {
  display: flex;
  align-items: baseline;
  gap: 4px;
  font-size: 11px;
}

.metric-label {
  color: #64748b;
}

.metric-val {
  font-weight: 700;
  font-feature-settings: 'tnum';
}

.text-green {
  color: #34d399;
}

.open-workbench-btn {
  background: #2563eb;
  color: #ffffff;
  border: none;
  font-size: 11px;
  font-weight: 700;
  padding: 5px 12px;
  border-radius: 9999px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.open-workbench-btn:hover {
  background: #1d4ed8;
  transform: translateY(-1px);
}
</style>
