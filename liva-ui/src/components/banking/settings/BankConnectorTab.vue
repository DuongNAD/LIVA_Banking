<script setup lang="ts">
/**
 * BankConnectorTab.vue — P102 Cổng Kết Nối Open Banking API
 * ==========================================================
 * Cấu hình kết nối API B2B đa ngân hàng (VCB, TCB, BIDV, MBB).
 * Hỗ trợ chuyển đổi Sandbox / Live, kiểm tra độ trễ (latency ping).
 */
import { useSettingsStore } from '../../../stores/settingsStore';

const store = useSettingsStore();
</script>

<template>
  <div class="settings-card">
    <div class="card-header">
      <div>
        <div class="header-tags">
          <span class="p-tag">P102 OPEN BANKING B2B</span>
          <span class="status-pill ok">
            Đang kết nối: {{ store.connectedBankCount }}/{{ store.bankConfigs.length }} Ngân hàng
          </span>
        </div>
        <h3 class="card-title">Cổng Kết Nối Open Banking API & Dịch Vụ Sao Kê Điện Tử</h3>
        <p class="card-desc">
          Chuẩn giao thức kết nối mTLS và ký số điện tử API B2B đa ngân hàng Việt Nam.
        </p>
      </div>
    </div>

    <!-- Banks Grid -->
    <div class="banks-grid">
      <div
        v-for="bank in store.bankConfigs"
        :key="bank.bankCode"
        class="bank-card"
        :class="{ active: bank.isConnected }"
      >
        <div class="bank-top">
          <div class="bank-identity">
            <span class="bank-code-badge">{{ bank.bankCode }}</span>
            <div class="bank-info">
              <h4 class="bank-name">{{ bank.bankName }}</h4>
              <span class="bank-sync">Đồng bộ gần nhất: {{ bank.lastSyncAt }}</span>
            </div>
          </div>
          <div class="bank-badges">
            <span class="env-badge" :class="{ sandbox: bank.isSandbox }">
              {{ bank.isSandbox ? 'SANDBOX' : 'PRODUCTION' }}
            </span>
            <span class="conn-badge" :class="{ ok: bank.isConnected }">
              {{ bank.isConnected ? 'Đang kết nối' : 'Đã ngắt' }}
            </span>
          </div>
        </div>

        <div class="endpoint-box">
          <span class="endpoint-label">API Endpoint:</span>
          <code class="endpoint-url font-mono">{{ bank.apiEndpoint }}</code>
        </div>

        <div class="bank-bottom">
          <div class="latency-indicator" v-if="bank.isConnected">
            <span class="ping-dot" />
            <span class="ping-text">Độ trễ phản hồi: <strong>{{ bank.latencyMs }} ms</strong></span>
          </div>
          <div class="latency-indicator text-muted" v-else>
            <span>Chưa kích hoạt</span>
          </div>

          <div class="bank-actions">
            <button
              class="action-btn"
              @click="store.toggleBankSandbox(bank.bankCode)"
            >
              {{ bank.isSandbox ? 'Sang Live' : 'Sang Sandbox' }}
            </button>
            <button
              class="action-btn"
              :class="bank.isConnected ? 'btn-disconnect' : 'btn-connect'"
              @click="store.toggleBankConnection(bank.bankCode)"
            >
              {{ bank.isConnected ? 'Ngắt kết nối' : 'Kết nối lại' }}
            </button>
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

.banks-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 14px;
}

@media (max-width: 900px) {
  .banks-grid {
    grid-template-columns: 1fr;
  }
}

.bank-card {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  transition: all 0.2s;
}

.bank-card.active {
  border-color: rgba(56, 189, 248, 0.25);
}

.bank-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 10px;
}

.bank-identity {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.bank-code-badge {
  font-size: 12px;
  font-weight: 700;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.12);
  border: 1px solid rgba(56, 189, 248, 0.25);
  padding: 4px 8px;
  border-radius: 6px;
  font-family: monospace;
}

.bank-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.bank-name {
  font-size: 13px;
  font-weight: 700;
  color: #f1f5f9;
  margin: 0;
}

.bank-sync {
  font-size: 11px;
  color: #64748b;
}

.bank-badges {
  display: flex;
  gap: 6px;
  flex-direction: column;
  align-items: flex-end;
}

.env-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
}

.env-badge.sandbox {
  background: rgba(56, 189, 248, 0.15);
  color: #38bdf8;
}

.conn-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(100, 116, 139, 0.2);
  color: #94a3b8;
}

.conn-badge.ok {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
}

.endpoint-box {
  background: #090d16;
  border: 1px solid rgba(255, 255, 255, 0.04);
  border-radius: 6px;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.endpoint-label {
  font-size: 10px;
  color: #64748b;
}

.endpoint-url {
  font-size: 11px;
  color: #cbd5e1;
  word-break: break-all;
}

.bank-bottom {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  padding-top: 10px;
}

.latency-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: #94a3b8;
}

.ping-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #10b981;
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.6);
}

.bank-actions {
  display: flex;
  gap: 6px;
}

.action-btn {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #cbd5e1;
  padding: 4px 8px;
  border-radius: 5px;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
}

.btn-connect {
  background: rgba(16, 185, 129, 0.15);
  border-color: rgba(16, 185, 129, 0.3);
  color: #10b981;
}

.btn-disconnect {
  background: rgba(239, 68, 68, 0.12);
  border-color: rgba(239, 68, 68, 0.3);
  color: #f87171;
}
</style>
