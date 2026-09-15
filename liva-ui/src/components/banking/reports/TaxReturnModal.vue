<script setup lang="ts">
/**
 * TaxReturnModal.vue — P94–P95 Tờ Khai Thuế GTGT (01/GTGT) & Quyết Toán Thuế TNDN (03/TNDN)
 * =========================================================================================
 * Chuẩn Thông tư 80/2021/TT-BTC gửi Tổng cục Thuế qua phần mềm Hỗ Trợ Kê Khai (HTKK).
 */
import { ref, computed } from 'vue';
import { useReportStore } from '../../../stores/reportStore';

defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const store = useReportStore();
const copyStatus = ref('Sao chép XML');

function formatVnd(val: number): string {
  return `${val.toLocaleString('vi-VN')} ₫`;
}

const xmlPayload = computed(() => {
  if (store.activeTaxModal === 'VAT') {
    return store.exportReportXml('VAT_01_GTGT');
  }
  if (store.activeTaxModal === 'CIT') {
    return store.exportReportXml('CIT_03_TNDN');
  }
  return '';
});

async function copyXml() {
  try {
    await navigator.clipboard.writeText(xmlPayload.value);
    copyStatus.value = 'Đã sao chép! ✓';
    setTimeout(() => {
      copyStatus.value = 'Sao chép XML';
    }, 2000);
  } catch {
    copyStatus.value = 'Lỗi sao chép';
  }
}

function downloadXml() {
  const blob = new Blob([xmlPayload.value], { type: 'application/xml;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  const filename = store.activeTaxModal === 'VAT'
    ? `ToKhai_01_GTGT_${store.currentPeriod}.xml`
    : `QuyetToan_03_TNDN_${store.citFinalization.taxYear}.xml`;
  link.setAttribute('href', url);
  link.setAttribute('download', filename);
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}
</script>

<template>
  <div v-if="isOpen" class="modal-backdrop" @click.self="emit('close')">
    <div class="modal-dialog">
      <!-- Header -->
      <div class="modal-header">
        <div class="modal-title-wrap">
          <span class="tt80-badge">THÔNG TƯ 80/2021/TT-BTC</span>
          <h3 v-if="store.activeTaxModal === 'VAT'" class="modal-title">
            Tờ Khai Thuế Giá Trị Gia Tăng (Mẫu 01/GTGT)
          </h3>
          <h3 v-else class="modal-title">
            Tờ Khai Quyết Toán Thuế TNDN (Mẫu 03/TNDN)
          </h3>
        </div>
        <button class="close-btn" @click="emit('close')">✕</button>
      </div>

      <!-- Body -->
      <div class="modal-body">
        <!-- VAT View -->
        <div v-if="store.activeTaxModal === 'VAT'" class="tax-grid">
          <div class="stat-card">
            <span class="stat-code">[25]</span>
            <div class="stat-info">
              <span class="stat-label">Thuế GTGT đầu vào được khấu trừ</span>
              <strong class="stat-val font-mono">{{ formatVnd(store.vatReturn.deductibleInputTax) }}</strong>
            </div>
          </div>
          <div class="stat-card">
            <span class="stat-code">[26] & [27]</span>
            <div class="stat-info">
              <span class="stat-label">Doanh số 8% & Thuế GTGT 8%</span>
              <strong class="stat-val font-mono text-accent">
                {{ formatVnd(store.vatReturn.taxableSales8pct) }} (Thuế: {{ formatVnd(store.vatReturn.outputTax8pct) }})
              </strong>
            </div>
          </div>
          <div class="stat-card">
            <span class="stat-code">[32] & [33]</span>
            <div class="stat-info">
              <span class="stat-label">Doanh số 10% & Thuế GTGT 10%</span>
              <strong class="stat-val font-mono text-accent">
                {{ formatVnd(store.vatReturn.taxableSales10pct) }} (Thuế: {{ formatVnd(store.vatReturn.outputTax10pct) }})
              </strong>
            </div>
          </div>
          <div class="stat-card">
            <span class="stat-code">[35]</span>
            <div class="stat-info">
              <span class="stat-label">Tổng thuế GTGT đầu ra phát sinh</span>
              <strong class="stat-val font-mono">{{ formatVnd(store.vatReturn.totalOutputTax) }}</strong>
            </div>
          </div>
          <div class="stat-card highlight">
            <span class="stat-code">[40a]</span>
            <div class="stat-info">
              <span class="stat-label">Thuế GTGT phải nộp trong kỳ</span>
              <strong class="stat-val font-mono text-gold">{{ formatVnd(store.vatReturn.netVatPayable) }}</strong>
            </div>
          </div>
          <div class="stat-card">
            <span class="stat-code">[43]</span>
            <div class="stat-info">
              <span class="stat-label">Thuế GTGT còn được khấu trừ chuyển kỳ sau</span>
              <strong class="stat-val font-mono">{{ formatVnd(store.vatReturn.carriedForwardTax) }}</strong>
            </div>
          </div>
        </div>

        <!-- CIT View -->
        <div v-else class="tax-grid">
          <div class="stat-card">
            <span class="stat-code">[A1]</span>
            <div class="stat-info">
              <span class="stat-label">Tổng lợi nhuận kế toán trước thuế (B02-DN)</span>
              <strong class="stat-val font-mono">{{ formatVnd(store.citFinalization.accountingPbt) }}</strong>
            </div>
          </div>
          <div class="stat-card">
            <span class="stat-code">[B4]</span>
            <div class="stat-info">
              <span class="stat-label">Các khoản chi không được trừ (chi phí không hợp lệ)</span>
              <strong class="stat-val font-mono text-neg">+ {{ formatVnd(store.citFinalization.nonDeductibleExpenses) }}</strong>
            </div>
          </div>
          <div class="stat-card">
            <span class="stat-code">[C4]</span>
            <div class="stat-info">
              <span class="stat-label">Thu nhập tính thuế TNDN (C4 = A1 + B4)</span>
              <strong class="stat-val font-mono text-accent">{{ formatVnd(store.citFinalization.taxableIncome) }}</strong>
            </div>
          </div>
          <div class="stat-card">
            <span class="stat-code">[C7] & [C8]</span>
            <div class="stat-info">
              <span class="stat-label">Thuế suất 20% & Tổng thuế TNDN phát sinh</span>
              <strong class="stat-val font-mono">{{ formatVnd(store.citFinalization.totalCitLiability) }}</strong>
            </div>
          </div>
          <div class="stat-card">
            <span class="stat-code">[E1]</span>
            <div class="stat-info">
              <span class="stat-label">Số thuế TNDN đã tạm nộp trong năm (4 quý)</span>
              <strong class="stat-val font-mono">- {{ formatVnd(store.citFinalization.provisionalTaxPaid) }}</strong>
            </div>
          </div>
          <div class="stat-card highlight">
            <span class="stat-code">[G]</span>
            <div class="stat-info">
              <span class="stat-label">Số thuế TNDN còn phải nộp vào NSNN</span>
              <strong class="stat-val font-mono text-gold">{{ formatVnd(store.citFinalization.remainingTaxPayable) }}</strong>
            </div>
          </div>
        </div>

        <!-- XML Preview for HTKK -->
        <div class="xml-section">
          <div class="xml-header">
            <span class="xml-title">Định dạng XML Hỗ Trợ Kê Khai (HTKK / e-Tax GDT)</span>
            <div class="xml-actions">
              <button class="action-btn" @click="copyXml">{{ copyStatus }}</button>
              <button class="action-btn primary" @click="downloadXml">Tải tệp .xml</button>
            </div>
          </div>
          <pre class="xml-preview"><code>{{ xmlPayload }}</code></pre>
        </div>
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <div class="footer-note">
          Dữ liệu đối soát tự động từ sổ cái kế toán và các giao dịch đã qua thẩm định Maker-Checker.
        </div>
        <button class="btn-close-modal" @click="emit('close')">Đóng</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 16px;
  backdrop-filter: blur(4px);
}

.modal-dialog {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  width: 100%;
  max-width: 760px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 18px 24px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.tt80-badge {
  font-size: 11px;
  font-weight: 700;
  color: #10b981;
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
  padding: 2px 8px;
  border-radius: 4px;
}

.modal-title {
  font-size: 17px;
  font-weight: 700;
  color: #f8fafc;
  margin: 6px 0 0 0;
}

.close-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 18px;
  cursor: pointer;
  padding: 4px;
}

.close-btn:hover {
  color: #fff;
}

.modal-body {
  padding: 20px 24px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.tax-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.stat-card {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 12px 14px;
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.stat-card.highlight {
  background: rgba(245, 158, 11, 0.08);
  border-color: rgba(245, 158, 11, 0.3);
}

.stat-code {
  font-size: 11px;
  font-weight: 700;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
}

.stat-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-label {
  font-size: 11px;
  color: #94a3b8;
}

.stat-val {
  font-size: 14px;
  color: #f1f5f9;
}

.xml-section {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 14px;
}

.xml-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.xml-title {
  font-size: 12px;
  font-weight: 600;
  color: #cbd5e1;
}

.xml-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #e2e8f0;
  padding: 4px 10px;
  border-radius: 5px;
  font-size: 11px;
  cursor: pointer;
}

.action-btn.primary {
  background: #3b82f6;
  border-color: #2563eb;
  color: #fff;
  font-weight: 600;
}

.xml-preview {
  margin: 0;
  max-height: 180px;
  overflow-y: auto;
  font-size: 11px;
  font-family: monospace;
  color: #a5f3fc;
  background: #090d16;
  padding: 10px;
  border-radius: 6px;
  white-space: pre-wrap;
  word-break: break-all;
}

.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 24px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.footer-note {
  font-size: 11px;
  color: #64748b;
  max-width: 500px;
}

.btn-close-modal {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #cbd5e1;
  padding: 6px 16px;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
}

.btn-close-modal:hover {
  background: rgba(255, 255, 255, 0.14);
  color: #fff;
}

.text-accent {
  color: #38bdf8;
}

.text-gold {
  color: #f59e0b;
}

.text-neg {
  color: #f87171;
}
</style>
