<script setup lang="ts">
/**
 * BankingSidebar.vue
 * Sidebar Navigation matching user mockup with 7 items and active indicator dots.
 */
import { ref, watch } from 'vue';

const props = withDefaults(
  defineProps<{
    activeItem?: string;
  }>(),
  {
    activeItem: 'overview',
  }
);

const emit = defineEmits<{
  (e: 'navigate', itemId: string): void;
}>();

const current = ref(props.activeItem);

watch(
  () => props.activeItem,
  (newVal) => {
    if (newVal) current.value = newVal;
  }
);

interface NavItem {
  id: string;
  label: string;
  hasActiveDot?: boolean;
}

const navItems: NavItem[] = [
  { id: 'overview', label: 'TỔNG QUAN' },
  { id: 'workbench', label: 'P42 WORKBENCH', hasActiveDot: true },
  { id: 'quarantine', label: 'P44 CÁCH LY', hasActiveDot: true },
  { id: 'ledger', label: 'P50 SỔ CÁI ERP', hasActiveDot: true },
  { id: 'reconcile', label: 'ĐỐI SOÁT' },
  { id: 'transactions', label: 'GIAO DỊCH' },
  { id: 'cashflow', label: 'THU CHI' },
  { id: 'treasury', label: 'NGÂN QUỸ' },
  { id: 'risk', label: 'P70 RỦI RO', hasActiveDot: true },
  { id: 'compliance', label: 'P80 TUÂN THỦ', hasActiveDot: true },
  { id: 'reports', label: 'BÁO CÁO' },
  { id: 'settings', label: 'CÀI ĐẶT' },
];

function selectItem(id: string) {
  current.value = id;
  emit('navigate', id);
}
</script>

<template>
  <aside class="banking-sidebar">
    <!-- Brand Section -->
    <div class="brand-section">
      <div class="brand-logo">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="2" y="2" width="9" height="9" rx="2" fill="#10b981" />
          <rect x="13" y="2" width="9" height="9" rx="2" fill="#059669" fill-opacity="0.8" />
          <rect x="2" y="13" width="9" height="9" rx="2" fill="#047857" fill-opacity="0.9" />
          <rect x="13" y="13" width="9" height="9" rx="2" fill="#34d399" />
        </svg>
      </div>
      <div class="brand-name">
        <span class="brand-title">LIVA</span>
        <span class="brand-sub">Reconciliation</span>
      </div>
    </div>

    <!-- Section Header -->
    <div class="sidebar-section-title">TỔNG QUAN</div>

    <!-- Nav List -->
    <nav class="nav-menu">
      <button
        v-for="item in navItems"
        :key="item.id"
        class="nav-item"
        :class="{ active: current === item.id }"
        @click="selectItem(item.id)"
      >
        <div class="nav-item-icon">
          <!-- Icon: TỔNG QUAN (Grid) -->
          <svg v-if="item.id === 'overview'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="7" height="7" />
            <rect x="14" y="3" width="7" height="7" />
            <rect x="14" y="14" width="7" height="7" />
            <rect x="3" y="14" width="7" height="7" />
          </svg>

          <!-- Icon: P42 WORKBENCH (Columns layout) -->
          <svg v-else-if="item.id === 'workbench'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="18" height="18" rx="2" />
            <line x1="9" y1="3" x2="9" y2="21" />
            <line x1="15" y1="3" x2="15" y2="21" />
          </svg>

          <!-- Icon: P44 CÁCH LY (Shield alert) -->
          <svg v-else-if="item.id === 'quarantine'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>

          <!-- Icon: P50 SỔ CÁI ERP (Book / Ledger) -->
          <svg v-else-if="item.id === 'ledger'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
            <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
            <line x1="8" y1="7" x2="16" y2="7" />
            <line x1="8" y1="11" x2="14" y2="11" />
          </svg>

          <!-- Icon: ĐỐI SOÁT (Check in circle) -->
          <svg v-else-if="item.id === 'reconcile'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
            <polyline points="22 4 12 14.01 9 11.01" />
          </svg>

          <!-- Icon: GIAO DỊCH (Exchange arrows) -->
          <svg v-else-if="item.id === 'transactions'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="m16 3 4 4-4 4" />
            <path d="M20 7H4" />
            <path d="m8 21-4-4 4-4" />
            <path d="M4 17h16" />
          </svg>

          <!-- Icon: THU CHI (Money / Receipt) -->
          <svg v-else-if="item.id === 'cashflow'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="2" y="6" width="20" height="12" rx="2" />
            <circle cx="12" cy="12" r="3" />
            <path d="M6 12h.01M18 12h.01" />
          </svg>

          <!-- Icon: NGÂN QUỸ (Bank building) -->
          <svg v-else-if="item.id === 'treasury'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 21h18M3 10h18M5 10v11M9 10v11M15 10v11M19 10v11M12 2l10 5H2z" />
          </svg>

          <!-- Icon: P70 RỦI RO (Shield Alert) -->
          <svg v-else-if="item.id === 'risk'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>

          <!-- Icon: P80 TUÂN THỦ (Shield Check) -->
          <svg v-else-if="item.id === 'compliance'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            <path d="m9 12 2 2 4-4" />
          </svg>

          <!-- Icon: BÁO CÁO (Bar chart) -->
          <svg v-else-if="item.id === 'reports'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="20" x2="18" y2="10" />
            <line x1="12" y1="20" x2="12" y2="4" />
            <line x1="6" y1="20" x2="6" y2="14" />
          </svg>

          <!-- Icon: CÀI ĐẶT (Gear) -->
          <svg v-else-if="item.id === 'settings'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </div>

        <span class="nav-item-label">{{ item.label }}</span>

        <!-- Green active status dot for Đối soát -->
        <span v-if="item.hasActiveDot" class="status-dot" title="Đang đối soát thời gian thực" />
      </button>
    </nav>
  </aside>
</template>

<style scoped>
.banking-sidebar {
  width: 220px;
  background-color: #1c2534;
  display: flex;
  flex-direction: column;
  border-right: 1px solid rgba(255, 255, 255, 0.05);
  padding: 16px 12px;
  flex-shrink: 0;
  user-select: none;
}

.brand-section {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 8px 20px 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.brand-logo {
  display: flex;
  align-items: center;
  justify-content: center;
}

.brand-name {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}

.brand-title {
  color: #ffffff;
  font-weight: 800;
  font-size: 16px;
  letter-spacing: 0.5px;
}

.brand-sub {
  color: #94a3b8;
  font-size: 11px;
  font-weight: 500;
}

.sidebar-section-title {
  color: #64748b;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 1px;
  padding: 20px 8px 8px 8px;
}

.nav-menu {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 10px 12px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: #94a3b8;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  position: relative;
  text-align: left;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #f1f5f9;
}

.nav-item.active {
  background: #283548;
  color: #ffffff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.nav-item-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  color: inherit;
}

.nav-item-label {
  flex: 1;
  letter-spacing: 0.3px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: #10b981;
  box-shadow: 0 0 8px rgba(16, 185, 129, 0.8);
  animation: pulse-dot 2s infinite ease-in-out;
}

@keyframes pulse-dot {
  0%, 100% {
    transform: scale(1);
    opacity: 0.9;
  }
  50% {
    transform: scale(1.25);
    opacity: 1;
  }
}
</style>
