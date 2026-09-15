<script setup lang="ts">
/**
 * ErpPayloadModal.vue — P51 ERP Integration Payload Inspector
 * ==============================================================
 * Displays the serialized payload formatted for MISA AMIS (JSON)
 * and FAST Business Online (XML) with Idempotency GUID verification.
 */
import { ref, computed } from 'vue';
import { useLedgerStore } from '../../../stores/ledgerStore';

const store = useLedgerStore();
const activeTab = ref<'MISA' | 'FAST'>('MISA');
const isCopied = ref(false);

const voucher = computed(() => store.activePayloadVoucher);

const payloadContent = computed(() => {
  if (!voucher.value) return '';
  if (activeTab.value === 'MISA') {
    return store.getMisaJson(voucher.value);
  } else {
    return store.getFastXml(voucher.value);
  }
});

async function copyPayload() {
  if (!payloadContent.value) return;
  try {
    await navigator.clipboard.writeText(payloadContent.value);
    isCopied.value = true;
    setTimeout(() => {
      isCopied.value = false;
    }, 2000);
  } catch (e) {
    // fallback
  }
}
</script>

<template>
  <div v-if="store.isPayloadModalOpen && voucher" class="modal-overlay" @click.self="store.closePayloadModal">
    <div class="payload-modal-box">
      <!-- Modal Header -->
      <div class="modal-header">
        <div class="header-left">
          <span class="modal-tag">P51 ERP PAYLOAD</span>
          <h2 class="modal-title">Gói Tin Đồng Bộ ERP — {{ voucher.voucherNumber }}</h2>
        </div>
        <button class="btn-close" @click="store.closePayloadModal">✕</button>
      </div>

      <!-- Idempotency & Security Banner -->
      <div class="idempotency-banner">
        <div class="guid-info">
          <span class="banner-lbl">Idempotency GUID:</span>
          <span class="banner-val">{{ voucher.voucherGuid }}</span>
        </div>
        <div class="hash-info">
          <span class="banner-lbl">Idempotency Hash:</span>
          <span class="banner-val">{{ voucher.idempotencyHash }}</span>
        </div>
      </div>

      <!-- Format Switcher & Action Bar -->
      <div class="format-toolbar">
        <div class="format-tabs">
          <button
            class="format-tab"
            :class="{ active: activeTab === 'MISA' }"
            @click="activeTab = 'MISA'"
          >
            MISA AMIS (REST JSON)
          </button>
          <button
            class="format-tab"
            :class="{ active: activeTab === 'FAST' }"
            @click="activeTab = 'FAST'"
          >
            FAST Business (XML)
          </button>
        </div>

        <button class="btn-copy" @click="copyPayload">
          <span v-if="isCopied">✓ Đã sao chép</span>
          <span v-else>📋 Sao chép gói tin</span>
        </button>
      </div>

      <!-- Code Viewer -->
      <div class="code-viewer-container">
        <pre class="code-block"><code>{{ payloadContent }}</code></pre>
      </div>

      <!-- Modal Footer -->
      <div class="modal-footer">
        <span class="compliance-note">
          🔒 Zero Data Egress: Gói tin chỉ được truyền tải qua cổng nội bộ hoặc API Gateway on-premise của doanh nghiệp.
        </span>
        <button class="btn btn-secondary" @click="store.closePayloadModal">Đóng</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
  backdrop-filter: blur(4px);
  padding: 20px;
}

.payload-modal-box {
  background: #111827;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  width: 100%;
  max-width: 780px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: #1e293b;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.modal-tag {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 800;
  background: rgba(56, 189, 248, 0.2);
  color: #38bdf8;
  padding: 2px 6px;
  border-radius: 4px;
}

.modal-title {
  font-size: 15px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0;
}

.btn-close {
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 16px;
  cursor: pointer;
  padding: 4px 8px;
}

.btn-close:hover {
  color: #f8fafc;
}

.idempotency-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #0f172a;
  padding: 10px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  font-size: 12px;
}

.guid-info, .hash-info {
  display: flex;
  align-items: center;
  gap: 6px;
}

.banner-lbl {
  color: #64748b;
  font-weight: 600;
}

.banner-val {
  font-family: 'JetBrains Mono', monospace;
  color: #cbd5e1;
  font-weight: 600;
}

.format-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 20px;
  background: #182234;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.format-tabs {
  display: flex;
  gap: 6px;
}

.format-tab {
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #94a3b8;
  padding: 5px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.format-tab.active {
  background: #3b82f6;
  border-color: #3b82f6;
  color: #ffffff;
}

.btn-copy {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #e2e8f0;
  padding: 5px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-copy:hover {
  background: rgba(255, 255, 255, 0.15);
}

.code-viewer-container {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  background: #090d16;
}

.code-block {
  margin: 0;
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  line-height: 1.5;
  color: #a7f3d0;
  white-space: pre-wrap;
  word-break: break-all;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: #1e293b;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.compliance-note {
  font-size: 11px;
  color: #94a3b8;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #f1f5f9;
  padding: 6px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.12);
}
</style>
