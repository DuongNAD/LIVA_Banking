<template>
  <div class="bg-slate-950/90 border-b border-slate-800/80 px-4 sm:px-6 lg:px-8 py-2 text-xs font-mono">
    <div class="max-w-7xl mx-auto flex flex-wrap items-center justify-between gap-2">
      <!-- Left: Server Connection & LocalStorage Token Status -->
      <div class="flex items-center space-x-2.5">
        <!-- Online/Offline Indicator -->
        <div
          class="flex items-center space-x-1.5 px-2.5 py-1 rounded-md border text-[11px] font-semibold transition"
          :class="
            authStore.isServerOnline
              ? 'bg-emerald-950/70 border-emerald-500/40 text-emerald-300'
              : 'bg-amber-950/70 border-amber-500/40 text-amber-300'
          "
        >
          <span
            class="w-2 h-2 rounded-full"
            :class="authStore.isServerOnline ? 'bg-emerald-400 animate-pulse' : 'bg-amber-400'"
          ></span>
          <span>{{ authStore.isServerOnline ? 'SERVER ONLINE (Port 3001)' : 'STANDALONE (Zero Cloud Egress)' }}</span>
        </div>

        <!-- Token Key Indicator -->
        <button
          type="button"
          class="flex items-center space-x-1 px-2.5 py-1 rounded-md bg-slate-900 border border-slate-700 text-slate-300 hover:text-white hover:border-slate-500 transition cursor-pointer"
          @click="showTokenModal = true"
          title="Bấm để xem chi tiết mã Token trong LocalStorage"
        >
          <svg class="w-3.5 h-3.5 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
          <span>LocalStorage: <strong class="text-emerald-400 underline">liva_auth_token</strong></span>
          <span v-if="authStore.token" class="text-[10px] text-slate-400">({{ tokenShort }})</span>
        </button>
      </div>

      <!-- Center / Right: Current Active Role & Fast 1-Click Role Switcher -->
      <div class="flex items-center space-x-2">
        <!-- Current Active User Badge -->
        <div v-if="authStore.currentUser" class="hidden md:flex items-center space-x-1.5 px-2.5 py-1 rounded-md bg-indigo-950/60 border border-indigo-500/30 text-indigo-300 text-[11px]">
          <span class="font-bold">{{ authStore.currentUser.fullName }}</span>
          <span
            class="px-1.5 py-0.2 rounded text-[10px] font-extrabold uppercase"
            :class="
              authStore.isChecker
                ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30'
                : 'bg-blue-500/20 text-blue-400 border border-blue-500/30'
            "
          >
            {{ authStore.currentUser.role }}
          </span>
        </div>

        <!-- 1-Click Fast Switch Buttons -->
        <div class="flex items-center space-x-1.5 text-[11px]">
          <span class="text-slate-500 hidden sm:inline">Chuyển vai trò:</span>

          <button
            type="button"
            class="px-2 py-1 rounded transition flex items-center space-x-1 cursor-pointer"
            :class="
              authStore.isMaker
                ? 'bg-blue-600 text-white font-bold shadow-xs'
                : 'bg-slate-800 text-slate-300 hover:bg-slate-700 hover:text-white border border-slate-700'
            "
            @click="authStore.loginAsMaker"
          >
            <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
            <span>Maker (Kế toán)</span>
          </button>

          <button
            type="button"
            class="px-2 py-1 rounded transition flex items-center space-x-1 cursor-pointer"
            :class="
              authStore.isChecker
                ? 'bg-amber-600 text-white font-bold shadow-xs'
                : 'bg-slate-800 text-slate-300 hover:bg-slate-700 hover:text-white border border-slate-700'
            "
            @click="authStore.loginAsChecker"
          >
            <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="20" height="12" x="2" y="6" rx="2"/><circle cx="12" cy="12" r="2"/><path d="M6 12h.01M18 12h.01"/></svg>
            <span>Checker (Giám đốc)</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Modal Xem Chi Tiết JWT Token trong LocalStorage -->
    <div
      v-if="showTokenModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-xs p-4"
    >
      <div class="bg-slate-900 border border-slate-700 rounded-2xl max-w-xl w-full p-6 shadow-2xl text-slate-200">
        <div class="flex items-center justify-between pb-4 border-b border-slate-800">
          <div class="flex items-center space-x-2">
            <svg class="w-5 h-5 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
            <div>
              <h3 class="font-bold text-base text-white font-sans">Chi Tiết JWT Auth Token (Client-Server)</h3>
              <p class="text-xs text-slate-400">Được lưu trực tiếp trong <code class="text-emerald-400 font-mono">localStorage['liva_auth_token']</code></p>
            </div>
          </div>
          <button
            type="button"
            class="text-slate-400 hover:text-white p-1 cursor-pointer"
            @click="showTokenModal = false"
          >
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
          </button>
        </div>

        <div class="mt-4 space-y-4 text-xs">
          <div>
            <label class="block font-bold text-slate-400 mb-1">Mã JWT Token thực tế (Authorization: Bearer):</label>
            <div class="p-3 bg-slate-950 rounded-xl border border-slate-800 font-mono text-[11px] text-emerald-300 break-all select-all">
              {{ authStore.token || 'Chưa có token (Vui lòng đăng nhập).' }}
            </div>
          </div>

          <div v-if="authStore.currentUser">
            <label class="block font-bold text-slate-400 mb-1">Dữ liệu Giải mã (Decoded Claims & RBAC):</label>
            <pre class="p-3 bg-slate-950 rounded-xl border border-slate-800 font-mono text-[11px] text-blue-300 overflow-x-auto">{{ JSON.stringify(authStore.currentUser, null, 2) }}</pre>
          </div>

          <div class="p-3 bg-emerald-950/40 border border-emerald-500/30 rounded-xl text-emerald-200 text-[11px] space-y-1 font-sans">
            <div class="font-bold flex items-center space-x-1.5">
              <svg class="w-4 h-4 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>
              <span>Cách tự kiểm tra trong trình duyệt của bạn:</span>
            </div>
            <ol class="list-decimal list-inside space-y-0.5 text-emerald-300">
              <li>Bấm phím <strong>F12</strong> trên bàn phím.</li>
              <li>Chọn tab <strong>Application</strong> &gt; mở menu <strong>Local storage</strong> bên trái.</li>
              <li>Click vào tên miền hiện tại &gt; Bạn sẽ thấy Key: <strong class="text-white">liva_auth_token</strong> và Value là chuỗi Token trên!</li>
            </ol>
          </div>
        </div>

        <div class="mt-6 flex justify-end">
          <button
            type="button"
            class="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-white rounded-xl font-bold font-sans transition"
            @click="showTokenModal = false"
          >
            Đóng
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useAuthStore } from '../../stores/authStore';

const authStore = useAuthStore();
const showTokenModal = ref(false);

const tokenShort = computed(() => {
  if (!authStore.token) return 'null';
  return authStore.token.length > 16
    ? authStore.token.substring(0, 8) + '...' + authStore.token.substring(authStore.token.length - 6)
    : authStore.token;
});

onMounted(() => {
  authStore.checkServerHealth();
});
</script>
