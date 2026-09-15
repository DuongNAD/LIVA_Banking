<template>
  <div class="min-h-screen bg-slate-950 text-slate-100 flex flex-col justify-center items-center p-4 sm:p-6 lg:p-8 font-sans selection:bg-emerald-500/30 selection:text-emerald-200">
    <!-- Main Login Container (Centered) -->
    <div class="max-w-md w-full space-y-6">
      <!-- Institutional Bank Header -->
      <div class="text-center space-y-3">
        <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-gradient-to-tr from-emerald-500 via-teal-500 to-blue-600 shadow-xl shadow-emerald-950/50 border border-emerald-400/30">
          <span class="font-black text-3xl text-white tracking-wider">L</span>
        </div>
        <div>
          <div class="flex items-center justify-center space-x-2">
            <h1 class="text-2xl sm:text-3xl font-extrabold tracking-tight text-white">
              LIVA COMMERCIAL BANK
            </h1>
            <span class="px-2 py-0.5 rounded text-[10px] font-black bg-emerald-500/20 text-emerald-400 border border-emerald-500/40">
              UNIVERSAL CORE
            </span>
          </div>
          <p class="text-sm font-semibold text-slate-400 mt-1">
            Cổng Đăng Nhập Hệ Thống Tác Nghiệp & Quản Trị Nội Bộ Ngân Hàng
          </p>
          <p class="text-xs text-slate-500">
            Internal Operations, Interbank Settlement & AML Surveillance Workstation
          </p>
        </div>
      </div>

      <!-- Regulatory & Security Warning Banner -->
      <div class="bg-slate-900/90 border border-amber-500/40 rounded-2xl p-4 shadow-lg flex items-start space-x-3.5">
        <div class="w-8 h-8 rounded-xl bg-amber-500/20 border border-amber-500/30 flex items-center justify-center shrink-0 text-amber-400 text-base">
          🛡️
        </div>
        <div class="text-xs space-y-1">
          <div class="flex items-center space-x-2">
            <strong class="font-bold text-amber-300 text-xs tracking-wide uppercase">
              Quy Định An Ninh Nội Bộ
            </strong>
            <span class="px-1.5 py-0.2 rounded text-[10px] font-bold bg-amber-500/20 text-amber-300">
              Nghị định 13/2023/NĐ-CP
            </span>
          </div>
          <p class="text-slate-300 leading-relaxed text-[11px]">
            Hệ thống <strong>không mở đăng ký tự do</strong>. Cán bộ sử dụng mã định danh và mật khẩu do <strong>Khối CNTT & An Ninh Thông Tin</strong> cấp phát để truy cập nghiệp vụ.
          </p>
        </div>
      </div>

      <!-- Centered Secure Login Form -->
      <div class="bg-slate-900/95 border border-slate-800 rounded-2xl p-6 sm:p-7 shadow-2xl space-y-4">
        <h2 class="text-sm font-bold text-white flex items-center space-x-2 pb-3 border-b border-slate-800">
          <span>🔐</span>
          <span>Xác Thực Danh Tính Cán Bộ</span>
        </h2>

        <!-- Error alert -->
        <div
          v-if="errorMessage"
          class="p-3 rounded-xl bg-rose-950/80 border border-rose-500/50 text-rose-300 text-xs flex items-center space-x-2"
        >
          <span>⚠️</span>
          <span>{{ errorMessage }}</span>
        </div>

        <form @submit.prevent="handleFormSubmit" class="space-y-4 text-xs">
          <div>
            <label class="block font-semibold text-slate-300 mb-1.5">
              Mã Cán Bộ / Email Ngân Hàng:
            </label>
            <div class="relative">
              <span class="absolute inset-y-0 left-0 pl-3.5 flex items-center text-slate-500 pointer-events-none">
                👤
              </span>
              <input
                v-model="inputEmail"
                type="text"
                required
                autocomplete="username"
                placeholder="VD: OPR-77092 hoặc maker@livabanking.vn"
                class="w-full pl-10 pr-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-700 text-slate-100 placeholder-slate-500 focus:outline-hidden focus:ring-2 focus:ring-emerald-500 focus:border-transparent font-mono text-xs transition"
              />
            </div>
          </div>

          <div>
            <label class="block font-semibold text-slate-300 mb-1.5">
              Mật Khẩu Truy Cập:
            </label>
            <div class="relative">
              <span class="absolute inset-y-0 left-0 pl-3.5 flex items-center text-slate-500 pointer-events-none">
                🔑
              </span>
              <input
                v-model="inputPassword"
                :type="showPassword ? 'text' : 'password'"
                required
                autocomplete="current-password"
                placeholder="Nhập mật khẩu truy cập"
                class="w-full pl-10 pr-11 py-2.5 rounded-xl bg-slate-950 border border-slate-700 text-slate-100 placeholder-slate-500 focus:outline-hidden focus:ring-2 focus:ring-emerald-500 focus:border-transparent font-mono text-xs transition"
              />
              <button
                type="button"
                class="absolute inset-y-0 right-0 pr-3.5 flex items-center text-slate-400 hover:text-slate-200 text-xs cursor-pointer"
                @click="showPassword = !showPassword"
              >
                {{ showPassword ? 'Ẩn' : 'Hiện' }}
              </button>
            </div>
          </div>

          <div>
            <label class="block font-semibold text-slate-300 mb-1.5">
              Chi Nhánh Làm Việc:
            </label>
            <select
              v-model="selectedBranch"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-700 text-slate-200 text-xs focus:ring-2 focus:ring-emerald-500 focus:outline-hidden transition"
            >
              <option value="HO-HN-001">HO-HN-001 — Hội Sở Chính Hà Nội</option>
              <option value="CN-HCM-002">CN-HCM-002 — Chi Nhánh TP. Hồ Chí Minh</option>
              <option value="CN-DN-003">CN-DN-003 — Chi Nhánh Đà Nẵng</option>
            </select>
          </div>

          <div>
            <label class="block font-semibold text-slate-300 mb-1.5">
              Trạm Làm Việc (Terminal ID):
            </label>
            <input
              v-model="selectedTerminal"
              type="text"
              readonly
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950/60 border border-slate-800 text-slate-400 font-mono text-xs"
            />
          </div>

          <button
            type="submit"
            :disabled="isLoading"
            class="w-full py-3 rounded-xl bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white font-bold text-xs shadow-lg transition flex items-center justify-center space-x-2 cursor-pointer disabled:opacity-50 mt-2"
          >
            <span v-if="isLoading" class="animate-spin">⏳</span>
            <span>{{ isLoading ? 'Đang xác thực hệ thống...' : 'Đăng Nhập Vào Hệ Thống' }}</span>
            <span v-if="!isLoading">→</span>
          </button>
        </form>

        <div class="pt-3 border-t border-slate-800/80 text-[11px] text-slate-400 text-center">
          Phiên kết nối bảo mật nội bộ (TLS 1.3 • E2EE • In-Memory Session)
        </div>
      </div>

      <!-- Institutional Footer Compliance Notice -->
      <div class="text-center text-[11px] text-slate-400 space-y-1">
        <p>
          Hệ Thống Tác Nghiệp Nội Bộ Ngân Hàng Thương Mại — Tuân thủ Thông tư 09/2020/TT-NHNN & Thông tư 09/2023/TT-NHNN.
        </p>
        <p class="text-slate-400">
          ● Bảo mật cục bộ tuyệt đối • Zero Data Egress Guaranteed • Không sao lưu trên đám mây công cộng
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useAuthStore, type UserRole } from '../stores/authStore';

const emit = defineEmits<{
  (e: 'login-success', role: UserRole): void;
}>();

const authStore = useAuthStore();

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
    const success = await authStore.login(inputEmail.value.trim(), inputPassword.value);
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
