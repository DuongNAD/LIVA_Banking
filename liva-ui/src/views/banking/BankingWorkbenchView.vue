<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useWorkbenchStore } from '../../stores/workbenchStore';
import BankColumn from '../../components/banking/workbench/BankColumn.vue';
import GlColumn from '../../components/banking/workbench/GlColumn.vue';
import MatchColumn from '../../components/banking/workbench/MatchColumn.vue';
import SplitSolverModal from '../../components/banking/workbench/SplitSolverModal.vue';
import UnmatchReasonModal from '../../components/banking/workbench/UnmatchReasonModal.vue';

const store = useWorkbenchStore();

function formatVnd(val: number): string {
  const formatted = new Intl.NumberFormat('vi-VN').format(Math.abs(val)) + ' ₫';
  return val < 0 ? `-${formatted}` : `+${formatted}`;
}

function handleResetBenchmark() {
  store.seedBenchmarkData();
}

function handleKeyDown(e: KeyboardEvent) {
  // F1: Manual match selected
  if (e.key === 'F1') {
    e.preventDefault();
    if (store.selectedBankLineIds.length > 0 && store.selectedGlRecordIds.length > 0) {
      store.createManualMatch();
    }
  }
  // F2: Move selected bank line to quarantine
  else if (e.key === 'F2') {
    e.preventDefault();
    if (store.selectedBankLineIds.length > 0) {
      const line = store.bankLines.find(b => b.id === store.selectedBankLineIds[0]);
      if (line) {
        line.isQuarantined = true;
      }
    }
  }
  // F4: Open Split Solver for selected bank line
  else if (e.key === 'F4') {
    e.preventDefault();
    if (store.selectedBankLineIds.length > 0) {
      const line = store.bankLines.find(b => b.id === store.selectedBankLineIds[0]);
      if (line) {
        store.splitSolverTarget = line;
      }
    }
  }
  // Ctrl + Enter: Submit batch
  else if (e.ctrlKey && e.key === 'Enter') {
    e.preventDefault();
    if (store.batchStatus === 'IN_PROGRESS') {
      store.submitBatchToChecker();
    }
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
  <div class="workbench-view-container">
    <!-- Top Master Control & Status Banner -->
    <header class="workbench-header">
      <div class="header-main-meta">
        <div class="batch-title-group">
          <div class="batch-code-row">
            <span class="page-tag">P42 WORKBENCH</span>
            <h1 class="batch-id">{{ store.activeBatchId }}</h1>
            <span class="status-tag" :class="store.batchStatus.toLowerCase()">
              {{ store.batchStatus === 'IN_PROGRESS' ? 'Đang thực hiện' : 'Đã trình duyệt' }}
            </span>
          </div>
          <div class="batch-sub-meta">
            <span>Maker: <strong>{{ store.makerUserId }}</strong></span>
            <span class="dot-sep">•</span>
            <span>Chế độ: <strong>Zero Floating-Point (i128/i64)</strong></span>
            <span class="dot-sep">•</span>
            <span>An toàn mạng: <strong>Air-gap Local Intranet</strong></span>
          </div>
        </div>

        <div class="header-kpi-group">
          <div class="kpi-mini-card">
            <span class="kpi-mini-label">Tỷ lệ khớp tự động</span>
            <span class="kpi-mini-value" :class="store.summaryStats.matchRate >= 90 ? 'text-success' : 'text-warn'">
              {{ store.summaryStats.matchRate }}%
            </span>
          </div>

          <div class="kpi-mini-card">
            <span class="kpi-mini-label">Tổng phát sinh Sao kê</span>
            <span class="kpi-mini-value text-mono">
              {{ formatVnd(store.summaryStats.totalBank) }}
            </span>
          </div>

          <div class="kpi-mini-card">
            <span class="kpi-mini-label">Tổng phát sinh Sổ cái</span>
            <span class="kpi-mini-value text-mono">
              {{ formatVnd(store.summaryStats.totalGl) }}
            </span>
          </div>

          <div class="kpi-mini-card invariant-card">
            <span class="kpi-mini-label">Bất biến số dư</span>
            <span class="invariant-badge" :class="store.summaryStats.balanceInvariantValid ? 'valid' : 'invalid'">
              {{ store.summaryStats.balanceInvariantValid ? '✓ Cân bằng 100%' : '⚠️ Lệch số dư' }}
            </span>
          </div>

          <button class="btn-reset-demo" title="Khôi phục bộ dữ liệu benchmark" @click="handleResetBenchmark">
            🔄 Reset Demo
          </button>
        </div>
      </div>
    </header>

    <!-- 3-Column Synchronized Layout Grid -->
    <main class="workbench-grid">
      <!-- Column 1: Bank Statement Lines -->
      <section class="grid-column bank-col-wrapper">
        <BankColumn />
      </section>

      <!-- Column 2: ERP General Ledger Records -->
      <section class="grid-column gl-col-wrapper">
        <GlColumn />
      </section>

      <!-- Column 3: Match Results & Staging Dock -->
      <section class="grid-column match-col-wrapper">
        <MatchColumn />
      </section>
    </main>

    <!-- P42 Hotkey Quick Actions Footer Bar -->
    <footer class="workbench-hotkey-bar">
      <div class="hotkey-item"><kbd>F1</kbd> <span>Khớp thủ công</span></div>
      <div class="hotkey-item"><kbd>F2</kbd> <span>Cách ly (Quarantine)</span></div>
      <div class="hotkey-item"><kbd>F4</kbd> <span>Gom/chẻ hóa đơn (Split Solver)</span></div>
      <div class="hotkey-item"><kbd>Ctrl</kbd> + <kbd>Enter</kbd> <span>Trình duyệt KTT</span></div>
      <div class="hotkey-spacer"></div>
      <div class="hotkey-item invariant-note">
        <span>Bất biến: Đóng sổ = Mở sổ + ΣCó - ΣNợ (Δ = 0đ)</span>
      </div>
    </footer>

    <!-- P45 Split Solver Dialog -->
    <SplitSolverModal />

    <!-- Mandatory Audit Unmatch Reason Dialog -->
    <UnmatchReasonModal />
  </div>
</template>

<style scoped>
.workbench-view-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: calc(100vh - 60px);
  background: #f1f5f9;
  padding: 16px 20px 20px 20px;
  gap: 14px;
  box-sizing: border-box;
}

.workbench-header {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 14px 20px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.header-main-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 16px;
}

.batch-title-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.batch-code-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.page-tag {
  font-size: 11px;
  font-weight: 800;
  padding: 2px 8px;
  background: #0f172a;
  color: #38bdf8;
  border-radius: 4px;
  letter-spacing: 0.5px;
}

.batch-id {
  font-size: 18px;
  font-weight: 800;
  color: #0f172a;
  font-family: monospace;
  margin: 0;
}

.status-tag {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 9999px;
}

.status-tag.in_progress { background: #fef08a; color: #854d0e; }
.status-tag.submitted_to_checker { background: #dbeafe; color: #1e40af; }

.batch-sub-meta {
  font-size: 12px;
  color: #64748b;
  display: flex;
  align-items: center;
  gap: 6px;
}

.dot-sep {
  color: #cbd5e1;
}

.header-kpi-group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.kpi-mini-card {
  display: flex;
  flex-direction: column;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 6px 12px;
  min-width: 120px;
}

.kpi-mini-label {
  font-size: 10px;
  color: #64748b;
  font-weight: 600;
  margin-bottom: 2px;
}

.kpi-mini-value {
  font-size: 14px;
  font-weight: 700;
}

.text-success { color: #16a34a; }
.text-warn { color: #d97706; }
.text-mono { font-family: monospace; color: #0f172a; }

.invariant-card {
  min-width: 110px;
}

.invariant-badge {
  font-size: 12px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 4px;
  text-align: center;
}

.invariant-badge.valid { background: #dcfce7; color: #15803d; }
.invariant-badge.invalid { background: #fee2e2; color: #b91c1c; }

.btn-reset-demo {
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 600;
  background: #f1f5f9;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  cursor: pointer;
  color: #334155;
  transition: all 0.15s;
}

.btn-reset-demo:hover {
  background: #e2e8f0;
}

.workbench-grid {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1fr 1.25fr;
  gap: 16px;
  min-height: 0; /* Quan trọng để flex/grid scroll độc lập */
}

.grid-column {
  height: calc(100vh - 170px);
  min-height: 500px;
}

@media (max-width: 1200px) {
  .workbench-grid {
    grid-template-columns: 1fr;
    height: auto;
  }

  .grid-column {
    height: 550px;
  }
}

.workbench-hotkey-bar {
  display: flex;
  align-items: center;
  gap: 16px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 8px 16px;
  font-size: 12px;
  color: #475569;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.03);
}

.hotkey-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.hotkey-item kbd {
  background: #0f172a;
  color: #38bdf8;
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
  font-size: 11px;
  font-weight: 700;
  border: 1px solid #1e293b;
}

.hotkey-spacer {
  flex: 1;
}

.invariant-note {
  font-family: monospace;
  font-size: 11px;
  color: #059669;
  font-weight: 600;
}
</style>
