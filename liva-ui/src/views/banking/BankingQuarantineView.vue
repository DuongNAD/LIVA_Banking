<script setup lang="ts">
/**
 * BankingQuarantineView.vue — P44 Quarantine Queue & Dual Control Hub View
 * =========================================================================
 * Host view for P44:
 * - Circular 09/2020/TT-NHNN 4-Eyes principle demonstration
 * - Master-Detail layout integrating QuarantineList & QuarantineDetailPane
 * - Role Switcher toolbar (R01 Maker vs R04 Checker)
 */
import { useWorkbenchStore } from '../../stores/workbenchStore';
import QuarantineList from '../../components/banking/quarantine/QuarantineList.vue';
import QuarantineDetailPane from '../../components/banking/quarantine/QuarantineDetailPane.vue';

const store = useWorkbenchStore();

function setRole(role: 'MAKER' | 'CHECKER') {
  store.switchOperatorRole(role);
}
</script>

<template>
  <div class="banking-quarantine-view">
    <!-- Top Role Control & Regulatory Bar -->
    <header class="quarantine-topbar">
      <div class="topbar-left">
        <div class="title-cluster">
          <span class="view-tag">P44 QUEUE</span>
          <h1 class="view-title">Hàng Đợi Cách Ly & Kiểm Soát Kép</h1>
        </div>
        <div class="reg-tags">
          <span class="reg-pill">TT 09/2020/TT-NHNN</span>
          <span class="reg-pill">QĐ 11/2023/QĐ-TTg (AML)</span>
          <span class="reg-pill">NĐ 13/2023/NĐ-CP</span>
        </div>
      </div>

      <!-- Interactive 4-Eyes Role Switcher -->
      <div class="topbar-right">
        <div class="role-switcher-box">
          <span class="switcher-label">Giả Lập Vai Trò 4 Mắt:</span>
          <div class="role-btn-group">
            <button
              class="role-btn role-maker"
              :class="{ active: store.currentOperatorRole === 'MAKER' }"
              @click="setRole('MAKER')"
            >
              <span class="role-icon">✍️</span>
              <span class="role-text">R01 Kế Toán Viên (Maker)</span>
            </button>
            <button
              class="role-btn role-checker"
              :class="{ active: store.currentOperatorRole === 'CHECKER' }"
              @click="setRole('CHECKER')"
            >
              <span class="role-icon">🛡️</span>
              <span class="role-text">R04 Kế Toán Trưởng (Checker)</span>
            </button>
          </div>
        </div>
      </div>
    </header>

    <!-- Master-Detail Split Grid -->
    <main class="quarantine-grid">
      <!-- Left Column: Filterable List -->
      <section class="grid-left-col">
        <QuarantineList />
      </section>

      <!-- Right Column: Investigation & Dual Control Workspace -->
      <section class="grid-right-col">
        <QuarantineDetailPane />
      </section>
    </main>
  </div>
</template>

<style scoped>
.banking-quarantine-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #090d16;
  color: #f8fafc;
  overflow: hidden;
}

.quarantine-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 24px;
  background: #111827;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  flex-shrink: 0;
}

.topbar-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.title-cluster {
  display: flex;
  align-items: center;
  gap: 10px;
}

.view-tag {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 800;
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.35);
  padding: 2px 6px;
  border-radius: 4px;
  letter-spacing: 0.05em;
}

.view-title {
  font-size: 16px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0;
}

.reg-tags {
  display: flex;
  align-items: center;
  gap: 6px;
}

.reg-pill {
  font-size: 10px;
  font-weight: 600;
  color: #94a3b8;
  background: rgba(255, 255, 255, 0.04);
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.topbar-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.role-switcher-box {
  display: flex;
  align-items: center;
  gap: 10px;
  background: #0f172a;
  padding: 4px 8px 4px 12px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.switcher-label {
  font-size: 11px;
  color: #94a3b8;
  font-weight: 600;
}

.role-btn-group {
  display: flex;
  gap: 4px;
}

.role-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid transparent;
  background: transparent;
  color: #94a3b8;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease-in-out;
}

.role-btn:hover {
  color: #f8fafc;
  background: rgba(255, 255, 255, 0.05);
}

.role-btn.role-maker.active {
  background: rgba(245, 158, 11, 0.2);
  color: #fbbf24;
  border-color: rgba(245, 158, 11, 0.4);
  box-shadow: 0 0 10px rgba(245, 158, 11, 0.15);
}

.role-btn.role-checker.active {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
  border-color: rgba(16, 185, 129, 0.4);
  box-shadow: 0 0 10px rgba(16, 185, 129, 0.15);
}

.quarantine-grid {
  flex: 1;
  display: grid;
  grid-template-columns: 380px 1fr;
  overflow: hidden;
}

.grid-left-col {
  height: 100%;
  overflow: hidden;
}

.grid-right-col {
  height: 100%;
  overflow: hidden;
}

@media (max-width: 1024px) {
  .quarantine-grid {
    grid-template-columns: 320px 1fr;
  }
}
</style>
