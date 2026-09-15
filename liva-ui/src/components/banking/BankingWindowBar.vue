<script setup lang="ts">
/**
 * BankingWindowBar.vue
 * Shell top window bar with traffic light window controls, local URL, and MVP badge.
 */
defineProps<{
  url?: string;
  badgeText?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'minimize'): void;
  (e: 'maximize'): void;
}>();

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

    <!-- Right Badge: MVP THỰC TẾ -->
    <div class="window-actions">
      <span class="mvp-badge">{{ badgeText || 'MVP THỰC TẾ' }}</span>
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
</style>
