<script setup lang="ts">
/**
 * BankingQuarantineView.vue — P44 Quarantine Queue & Dual Control Hub View
 * =========================================================================
 * Host view for P44:
 * - Circular 09/2020/TT-NHNN 4-Eyes principle demonstration
 * - Master-Detail layout integrating QuarantineList & QuarantineDetailPane
 * - Authenticated session badge (Bound to authStore.currentUser, switcher eliminated)
 */
import { useAuthStore } from '../../stores/authStore';
import QuarantineList from '../../components/banking/quarantine/QuarantineList.vue';
import QuarantineDetailPane from '../../components/banking/quarantine/QuarantineDetailPane.vue';

const authStore = useAuthStore();
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

      <!-- Static Authenticated Session Badge (Bound to authStore.currentUser) -->
      <div class="topbar-right">
        <div class="session-officer-badge">
          <span
            class="officer-avatar-dot"
            :style="{ backgroundColor: authStore.currentUser.avatarColor || '#2563eb' }"
          >
            {{ authStore.currentUser.avatarInitials }}
          </span>
          <div class="officer-identity">
            <span class="officer-name">{{ authStore.currentUser.fullName }}</span>
            <span class="officer-role-pill" :class="authStore.currentUser.role.toLowerCase()">
              {{ authStore.currentUser.roleTitle }}
            </span>
          </div>
          <span v-if="authStore.isReadOnly" class="readonly-pill">[CHỈ ĐỌC]</span>
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

.session-officer-badge {
  display: flex;
  align-items: center;
  gap: 10px;
  background: #0f172a;
  padding: 6px 14px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.officer-avatar-dot {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 800;
  color: #ffffff;
}

.officer-identity {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.officer-name {
  font-size: 12px;
  font-weight: 700;
  color: #f8fafc;
}

.officer-role-pill {
  font-size: 10px;
  font-weight: 600;
  color: #94a3b8;
}

.officer-role-pill.maker { color: #fbbf24; }
.officer-role-pill.checker { color: #34d399; }
.officer-role-pill.cfo { color: #a78bfa; }
.officer-role-pill.auditor { color: #f59e0b; }
.officer-role-pill.admin { color: #ef4444; }

.readonly-pill {
  font-size: 10px;
  font-weight: 800;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(245, 158, 11, 0.2);
  color: #fbbf24;
  border: 1px solid rgba(245, 158, 11, 0.4);
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
