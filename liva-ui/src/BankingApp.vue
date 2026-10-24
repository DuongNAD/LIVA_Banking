<script setup lang="ts">
/**
 * BankingApp.vue — Main 2D Banking & Treasury UI Shell
 * =======================================================
 * Implements dynamic component routing:
 * - Dynamic view registry mapping 7 tabs to dedicated views
 * - URL Hash synchronization (#reconcile, #transactions, #settings...)
 * - Smooth transition animations between views
 * - Window Bar with URL liva://reconciliation.local and MVP THỰC TẾ badge
 * - Pure 2D Conversational Financial Assistant Drawer
 */
import { ref, computed, markRaw, onMounted, onUnmounted, watch } from 'vue';
import type { Component } from 'vue';
import BankingWindowBar from './components/banking/BankingWindowBar.vue';
import BankingSidebar from './components/banking/BankingSidebar.vue';
import BankingHeader from './components/banking/BankingHeader.vue';
import FinancialAssistantDrawer from './components/banking/FinancialAssistantDrawer.vue';
import ToastContainer from './components/ToastContainer.vue';
import LoginModal from './components/banking/LoginModal.vue';
import { useAuthStore } from './stores/authStore';
import { useToast } from './composables/useToast';

// Dynamic Sub-views
import BankingOverviewView from './views/banking/BankingOverviewView.vue';
import BankingWorkbenchView from './views/banking/BankingWorkbenchView.vue';
import BankingQuarantineView from './views/banking/BankingQuarantineView.vue';
import BankingLedgerView from './views/banking/BankingLedgerView.vue';
import BankingReconciliationView from './views/banking/BankingReconciliationView.vue';
import BankingTransactionsView from './views/banking/BankingTransactionsView.vue';
import BankingCashflowView from './views/banking/BankingCashflowView.vue';
import BankingTreasuryView from './views/banking/BankingTreasuryView.vue';
import BankingRiskView from './views/banking/BankingRiskView.vue';
import BankingComplianceView from './views/banking/BankingComplianceView.vue';
import BankingReportsView from './views/banking/BankingReportsView.vue';
import BankingSettingsView from './views/banking/BankingSettingsView.vue';

// Dynamic Component Registry (Registry Pattern)
const viewRegistry: Record<string, Component> = {
  overview: markRaw(BankingOverviewView),
  workbench: markRaw(BankingWorkbenchView),
  quarantine: markRaw(BankingQuarantineView),
  ledger: markRaw(BankingLedgerView),
  reconcile: markRaw(BankingReconciliationView),
  transactions: markRaw(BankingTransactionsView),
  cashflow: markRaw(BankingCashflowView),
  treasury: markRaw(BankingTreasuryView),
  risk: markRaw(BankingRiskView),
  compliance: markRaw(BankingComplianceView),
  reports: markRaw(BankingReportsView),
  settings: markRaw(BankingSettingsView),
};

const authStore = useAuthStore();
const toast = useToast();

const currentTab = ref('overview');
const isAssistantOpen = ref(false);

const activeComponent = computed(() => {
  return viewRegistry[currentTab.value] || viewRegistry.overview;
});

function isTabAuthorized(tab: string): boolean {
  return authStore.canAccessRoute(tab);
}

function guardAndResolveTab(rawTab: string): string {
  if (!viewRegistry[rawTab]) {
    if (typeof window !== 'undefined' && window.location.hash && window.location.hash !== '#overview') {
      window.location.hash = '#overview';
    }
    return 'overview';
  }
  if (!isTabAuthorized(rawTab)) {
    toast.error(
      `403 Forbidden: Vai trò ${authStore.currentUser.role} (${authStore.currentUser.roleTitle}) không có quyền truy cập màn hình [${rawTab.toUpperCase()}].`
    );
    if (typeof window !== 'undefined' && window.location.hash !== '#overview') {
      window.location.hash = '#overview';
    }
    return 'overview';
  }
  return rawTab;
}

function resolveTabFromHash(): string {
  if (typeof window === 'undefined') return 'overview';
  const hash = window.location.hash.replace(/^#\/?/, '').toLowerCase();
  return hash || 'overview';
}

function onNavigate(itemId: string) {
  if (!viewRegistry[itemId]) return;
  if (!isTabAuthorized(itemId)) {
    toast.error(
      `403 Forbidden: Vai trò ${authStore.currentUser.role} (${authStore.currentUser.roleTitle}) không có quyền truy cập màn hình [${itemId.toUpperCase()}].`
    );
    if (currentTab.value !== 'overview') {
      currentTab.value = 'overview';
      if (typeof window !== 'undefined' && window.location.hash !== '#overview') {
        window.location.hash = '#overview';
      }
    }
    return;
  }
  currentTab.value = itemId;
  if (typeof window !== 'undefined' && window.location.hash !== `#${itemId}`) {
    window.location.hash = `#${itemId}`;
  }
}

function onHashChange() {
  const requested = resolveTabFromHash();
  const valid = guardAndResolveTab(requested);
  if (currentTab.value !== valid) {
    currentTab.value = valid;
  }
  if (typeof window !== 'undefined' && window.location.hash && window.location.hash !== `#${valid}`) {
    window.location.hash = `#${valid}`;
  }
}

// Reactive guard on role switch: if current tab is no longer permitted, bounce to overview
watch(
  () => authStore.currentUser.role,
  (newRole) => {
    if (!authStore.canAccessRoute(currentTab.value)) {
      toast.warning(
        `Chuyển đổi vai trò sang [${newRole}]: Màn hình [${currentTab.value.toUpperCase()}] không còn hợp lệ. Tự động chuyển về Tổng quan.`
      );
      currentTab.value = 'overview';
      if (typeof window !== 'undefined' && window.location.hash !== '#overview') {
        window.location.hash = '#overview';
      }
    }
  }
);

onMounted(() => {
  const initial = resolveTabFromHash();
  const valid = guardAndResolveTab(initial);
  currentTab.value = valid;
  if (typeof window !== 'undefined' && window.location.hash && window.location.hash !== `#${valid}`) {
    window.location.hash = `#${valid}`;
  }
  window.addEventListener('hashchange', onHashChange);
});

onUnmounted(() => {
  window.removeEventListener('hashchange', onHashChange);
});

function toggleAssistant() {
  isAssistantOpen.value = !isAssistantOpen.value;
}

function onSearch(query: string) {
  if (query.length > 2) {
    // Global search feedback
  }
}

defineExpose({
  currentTab,
  onNavigate,
  onHashChange,
  guardAndResolveTab,
});
</script>

<template>
  <div class="banking-app-shell">
    <!-- Shell Window Top Bar -->
    <BankingWindowBar
      url="reconciliation.local"
      badge-text="MVP THỰC TẾ"
    />

    <!-- Main App Body: Sidebar + Main Content -->
    <div class="banking-app-body">
      <!-- Sidebar Navigation (7 items) -->
      <BankingSidebar
        :active-item="currentTab"
        @navigate="onNavigate"
      />

      <!-- Content Area -->
      <main class="banking-main-content">
        <!-- Top Dashboard Header -->
        <BankingHeader
          :has-unread-notifications="true"
          @toggle-assistant="toggleAssistant"
          @search="onSearch"
        />

        <!-- Dynamic Viewport with Transition -->
        <div class="banking-viewport">
          <Transition name="view-fade" mode="out-in">
            <component :is="activeComponent" :key="currentTab" />
          </Transition>
        </div>
      </main>
    </div>

    <!-- Pure 2D Financial Assistant Drawer -->
    <FinancialAssistantDrawer
      :is-open="isAssistantOpen"
      @close="isAssistantOpen = false"
    />

    <!-- Global Toast Container -->
    <ToastContainer />

    <!-- Role-based Authentication & Login Modal -->
    <LoginModal />
  </div>
</template>

<style scoped>
.banking-app-shell {
  display: flex;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
  background-color: #f8fafc;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  color: #0f172a;
}

.banking-app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.banking-main-content {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
  background-color: #f8fafc;
}

.banking-viewport {
  flex: 1;
  overflow-y: auto;
  position: relative;
}

/* View Transition Animations */
.view-fade-enter-active,
.view-fade-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.view-fade-enter-from {
  opacity: 0;
  transform: translateY(6px);
}

.view-fade-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
