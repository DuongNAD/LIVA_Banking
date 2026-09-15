<script setup lang="ts">
/**
 * ErpBridgeTab.vue — P103 Cấu Hình Cầu Nối Đồng Bộ ERP
 * =====================================================
 * Cấu hình kết nối API ERP (MISA AMIS, FAST Accounting, SAP Business One).
 * Quản lý chu kỳ đồng bộ hóa sổ cái và tự động sinh bút toán hạch toán.
 */
import { useSettingsStore } from '../../../stores/settingsStore';

const store = useSettingsStore();
</script>

<template>
  <div class="settings-card">
    <div class="card-header">
      <div>
        <div class="header-tags">
          <span class="p-tag">P103 ERP BRIDGES</span>
          <span class="status-pill ok">
            Kích hoạt: {{ store.activeErpCount }}/{{ store.erpConfigs.length }} Hệ thống
          </span>
        </div>
        <h3 class="card-title">Cấu Hình Cầu Nối Đồng Bộ Kế Toán & ERP Doanh Nghiệp</h3>
        <p class="card-desc">
          Tự động sinh và truyền tải chứng từ đối soát sang phần mềm kế toán theo chuẩn XML/JSON.
        </p>
      </div>
    </div>

    <!-- ERP Grid -->
    <div class="erp-list">
      <div
        v-for="erp in store.erpConfigs"
        :key="erp.erpName"
        class="erp-item-card"
        :class="{ active: erp.isActive }"
      >
        <div class="erp-top">
          <div>
            <div class="erp-name-wrap">
              <h4 class="erp-name">{{ erp.erpName }}</h4>
              <span class="status-badge" :class="{ ok: erp.isActive }">
                {{ erp.isActive ? 'ĐANG KÍCH HOẠT' : 'CHƯA KÍCH HOẠT' }}
              </span>
            </div>
            <p class="erp-desc">{{ erp.description }}</p>
          </div>

          <button
            class="toggle-btn"
            :class="{ active: erp.isActive }"
            @click="store.toggleErpActive(erp.erpName)"
          >
            {{ erp.isActive ? 'Tạm ngắt đồng bộ' : 'Kích hoạt đồng bộ' }}
          </button>
        </div>

        <div class="erp-details">
          <div class="detail-col">
            <span class="label">API Gateway Endpoint:</span>
            <code class="val font-mono">{{ erp.endpoint }}</code>
          </div>
          <div class="detail-row">
            <div>
              <span class="label">Chu kỳ đồng bộ tự động:</span>
              <strong class="val">{{ erp.syncIntervalMins }} phút / lần</strong>
            </div>
            <div>
              <span class="label">Đồng bộ gần nhất:</span>
              <span class="val text-muted">{{ erp.lastSyncAt }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-card {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.header-tags {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.p-tag {
  font-size: 11px;
  font-weight: 700;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.12);
  border: 1px solid rgba(56, 189, 248, 0.3);
  padding: 2px 8px;
  border-radius: 4px;
}

.status-pill {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 4px;
}

.status-pill.ok {
  color: #10b981;
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.card-title {
  font-size: 16px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0 0 4px 0;
}

.card-desc {
  font-size: 12px;
  color: #94a3b8;
  margin: 0;
}

.erp-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.erp-item-card {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.erp-item-card.active {
  border-color: rgba(56, 189, 248, 0.25);
}

.erp-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
}

.erp-name-wrap {
  display: flex;
  align-items: center;
  gap: 10px;
}

.erp-name {
  font-size: 15px;
  font-weight: 700;
  color: #f1f5f9;
  margin: 0;
}

.status-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(100, 116, 139, 0.2);
  color: #94a3b8;
}

.status-badge.ok {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
}

.erp-desc {
  font-size: 12px;
  color: #94a3b8;
  margin: 4px 0 0 0;
}

.toggle-btn {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #cbd5e1;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s;
}

.toggle-btn.active {
  background: rgba(239, 68, 68, 0.15);
  border-color: rgba(239, 68, 68, 0.3);
  color: #f87171;
}

.toggle-btn:not(.active) {
  background: rgba(16, 185, 129, 0.15);
  border-color: rgba(16, 185, 129, 0.3);
  color: #10b981;
}

.erp-details {
  background: #090d16;
  border: 1px solid rgba(255, 255, 255, 0.04);
  border-radius: 6px;
  padding: 10px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.detail-col {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  border-top: 1px solid rgba(255, 255, 255, 0.04);
  padding-top: 6px;
  font-size: 12px;
}

.label {
  font-size: 11px;
  color: #64748b;
  margin-right: 6px;
}

.val {
  color: #cbd5e1;
}

.text-muted {
  color: #64748b;
}
</style>
