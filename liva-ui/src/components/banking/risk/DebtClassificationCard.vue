<script setup lang="ts">
/**
 * DebtClassificationCard.vue — P72 Phân Loại Nợ 5 Nhóm theo Thông tư 11/2021/TT-NHNN
 * =================================================================================
 * Giám sát trạng thái quá hạn nợ vay, tỷ lệ trích lập dự phòng rủi ro chung (0.75%)
 * và dự phòng cụ thể (0% - 100%) cho từng nhóm nợ.
 */
import { useRiskStore, type DebtGroup } from '../../../stores/riskStore';

const store = useRiskStore();

function formatVnd(val: number): string {
  return `${val.toLocaleString('vi-VN')} ₫`;
}

const groups: {
  id: DebtGroup;
  name: string;
  overdueText: string;
  specRateText: string;
  badgeClass: string;
}[] = [
  {
    id: 'GROUP_1',
    name: 'Nhóm 1 — Nợ Đủ Tiêu Chuẩn',
    overdueText: '< 10 ngày',
    specRateText: '0%',
    badgeClass: 'badge-g1',
  },
  {
    id: 'GROUP_2',
    name: 'Nhóm 2 — Nợ Cần Chú Ý',
    overdueText: '10–90 ngày',
    specRateText: '5%',
    badgeClass: 'badge-g2',
  },
  {
    id: 'GROUP_3',
    name: 'Nhóm 3 — Nợ Dưới Tiêu Chuẩn',
    overdueText: '91–180 ngày',
    specRateText: '20%',
    badgeClass: 'badge-g3',
  },
  {
    id: 'GROUP_4',
    name: 'Nhóm 4 — Nợ Nghi Ngờ',
    overdueText: '181–360 ngày',
    specRateText: '50%',
    badgeClass: 'badge-g4',
  },
  {
    id: 'GROUP_5',
    name: 'Nhóm 5 — Nợ Có Khả Năng Mất Vốn',
    overdueText: '> 360 ngày',
    specRateText: '100%',
    badgeClass: 'badge-g5',
  },
];
</script>

<template>
  <div class="debt-card">
    <div class="debt-card-header">
      <div>
        <div class="header-tag-wrap">
          <span class="p-tag">P72 TT 11/2021/TT-NHNN</span>
          <h3 class="debt-card-title">Phân Loại Nhóm Nợ & Dự Phòng Rủi Ro Tín Dụng</h3>
        </div>
        <p class="debt-card-subtitle">
          Quy chế phân loại tài sản có và trích lập dự phòng theo quy định của Ngân hàng Nhà nước Việt Nam.
        </p>
      </div>

      <!-- Current Group Pill -->
      <div class="current-group-badge" :class="store.debtClassification.group.toLowerCase()">
        <span class="dot" />
        <span class="group-name">{{ store.debtClassification.groupName }}</span>
      </div>
    </div>

    <!-- 5-Group Stepper Indicator -->
    <div class="groups-stepper">
      <div
        v-for="g in groups"
        :key="g.id"
        class="step-item"
        :class="{ active: store.debtClassification.group === g.id }"
        @click="
          store.setDaysOverdue(
            g.id === 'GROUP_1' ? 0 : g.id === 'GROUP_2' ? 30 : g.id === 'GROUP_3' ? 120 : g.id === 'GROUP_4' ? 240 : 400
          )
        "
      >
        <div class="step-badge" :class="g.badgeClass">
          {{ g.id.replace('GROUP_', 'N') }}
        </div>
        <div class="step-info">
          <span class="step-name">{{ g.name.split('—')[1] }}</span>
          <span class="step-meta">Quá hạn: {{ g.overdueText }} · DPCT: {{ g.specRateText }}</span>
        </div>
      </div>
    </div>

    <!-- Provisioning Calculation Matrix -->
    <div class="provision-summary-grid">
      <div class="provision-item">
        <span class="lbl">Dư Nợ Vay Đang Đánh Giá</span>
        <span class="val font-mono">{{ formatVnd(store.financialInput.outstandingLoan) }}</span>
      </div>
      <div class="provision-item">
        <span class="lbl">Số Ngày Quá Hạn Thực Tế</span>
        <div class="overdue-input-wrap">
          <input
            v-model.number="store.financialInput.daysOverdue"
            type="number"
            min="0"
            max="720"
            class="days-input"
          />
          <span class="days-unit">ngày</span>
        </div>
      </div>
      <div class="provision-item">
        <span class="lbl">Dự Phòng Chung (0.75%)</span>
        <span class="val text-amber font-mono">{{ formatVnd(store.debtClassification.requiredGeneral) }}</span>
      </div>
      <div class="provision-item">
        <span class="lbl">Dự Phòng Cụ Thể ({{ (store.debtClassification.specificRateBps / 100).toFixed(1) }}%)</span>
        <span class="val text-red font-mono">{{ formatVnd(store.debtClassification.requiredSpecific) }}</span>
      </div>
      <div class="provision-item total-item">
        <span class="lbl">Tổng Chi Phí Dự Phòng Rủi Ro</span>
        <span class="val text-red font-mono total-val">{{ formatVnd(store.debtClassification.totalProvision) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.debt-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.debt-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
}

.header-tag-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.p-tag {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 800;
  background: #eff6ff;
  color: #2563eb;
  padding: 2px 6px;
  border-radius: 4px;
}

.debt-card-title {
  font-size: 15px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.debt-card-subtitle {
  font-size: 12px;
  color: #64748b;
  margin: 0;
}

.current-group-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 700;
}

.current-group-badge.group_1 {
  background: #dcfce7;
  color: #15803d;
  border: 1px solid #86efac;
}

.current-group-badge.group_2 {
  background: #fef3c7;
  color: #b45309;
  border: 1px solid #fcd34d;
}

.current-group-badge.group_3,
.current-group-badge.group_4,
.current-group-badge.group_5 {
  background: #fee2e2;
  color: #b91c1c;
  border: 1px solid #fca5a5;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
}

.groups-stepper {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 10px;
}

.step-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.step-item:hover {
  background: #f1f5f9;
}

.step-item.active {
  background: #eff6ff;
  border-color: #3b82f6;
  box-shadow: 0 0 0 1px #3b82f6;
}

.step-badge {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'JetBrains Mono', monospace;
  font-weight: 800;
  font-size: 13px;
  flex-shrink: 0;
}

.badge-g1 {
  background: #dcfce7;
  color: #15803d;
}

.badge-g2 {
  background: #fef3c7;
  color: #b45309;
}

.badge-g3 {
  background: #fed7aa;
  color: #c2410c;
}

.badge-g4 {
  background: #fecaca;
  color: #b91c1c;
}

.badge-g5 {
  background: #f3e8ff;
  color: #7e22ce;
}

.step-info {
  display: flex;
  flex-direction: column;
}

.step-name {
  font-size: 12px;
  font-weight: 700;
  color: #1e293b;
}

.step-meta {
  font-size: 10px;
  color: #64748b;
}

.provision-summary-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 14px;
}

.provision-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.provision-item .lbl {
  font-size: 11px;
  color: #64748b;
}

.provision-item .val {
  font-size: 14px;
  font-weight: 700;
  color: #0f172a;
}

.overdue-input-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
}

.days-input {
  width: 70px;
  padding: 4px 8px;
  border: 1px solid #cbd5e1;
  border-radius: 4px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  font-weight: 700;
  color: #0f172a;
}

.days-unit {
  font-size: 12px;
  color: #64748b;
}

.font-mono {
  font-family: 'JetBrains Mono', monospace;
}

.text-amber {
  color: #d97706;
}

.text-red {
  color: #dc2626;
}

.total-item {
  grid-column: span 1;
}

.total-val {
  font-size: 16px !important;
}
</style>
