<script setup lang="ts">
/**
 * AmlAlertList.vue — P81 Danh Sách Cảnh Báo Giao Dịch Đáng Ngờ (AML Alerts)
 * ===========================================================================
 * Bảng hiển thị các giao dịch chạm ngưỡng Quyết định 11/2023/QĐ-TTg,
 * dấu hiệu chia nhỏ (Smurfing), và tài khoản trung chuyển (Mule).
 */
import { useComplianceStore } from '../../../stores/complianceStore';

const store = useComplianceStore();

function formatVnd(val: number): string {
  return `${val.toLocaleString('vi-VN')} ₫`;
}
</script>

<template>
  <div class="aml-alert-table-card">
    <div class="card-header">
      <div>
        <h3 class="card-title">Cảnh Báo Phòng Chống Rửa Tiền Đang Kích Hoạt (Active AML Alerts)</h3>
        <p class="card-desc">
          Tự động phát hiện theo Điều 25 & 26 Luật PCRT 2022 và Quyết định 11/2023/QĐ-TTg.
        </p>
      </div>

      <div class="badge-group">
        <span class="count-badge critical">
          {{ store.criticalCount }} Khẩn cấp (Critical)
        </span>
        <span class="count-badge high">
          {{ store.highCount }} Cao (High)
        </span>
      </div>
    </div>

    <!-- Alert Table -->
    <div class="table-wrap">
      <table class="alert-table">
        <thead>
          <tr>
            <th>Mã Giao Dịch</th>
            <th>Ngân Hàng</th>
            <th>Đối Tác & Tài Khoản</th>
            <th>Dấu Hiệu AML</th>
            <th class="text-right">Số Tiền (VND)</th>
            <th>Mức Độ</th>
            <th>Hành Động</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="item in store.alerts" :key="item.id" :class="item.severity.toLowerCase()">
            <td class="font-mono text-bold">{{ item.txId }}</td>
            <td>
              <span class="bank-pill">{{ item.bankCode }}</span>
            </td>
            <td>
              <div class="counterparty-wrap">
                <span class="cp-name">{{ item.counterpartyName }}</span>
                <span class="cp-acc font-mono">{{ item.counterpartyAccount }}</span>
              </div>
            </td>
            <td>
              <div class="rule-wrap">
                <span class="rule-name">{{ item.ruleCode.replace(/_/g, ' ') }}</span>
                <span class="rule-desc">{{ item.narrative }}</span>
              </div>
            </td>
            <td class="text-right font-mono text-bold amount-cell">
              {{ formatVnd(item.amount) }}
            </td>
            <td>
              <span class="severity-tag" :class="item.severity.toLowerCase()">
                {{ item.severity }}
              </span>
            </td>
            <td>
              <button
                class="btn-str-action"
                :class="{ 'has-report': item.hasStrReport }"
                @click="store.generateStrReport(item.id)"
              >
                {{ item.hasStrReport ? '📄 Xem STR' : '⚡ Lập Mẫu STR' }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.aml-alert-table-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.card-header {
  padding: 16px 20px;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-title {
  font-size: 14px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 2px 0;
}

.card-desc {
  font-size: 12px;
  color: #64748b;
  margin: 0;
}

.badge-group {
  display: flex;
  gap: 8px;
}

.count-badge {
  font-size: 11px;
  font-weight: 700;
  padding: 3px 10px;
  border-radius: 20px;
}

.count-badge.critical {
  background: #fee2e2;
  color: #b91c1c;
}

.count-badge.high {
  background: #fef3c7;
  color: #b45309;
}

.table-wrap {
  overflow-x: auto;
}

.alert-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}

.alert-table th {
  background: #f8fafc;
  color: #475569;
  font-weight: 700;
  padding: 10px 14px;
  border-bottom: 1px solid #e2e8f0;
  text-align: left;
}

.alert-table td {
  padding: 12px 14px;
  border-bottom: 1px solid #f1f5f9;
  vertical-align: middle;
}

.alert-table tr:hover {
  background: #f8fafc;
}

.alert-table tr.critical {
  background: #fffafa;
}

.bank-pill {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 700;
  background: #f1f5f9;
  color: #334155;
  padding: 2px 6px;
  border-radius: 4px;
}

.counterparty-wrap {
  display: flex;
  flex-direction: column;
}

.cp-name {
  font-weight: 600;
  color: #1e293b;
}

.cp-acc {
  font-size: 11px;
  color: #64748b;
}

.rule-wrap {
  display: flex;
  flex-direction: column;
  max-width: 320px;
}

.rule-name {
  font-weight: 700;
  color: #0f172a;
  font-size: 11px;
}

.rule-desc {
  font-size: 11px;
  color: #64748b;
  line-height: 1.3;
}

.amount-cell {
  font-size: 13px;
  color: #0f172a;
}

.severity-tag {
  font-size: 10px;
  font-weight: 800;
  padding: 2px 8px;
  border-radius: 4px;
  text-transform: uppercase;
}

.severity-tag.critical {
  background: #ef4444;
  color: #ffffff;
}

.severity-tag.high {
  background: #f59e0b;
  color: #ffffff;
}

.btn-str-action {
  background: #0f172a;
  color: #ffffff;
  border: none;
  font-size: 11px;
  font-weight: 600;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.btn-str-action:hover {
  background: #334155;
}

.btn-str-action.has-report {
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #bfdbfe;
}

.text-right {
  text-align: right;
}

.font-mono {
  font-family: 'JetBrains Mono', monospace;
}

.text-bold {
  font-weight: 700;
}
</style>
