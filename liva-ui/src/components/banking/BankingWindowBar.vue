<script setup lang="ts">
/**
 * BankingWindowBar.vue
 * Shell top window bar with traffic light window controls, local URL, and MVP badge.
 */
import { ref, computed, onMounted } from 'vue';
import {
  getConfiguredBackendUrl,
  setConfiguredBackendUrl,
  isTauriEnvironment,
} from '../../platform';

defineProps<{
  url?: string;
  badgeText?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'minimize'): void;
  (e: 'maximize'): void;
}>();

const isTauri = ref(false);
const backendUrl = ref('');
const isConfigModalOpen = ref(false);
const inputUrl = ref('');

onMounted(() => {
  isTauri.value = isTauriEnvironment();
  backendUrl.value = getConfiguredBackendUrl() || '';
  inputUrl.value = backendUrl.value;
});

const backendBadgeLabel = computed(() => {
  if (!backendUrl.value) return 'Web Demo';
  try {
    const u = new URL(backendUrl.value);
    return `Core: ${u.host}`;
  } catch {
    return 'Core: Live';
  }
});

function openConfigModal() {
  inputUrl.value = backendUrl.value;
  isConfigModalOpen.value = true;
}

function handleSaveBackend() {
  const target = inputUrl.value.trim();
  setConfiguredBackendUrl(target || null);
  isConfigModalOpen.value = false;
  if (typeof window !== 'undefined') {
    window.location.reload();
  }
}

function handleRunDemo() {
  setConfiguredBackendUrl(null);
  isConfigModalOpen.value = false;
  if (typeof window !== 'undefined') {
    window.location.reload();
  }
}

async function handleClose() {
  emit('close');
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().close();
    } catch {
      // Ignored in non-Tauri browser previews
    }
  }
}

async function handleMinimize() {
  emit('minimize');
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().minimize();
    } catch {
      // Ignored in non-Tauri browser previews
    }
  }
}

async function handleMaximize() {
  emit('maximize');
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().toggleMaximize();
    } catch {
      // Ignored in non-Tauri browser previews
    }
  }
}
</script>

<template>
  <div class="banking-window-bar" data-tauri-drag-region>
    <!-- Window Controls: Traffic light dots -->
    <div class="window-controls">
      <button class="dot dot-close" title="Đóng" @click="handleClose" />
      <button class="dot dot-minimize" title="Thu nhỏ" @click="handleMinimize" />
      <button class="dot dot-maximize" title="Phóng to" @click="handleMaximize" />
    </div>

    <!-- Center URL / Host address -->
    <div class="window-url-badge">
      <span class="url-protocol">liva://</span><span class="url-host">{{ url || 'reconciliation.local' }}</span>
    </div>

    <!-- Right Badge: Backend Tunnel Status + MVP THỰC TẾ -->
    <div class="window-actions">
      <button
        v-if="!isTauri"
        class="backend-status-btn"
        :class="{ 'is-connected': !!backendUrl, 'is-demo': !backendUrl }"
        :title="backendUrl ? `Đang kết nối Backend: ${backendUrl} (Bấm để đổi)` : 'Đang chạy Web Demo Offline (Bấm để nối Core cục bộ)'"
        @click="openConfigModal"
      >
        <span class="status-dot" />
        <span class="status-text">{{ backendBadgeLabel }}</span>
      </button>
      <span class="mvp-badge">{{ badgeText || 'MVP THỰC TẾ' }}</span>
    </div>

    <!-- Backend Tunnel Configuration Modal -->
    <div v-if="isConfigModalOpen" class="backend-modal-overlay" @click.self="isConfigModalOpen = false">
      <div class="backend-modal">
        <div class="modal-header">
          <h3>🔌 Kết Nối LIVA Native Core Cục Bộ</h3>
          <button class="modal-close" @click="isConfigModalOpen = false">✕</button>
        </div>
        <div class="modal-body">
          <p class="modal-tip">
            Khi triển khai trên <strong>Cloudflare Pages</strong> hoặc mở web từ xa, nhập URL Secure Tunnel (ngrok / Cloudflare Tunnel) trỏ vào cổng <code>8002</code> trên máy tính của bạn:
          </p>
          <div class="form-group">
            <label>URL Backend Tunnel (HTTPS):</label>
            <input
              v-model="inputUrl"
              type="text"
              placeholder="Ví dụ: https://xxx.ngrok-free.app hoặc https://xxx.trycloudflare.com"
              class="modal-input"
              @keyup.enter="handleSaveBackend"
            />
          </div>
          <div class="modal-quick-command">
            <span>Mở tunnel nhanh trên máy bạn:</span>
            <code>npm run tunnel:backend</code>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn-demo" @click="handleRunDemo">
            Chạy Web Demo (Mock)
          </button>
          <button class="btn-save" @click="handleSaveBackend">
            Lưu & Kết nối lại
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.banking-window-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 38px;
  background-color: #1a2332;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  padding: 0 16px;
  user-select: none;
  z-index: 50;
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: none;
  cursor: pointer;
  padding: 0;
  transition: transform 0.15s ease, opacity 0.15s ease;
}

.dot:hover {
  transform: scale(1.15);
  opacity: 0.9;
}

.dot-close {
  background-color: #ef4444;
}

.dot-minimize {
  background-color: #f59e0b;
}

.dot-maximize {
  background-color: #10b981;
}

.window-url-badge {
  display: flex;
  align-items: center;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 13px;
  color: #94a3b8;
  letter-spacing: 0.3px;
  background: rgba(0, 0, 0, 0.25);
  padding: 3px 12px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.04);
}

.url-protocol {
  color: #64748b;
}

.url-host {
  color: #cbd5e1;
  font-weight: 500;
}

.window-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.backend-status-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 4px;
  padding: 3px 8px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  color: #cbd5e1;
  transition: all 0.2s ease;
}

.backend-status-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  border-color: rgba(255, 255, 255, 0.2);
}

.status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  display: inline-block;
}

.backend-status-btn.is-connected .status-dot {
  background-color: #10b981;
  box-shadow: 0 0 6px #10b981;
}

.backend-status-btn.is-demo .status-dot {
  background-color: #f59e0b;
}

.mvp-badge {
  background: #059669;
  color: #ffffff;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.5px;
  padding: 3px 10px;
  border-radius: 4px;
  box-shadow: 0 1px 3px rgba(5, 150, 105, 0.3);
}

/* Modal Overlay & Card */
.backend-modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.backend-modal {
  width: 90%;
  max-width: 520px;
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 10px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5), 0 8px 10px -6px rgba(0, 0, 0, 0.4);
  color: #f1f5f9;
  overflow: hidden;
  animation: modal-fade-in 0.2s ease-out;
}

@keyframes modal-fade-in {
  from {
    opacity: 0;
    transform: scale(0.96);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: #0f172a;
}

.modal-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #38bdf8;
}

.modal-close {
  background: none;
  border: none;
  color: #94a3b8;
  font-size: 16px;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.modal-close:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
}

.modal-body {
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.modal-tip {
  margin: 0;
  font-size: 13px;
  line-height: 1.5;
  color: #94a3b8;
}

.modal-tip strong {
  color: #e2e8f0;
}

.modal-tip code {
  background: rgba(0, 0, 0, 0.3);
  padding: 2px 5px;
  border-radius: 4px;
  color: #38bdf8;
  font-size: 12px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 12px;
  font-weight: 500;
  color: #cbd5e1;
}

.modal-input {
  width: 100%;
  box-sizing: border-box;
  padding: 9px 12px;
  background: #0f172a;
  border: 1px solid #334155;
  border-radius: 6px;
  color: #f8fafc;
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
}

.modal-input:focus {
  border-color: #38bdf8;
  box-shadow: 0 0 0 2px rgba(56, 189, 248, 0.2);
}

.modal-quick-command {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(15, 23, 42, 0.6);
  border: 1px dashed rgba(255, 255, 255, 0.12);
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 12px;
  color: #94a3b8;
}

.modal-quick-command code {
  color: #a7f3d0;
  background: rgba(0, 0, 0, 0.4);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  padding: 12px 18px;
  background: #0f172a;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.btn-demo {
  background: transparent;
  border: 1px solid #475569;
  color: #cbd5e1;
  padding: 7px 14px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-demo:hover {
  background: rgba(255, 255, 255, 0.05);
  border-color: #94a3b8;
}

.btn-save {
  background: #0284c7;
  border: none;
  color: #fff;
  padding: 7px 16px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-save:hover {
  background: #0369a1;
}
</style>
