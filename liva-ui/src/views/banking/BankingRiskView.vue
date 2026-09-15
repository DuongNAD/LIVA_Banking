<script setup lang="ts">
/**
 * BankingRiskView.vue — Màn hình Thẩm Định Rủi Ro & Tín Dụng Doanh Nghiệp (P70–P75)
 * ===================================================================================
 * Định lượng chỉ số DSCR, tỷ số thanh toán nhanh/hiện hành/tức thời, phân loại nợ
 * theo Thông tư 11/2021/TT-NHNN và thử nghiệm ứng kích dòng tiền (Stress Testing).
 */
import { ref } from 'vue';
import { useRiskStore } from '../../stores/riskStore';
import SolvencyRatioGrid from '../../components/banking/risk/SolvencyRatioGrid.vue';
import DebtClassificationCard from '../../components/banking/risk/DebtClassificationCard.vue';
import StressTestScenarioModal from '../../components/banking/risk/StressTestScenarioModal.vue';

const store = useRiskStore();
const isStressModalOpen = ref(false);
const isEditingFinancials = ref(false);

function formatVnd(val: number): string {
  return `${val.toLocaleString('vi-VN')} ₫`;
}
</script>

<template>
  <div class="banking-view-container">
    <!-- Corporate Profile & Underwriting Header Banner -->
    <div class="risk-header-banner">
      <div class="header-left">
        <div class="title-with-pill">
          <span class="p-tag">P70–P75</span>
          <h2 class="banner-title">Thẩm Định Rủi Ro Tín Dụng & Khả Năng Trả Nợ Doanh Nghiệp</h2>
        </div>
        <p class="banner-desc">
          Doanh nghiệp: <strong>{{ store.applicantName }}</strong> · MST: <code>{{ store.taxCode }}</code> ·
          Nhu cầu: <strong>{{ store.creditFacilityType }}</strong>
        </p>
      </div>

      <div class="header-right">
        <!-- Decision Chip -->
        <div class="decision-box" :class="store.underwritingRecommendation.decision.toLowerCase()">
          <span class="d-lbl">Khuyến nghị cấp tín dụng:</span>
          <span class="d-val">
            {{
              store.underwritingRecommendation.decision === 'APPROVED'
                ? '✅ CHẤP THUẬN'
                : store.underwritingRecommendation.decision === 'CONDITIONAL'
                ? '⚠️ CÓ ĐIỀU KIỆN'
                : '⛔ TỪ CHỐI'
            }}
          </span>
          <span class="d-limit">
            Hạn mức tối đa: <strong>{{ formatVnd(store.underwritingRecommendation.recommendedLimit) }}</strong>
          </span>
        </div>

        <div class="action-btn-group">
          <button class="btn btn-stress" @click="isStressModalOpen = true">
            ⚡ Thử Nghiệm Ứng Kích (Stress Test)
          </button>
          <button
            class="btn btn-toggle-edit"
            :class="{ active: isEditingFinancials }"
            @click="isEditingFinancials = !isEditingFinancials"
          >
            {{ isEditingFinancials ? 'Đóng Biểu Nhập' : '📝 Tùy Biến BCTC' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Collapsible Financial Statement Inputs Form -->
    <div v-if="isEditingFinancials" class="financial-inputs-panel">
      <div class="panel-header">
        <span class="panel-title">Tham Số Báo Cáo Tài Chính & Dư Nợ (Đơn vị: VNĐ)</span>
        <span class="panel-sub">Điều chỉnh số liệu để kiểm tra lại các tỷ số tự động</span>
      </div>
      <div class="inputs-grid">
        <div class="input-item">
          <label>EBITDA (Lợi nhuận trước thuế, lãi vay & KH)</label>
          <input v-model.number="store.financialInput.ebitda" type="number" step="10000000" />
        </div>
        <div class="input-item">
          <label>Chi phí vốn đầu tư (CAPEX)</label>
          <input v-model.number="store.financialInput.capex" type="number" step="10000000" />
        </div>
        <div class="input-item">
          <label>Nghĩa vụ trả gốc vay hàng năm</label>
          <input v-model.number="store.financialInput.debtServicePrincipal" type="number" step="10000000" />
        </div>
        <div class="input-item">
          <label>Nghĩa vụ trả lãi vay hàng năm</label>
          <input v-model.number="store.financialInput.debtServiceInterest" type="number" step="5000000" />
        </div>
        <div class="input-item">
          <label>Tiền mặt & Tương đương tiền</label>
          <input v-model.number="store.financialInput.cashAndEquivalents" type="number" step="10000000" />
        </div>
        <div class="input-item">
          <label>Chứng khoán kinh doanh / tiền gửi</label>
          <input v-model.number="store.financialInput.marketableSecurities" type="number" step="10000000" />
        </div>
        <div class="input-item">
          <label>Phải thu ngắn hạn khách hàng</label>
          <input v-model.number="store.financialInput.accountsReceivable" type="number" step="10000000" />
        </div>
        <div class="input-item">
          <label>Hàng tồn kho</label>
          <input v-model.number="store.financialInput.inventory" type="number" step="10000000" />
        </div>
        <div class="input-item">
          <label>Nợ ngắn hạn (Phải trả người bán & vay NH)</label>
          <input v-model.number="store.financialInput.currentLiabilities" type="number" step="10000000" />
        </div>
        <div class="input-item">
          <label>Dư nợ vay hiện hữu tại hệ thống</label>
          <input v-model.number="store.financialInput.outstandingLoan" type="number" step="50000000" />
        </div>
      </div>
    </div>

    <!-- P71 Solvency & Liquidity Ratios Grid -->
    <div class="section-title">Hệ Thống Chỉ Số Khả Năng Trả Nợ & Thanh Khoản (P71 Solvency Ratios)</div>
    <SolvencyRatioGrid />

    <!-- P72 Circular 11 Debt Classification Card -->
    <div class="section-title">Giám Sát Nhóm Nợ & Dự Phòng Rủi Ro (P72 TT 11/2021/TT-NHNN)</div>
    <DebtClassificationCard />

    <!-- P74 Stress Testing Modal -->
    <StressTestScenarioModal
      :is-open="isStressModalOpen"
      @close="isStressModalOpen = false"
    />
  </div>
</template>

<style scoped>
.banking-view-container {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 20px 24px 40px 24px;
  background-color: #f8fafc;
  min-height: 100%;
}

.risk-header-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 18px 24px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
  flex-wrap: wrap;
  gap: 16px;
}

.title-with-pill {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 4px;
}

.p-tag {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 800;
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #bfdbfe;
  padding: 2px 6px;
  border-radius: 4px;
}

.banner-title {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.banner-desc {
  font-size: 13px;
  color: #64748b;
  margin: 0;
}

.banner-desc code {
  font-family: 'JetBrains Mono', monospace;
  background: #f1f5f9;
  padding: 1px 5px;
  border-radius: 4px;
  color: #334155;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-wrap: wrap;
}

.decision-box {
  display: flex;
  flex-direction: column;
  padding: 8px 14px;
  border-radius: 8px;
  gap: 2px;
}

.decision-box.approved {
  background: #dcfce7;
  border: 1px solid #86efac;
  color: #15803d;
}

.decision-box.conditional {
  background: #fef3c7;
  border: 1px solid #fcd34d;
  color: #b45309;
}

.decision-box.rejected {
  background: #fee2e2;
  border: 1px solid #fca5a5;
  color: #b91c1c;
}

.d-lbl {
  font-size: 10px;
  text-transform: uppercase;
  font-weight: 700;
  opacity: 0.8;
}

.d-val {
  font-size: 14px;
  font-weight: 800;
}

.d-limit {
  font-size: 11px;
}

.action-btn-group {
  display: flex;
  gap: 8px;
}

.btn {
  padding: 8px 14px;
  font-size: 12px;
  font-weight: 600;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-stress {
  background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
  color: #ffffff;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.btn-stress:hover {
  background: #334155;
  transform: translateY(-1px);
}

.btn-toggle-edit {
  background: #ffffff;
  border: 1px solid #cbd5e1;
  color: #475569;
}

.btn-toggle-edit:hover,
.btn-toggle-edit.active {
  background: #f1f5f9;
  border-color: #94a3b8;
}

.financial-inputs-panel {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
}

.panel-title {
  font-size: 13px;
  font-weight: 700;
  color: #0f172a;
}

.panel-sub {
  font-size: 11px;
  color: #64748b;
}

.inputs-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 12px;
}

.input-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.input-item label {
  font-size: 11px;
  color: #475569;
  font-weight: 600;
}

.input-item input {
  padding: 6px 10px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  font-weight: 700;
  color: #0f172a;
}

.input-item input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 1px #3b82f6;
}

.section-title {
  font-size: 13px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #475569;
}
</style>
