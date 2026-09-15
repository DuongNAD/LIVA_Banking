<script setup lang="ts">
/**
 * ThresholdConfigTab.vue — P104 Tham Số Dung Sai & Quy Tắc Đối Soát
 * ===================================================================
 * Cấu hình các ngưỡng dung sai đối soát (Exact Match, Fee Variance),
 * độ sâu giải thuật phân rã tổ hợp con (Split Solver k <= 8), và TTL hàng đợi cách ly P44.
 */
import { ref } from 'vue';
import { useSettingsStore } from '../../../stores/settingsStore';

const store = useSettingsStore();

const feeVariance = ref(store.thresholds.maxFeeVarianceVnd);
const splitK = ref(store.thresholds.splitSolverMaxK);
const ttlSecs = ref(store.thresholds.quarantineTtlSecs);
const amlThreshold = ref(store.thresholds.amlThresholdVnd);
const saveFeedback = ref('');

function saveThresholds() {
  store.updateThresholds({
    maxFeeVarianceVnd: Number(feeVariance.value),
    splitSolverMaxK: Number(splitK.value),
    quarantineTtlSecs: Number(ttlSecs.value),
    amlThresholdVnd: Number(amlThreshold.value),
  });
  saveFeedback.value = 'Đã lưu tham số thành công! ✓';
  setTimeout(() => {
    saveFeedback.value = '';
  }, 2000);
}
</script>

<template>
  <div class="settings-card">
    <div class="card-header">
      <div>
        <div class="header-tags">
          <span class="p-tag">P104 THAM SỐ ĐỐI SOÁT</span>
          <span class="status-pill ok">Zero Float Drift</span>
        </div>
        <h3 class="card-title">Tham Số Dung Sai & Quy Tắc Đối Soát Tự Động</h3>
        <p class="card-desc">
          Điều chỉnh biên độ lọc nhiễu, tách phí giao dịch ngân hàng và kiểm soát tài nguyên giải thuật.
        </p>
      </div>
    </div>

    <!-- Form Controls -->
    <div class="form-grid">
      <!-- 1. Tier 1 -->
      <div class="form-group">
        <label class="form-label">
          <span>Ngưỡng Lệch Tuyệt Đối (Tier 1 Exact Match):</span>
          <span class="label-sub">Số tiền VND chênh lệch tối đa cho phép khớp 1:1</span>
        </label>
        <div class="input-wrap disabled">
          <input type="number" :value="store.thresholds.exactToleranceVnd" disabled class="form-input" />
          <span class="unit">VND (Bất biến)</span>
        </div>
      </div>

      <!-- 2. Tier 2 Fee Variance -->
      <div class="form-group">
        <label class="form-label">
          <span>Dung Sai Phí Ngân Hàng Tối Đa (Tier 2 Fuzzy Matching):</span>
          <span class="label-sub">Tự động hạch toán vào TK 6425 nếu độ lệch &le; mức này</span>
        </label>
        <div class="input-wrap">
          <input v-model.number="feeVariance" type="number" class="form-input" />
          <span class="unit">VND</span>
        </div>
      </div>

      <!-- 3. Tier 3 Split Solver Depth -->
      <div class="form-group">
        <label class="form-label">
          <span>Độ Sâu Giải Thuật Chia Tách (Tier 3 Split Solver Depth k):</span>
          <span class="label-sub">Số lượng hóa đơn tối đa trong tổ hợp con Subset-Sum (2 đến 12)</span>
        </label>
        <div class="slider-wrap">
          <input
            v-model.number="splitK"
            type="range"
            min="2"
            max="12"
            class="form-slider"
          />
          <span class="slider-val font-mono">k = {{ splitK }}</span>
        </div>
      </div>

      <!-- 4. P44 TTL -->
      <div class="form-group">
        <label class="form-label">
          <span>Thời Gian Hết Hạn Hàng Đợi Cách Ly (P44 Quarantine TTL):</span>
          <span class="label-sub">Thời gian đếm ngược trước khi trả giao dịch về hàng đợi chung</span>
        </label>
        <div class="input-wrap">
          <input v-model.number="ttlSecs" type="number" class="form-input" />
          <span class="unit">giây ({{ Math.round(ttlSecs / 60) }} phút)</span>
        </div>
      </div>

      <!-- 5. AML Threshold -->
      <div class="form-group">
        <label class="form-label">
          <span>Ngưỡng Rà Soát AML Giá Trị Lớn (QĐ 11/2023/QĐ-TTg):</span>
          <span class="label-sub">Giao dịch vượt mức này bắt buộc đưa vào diện kiểm tra tuân thủ</span>
        </label>
        <div class="input-wrap">
          <input v-model.number="amlThreshold" type="number" class="form-input" />
          <span class="unit">VND</span>
        </div>
      </div>
    </div>

    <!-- Actions -->
    <div class="form-footer">
      <span v-if="saveFeedback" class="save-msg">{{ saveFeedback }}</span>
      <button class="btn-save" @click="saveThresholds">
        Lưu Thay Đổi Tham Số
      </button>
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

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

@media (max-width: 900px) {
  .form-grid {
    grid-template-columns: 1fr;
  }
}

.form-group {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 10px;
}

.form-label {
  display: flex;
  flex-direction: column;
  gap: 3px;
  font-size: 13px;
  font-weight: 600;
  color: #e2e8f0;
}

.label-sub {
  font-size: 11px;
  color: #64748b;
  font-weight: 400;
}

.input-wrap {
  display: flex;
  align-items: center;
  background: #090d16;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  padding: 0 10px;
}

.input-wrap.disabled {
  opacity: 0.6;
  background: rgba(255, 255, 255, 0.02);
}

.form-input {
  background: transparent;
  border: none;
  color: #f1f5f9;
  font-size: 14px;
  padding: 8px 0;
  width: 100%;
  outline: none;
  font-family: monospace;
}

.unit {
  font-size: 12px;
  color: #64748b;
  white-space: nowrap;
}

.slider-wrap {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
}

.form-slider {
  flex: 1;
  accent-color: #38bdf8;
  cursor: pointer;
}

.slider-val {
  font-size: 14px;
  font-weight: 700;
  color: #38bdf8;
  min-width: 50px;
}

.form-footer {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  padding-top: 12px;
}

.save-msg {
  color: #10b981;
  font-size: 12px;
  font-weight: 600;
}

.btn-save {
  background: #2563eb;
  border: 1px solid #3b82f6;
  color: #fff;
  padding: 8px 20px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-save:hover {
  background: #1d4ed8;
}
</style>
