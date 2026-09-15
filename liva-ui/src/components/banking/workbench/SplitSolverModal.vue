<script setup lang="ts">
import { computed } from 'vue';
import { useWorkbenchStore, type GlRecord } from '../../../stores/workbenchStore';

const store = useWorkbenchStore();

const target = computed(() => store.splitSolverTarget);

const solverResults = computed(() => {
  if (!target.value) return null;
  return store.solveSubsetSum(target.value, 8);
});

function formatVnd(val: number): string {
  const formatted = new Intl.NumberFormat('vi-VN').format(Math.abs(val)) + ' ₫';
  return val < 0 ? `-${formatted}` : `+${formatted}`;
}

function handleApply(gls: GlRecord[]) {
  if (!target.value) return;
  store.applySplitSolution(target.value, gls);
  closeModal();
}

function closeModal() {
  store.splitSolverTarget = null;
}
</script>

<template>
  <div v-if="target" class="modal-backdrop" @click.self="closeModal">
    <div class="modal-dialog split-modal">
      <!-- Header -->
      <div class="modal-header">
        <div class="header-left">
          <span class="modal-badge">P45 SOLVER</span>
          <h3 class="modal-title">Split Solver (Tách gộp 1:N / N:1)</h3>
        </div>
        <button class="btn-close" @click="closeModal">✕</button>
      </div>

      <!-- Target Summary Box -->
      <div class="target-box">
        <div class="target-title">GIAO DỊCH SAO KÊ CẦN TÌM TỔ HỢP HÓA ĐƠN ĐỐI ỨNG</div>
        <div class="target-row">
          <div>
            <span class="tx-code">{{ target.txCode }}</span>
            <span class="bank-pill" :class="target.bankCode.toLowerCase()">{{ target.bankCode }}</span>
            <div class="target-memo">{{ target.memo }}</div>
          </div>
          <div class="target-amount">
            {{ formatVnd(target.amount) }}
          </div>
        </div>
      </div>

      <!-- Solver Candidate Solutions -->
      <div class="modal-body">
        <div class="body-intro">
          <span class="intro-icon">🧮</span>
          <span>
            Thuật toán tìm kiếm tổ hợp tập con (Subset-Sum bounded $k \le 8$) đã quét các chứng từ sổ cái chưa đối soát:
          </span>
        </div>

        <div v-if="!solverResults || solverResults.combinations.length === 0" class="no-solution-box">
          <p>Không tìm thấy tổ hợp hóa đơn nào có tổng chính xác {{ formatVnd(target.amount) }}.</p>
          <p class="sub-hint">Bạn có thể chọn thủ công các dòng bên Cột 1 và Cột 2 rồi bấm "Ghép Cặp".</p>
        </div>

        <div v-else class="solution-list">
          <div
            v-for="(sol, idx) in solverResults.combinations"
            :key="idx"
            class="solution-card"
          >
            <div class="solution-top">
              <div class="solution-tag">
                Phương án {{ idx + 1 }} · Khớp 1:{{ sol.glRecords.length }}
                <span class="confidence-tag">{{ Math.round(sol.confidenceScore * 100) }}% tin cậy</span>
              </div>
              <div class="solution-amount">
                Tổng: {{ formatVnd(sol.totalAmount) }}
              </div>
            </div>

            <!-- List of GL Invoices in this solution -->
            <div class="gl-breakdown">
              <div v-for="g in sol.glRecords" :key="g.id" class="gl-row">
                <span class="gl-code">{{ g.voucherNumber }} ({{ g.erpSource }})</span>
                <span class="gl-partner">{{ g.partnerName }}</span>
                <span class="gl-amount">{{ formatVnd(g.amount) }}</span>
              </div>
            </div>

            <!-- Footer Action -->
            <div class="solution-footer">
              <span class="zero-diff-check">✓ Chênh lệch: 0 ₫ (Toán học chính xác 100%)</span>
              <button class="btn-apply-solution" @click="handleApply(sol.glRecords)">
                Áp Dụng Phương Án Này
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.65);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.modal-dialog {
  background: #ffffff;
  border-radius: 12px;
  width: 90%;
  max-width: 680px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.2);
}

.modal-header {
  padding: 16px 20px;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.modal-badge {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  background: #f3e8ff;
  color: #7e22ce;
  border-radius: 4px;
}

.modal-title {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.btn-close {
  background: none;
  border: none;
  font-size: 16px;
  color: #94a3b8;
  cursor: pointer;
}

.btn-close:hover {
  color: #0f172a;
}

.target-box {
  padding: 14px 20px;
  background: #eff6ff;
  border-bottom: 1px solid #dbeafe;
}

.target-title {
  font-size: 10px;
  font-weight: 700;
  color: #1d4ed8;
  letter-spacing: 0.5px;
  margin-bottom: 6px;
}

.target-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.tx-code {
  font-size: 13px;
  font-weight: 700;
  font-family: monospace;
  color: #0f172a;
  margin-right: 6px;
}

.bank-pill {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
}

.bank-pill.vcb { background: #dcfce7; color: #15803d; }
.bank-pill.tcb { background: #fee2e2; color: #b91c1c; }
.bank-pill.bidv { background: #dbeafe; color: #1d4ed8; }

.target-memo {
  font-size: 12px;
  color: #334155;
  margin-top: 4px;
}

.target-amount {
  font-size: 18px;
  font-weight: 800;
  color: #16a34a;
  font-family: monospace;
}

.modal-body {
  padding: 16px 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.body-intro {
  font-size: 12px;
  color: #475569;
  display: flex;
  align-items: center;
  gap: 8px;
}

.intro-icon {
  font-size: 16px;
}

.no-solution-box {
  padding: 30px 20px;
  background: #f8fafc;
  border: 1px dashed #cbd5e1;
  border-radius: 8px;
  text-align: center;
  color: #64748b;
  font-size: 13px;
}

.sub-hint {
  font-size: 11px;
  color: #94a3b8;
  margin-top: 6px;
}

.solution-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.solution-card {
  padding: 14px;
  background: #ffffff;
  border: 1px solid #c4b5fd;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  box-shadow: 0 2px 6px rgba(126, 34, 206, 0.06);
}

.solution-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.solution-tag {
  font-size: 12px;
  font-weight: 700;
  color: #6d28d9;
  display: flex;
  align-items: center;
  gap: 8px;
}

.confidence-tag {
  font-size: 10px;
  padding: 1px 6px;
  background: #ede9fe;
  color: #7c3aed;
  border-radius: 4px;
}

.solution-amount {
  font-size: 14px;
  font-weight: 700;
  font-family: monospace;
  color: #0f172a;
}

.gl-breakdown {
  background: #f8fafc;
  border-radius: 6px;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.gl-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.gl-code {
  font-weight: 700;
  font-family: monospace;
  color: #334155;
}

.gl-partner {
  color: #64748b;
  flex: 1;
  margin: 0 10px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.gl-amount {
  font-weight: 700;
  font-family: monospace;
  color: #0f172a;
}

.solution-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 6px;
  border-top: 1px solid #f1f5f9;
}

.zero-diff-check {
  font-size: 11px;
  color: #16a34a;
  font-weight: 600;
}

.btn-apply-solution {
  padding: 6px 14px;
  font-size: 12px;
  font-weight: 700;
  background: #7c3aed;
  color: #ffffff;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  transition: background 0.15s;
}

.btn-apply-solution:hover {
  background: #6d28d9;
}
</style>
