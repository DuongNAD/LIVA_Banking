<template>
  <aside aria-label="Core Banking Terminal Status" class="bg-slate-900 border-b border-slate-800 px-4 sm:px-6 py-2 text-xs text-slate-300">
    <div class="max-w-7xl mx-auto flex flex-wrap items-center justify-between gap-3">
      <!-- Left: Branch, Terminal & Officer Role -->
      <div class="flex items-center space-x-2 sm:space-x-3 text-[11px]">
        <div class="flex items-center space-x-1.5 text-slate-200">
          <span>🏛️</span>
          <span class="font-medium">{{ authStore.currentBranchName }}</span>
        </div>
        <span class="text-slate-600">|</span>
        <div class="flex items-center space-x-1 text-slate-400">
          <span>💻</span>
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
          <span>{{ authStore.currentUser.avatar }}</span>
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

        <!-- Session Time -->
        <div class="flex items-center space-x-1 text-slate-400 font-mono text-[11px]">
          <span>⏱️</span>
          <span class="text-amber-300">{{ sessionElapsedFormatted }}</span>
        </div>

        <!-- Logout Button -->
        <button
          type="button"
          class="px-2.5 py-1 rounded-md bg-slate-800 hover:bg-rose-900/60 hover:text-rose-200 border border-slate-700 hover:border-rose-500/50 text-slate-300 transition font-medium flex items-center space-x-1 cursor-pointer text-[11px]"
          title="Đăng xuất khỏi phiên làm việc"
          @click="handleLogout"
        >
          <span>🚪</span>
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

// Session elapsed timer (hh:mm:ss)
const sessionSeconds = ref(0);
let sessionTimer: any = null;

const sessionElapsedFormatted = computed(() => {
  const hrs = Math.floor(sessionSeconds.value / 3600).toString().padStart(2, '0');
  const mins = Math.floor((sessionSeconds.value % 3600) / 60).toString().padStart(2, '0');
  const secs = (sessionSeconds.value % 60).toString().padStart(2, '0');
  return `${hrs}:${mins}:${secs}`;
});

onMounted(() => {
  sessionTimer = setInterval(() => {
    sessionSeconds.value++;
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
