<script setup lang="ts">
/**
 * LoginModal.vue — Màn hình Đăng nhập & Xác thực Đa Vai trò (RBAC)
 * ==============================================================
 * Hỗ trợ:
 * - Đăng nhập tài khoản & mật khẩu chuẩn ngân hàng
 * - Đăng nhập 1-chạm (Quick Login) dành cho tài khoản kiểm thử theo từng Role:
 *   + MAKER: Kế toán viên lập lệnh
 *   + CHECKER: Kế toán trưởng kiểm soát kép (Thông tư 09/2020/TT-NHNN)
 *   + CFO: Giám đốc Tài chính
 *   + AUDITOR: Kiểm toán viên Độc lập (Chỉ đọc)
 *   + ADMIN: Quản trị viên Hệ thống
 */
import { ref, onMounted } from 'vue';
import { useAuthStore, type UserProfile } from '../../stores/authStore';

const authStore = useAuthStore();

const inputUsername = ref('');
const inputPassword = ref('');
const errorMessage = ref<string | null>(null);
const isLoading = ref(false);

onMounted(() => {
  authStore.fetchAccounts().catch(() => {});
});

async function handleLogin() {
  errorMessage.value = null;
  if (!inputUsername.value.trim()) {
    errorMessage.value = 'Vui lòng nhập tên đăng nhập.';
    return;
  }
  if (!inputPassword.value) {
    errorMessage.value = 'Vui lòng nhập mật khẩu.';
    return;
  }

  isLoading.value = true;
  try {
    const res = await authStore.loginAsync(inputUsername.value, inputPassword.value, false);
    if (!res.success) {
      errorMessage.value = res.error || 'Đăng nhập thất bại.';
    } else {
      inputUsername.value = '';
      inputPassword.value = '';
    }
  } catch (err: any) {
    errorMessage.value = err?.message || 'Lỗi hệ thống khi đăng nhập.';
  } finally {
    isLoading.value = false;
  }
}

async function handleQuickSelect(account: UserProfile) {
  errorMessage.value = null;
  inputUsername.value = account.username;
  inputPassword.value = '';
  isLoading.value = true;
  try {
    const res = await authStore.loginAsync(
      account.username,
      undefined,
      true
    );
    if (!res.success) {
      errorMessage.value = res.error || 'Đăng nhập thất bại.';
    }
  } catch (err: any) {
    errorMessage.value = err?.message || 'Lỗi hệ thống khi đăng nhập.';
  } finally {
    isLoading.value = false;
  }
}

function handleClose() {
  authStore.closeLoginModal();
}
</script>

<template>
  <div v-if="authStore.isLoginModalOpen" class="login-modal-overlay">
    <div class="login-modal-container">
      <!-- Modal Header -->
      <div class="login-modal-header">
        <div class="header-brand">
          <div class="brand-logo">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="5" width="20" height="14" rx="2" />
              <line x1="2" y1="10" x2="22" y2="10" />
            </svg>
          </div>
          <div>
            <h2 class="brand-title">LIVA Banking Authentication</h2>
            <span class="brand-sub">Hệ thống Quản trị Ngân quỹ & Đối soát Chuẩn TT 09/2020</span>
          </div>
        </div>

        <button
          v-if="authStore.isAuthenticated"
          class="close-btn"
          title="Đóng cửa sổ"
          @click="handleClose"
        >
          ✕
        </button>
      </div>

      <!-- Compliance Notice -->
      <div class="compliance-banner">
        <span class="compliance-icon">🔒</span>
        <span class="compliance-text">
          Chế độ Zero-Egress kích hoạt. Phiên làm việc được bảo vệ bằng mã hóa per-machine DPAPI và kiểm soát kép Maker-Checker.
        </span>
      </div>

      <!-- Error Alert -->
      <div v-if="errorMessage" class="error-alert">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
        <span>{{ errorMessage }}</span>
      </div>

      <!-- Form Inputs -->
      <form class="login-form" @submit.prevent="handleLogin">
        <div class="form-group">
          <label class="form-label">Tên đăng nhập (Username)</label>
          <input
            v-model="inputUsername"
            type="text"
            class="form-input"
            placeholder="vd: maker_nam, checker_tri, cfo_hoang..."
            autocapitalize="none"
            autocomplete="username"
          />
        </div>

        <div class="form-group">
          <label class="form-label">Mật khẩu (Password)</label>
          <input
            v-model="inputPassword"
            type="password"
            class="form-input"
            placeholder="Nhập mật khẩu..."
            autocomplete="current-password"
          />
        </div>

        <button type="submit" class="submit-btn" :disabled="isLoading">
          <span v-if="isLoading">Đang xác thực...</span>
          <span v-else>Đăng Nhập Vào Hệ Thống</span>
        </button>
      </form>

      <!-- Divider -->
      <div class="login-divider">
        <span class="divider-text">HOẶC ĐĂNG NHẬP NHANH THEO VAI TRÒ (DEMO / AUDIT)</span>
      </div>

      <!-- Quick Role Selection List -->
      <div class="quick-accounts-list">
        <div
          v-for="acc in authStore.accounts"
          :key="acc.username"
          class="quick-account-card"
          :class="{ 'card-active': authStore.currentUser.username === acc.username && authStore.isAuthenticated }"
          @click="handleQuickSelect(acc)"
        >
          <div class="account-avatar" :style="{ backgroundColor: acc.avatarColor }">
            {{ acc.avatarInitials }}
          </div>

          <div class="account-details">
            <div class="account-header-line">
              <span class="account-name">{{ acc.fullName }}</span>
              <span class="role-badge" :class="`badge-${acc.role.toLowerCase()}`">{{ acc.role }}</span>
            </div>
            <div class="account-user-meta">
              <code>{{ acc.username }}</code> · {{ acc.department }}
            </div>
            <div class="account-desc">
              {{ acc.description }}
            </div>
          </div>

          <button type="button" class="quick-select-btn" title="Đăng nhập tài khoản này">
            Chọn
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.login-modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background-color: rgba(15, 23, 42, 0.7);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.login-modal-container {
  background: #ffffff;
  width: 100%;
  max-width: 580px;
  max-height: 90vh;
  border-radius: 16px;
  box-shadow: 0 20px 40px -15px rgba(0, 0, 0, 0.25), 0 0 0 1px rgba(226, 232, 240, 0.8);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  padding: 24px;
}

.login-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.header-brand {
  display: flex;
  align-items: center;
  gap: 12px;
}

.brand-logo {
  width: 42px;
  height: 42px;
  border-radius: 10px;
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
  color: #10b981;
  display: flex;
  align-items: center;
  justify-content: center;
}

.brand-title {
  font-size: 17px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
  letter-spacing: -0.2px;
}

.brand-sub {
  font-size: 12px;
  color: #64748b;
  display: block;
}

.close-btn {
  background: transparent;
  border: none;
  font-size: 18px;
  color: #94a3b8;
  cursor: pointer;
  padding: 6px 10px;
  border-radius: 6px;
  transition: all 0.15s;
}

.close-btn:hover {
  background: #f1f5f9;
  color: #0f172a;
}

.compliance-banner {
  background-color: #ecfdf5;
  border: 1px solid #a7f3d0;
  color: #065f46;
  border-radius: 8px;
  padding: 10px 14px;
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 18px;
}

.compliance-icon {
  font-size: 16px;
}

.error-alert {
  background-color: #fef2f2;
  border: 1px solid #fecaca;
  color: #b91c1c;
  border-radius: 8px;
  padding: 10px 14px;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: #334155;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.form-input {
  height: 40px;
  border-radius: 8px;
  border: 1px solid #cbd5e1;
  padding: 0 14px;
  font-size: 14px;
  background-color: #f8fafc;
  color: #0f172a;
  outline: none;
  transition: all 0.15s ease;
}

.form-input:focus {
  border-color: #10b981;
  background-color: #ffffff;
  box-shadow: 0 0 0 3px rgba(16, 185, 129, 0.15);
}

.submit-btn {
  height: 42px;
  background-color: #0f172a;
  color: #ffffff;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.15s ease;
  margin-top: 4px;
}

.submit-btn:hover {
  background-color: #1e293b;
}

.login-divider {
  display: flex;
  align-items: center;
  text-align: center;
  margin: 22px 0 14px 0;
}

.login-divider::before,
.login-divider::after {
  content: '';
  flex: 1;
  border-bottom: 1px solid #e2e8f0;
}

.divider-text {
  padding: 0 10px;
  font-size: 11px;
  font-weight: 700;
  color: #94a3b8;
  letter-spacing: 0.5px;
}

.quick-accounts-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 280px;
  overflow-y: auto;
  padding-right: 4px;
}

.quick-account-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  background: #f8fafc;
  cursor: pointer;
  transition: all 0.15s ease;
}

.quick-account-card:hover {
  background: #ffffff;
  border-color: #94a3b8;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.04);
}

.quick-account-card.card-active {
  border-color: #10b981;
  background: #f0fdf4;
}

.account-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  color: #ffffff;
  font-size: 13px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.account-details {
  flex: 1;
  min-width: 0;
}

.account-header-line {
  display: flex;
  align-items: center;
  gap: 8px;
}

.account-name {
  font-size: 13px;
  font-weight: 600;
  color: #0f172a;
}

.role-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.badge-maker { background: #dbeafe; color: #1e40af; }
.badge-checker { background: #d1fae5; color: #065f46; }
.badge-cfo { background: #ede9fe; color: #5b21b6; }
.badge-auditor { background: #fef3c7; color: #92400e; }
.badge-admin { background: #fee2e2; color: #991b1b; }

.account-user-meta {
  font-size: 11px;
  color: #64748b;
  margin-top: 2px;
}

.account-user-meta code {
  font-family: monospace;
  background: #e2e8f0;
  padding: 1px 4px;
  border-radius: 3px;
  color: #334155;
}

.account-desc {
  font-size: 11px;
  color: #64748b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

.quick-select-btn {
  padding: 4px 10px;
  background: #ffffff;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  color: #334155;
  cursor: pointer;
  transition: all 0.12s;
  flex-shrink: 0;
}

.quick-account-card:hover .quick-select-btn {
  background: #0f172a;
  color: #ffffff;
  border-color: #0f172a;
}
</style>
