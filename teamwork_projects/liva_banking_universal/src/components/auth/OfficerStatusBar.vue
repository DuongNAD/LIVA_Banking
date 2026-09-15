<template>
  <aside aria-label="Core Banking Terminal Status" class="bg-slate-900 border-b border-slate-800 px-4 sm:px-6 py-1.5 text-xs text-slate-300">
    <div class="max-w-7xl mx-auto flex flex-wrap items-center justify-between gap-3">
      <!-- Left: Branch, Terminal & Officer Role -->
      <div class="flex items-center space-x-2 sm:space-x-3 text-[11px]">
        <div class="flex items-center space-x-1.5 text-slate-200">
          <svg class="w-3.5 h-3.5 text-slate-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 22V4a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v18Z"/><path d="M6 12H4a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2h2"/><path d="M18 9h2a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-2"/><path d="M10 6h4"/><path d="M10 10h4"/><path d="M10 14h4"/><path d="M10 18h4"/></svg>
          <span class="font-medium">{{ authStore.currentBranchName }}</span>
        </div>
        <span class="text-slate-600">|</span>
        <div class="flex items-center space-x-1 text-slate-400">
          <svg class="w-3.5 h-3.5 text-slate-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="20" height="14" x="2" y="3" rx="2"/><line x1="8" x2="16" y1="21" y2="21"/><line x1="12" x2="12" y1="17" y2="21"/></svg>
          <span class="font-mono">{{ authStore.currentTerminalId }}</span>
        </div>
        <span class="text-slate-600">|</span>
        <div class="flex items-center space-x-1.5">
          <span class="text-slate-400">Nghiệp vụ:</span>
          <span class="font-bold text-emerald-400">{{ activeWorkspaceTitle }}</span>
        </div>
      </div>

      <!-- Right: Officer Profile, Session Timer & Logout -->
      <div class="flex items-center space-x-3 text-[11px] ml-auto">
        <!-- Officer Badge -->
        <div v-if="authStore.currentUser" class="flex items-center space-x-1.5 text-slate-300">
          <svg class="w-3.5 h-3.5 text-slate-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
          <span class="font-mono text-slate-400">[{{ authStore.currentOfficerId }}]</span>
          <span class="font-semibold text-white">{{ authStore.currentUser.fullName }}</span>
          <span
            class="px-1.5 py-0.2 rounded text-[10px] font-bold uppercase ml-1"
            :class="roleBadgeClass"
          >
            {{ authStore.currentUser.role }}
          </span>
        </div>

        <span class="text-slate-700">|</span>

        <!-- Session Time (4h limit) -->
        <div class="flex items-center space-x-1 text-slate-400 font-mono text-[11px]" title="Thời hạn phiên làm việc JWT (4 tiếng)">
          <svg class="w-3.5 h-3.5 text-amber-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
          <span class="text-amber-300 font-bold">{{ sessionRemainingFormatted }}</span>
          <span class="text-slate-500 text-[10px] hidden md:inline">/ 4h</span>
        </div>

        <!-- Logout Button -->
        <button
          type="button"
          class="px-2.5 py-1 rounded-md bg-slate-800 hover:bg-rose-900/60 hover:text-rose-200 border border-slate-700 hover:border-rose-500/50 text-slate-300 transition font-medium flex items-center space-x-1 cursor-pointer text-[11px]"
          title="Đăng xuất khỏi phiên làm việc"
          @click="handleLogout"
        >
          <svg class="w-3 h-3 text-slate-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" x2="9" y1="12" y2="12"/></svg>
          <span>Đăng Xuất</span>
        </button>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useAuthStore } from '../../stores/authStore';

const authStore = useAuthStore();

const emit = defineEmits<{
  (e: 'switch-tab', tab: 'dashboard' | 'reconciliation' | 'aml' | 'treasury'): void;
  (e: 'logout'): void;
}>();

// Session elapsed timer (hh:mm:ss) & 4-Hour JWT Expiry limit
const MAX_SESSION_SECONDS = 14400; // 4 hours (14,400s)
const sessionSeconds = ref(0);
let sessionTimer: any = null;

const sessionElapsedFormatted = computed(() => {
  const hrs = Math.floor(sessionSeconds.value / 3600).toString().padStart(2, '0');
  const mins = Math.floor((sessionSeconds.value % 3600) / 60).toString().padStart(2, '0');
  const secs = (sessionSeconds.value % 60).toString().padStart(2, '0');
  return `${hrs}:${mins}:${secs}`;
});

const remainingSeconds = computed(() => Math.max(0, MAX_SESSION_SECONDS - sessionSeconds.value));
const sessionRemainingFormatted = computed(() => {
  const hrs = Math.floor(remainingSeconds.value / 3600).toString().padStart(2, '0');
  const mins = Math.floor((remainingSeconds.value % 3600) / 60).toString().padStart(2, '0');
  const secs = (remainingSeconds.value % 60).toString().padStart(2, '0');
  return `${hrs}:${mins}:${secs}`;
});

onMounted(() => {
  sessionTimer = setInterval(() => {
    sessionSeconds.value++;
    if (sessionSeconds.value >= MAX_SESSION_SECONDS) {
      handleLogout();
    }
  }, 1000);
});

onUnmounted(() => {
  if (sessionTimer) clearInterval(sessionTimer);
});

// Role workspace metadata
const activeWorkspaceTitle = computed(() => {
  if (authStore.isMaker) return 'Vận Hành & Đối Soát (Maker)';
  if (authStore.isChecker) return 'Kiểm Soát & Phê Duyệt Lệnh (Checker)';
  if (authStore.isAml) return 'Giám Sát Tuân Thủ & PCRT (AML)';
  if (authStore.isTreasury) return 'Quản Trị Thanh Khoản (Treasury)';
  return 'Tác Nghiệp Ngân Hàng';
});

// Role badge styling
const roleBadgeClass = computed(() => {
  if (authStore.isChecker) return 'bg-amber-500/20 text-amber-400 border border-amber-500/30';
  if (authStore.isAml) return 'bg-rose-500/20 text-rose-400 border border-rose-500/30';
  if (authStore.isTreasury) return 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30';
  return 'bg-blue-500/20 text-blue-400 border border-blue-500/30';
});

// Logout handler
function handleLogout() {
  authStore.logout();
  emit('logout');
}
</script>
