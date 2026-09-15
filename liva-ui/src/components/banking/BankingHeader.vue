<script setup lang="ts">
/**
 * BankingHeader.vue
 * Top dashboard header bar matching user mockup.
 */
import { ref, onMounted, onUnmounted } from 'vue';
import { useAuthStore } from '../../stores/authStore';

const props = defineProps<{
  userName?: string;
  hasUnreadNotifications?: boolean;
}>();

const emit = defineEmits<{
  (e: 'search', query: string): void;
  (e: 'toggleAssistant'): void;
}>();

const authStore = useAuthStore();
const searchText = ref('');
const isDropdownOpen = ref(false);
const dropdownRef = ref<HTMLElement | null>(null);

function onSearchInput() {
  emit('search', searchText.value);
}

function toggleDropdown() {
  isDropdownOpen.value = !isDropdownOpen.value;
}

function handleSwitchUser(username: string) {
  authStore.switchUser(username);
  isDropdownOpen.value = false;
}

function handleOpenLoginModal() {
  authStore.openLoginModal();
  isDropdownOpen.value = false;
}

function handleLogout() {
  authStore.logout();
  isDropdownOpen.value = false;
}

function handleClickOutside(event: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target as Node)) {
    isDropdownOpen.value = false;
  }
}

onMounted(() => {
  if (typeof window !== 'undefined') {
    window.addEventListener('click', handleClickOutside);
  }
});

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('click', handleClickOutside);
  }
});
</script>

<template>
  <header class="banking-header">
    <div class="header-left">
      <h1 class="header-title">LIVA Reconciliation Dashboard</h1>
    </div>

    <div class="header-right">
      <!-- Search Input -->
      <div class="search-box">
        <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input
          v-model="searchText"
          type="text"
          class="search-input"
          placeholder="Tìm kiếm giao dịch, mã sao kê..."
          @input="onSearchInput"
        />
      </div>

      <!-- Financial Assistant Trigger Pill -->
      <button class="assistant-btn" title="Mở Trợ lý Tài chính 2D" @click="emit('toggleAssistant')">
        <span class="assistant-wave-icon">
          <span class="wave-bar" />
          <span class="wave-bar" />
          <span class="wave-bar" />
        </span>
        <span class="assistant-label">Trợ lý AI</span>
      </button>

      <!-- Notification Bell with Red Dot -->
      <button class="bell-btn" title="Thông báo hệ thống">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" />
          <path d="M13.73 21a2 2 0 0 1-3.46 0" />
        </svg>
        <span class="bell-badge" />
      </button>

      <!-- User Profile Badge & Dropdown -->
      <div ref="dropdownRef" class="user-profile-wrapper">
        <div class="user-profile-badge" @click.stop="toggleDropdown">
          <div
            class="user-avatar"
            :style="{ backgroundColor: authStore.currentUser.avatarColor, color: '#ffffff' }"
          >
            {{ authStore.currentUser.avatarInitials || 'MT' }}
          </div>
          <div class="user-meta-column">
            <span class="user-name">{{ userName || authStore.currentUser.fullName }}</span>
            <span class="user-role-tag" :class="`tag-${authStore.currentUser.role.toLowerCase()}`">
              {{ authStore.currentUser.role }}
            </span>
          </div>
          <svg
            class="chevron-icon"
            :class="{ 'rotate-180': isDropdownOpen }"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </div>

        <!-- Dropdown Menu -->
        <div v-if="isDropdownOpen" class="user-dropdown-menu">
          <!-- Active User Profile Section -->
          <div class="dropdown-header">
            <div class="dropdown-user-name">{{ authStore.currentUser.fullName }}</div>
            <div class="dropdown-user-role">{{ authStore.currentUser.roleTitle }}</div>
            <div class="dropdown-user-dept">{{ authStore.currentUser.department }}</div>
          </div>

          <div class="dropdown-divider" />

          <!-- Switch Role Header -->
          <div class="dropdown-section-title">
            CHUYỂN NHANH VAI TRÒ (RBAC)
          </div>

          <!-- Accounts List -->
          <div class="dropdown-accounts-list">
            <button
              v-for="acc in authStore.accounts"
              :key="acc.username"
              type="button"
              class="dropdown-account-item"
              :class="{ 'item-active': acc.username === authStore.currentUser.username }"
              @click="handleSwitchUser(acc.username)"
            >
              <span class="item-avatar" :style="{ backgroundColor: acc.avatarColor }">
                {{ acc.avatarInitials }}
              </span>
              <div class="item-info">
                <div class="item-top">
                  <span class="item-name">{{ acc.fullName }}</span>
                  <span class="item-badge" :class="`badge-${acc.role.toLowerCase()}`">{{ acc.role }}</span>
                </div>
                <div class="item-sub"><code>{{ acc.username }}</code> · {{ acc.roleTitle }}</div>
              </div>
              <span v-if="acc.username === authStore.currentUser.username" class="item-check">✓</span>
            </button>
          </div>

          <div class="dropdown-divider" />

          <!-- Action buttons -->
          <button type="button" class="dropdown-action-btn" @click="handleOpenLoginModal">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />
              <polyline points="10 17 15 12 10 7" />
              <line x1="15" y1="12" x2="3" y2="12" />
            </svg>
            <span>Đăng nhập tài khoản khác...</span>
          </button>

          <button type="button" class="dropdown-action-btn btn-danger" @click="handleLogout">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
              <polyline points="16 17 21 12 16 7" />
              <line x1="21" y1="12" x2="9" y2="12" />
            </svg>
            <span>Đăng xuất phiên làm việc</span>
          </button>
        </div>
      </div>
    </div>
  </header>
</template>

<style scoped>
.banking-header {
  height: 64px;
  background-color: #ffffff;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.03);
}

.header-title {
  font-size: 20px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
  letter-spacing: -0.3px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: #94a3b8;
  pointer-events: none;
}

.search-input {
  width: 240px;
  height: 36px;
  padding: 0 12px 0 36px;
  border-radius: 20px;
  border: 1px solid #cbd5e1;
  background-color: #f8fafc;
  color: #1e293b;
  font-size: 13px;
  outline: none;
  transition: all 0.15s ease;
}

.search-input:focus {
  width: 280px;
  border-color: #10b981;
  background-color: #ffffff;
  box-shadow: 0 0 0 3px rgba(16, 185, 129, 0.15);
}

.assistant-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 36px;
  padding: 0 14px;
  background: #ecfdf5;
  border: 1px solid #a7f3d0;
  border-radius: 18px;
  color: #065f46;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.assistant-btn:hover {
  background: #d1fae5;
  border-color: #6ee7b7;
  transform: translateY(-1px);
}

.assistant-wave-icon {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 12px;
}

.wave-bar {
  width: 3px;
  height: 100%;
  background: #10b981;
  border-radius: 1px;
  animation: wave-anim 1.2s infinite ease-in-out;
}

.wave-bar:nth-child(2) {
  height: 70%;
  animation-delay: 0.2s;
}

.wave-bar:nth-child(3) {
  height: 90%;
  animation-delay: 0.4s;
}

@keyframes wave-anim {
  0%, 100% { transform: scaleY(0.5); }
  50% { transform: scaleY(1); }
}

.bell-btn {
  position: relative;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  border: 1px solid #e2e8f0;
  background: #ffffff;
  color: #64748b;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s ease;
}

.bell-btn:hover {
  background: #f8fafc;
  color: #1e293b;
}

.bell-badge {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 8px;
  height: 8px;
  background-color: #ef4444;
  border-radius: 50%;
  border: 2px solid #ffffff;
}

.user-profile-wrapper {
  position: relative;
}

.user-profile-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 10px 4px 4px;
  border-radius: 20px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  cursor: pointer;
  transition: all 0.15s ease;
  user-select: none;
}

.user-profile-badge:hover {
  background: #f1f5f9;
  border-color: #cbd5e1;
}

.user-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  font-size: 12px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.user-meta-column {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  line-height: 1.2;
}

.user-name {
  font-size: 13px;
  font-weight: 600;
  color: #0f172a;
}

.user-role-tag {
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  padding: 1px 4px;
  border-radius: 3px;
  margin-top: 2px;
}

.tag-maker { background: #dbeafe; color: #1e40af; }
.tag-checker { background: #d1fae5; color: #065f46; }
.tag-cfo { background: #ede9fe; color: #5b21b6; }
.tag-auditor { background: #fef3c7; color: #92400e; }
.tag-admin { background: #fee2e2; color: #991b1b; }

.chevron-icon {
  color: #94a3b8;
  transition: transform 0.2s ease;
  margin-left: 2px;
}

.rotate-180 {
  transform: rotate(180deg);
}

.user-dropdown-menu {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  width: 310px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.15), 0 0 0 1px rgba(226, 232, 240, 0.8);
  padding: 12px;
  z-index: 500;
  animation: dropdownFadeIn 0.15s ease-out;
}

@keyframes dropdownFadeIn {
  from { opacity: 0; transform: translateY(-6px); }
  to { opacity: 1; transform: translateY(0); }
}

.dropdown-header {
  padding: 4px 6px 8px 6px;
}

.dropdown-user-name {
  font-size: 14px;
  font-weight: 700;
  color: #0f172a;
}

.dropdown-user-role {
  font-size: 12px;
  font-weight: 600;
  color: #059669;
  margin-top: 2px;
}

.dropdown-user-dept {
  font-size: 11px;
  color: #64748b;
  margin-top: 1px;
}

.dropdown-divider {
  height: 1px;
  background-color: #f1f5f9;
  margin: 8px 0;
}

.dropdown-section-title {
  font-size: 10px;
  font-weight: 700;
  color: #94a3b8;
  letter-spacing: 0.5px;
  padding: 4px 6px;
}

.dropdown-accounts-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 220px;
  overflow-y: auto;
}

.dropdown-account-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 6px 8px;
  border-radius: 8px;
  background: transparent;
  border: 1px solid transparent;
  text-align: left;
  cursor: pointer;
  transition: all 0.12s ease;
}

.dropdown-account-item:hover {
  background: #f8fafc;
  border-color: #e2e8f0;
}

.dropdown-account-item.item-active {
  background: #f0fdf4;
  border-color: #a7f3d0;
}

.item-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  color: #ffffff;
  font-size: 11px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.item-info {
  flex: 1;
  min-width: 0;
}

.item-top {
  display: flex;
  align-items: center;
  gap: 6px;
}

.item-name {
  font-size: 12px;
  font-weight: 600;
  color: #1e293b;
}

.item-badge {
  font-size: 9px;
  font-weight: 700;
  padding: 1px 4px;
  border-radius: 3px;
  text-transform: uppercase;
}

.badge-maker { background: #dbeafe; color: #1e40af; }
.badge-checker { background: #d1fae5; color: #065f46; }
.badge-cfo { background: #ede9fe; color: #5b21b6; }
.badge-auditor { background: #fef3c7; color: #92400e; }
.badge-admin { background: #fee2e2; color: #991b1b; }

.item-sub {
  font-size: 10px;
  color: #64748b;
  margin-top: 1px;
}

.item-sub code {
  font-family: monospace;
  background: #f1f5f9;
  padding: 1px 3px;
  border-radius: 2px;
}

.item-check {
  color: #10b981;
  font-weight: 800;
  font-size: 14px;
}

.dropdown-action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px;
  border-radius: 6px;
  background: transparent;
  border: none;
  font-size: 12px;
  font-weight: 500;
  color: #475569;
  cursor: pointer;
  transition: all 0.12s ease;
  margin-top: 2px;
}

.dropdown-action-btn:hover {
  background: #f1f5f9;
  color: #0f172a;
}

.dropdown-action-btn.btn-danger {
  color: #dc2626;
}

.dropdown-action-btn.btn-danger:hover {
  background: #fef2f2;
  color: #b91c1c;
}
</style>
