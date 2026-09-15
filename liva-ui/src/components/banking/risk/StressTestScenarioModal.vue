<script setup lang="ts">
/**
 * StressTestScenarioModal.vue — P74 Thử Nghiệm Ứng Kích Dòng Tiền (Stress Testing)
 * ==============================================================================
 * Mô phỏng 3 kịch bản chịu đựng rủi ro: Base, Moderate Stress, Severe Stress
 * và đánh giá sự suy thoái chỉ số DSCR, đệm dòng tiền an toàn.
 */
import { useRiskStore, type StressScenarioKey } from '../../../stores/riskStore';

defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const store = useRiskStore();

function formatVnd(val: number): string {
  return `${val.toLocaleString('vi-VN')} ₫`;
}

const scenarioList: {
  key: StressScenarioKey;
  title: string;
  desc: string;
  tag: string;
}[] = [
  {
    key: 'BASE',
    title: 'Kịch Bản Cơ Sở (Base Case)',
    desc: 'Hoạt động kinh doanh và doanh thu diễn ra theo kế hoạch bình thường.',
    tag: 'BÌNH THƯỜNG',
  },
  {
    key: 'MODERATE',
    title: 'Căng Thẳng Vừa Phải (Moderate Stress)',
    desc: 'Doanh thu suy giảm -15%, chậm thu hồi nợ 30 ngày, lãi suất vay thả nổi tăng +150 bps.',
    tag: 'ÁP LỰC TRUNG BÌNH',
  },
  {
    key: 'SEVERE',
    title: 'Căng Thẳng Nghiêm Trọng (Severe Stress)',
    desc: 'Doanh thu suy giảm -30%, chậm thu nợ 60 ngày, lãi suất tăng +300 bps, phát sinh CAPEX +10%.',
    tag: 'ÁP LỰC CAO',
  },
];
</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="emit('close')">
    <div class="modal-container">
      <div class="modal-header">
        <div class="header-branding">
          <span class="p-pill">P74 STRESS TEST</span>
          <h3 class="modal-title">Thử Nghiệm Ứng Kích Khả Năng Trả Nợ Vay (Stress Testing Matrix)</h3>
        </div>
        <button class="btn-close" @click="emit('close')">✕</button>
      </div>

      <div class="modal-body">
        <!-- Scenario Switcher Buttons -->
        <div class="scenario-selector">
          <button
            v-for="s in scenarioList"
            :key="s.key"
            class="scenario-card-btn"
            :class="{ active: store.activeStressScenario === s.key }"
            @click="store.setStressScenario(s.key)"
          >
            <div class="btn-top">
              <span class="s-tag">{{ s.tag }}</span>
              <span v-if="store.stressScenarios[s.key].passes" class="s-badge pass">VƯỢT QUA</span>
              <span v-else class="s-badge fail">KHÔNG ĐẠT</span>
            </div>
            <div class="s-title">{{ s.title }}</div>
            <div class="s-desc">{{ s.desc }}</div>
            <div class="s-metric">
              <span class="m-lbl">DSCR mô phỏng:</span>
              <span
                class="m-val"
                :class="store.stressScenarios[s.key].passes ? 'text-green' : 'text-red'"
              >
                {{ store.stressScenarios[s.key].stressedDscr.toFixed(2) }}x
              </span>
            </div>
          </button>
        </div>

        <!-- Comparative Matrix Table -->
        <div class="comparison-table-wrap">
          <h4 class="table-heading">Bảng So Sánh Chỉ Tiêu Giữa Các Kịch Bản</h4>
          <table class="matrix-table">
            <thead>
              <tr>
                <th>Chỉ Tiêu Đánh Giá</th>
                <th>Cơ Sở (Base)</th>
                <th>Căng Thẳng Vừa (Moderate)</th>
                <th>Căng Thẳng Nặng (Severe)</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td class="row-label">Dòng tiền thuần (NOI = EBITDA - CAPEX)</td>
                <td class="font-mono">{{ formatVnd(store.stressScenarios.BASE.stressedNoi) }}</td>
                <td class="font-mono">{{ formatVnd(store.stressScenarios.MODERATE.stressedNoi) }}</td>
                <td class="font-mono text-amber">{{ formatVnd(store.stressScenarios.SEVERE.stressedNoi) }}</td>
              </tr>
              <tr>
                <td class="row-label">Tổng nghĩa vụ trả nợ (Gốc + Lãi)</td>
                <td class="font-mono">{{ formatVnd(store.stressScenarios.BASE.stressedDebtService) }}</td>
                <td class="font-mono">{{ formatVnd(store.stressScenarios.MODERATE.stressedDebtService) }}</td>
                <td class="font-mono text-red">{{ formatVnd(store.stressScenarios.SEVERE.stressedDebtService) }}</td>
              </tr>
              <tr>
                <td class="row-label">Hệ số phủ nợ vay (DSCR)</td>
                <td class="font-mono bold text-green">{{ store.stressScenarios.BASE.stressedDscr.toFixed(2) }}x</td>
                <td
                  class="font-mono bold"
                  :class="store.stressScenarios.MODERATE.passes ? 'text-green' : 'text-amber'"
                >
                  {{ store.stressScenarios.MODERATE.stressedDscr.toFixed(2) }}x
                </td>
                <td
                  class="font-mono bold"
                  :class="store.stressScenarios.SEVERE.passes ? 'text-green' : 'text-red'"
                >
                  {{ store.stressScenarios.SEVERE.stressedDscr.toFixed(2) }}x
                </td>
              </tr>
              <tr>
                <td class="row-label">Đệm an toàn dòng tiền (NOI - Nợ)</td>
                <td class="font-mono text-green">{{ formatVnd(store.stressScenarios.BASE.buffer) }}</td>
                <td
                  class="font-mono"
                  :class="store.stressScenarios.MODERATE.buffer >= 0 ? 'text-green' : 'text-red'"
                >
                  {{ formatVnd(store.stressScenarios.MODERATE.buffer) }}
                </td>
                <td
                  class="font-mono"
                  :class="store.stressScenarios.SEVERE.buffer >= 0 ? 'text-green' : 'text-red'"
                >
                  {{ formatVnd(store.stressScenarios.SEVERE.buffer) }}
                </td>
              </tr>
              <tr>
                <td class="row-label">Trạng thái chịu đựng rủi ro</td>
                <td><span class="status-chip safe">AN TOÀN</span></td>
                <td>
                  <span
                    class="status-chip"
                    :class="store.stressScenarios.MODERATE.passes ? 'safe' : 'warning'"
                  >
                    {{ store.stressScenarios.MODERATE.passes ? 'ĐẠT CHUẨN' : 'ÁP LỰC' }}
                  </span>
                </td>
                <td>
                  <span
                    class="status-chip"
                    :class="store.stressScenarios.SEVERE.passes ? 'safe' : 'critical'"
                  >
                    {{ store.stressScenarios.SEVERE.passes ? 'CHỊU ĐƯỢC' : 'THÂM HỤT' }}
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="modal-footer">
        <div class="footer-note">
          <span>Khuyến nghị thẩm định: </span>
          <strong>{{ store.underwritingRecommendation.rationale }}</strong>
        </div>
        <button class="btn btn-secondary" @click="emit('close')">Đóng</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.65);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1200;
  padding: 20px;
}

.modal-container {
  background: #ffffff;
  border-radius: 12px;
  width: 100%;
  max-width: 860px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.2);
  overflow: hidden;
}

.modal-header {
  padding: 16px 20px;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-branding {
  display: flex;
  align-items: center;
  gap: 10px;
}

.p-pill {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 800;
  background: #eff6ff;
  color: #2563eb;
  padding: 2px 8px;
  border-radius: 4px;
}

.modal-title {
  font-size: 15px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.btn-close {
  background: transparent;
  border: none;
  font-size: 16px;
  color: #64748b;
  cursor: pointer;
}

.modal-body {
  padding: 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.scenario-selector {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.scenario-card-btn {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 14px;
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.scenario-card-btn:hover {
  background: #f1f5f9;
}

.scenario-card-btn.active {
  background: #eff6ff;
  border-color: #3b82f6;
  box-shadow: 0 0 0 1px #3b82f6;
}

.btn-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.s-tag {
  font-size: 10px;
  font-weight: 700;
  color: #64748b;
  text-transform: uppercase;
}

.s-badge {
  font-size: 9px;
  font-weight: 800;
  padding: 2px 6px;
  border-radius: 4px;
}

.s-badge.pass {
  background: #dcfce7;
  color: #15803d;
}

.s-badge.fail {
  background: #fee2e2;
  color: #b91c1c;
}

.s-title {
  font-size: 13px;
  font-weight: 700;
  color: #0f172a;
}

.s-desc {
  font-size: 11px;
  color: #64748b;
  line-height: 1.4;
  flex: 1;
}

.s-metric {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  border-top: 1px solid rgba(0, 0, 0, 0.05);
  padding-top: 8px;
}

.m-lbl {
  font-size: 11px;
  color: #64748b;
}

.m-val {
  font-family: 'JetBrains Mono', monospace;
  font-size: 16px;
  font-weight: 800;
}

.comparison-table-wrap {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.table-heading {
  font-size: 13px;
  font-weight: 700;
  color: #334155;
  margin: 0;
}

.matrix-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}

.matrix-table th,
.matrix-table td {
  padding: 10px 12px;
  border: 1px solid #e2e8f0;
  text-align: right;
}

.matrix-table th {
  background: #f8fafc;
  font-weight: 700;
  color: #475569;
}

.matrix-table th:first-child,
.matrix-table td:first-child {
  text-align: left;
}

.row-label {
  font-weight: 600;
  color: #334155;
}

.font-mono {
  font-family: 'JetBrains Mono', monospace;
}

.bold {
  font-weight: 800;
}

.text-green {
  color: #10b981;
}

.text-amber {
  color: #d97706;
}

.text-red {
  color: #dc2626;
}

.status-chip {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 10px;
}

.status-chip.safe {
  background: #dcfce7;
  color: #15803d;
}

.status-chip.warning {
  background: #fef3c7;
  color: #b45309;
}

.status-chip.critical {
  background: #fee2e2;
  color: #b91c1c;
}

.modal-footer {
  padding: 14px 20px;
  border-top: 1px solid #e2e8f0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #f8fafc;
}

.footer-note {
  font-size: 12px;
  color: #475569;
  max-width: 75%;
}

.footer-note strong {
  color: #0f172a;
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.btn-secondary {
  background: #ffffff;
  border: 1px solid #cbd5e1;
  color: #475569;
}

.btn-secondary:hover {
  background: #f1f5f9;
}
</style>
