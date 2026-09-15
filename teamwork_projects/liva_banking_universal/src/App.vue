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
            <div class="flex items-center space-x-3 cursor-pointer" @click="handleDefaultTabForRole">
              <div class="w-10 h-10 rounded-xl bg-gradient-to-tr from-emerald-500 via-teal-500 to-blue-600 flex items-center justify-center font-black text-xl text-white shadow-md">
                L
              </div>
              <div>
                <div class="flex items-center space-x-2">
                  <span class="text-base font-bold tracking-tight text-white">LIVA COMMERCIAL BANK</span>
                  <span class="px-2 py-0.2 rounded text-[10px] font-extrabold bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
                    UNIVERSAL INTERBANK CORE
                  </span>
                </div>
                <p class="text-[11px] text-slate-400 font-medium">Hệ Thống Tác Nghiệp Nội Bộ Ngân Hàng Thương Mại</p>
              </div>
            </div>

            <!-- Role-Specific Navigation Tabs (Only show what is relevant to the logged-in role) -->
            <nav class="hidden md:flex items-center space-x-1 p-1 bg-slate-800/80 rounded-xl border border-slate-700/80 text-xs font-semibold">
              <!-- Maker Tabs -->
              <template v-if="authStore.isMaker">
                <button
                  type="button"
                  class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-1.5 cursor-pointer"
                  :class="currentTab === 'reconciliation' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                  @click="currentTab = 'reconciliation'"
                >
                  <span>⚖️</span>
                  <span>Đối Soát Quyết Toán</span>
                </button>
                <button
                  type="button"
                  class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-1.5 cursor-pointer"
                  :class="currentTab === 'dashboard' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                  @click="currentTab = 'dashboard'"
                >
                  <span>📊</span>
                  <span>Vị Thế Thanh Khoản</span>
                </button>
              </template>

              <!-- Checker Tabs -->
              <template v-else-if="authStore.isChecker">
                <button
                  type="button"
                  class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-1.5 cursor-pointer"
                  :class="currentTab === 'treasury' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                  @click="currentTab = 'treasury'"
                >
                  <span>🏦</span>
                  <span>Phê Duyệt Lệnh Chi (Maker-Checker)</span>
                  <span
                    v-if="treasuryStore.pendingVouchers.length > 0"
                    class="ml-1 px-1.5 py-0.2 rounded-full text-[10px] bg-amber-500 text-slate-950 font-bold animate-pulse"
                  >
                    {{ treasuryStore.pendingVouchers.length }}
                  </span>
                </button>
              </template>

              <!-- AML Specialist Tabs -->
              <template v-else-if="authStore.isAml">
                <button
                  type="button"
                  class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-1.5 cursor-pointer"
                  :class="currentTab === 'aml' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                  @click="currentTab = 'aml'"
                >
                  <span>🛡️</span>
                  <span>Giám Sát AML/STR</span>
                  <span
                    v-if="amlStore.totalAlertsCount > 0"
                    class="ml-1 px-1.5 py-0.2 rounded-full text-[10px] bg-rose-500 text-white font-bold"
                  >
                    {{ amlStore.totalAlertsCount }}
                  </span>
                </button>
              </template>

              <!-- Treasury Desk Tabs -->
              <template v-else-if="authStore.isTreasury">
                <button
                  type="button"
                  class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-1.5 cursor-pointer"
                  :class="currentTab === 'dashboard' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                  @click="currentTab = 'dashboard'"
                >
                  <span>📊</span>
                  <span>Quản Trị Thanh Khoản</span>
                </button>
                <button
                  type="button"
                  class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-1.5 cursor-pointer"
                  :class="currentTab === 'treasury' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                  @click="currentTab = 'treasury'"
                >
                  <span>🏦</span>
                  <span>Điều Chuyển Vốn</span>
                </button>
              </template>

              <!-- Default Fallback All Tabs for general exploration -->
              <template v-else>
                <button
                  type="button"
                  class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-1.5 cursor-pointer"
                  :class="currentTab === 'dashboard' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                  @click="currentTab = 'dashboard'"
                >
                  <span>📊</span>
                  <span>Tổng Quan</span>
                </button>
                <button
                  type="button"
                  class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-1.5 cursor-pointer"
                  :class="currentTab === 'reconciliation' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                  @click="currentTab = 'reconciliation'"
                >
                  <span>⚖️</span>
                  <span>Đối Soát</span>
                </button>
                <button
                  type="button"
                  class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-1.5 cursor-pointer"
                  :class="currentTab === 'aml' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                  @click="currentTab = 'aml'"
                >
                  <span>🛡️</span>
                  <span>AML/STR</span>
                </button>
                <button
                  type="button"
                  class="px-3.5 py-2 rounded-lg transition-all duration-150 flex items-center space-x-1.5 cursor-pointer"
                  :class="currentTab === 'treasury' ? 'bg-white text-slate-900 font-bold shadow-xs' : 'text-slate-300 hover:text-white hover:bg-slate-700/50'"
                  @click="currentTab = 'treasury'"
                >
                  <span>🏦</span>
                  <span>Thanh Khoản</span>
                </button>
              </template>
            </nav>

            <!-- Right Status -->
            <div class="flex items-center space-x-3 text-xs">
              <!-- Simulated Vietnam Live Clock -->
              <div class="flex items-center space-x-1.5 px-3 py-1.5 rounded-xl bg-slate-800 border border-slate-700 text-slate-300 font-mono text-[11px]">
                <span>🕒</span>
                <span>{{ liveTimeString }}</span>
              </div>
            </div>
          </div>

          <!-- Mobile Navigation Row -->
          <div class="md:hidden flex items-center space-x-1 pb-3 pt-1 border-t border-slate-800 text-[11px] font-semibold overflow-x-auto">
            <template v-if="authStore.isMaker">
              <button
                type="button"
                class="px-2.5 py-1.5 rounded-lg whitespace-nowrap"
                :class="currentTab === 'reconciliation' ? 'bg-white text-slate-900 font-bold' : 'text-slate-300'"
                @click="currentTab = 'reconciliation'"
              >
                Đối Soát Quyết Toán
              </button>
              <button
                type="button"
                class="px-2.5 py-1.5 rounded-lg whitespace-nowrap"
                :class="currentTab === 'dashboard' ? 'bg-white text-slate-900 font-bold' : 'text-slate-300'"
                @click="currentTab = 'dashboard'"
              >
                Thanh Khoản
              </button>
            </template>
            <template v-else-if="authStore.isChecker">
              <button
                type="button"
                class="px-2.5 py-1.5 rounded-lg whitespace-nowrap"
                :class="currentTab === 'treasury' ? 'bg-white text-slate-900 font-bold' : 'text-slate-300'"
                @click="currentTab = 'treasury'"
              >
                Phê Duyệt Lệnh Chi
              </button>
            </template>
            <template v-else-if="authStore.isAml">
              <button
                type="button"
                class="px-2.5 py-1.5 rounded-lg whitespace-nowrap"
                :class="currentTab === 'aml' ? 'bg-white text-slate-900 font-bold' : 'text-slate-300'"
                @click="currentTab = 'aml'"
              >
                Giám Sát AML/STR
              </button>
            </template>
            <template v-else>
              <button
                type="button"
                class="px-2.5 py-1.5 rounded-lg whitespace-nowrap"
                :class="currentTab === 'dashboard' ? 'bg-white text-slate-900 font-bold' : 'text-slate-300'"
                @click="currentTab = 'dashboard'"
              >
                Thanh Khoản
              </button>
              <button
                type="button"
                class="px-2.5 py-1.5 rounded-lg whitespace-nowrap"
                :class="currentTab === 'treasury' ? 'bg-white text-slate-900 font-bold' : 'text-slate-300'"
                @click="currentTab = 'treasury'"
              >
                Điều Chuyển Vốn
              </button>
            </template>
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
      <footer class="bg-white border-t border-slate-200 py-4 text-xs text-slate-500">
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col sm:flex-row items-center justify-between gap-3 text-center sm:text-left">
          <div>
            <strong class="text-slate-800">LIVA Commercial Bank Universal Core</strong> — Nền Tảng Tác Nghiệp & Quản Trị Thanh Khoản Liên Ngân Hàng Cục Bộ.
            <span class="block text-[11px] text-slate-400">
              Tuân thủ: Thông tư 09/2020/TT-NHNN, Thông tư 09/2023/TT-NHNN, Quyết định 11/2023/QĐ-TTg & Nghị định 13/2023/NĐ-CP
            </span>
          </div>

          <div class="flex items-center space-x-3 text-[11px]">
            <span class="text-emerald-700 font-semibold">● Zero Cloud Egress Guaranteed</span>
            <span class="text-slate-300">|</span>
            <span class="font-mono text-slate-600">Tamper-Evident SHA-256</span>
            <span class="text-slate-300">|</span>
            <span class="text-slate-500 font-medium">Hệ Thống Tác Nghiệp Cán Bộ Ngân Hàng</span>
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
