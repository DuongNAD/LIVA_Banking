<script setup lang="ts">
/**
 * BankingHeader.vue
 * Top dashboard header bar matching user mockup.
 */
import { ref } from 'vue';

defineProps<{
  userName?: string;
  hasUnreadNotifications?: boolean;
}>();

const emit = defineEmits<{
  (e: 'search', query: string): void;
  (e: 'toggleAssistant'): void;
}>();

const searchText = ref('');

function onSearchInput() {
  emit('search', searchText.value);
}
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

      <!-- User Profile Badge -->
      <div class="user-profile-badge">
        <div class="user-avatar">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
            <circle cx="12" cy="7" r="4" />
          </svg>
        </div>
        <span class="user-name">{{ userName || 'Nguyễn Minh Trí' }}</span>
        <svg class="chevron-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 12 15 18 9" />
        </svg>
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

.user-profile-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px 4px 4px;
  border-radius: 20px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  cursor: pointer;
  transition: all 0.15s ease;
}

.user-profile-badge:hover {
  background: #f1f5f9;
}

.user-avatar {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  background: #e2e8f0;
  color: #475569;
  display: flex;
  align-items: center;
  justify-content: center;
}

.user-name {
  font-size: 13px;
  font-weight: 600;
  color: #1e293b;
}

.chevron-icon {
  color: #94a3b8;
}
</style>
