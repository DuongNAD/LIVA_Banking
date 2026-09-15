<template>
  <div class="min-h-screen bg-slate-100 text-slate-900 flex flex-col font-sans antialiased selection:bg-indigo-100 selection:text-indigo-900">
    <!-- 1. If not authenticated, render Clean Dedicated Login View -->
    <LoginView
      v-if="!authStore.isAuthenticated"
      @login-success="handleLoginSuccess"
    />

    <!-- 2. If authenticated, render Core Banking Operational Interface -->
    <template v-else>
      <!-- Top Enterprise Navigation Header -->
      <header class="sticky top-0 z-40 bg-slate-900 text-white border-b border-slate-800 shadow-md">
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div class="flex items-center justify-between h-16">
            <!-- Logo & Brand Identity -->
            <div class="flex items-center space-x-2.5 cursor-pointer select-none" @click="handleDefaultTabForRole">
              <div class="w-9 h-9 rounded-xl bg-gradient-to-tr from-emerald-500 via-teal-500 to-blue-600 flex items-center justify-center font-black text-lg text-white shadow-md">
                L
              </div>
              <div>
                <div class="flex items-center space-x-2">
                  <span class="text-sm sm:text-base font-bold tracking-tight text-white">LIVA BANK</span>
                  <span class="px-1.5 py-0.5 rounded text-[9px] font-bold bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
                    INTERBANK CORE
                  </span>
                </div>
                <p class="text-[10px] text-slate-400 font-medium hidden sm:block">Hệ Thống Tác Nghiệp Nội Bộ</p>
              </div>
            </div>

            <!-- Primary Enterprise Navigation Tabs (All 4 Core Banking Functional Workstations) -->
            <nav class="hidden md:flex items-center space-x-1 p-1 bg-slate-800/80 rounded-xl border border-slate-700/80 text-xs font-semibold">
              <!-- Tab 1: Reconciliation -->
              <button
                type="button"
                class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-2 cursor-pointer"
                :class="currentTab === 'reconciliation' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                @click="currentTab = 'reconciliation'"
              >
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m16 16 3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z"/><path d="m2 16 3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z"/><path d="M7 21h10"/><path d="M12 3v18"/><path d="M3 7h2c2 0 5-1 7-2 2 1 5 2 7 2h2"/></svg>
                <span>Đối Soát Quyết Toán</span>
              </button>

              <!-- Tab 2: Liquidity Desk -->
              <button
                type="button"
                class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-2 cursor-pointer"
                :class="currentTab === 'dashboard' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                @click="currentTab = 'dashboard'"
              >
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 3v18h18"/><path d="m19 9-5 5-4-4-3 3"/></svg>
                <span>Vị Thế Thanh Khoản</span>
                <span
                  v-if="bankingStore.hasReserveBreach"
                  class="ml-1 px-1.5 py-0.2 rounded-full text-[9px] bg-rose-500 text-white font-bold animate-pulse"
                >
                  Cảnh báo
                </span>
              </button>

              <!-- Tab 3: Treasury Maker-Checker Approval -->
              <button
                type="button"
                class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-2 cursor-pointer"
                :class="currentTab === 'treasury' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                @click="currentTab = 'treasury'"
              >
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="20" height="12" x="2" y="6" rx="2"/><circle cx="12" cy="12" r="2"/><path d="M6 12h.01M18 12h.01"/></svg>
                <span>Phê Duyệt Lệnh Chi</span>
                <span
                  v-if="treasuryStore.pendingVouchers.length > 0"
                  class="ml-1 px-1.5 py-0.2 rounded-full text-[10px] bg-amber-500 text-slate-950 font-bold animate-pulse"
                >
                  {{ treasuryStore.pendingVouchers.length }}
                </span>
              </button>

              <!-- Tab 4: AML & Compliance -->
              <button
                type="button"
                class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-2 cursor-pointer"
                :class="currentTab === 'aml' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                @click="currentTab = 'aml'"
              >
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/></svg>
                <span>Giám Sát AML/STR</span>
                <span
                  v-if="amlStore.totalAlertsCount > 0"
                  class="ml-1 px-1.5 py-0.2 rounded-full text-[10px] bg-rose-500 text-white font-bold"
                >
                  {{ amlStore.totalAlertsCount }}
                </span>
              </button>
            </nav>

            <!-- Right Status -->
            <div class="flex items-center space-x-2.5 text-xs">
              <!-- Copilot Quick Toggle -->
              <button
                type="button"
                class="px-2.5 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-300 hover:text-white text-[11px] font-medium transition flex items-center space-x-1.5 cursor-pointer"
                title="Mở Trợ lý AI (Copilot)"
                @click="isCopilotOpen = true"
              >
                <svg class="w-3.5 h-3.5 text-blue-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
                <span class="hidden sm:inline">Trợ lý AI</span>
              </button>

              <!-- Simulated Vietnam Live Clock -->
              <div class="flex items-center space-x-1.5 px-3 py-1.5 rounded-xl bg-slate-800 border border-slate-700 text-slate-300 font-mono text-[11px]">
                <svg class="w-3.5 h-3.5 text-slate-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                <span>{{ liveTimeString }}</span>
              </div>
            </div>
          </div>

          <!-- Mobile Navigation Row -->
          <div class="md:hidden flex items-center space-x-1 pb-3 pt-1 border-t border-slate-800 text-[11px] font-semibold overflow-x-auto">
            <button
              type="button"
              class="px-2.5 py-1.5 rounded-lg whitespace-nowrap cursor-pointer"
              :class="currentTab === 'reconciliation' ? 'bg-white text-slate-900 font-bold' : 'text-slate-300'"
              @click="currentTab = 'reconciliation'"
            >
              Đối Soát Quyết Toán
            </button>
            <button
              type="button"
              class="px-2.5 py-1.5 rounded-lg whitespace-nowrap cursor-pointer"
              :class="currentTab === 'dashboard' ? 'bg-white text-slate-900 font-bold' : 'text-slate-300'"
              @click="currentTab = 'dashboard'"
            >
              Vị Thế Thanh Khoản
            </button>
            <button
              type="button"
              class="px-2.5 py-1.5 rounded-lg whitespace-nowrap cursor-pointer"
              :class="currentTab === 'treasury' ? 'bg-white text-slate-900 font-bold' : 'text-slate-300'"
              @click="currentTab = 'treasury'"
            >
              Phê Duyệt Lệnh Chi
            </button>
            <button
              type="button"
              class="px-2.5 py-1.5 rounded-lg whitespace-nowrap cursor-pointer"
              :class="currentTab === 'aml' ? 'bg-white text-slate-900 font-bold' : 'text-slate-300'"
              @click="currentTab = 'aml'"
            >
              Giám Sát AML/STR
            </button>
          </div>
        </div>
      </header>

      <!-- Professional Core Banking Operations Workstation Status Bar (Feature F03 & F04) -->
      <OfficerStatusBar
        :current-tab="currentTab"
        @switch-tab="handleTabNavigation"
        @logout="handleLogout"
      />

      <!-- Main Workspace Container -->
      <main class="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-6">
        <BankingDashboardView
          v-if="currentTab === 'dashboard'"
          @navigate="handleTabNavigation"
        />

        <ReconciliationWorkbenchView
          v-else-if="currentTab === 'reconciliation'"
        />

        <ComplianceAmlView
          v-else-if="currentTab === 'aml'"
        />

        <TreasuryPaymentView
          v-else-if="currentTab === 'treasury'"
        />
      </main>

      <!-- Footer Security & Regulatory Compliance Banner -->
      <footer class="bg-white border-t border-slate-200 py-2.5 text-[11px] text-slate-500">
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col sm:flex-row items-center justify-between gap-2 text-center sm:text-left">
          <div>
            <strong class="text-slate-700">LIVA Commercial Bank Universal Core</strong> — Hệ Thống Tác Nghiệp Cán Bộ Ngân Hàng.
          </div>

          <div class="flex items-center space-x-2 text-slate-400">
            <span class="text-emerald-700 font-semibold">● Zero Cloud Egress Guaranteed</span>
            <span>•</span>
            <span>Thông tư 09/2020/TT-NHNN</span>
            <span>•</span>
            <span>Nghị định 13/2023/NĐ-CP</span>
            <span>•</span>
            <span class="font-mono text-slate-600">SHA-256</span>
          </div>
        </div>
      </footer>

      <!-- Global Floating 2D Financial Copilot Drawer (Feature F19) -->
      <FinancialCopilotDrawer
        :is-open="treasuryStore.isCopilotOpen"
        :messages="treasuryStore.copilotMessages"
        :live-balance="bankingStore.totalLiquidVnd"
        :match-rate="reconStore.matchRate"
        :pending-alerts-count="amlStore.totalAlertsCount"
        @update:is-open="treasuryStore.isCopilotOpen = $event"
        @send="treasuryStore.sendCopilotMessage"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import LoginView from './views/LoginView.vue';
import BankingDashboardView from './views/BankingDashboardView.vue';
import ReconciliationWorkbenchView from './views/ReconciliationWorkbenchView.vue';
import ComplianceAmlView from './views/ComplianceAmlView.vue';
import TreasuryPaymentView from './views/TreasuryPaymentView.vue';
import FinancialCopilotDrawer from './components/copilot/FinancialCopilotDrawer.vue';
import OfficerStatusBar from './components/auth/OfficerStatusBar.vue';

import { useAuthStore, type UserRole } from './stores/authStore';
import { useBankingStore } from './stores/bankingStore';
import { useReconciliationStore } from './stores/reconciliationStore';
import { useAmlStore } from './stores/amlStore';
import { useTreasuryStore } from './stores/treasuryStore';

type AppTab = 'dashboard' | 'reconciliation' | 'aml' | 'treasury';

const authStore = useAuthStore();
const bankingStore = useBankingStore();
const reconStore = useReconciliationStore();
const amlStore = useAmlStore();
const treasuryStore = useTreasuryStore();

const currentTab = ref<AppTab>('dashboard');

// Auto-route to dedicated role dashboard whenever role changes
watch(
  () => authStore.currentUser?.role,
  (newRole) => {
    if (newRole === 'MAKER') currentTab.value = 'reconciliation';
    else if (newRole === 'CHECKER') currentTab.value = 'treasury';
    else if (newRole === 'AML' || (newRole as any) === 'AUDITOR') currentTab.value = 'aml';
    else if (newRole === 'TREASURY') currentTab.value = 'dashboard';
  },
  { immediate: true }
);

// Live Simulated Clock
const liveTimeString = ref('');
let clockTimer: any = null;

function updateClock() {
  const now = new Date();
  liveTimeString.value = now.toLocaleTimeString('vi-VN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

function handleDefaultTabForRole() {
  if (authStore.isMaker) currentTab.value = 'reconciliation';
  else if (authStore.isChecker) currentTab.value = 'treasury';
  else if (authStore.isAml) currentTab.value = 'aml';
  else if (authStore.isTreasury) currentTab.value = 'dashboard';
  else currentTab.value = 'dashboard';
}

function handleLoginSuccess(role: UserRole) {
  if (role === 'MAKER') currentTab.value = 'reconciliation';
  else if (role === 'CHECKER') currentTab.value = 'treasury';
  else if (role === 'AML') currentTab.value = 'aml';
  else if (role === 'TREASURY') currentTab.value = 'dashboard';
  else currentTab.value = 'dashboard';
}

function handleLogout() {
  authStore.logout();
}

onMounted(() => {
  document.title = 'LIVA Banking — Cổng Nghiệp Vụ & Giám Sát Ngân Hàng';
  updateClock();
  clockTimer = setInterval(updateClock, 1000);
  if (authStore.isAuthenticated && authStore.currentUser) {
    handleDefaultTabForRole();
  }
});

onUnmounted(() => {
  if (clockTimer) clearInterval(clockTimer);
});

function handleTabNavigation(tab: AppTab) {
  currentTab.value = tab;
}
</script>
