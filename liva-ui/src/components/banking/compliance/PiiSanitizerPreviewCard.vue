<script setup lang="ts">
/**
 * PiiSanitizerPreviewCard.vue — P80 Bộ Lọc Ẩn Danh Hóa Dữ Liệu Cá Nhân (Decree 13 PII)
 * ==================================================================================
 * Trực quan hóa cơ chế che giấu dữ liệu định danh cá nhân nhạy cảm (CCCD, SĐT, STK)
 * và bảo toàn tuyệt đối tên pháp nhân doanh nghiệp để phục vụ đối soát công nợ.
 */
import { useComplianceStore } from '../../../stores/complianceStore';

const store = useComplianceStore();
</script>

<template>
  <div class="pii-card">
    <div class="card-header">
      <div>
        <div class="tag-wrap">
          <span class="p-tag">P80 NGHỊ ĐỊNH 13/2023</span>
          <h3 class="card-title">Bộ Lọc Ẩn Danh Hóa Dữ Liệu Cá Nhân (PII Sanitizer)</h3>
        </div>
        <p class="card-desc">
          Bảo vệ quyền riêng tư cá nhân theo Nghị định 13/2023/NĐ-CP trong toàn bộ quy trình đối soát tự động.
        </p>
      </div>

      <div class="toggle-wrap">
        <label class="toggle-label">
          <input
            type="checkbox"
            :checked="store.isPiiRedactionEnabled"
            @change="store.togglePiiRedaction()"
          />
          <span class="toggle-slider" />
        </label>
        <span class="status-lbl">
          {{ store.isPiiRedactionEnabled ? 'Bộ Lọc Đang BẬT' : 'Bộ Lọc Đang TẮT' }}
        </span>
      </div>
    </div>

    <!-- Side-by-side Diff View -->
    <div class="diff-grid">
      <!-- Raw Input -->
      <div class="text-box raw">
        <div class="box-header">
          <span class="box-title">Dữ liệu thô từ sao kê ngân hàng (Raw Statement Narration):</span>
        </div>
        <textarea
          v-model="store.piiSampleInput"
          rows="4"
          class="raw-textarea"
          placeholder="Nhập chuỗi diễn giải giao dịch cần kiểm tra..."
        />
      </div>

      <!-- Sanitized Output -->
      <div class="text-box sanitized">
        <div class="box-header">
          <span class="box-title">Dữ liệu sau khi qua bộ lọc Nghị định 13 (Sanitized Output):</span>
          <span class="shield-badge">🛡️ Zero Cloud Leakage</span>
        </div>
        <div class="sanitized-content">
          {{ store.sanitizedPiiPreview }}
        </div>
      </div>
    </div>

    <!-- Preservation Rules Footnote -->
    <div class="rules-footnote">
      <span class="fn-icon">💡</span>
      <span class="fn-text">
        <strong>Nguyên tắc bảo toàn pháp nhân:</strong> Các pháp nhân có tiền tố <code>CONG TY</code>, <code>TNHH</code>, <code>CO PHAN</code>, <code>NGAN HANG</code> được giữ nguyên 100% để bảo đảm tính hợp pháp khi hạch toán hóa đơn điện tử và chứng từ kế toán.
      </span>
    </div>
  </div>
</template>

<style scoped>
.pii-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 18px 22px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
}

.tag-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 2px;
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

.card-title {
  font-size: 14px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.card-desc {
  font-size: 11px;
  color: #64748b;
  margin: 0;
}

.toggle-wrap {
  display: flex;
  align-items: center;
  gap: 10px;
}

.toggle-label {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 22px;
}

.toggle-label input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background-color: #cbd5e1;
  border-radius: 22px;
  transition: 0.2s;
}

.toggle-slider:before {
  position: absolute;
  content: '';
  height: 16px;
  width: 16px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  border-radius: 50%;
  transition: 0.2s;
}

input:checked + .toggle-slider {
  background-color: #10b981;
}

input:checked + .toggle-slider:before {
  transform: translateX(18px);
}

.status-lbl {
  font-size: 12px;
  font-weight: 600;
  color: #334155;
}

.diff-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 14px;
}

@media (max-width: 860px) {
  .diff-grid {
    grid-template-columns: 1fr;
  }
}

.text-box {
  border-radius: 8px;
  border: 1px solid #e2e8f0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.box-header {
  padding: 8px 12px;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.box-title {
  font-size: 11px;
  font-weight: 700;
  color: #475569;
}

.shield-badge {
  font-size: 10px;
  font-weight: 700;
  color: #15803d;
  background: #dcfce7;
  padding: 2px 6px;
  border-radius: 4px;
}

.raw-textarea {
  width: 100%;
  padding: 10px 12px;
  border: none;
  font-size: 12px;
  line-height: 1.5;
  color: #1e293b;
  resize: vertical;
  box-sizing: border-box;
}

.raw-textarea:focus {
  outline: none;
}

.sanitized-content {
  padding: 10px 12px;
  font-size: 12px;
  line-height: 1.5;
  color: #0f172a;
  background: #fafafa;
  min-height: 80px;
}

.rules-footnote {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  background: #f0fdf4;
  border: 1px solid #bbf7d0;
  border-radius: 8px;
  padding: 10px 14px;
}

.fn-icon {
  font-size: 16px;
}

.fn-text {
  font-size: 11px;
  color: #166534;
  line-height: 1.4;
}

.fn-text code {
  font-family: 'JetBrains Mono', monospace;
  background: #dcfce7;
  padding: 1px 4px;
  border-radius: 3px;
}
</style>
