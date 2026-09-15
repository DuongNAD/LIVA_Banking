<template>
  <div class="min-h-screen bg-gradient-to-b from-slate-50 via-slate-50 to-slate-100 text-slate-800 flex flex-col justify-center items-center p-4 sm:p-6 lg:p-8 font-sans relative overflow-hidden selection:bg-emerald-100 selection:text-emerald-900">
    <!-- Ambient subtle background glow -->
    <div class="absolute -top-32 left-1/2 -translate-x-1/2 w-[620px] h-[360px] bg-gradient-to-br from-emerald-100/50 to-teal-100/30 rounded-full blur-3xl pointer-events-none -z-10"></div>
    <div class="absolute -bottom-32 right-1/4 w-[420px] h-[300px] bg-gradient-to-tr from-slate-200/40 to-emerald-100/30 rounded-full blur-3xl pointer-events-none -z-10"></div>

    <!-- Main Login Container (Centered) -->
    <div class="max-w-md w-full space-y-5">
      <!-- Institutional Bank Header (Compact & Clean) -->
      <div class="text-center space-y-2">
        <div class="inline-flex items-center justify-center w-12 h-12 rounded-2xl bg-gradient-to-tr from-emerald-600 to-teal-600 shadow-md shadow-emerald-600/20 ring-4 ring-emerald-50">
          <span class="font-extrabold text-xl text-white tracking-wider">L</span>
        </div>
        <div>
          <div class="flex items-center justify-center gap-2">
            <h1 class="text-xl sm:text-2xl font-extrabold tracking-tight text-slate-900">
              LIVA COMMERCIAL BANK
            </h1>
            <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-emerald-50 text-emerald-700 border border-emerald-200/80">
              UNIVERSAL CORE
            </span>
          </div>
          <p class="text-xs font-medium text-slate-500 mt-1">
            Cổng Đăng Nhập Hệ Thống Tác Nghiệp & Quản Trị Nội Bộ Ngân Hàng
          </p>
        </div>
      </div>

      <!-- Centered Secure Login Form -->
      <div class="bg-white border border-slate-200/90 rounded-2xl p-6 sm:p-7 shadow-xl shadow-slate-200/50 space-y-4">
        <!-- Form Header -->
        <div class="flex items-center justify-between pb-3.5 border-b border-slate-100">
          <h2 class="text-sm font-bold text-slate-800 flex items-center space-x-2">
            <div class="w-6 h-6 rounded-lg bg-emerald-50 text-emerald-600 flex items-center justify-center">
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>
                <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
              </svg>
            </div>
            <span>Xác Thực Danh Tính Cán Bộ</span>
          </h2>
          <span class="inline-flex items-center px-2 py-0.5 rounded-md text-[10px] font-medium bg-slate-100 text-slate-600">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 mr-1.5"></span>
            SSO Ready
          </span>
        </div>

        <!-- Role / Duty Selection (User chooses role/duty, but enters credentials manually) -->
        <div class="space-y-1.5">
          <label class="block font-semibold text-slate-700 text-xs">
            Vai Trò / Nhiệm Vụ Tác Nghiệp:
          </label>
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-1.5 text-[11px]">
            <button
              v-for="r in AVAILABLE_ROLES"
              :key="r.role"
              type="button"
              class="px-2 py-2 rounded-xl border text-center transition font-medium cursor-pointer select-none"
              :class="selectedRole === r.role
                ? 'bg-emerald-50 border-emerald-500 text-emerald-700 font-bold shadow-xs ring-1 ring-emerald-500/20'
                : 'bg-slate-50/70 border-slate-200 text-slate-600 hover:bg-slate-100 hover:border-slate-300'"
              @click="handleRoleChange(r.role)"
            >
              {{ r.label }}
            </button>
          </div>
        </div>

        <!-- Error alert -->
        <div
          v-if="errorMessage"
          class="p-3.5 rounded-xl bg-rose-50 border border-rose-200 text-rose-800 text-xs flex items-start space-x-2.5 leading-relaxed shadow-xs"
        >
          <svg class="w-4 h-4 shrink-0 text-rose-600 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          <span class="font-medium">{{ errorMessage }}</span>
        </div>

        <form @submit.prevent="handleFormSubmit" class="space-y-3.5 text-xs">
          <!-- Username / Email Input -->
          <div>
            <label class="block font-semibold text-slate-700 mb-1.5">
              Mã Cán Bộ / Email Ngân Hàng:
            </label>
            <div class="relative rounded-xl border border-slate-300 bg-white hover:border-slate-400 focus-within:border-emerald-600 focus-within:ring-4 focus-within:ring-emerald-500/10 transition">
              <span class="absolute inset-y-0 left-0 pl-3.5 flex items-center text-slate-400 pointer-events-none">
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/>
                  <circle cx="12" cy="7" r="4"/>
                </svg>
              </span>
              <input
                v-model="inputEmail"
                type="text"
                required
                autocomplete="username"
                :placeholder="currentRoleMeta.placeholder"
                class="w-full pl-10 pr-3.5 py-2.5 rounded-xl bg-transparent text-slate-900 placeholder-slate-400 font-mono text-xs focus:outline-hidden"
              />
            </div>
          </div>

          <!-- Password Input -->
          <div>
            <label class="block font-semibold text-slate-700 mb-1.5">
              Mật Khẩu Truy Cập:
            </label>
            <div class="relative rounded-xl border border-slate-300 bg-white hover:border-slate-400 focus-within:border-emerald-600 focus-within:ring-4 focus-within:ring-emerald-500/10 transition">
              <span class="absolute inset-y-0 left-0 pl-3.5 flex items-center text-slate-400 pointer-events-none">
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>
                  <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
                </svg>
              </span>
              <input
                v-model="inputPassword"
                :type="showPassword ? 'text' : 'password'"
                required
                autocomplete="current-password"
                placeholder="Nhập mật khẩu truy cập"
                class="w-full pl-10 pr-11 py-2.5 rounded-xl bg-transparent text-slate-900 placeholder-slate-400 font-mono text-xs focus:outline-hidden"
              />
              <button
                type="button"
                class="absolute inset-y-0 right-0 pr-3.5 flex items-center text-slate-400 hover:text-slate-600 transition cursor-pointer"
                :title="showPassword ? 'Ẩn mật khẩu' : 'Hiện mật khẩu'"
                @click="showPassword = !showPassword"
              >
                <svg v-if="!showPassword" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/>
                  <circle cx="12" cy="12" r="3"/>
                </svg>
                <svg v-else class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24"/>
                  <path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"/>
                  <path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"/>
                  <line x1="2" y1="2" x2="22" y2="22"/>
                </svg>
              </button>
            </div>
          </div>

          <!-- Branch Selection -->
          <div>
            <label class="block font-semibold text-slate-700 mb-1.5">
              Chi Nhánh Làm Việc:
            </label>
            <div class="relative rounded-xl border border-slate-300 bg-white hover:border-slate-400 focus-within:border-emerald-600 focus-within:ring-4 focus-within:ring-emerald-500/10 transition">
              <span class="absolute inset-y-0 left-0 pl-3.5 flex items-center text-slate-400 pointer-events-none">
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect width="16" height="20" x="4" y="2" rx="2" ry="2"/>
                  <path d="M9 22v-4h6v4"/>
                  <path d="M8 6h.01M16 6h.01M12 6h.01M8 10h.01M16 10h.01M12 10h.01M8 14h.01M16 14h.01M12 14h.01"/>
                </svg>
              </span>
              <select
                v-model="selectedBranch"
                class="w-full pl-10 pr-8 py-2.5 rounded-xl bg-transparent text-slate-800 text-xs focus:outline-hidden appearance-none cursor-pointer"
              >
                <option value="HO-HN-001">HO-HN-001 — Hội Sở Chính Hà Nội</option>
                <option value="CN-HCM-002">CN-HCM-002 — Chi Nhánh TP. Hồ Chí Minh</option>
                <option value="CN-DN-003">CN-DN-003 — Chi Nhánh Đà Nẵng</option>
              </select>
              <span class="absolute inset-y-0 right-0 pr-3.5 flex items-center text-slate-400 pointer-events-none">
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg>
              </span>
            </div>
          </div>

          <!-- Terminal ID -->
          <div>
            <label class="block font-semibold text-slate-700 mb-1.5">
              Trạm Làm Việc (Terminal ID):
            </label>
            <div class="relative rounded-xl border border-slate-200 bg-slate-50 text-slate-500">
              <span class="absolute inset-y-0 left-0 pl-3.5 flex items-center text-slate-400 pointer-events-none">
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="4 17 10 11 4 5"/>
                  <line x1="12" y1="19" x2="20" y2="19"/>
                </svg>
              </span>
              <input
                v-model="selectedTerminal"
                type="text"
                readonly
                class="w-full pl-10 pr-16 py-2.5 rounded-xl bg-transparent text-slate-600 font-mono text-xs focus:outline-hidden select-all"
              />
              <span class="absolute inset-y-0 right-0 pr-3.5 flex items-center text-[10px] font-mono font-medium text-slate-400">
                LOCKED
              </span>
            </div>
          </div>

          <!-- Submit Button -->
          <button
            type="submit"
            :disabled="isLoading"
            class="w-full py-3 px-4 rounded-xl bg-emerald-600 hover:bg-emerald-700 active:bg-emerald-800 text-white font-bold text-xs shadow-md shadow-emerald-600/20 hover:shadow-lg hover:shadow-emerald-600/30 active:scale-[0.99] transition-all flex items-center justify-center space-x-2 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed mt-2"
          >
            <svg v-if="isLoading" class="w-4 h-4 animate-spin text-white" viewBox="0 0 24 24" fill="none">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <span>{{ isLoading ? 'Đang xác thực hệ thống...' : 'Đăng Nhập Vào Hệ Thống' }}</span>
            <svg v-if="!isLoading" class="w-4 h-4 ml-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M5 12h14M12 5l7 7-7 7"/>
            </svg>
          </button>

          <!-- Standard Demo Credentials Reference -->
          <div class="pt-2 border-t border-slate-100 space-y-1.5 text-[11px] text-slate-500">
            <div class="font-bold text-slate-700 flex items-center justify-between">
              <span class="flex items-center gap-1.5">
                <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
                Tài khoản tác nghiệp chuẩn:
              </span>
              <span class="text-[10px] font-normal text-slate-400">Kiểm soát kép (Maker-Checker)</span>
            </div>
            <div class="grid grid-cols-2 gap-1.5 text-[10px] font-mono">
              <div class="p-1.5 rounded-lg bg-slate-50 border border-slate-200/70">
                <span class="text-emerald-700 font-bold block">Kế toán (Maker):</span>
                <span>maker_nam</span> / <span class="text-slate-600">LivaMaker@2026</span>
              </div>
              <div class="p-1.5 rounded-lg bg-slate-50 border border-slate-200/70">
                <span class="text-blue-700 font-bold block">Kiểm soát (Checker):</span>
                <span>checker_tri</span> / <span class="text-slate-600">LivaChecker@2026</span>
              </div>
            </div>
          </div>
        </form>
      </div>

      <!-- Institutional Footer Compliance Notice -->
      <div class="text-center text-[11px] text-slate-500 space-y-1 mt-6">
        <p>
          Hệ Thống Tác Nghiệp Nội Bộ Ngân Hàng Thương Mại — Tuân thủ Thông tư 09/2020/TT-NHNN & Nghị định 13/2023/NĐ-CP.
        </p>
        <p class="text-slate-400">
          ● Bảo mật cục bộ tuyệt đối • Zero Data Egress Guaranteed • Không sao lưu trên đám mây công cộng
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAuthStore, type UserRole } from '../stores/authStore';

const emit = defineEmits<{
  (e: 'login-success', role: UserRole): void;
}>();

const authStore = useAuthStore();

const AVAILABLE_ROLES: { role: UserRole; label: string; placeholder: string }[] = [
  { role: 'MAKER', label: 'Kế toán (Maker)', placeholder: 'VD: maker_nam hoặc OPR-77092' },
  { role: 'CHECKER', label: 'Kiểm soát (Checker)', placeholder: 'VD: checker_tri hoặc SUP-88214' },
  { role: 'AML', label: 'Giám sát AML', placeholder: 'VD: auditor_lan hoặc CMP-99015' },
  { role: 'TREASURY', label: 'Quản trị Vốn (Treasury)', placeholder: 'VD: cfo_hoang hoặc TRZ-55038' },
];

const selectedRole = ref<UserRole>('MAKER');
const currentRoleMeta = computed(() => AVAILABLE_ROLES.find(r => r.role === selectedRole.value) || AVAILABLE_ROLES[0]);

function handleRoleChange(role: UserRole) {
  selectedRole.value = role;
  // NOTE: Tuân thủ bảo mật - Tuyệt đối KHÔNG tự động điền tài khoản hay mật khẩu!
  // Cán bộ tự nhập thông tin xác thực của mình.
}

const inputEmail = ref('');
const inputPassword = ref('');
const showPassword = ref(false);
const selectedBranch = ref('HO-HN-001');
const selectedTerminal = ref('WS-OPER-04');
const errorMessage = ref<string | null>(null);
const isLoading = ref(false);

async function handleFormSubmit() {
  if (!inputEmail.value.trim() || !inputPassword.value) {
    errorMessage.value = 'Vui lòng nhập đầy đủ mã cán bộ / email và mật khẩu.';
    return;
  }
  errorMessage.value = null;
  isLoading.value = true;
  try {
    const success = await authStore.login(inputEmail.value.trim(), inputPassword.value, selectedRole.value);
    if (success && authStore.currentUser) {
      emit('login-success', authStore.currentUser.role);
    } else {
      errorMessage.value = authStore.authError || 'Thông tin đăng nhập không chính xác hoặc tài khoản bị khóa.';
    }
  } catch (err: any) {
    errorMessage.value = err?.message || 'Không thể kết nối máy chủ xác thực.';
  } finally {
    isLoading.value = false;
  }
}
</script>
