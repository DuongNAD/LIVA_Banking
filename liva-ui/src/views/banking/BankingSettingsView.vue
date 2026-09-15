<script setup lang="ts">
/**
 * BankingSettingsView.vue — Màn hình Cài đặt Cấu hình Ngân quỹ & Đối soát
 * Cấu hình các tham số ngưỡng an toàn thanh khoản, Maker-Checker, và liên kết đến Quản trị Hệ thống.
 */
import { ref } from 'vue';
import { useToast } from '../../composables/useToast';

const toast = useToast();

const minLiquidityThreshold = ref('500,000,000');
const autoReconcileTolerance = ref('11,000');
const enableMakerChecker = ref(true);
const enableZeroEgress = ref(true);
const autoPruneOldLogs = ref(false);

function saveSettings() {
  toast.success('Đã lưu cấu hình an toàn ngân quỹ thành công!', { title: 'Cài Đặt' });
}

function openSystemDashboard() {
  window.open('/dashboard.html', '_blank');
}
</script>

<template>
  <div class="banking-view-container">
    <!-- Header Banner -->
    <div class="settings-header-banner">
      <div>
        <h2 class="banner-title">Cài Đặt Cấu Hình Ngân Quỹ & Quy Tắc Đối Soát</h2>
        <p class="banner-desc">
          Thiết lập các ngưỡng kiểm soát rủi ro, chính sách Maker-Checker và bảo mật theo chuẩn Nghị định 13/2023/NĐ-CP.
        </p>
      </div>
      <button class="save-btn" @click="saveSettings">
        Lưu Thay Đổi
      </button>
    </div>

    <!-- Configuration Cards Grid -->
    <div class="settings-grid">
      <!-- Card 1: Treasury Risk Controls -->
      <div class="settings-card">
        <h3 class="card-title">1. Kiểm Soát Rủi Ro Ngân Quỹ (Treasury Safeguards)</h3>
        
        <div class="form-group">
          <label class="form-label">Ngưỡng đệm an toàn thanh khoản tối thiểu (VND)</label>
          <input
            v-model="minLiquidityThreshold"
            type="text"
            class="form-input"
            placeholder="500,000,000"
          />
          <span class="form-hint">Cảnh báo thâm hụt số dư sẽ kích hoạt nếu dự báo dưới ngưỡng này.</span>
        </div>

        <div class="form-group">
          <label class="form-label">Dung sai sai lệch phí ngân hàng cho phép (VND)</label>
          <input
            v-model="autoReconcileTolerance"
            type="text"
            class="form-input"
            placeholder="11,000"
          />
          <span class="form-hint">Phù hợp với các giao dịch chênh lệch phí chuyển khoản liên ngân hàng (&plusmn;11,000 VND).</span>
        </div>
      </div>

      <!-- Card 2: Compliance & Security -->
      <div class="settings-card">
        <h3 class="card-title">2. Chính Sách Tuân Thủ & Bảo Mật Dữ Liệu</h3>

        <div class="toggle-group">
          <div class="toggle-info">
            <span class="toggle-title">Chế độ Zero Data Egress (Nghị định 13/2023/NĐ-CP)</span>
            <span class="toggle-desc">Ngăn chặn 100% dữ liệu số dư, sao kê và PII truyền ra ngoài mạng nội bộ.</span>
          </div>
          <input v-model="enableZeroEgress" type="checkbox" class="toggle-switch" disabled />
        </div>

        <div class="toggle-group">
          <div class="toggle-info">
            <span class="toggle-title">Quy tắc Phê duyệt Hai pha Maker-Checker (TT 09/2020)</span>
            <span class="toggle-desc">Bắt buộc Kế toán trưởng xác nhận bằng mã token UUIDv4 trước khi ghi sổ lệch.</span>
          </div>
          <input v-model="enableMakerChecker" type="checkbox" class="toggle-switch" />
        </div>

        <div class="toggle-group">
          <div class="toggle-info">
            <span class="toggle-title">Tự động dọn dẹp log sau 30 ngày (Data Retention)</span>
            <span class="toggle-desc">Chỉ lưu trữ vết kiểm toán Merkle Tree, thu hồi dung lượng đĩa.</span>
          </div>
          <input v-model="autoPruneOldLogs" type="checkbox" class="toggle-switch" />
        </div>
      </div>
    </div>

    <!-- Advanced System Admin Callout -->
    <div class="system-admin-callout">
      <div class="callout-left">
        <div class="callout-icon">⚙️</div>
        <div>
          <h4 class="callout-title">Quản Trị Hệ Thống Toàn Diện (System & AI Dashboard)</h4>
          <p class="callout-desc">
            Truy cập giao diện nâng cao để cấu hình Mô hình AI Offline, VieNeu TTS Giọng nói, Quản lý Ký ức L0–L3 và Kết nối Obsidian Vault.
          </p>
        </div>
      </div>
      <button class="open-system-btn" @click="openSystemDashboard">
        <span>Mở Dashboard Hệ Thống</span>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
          <polyline points="15 3 21 3 21 9" />
          <line x1="10" y1="14" x2="21" y2="3" />
        </svg>
      </button>
    </div>
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

.settings-header-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 18px 24px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.banner-title {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 4px 0;
}

.banner-desc {
  font-size: 13px;
  color: #64748b;
  margin: 0;
}

.save-btn {
  padding: 10px 22px;
  background: #10b981;
  color: #ffffff;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  transition: background 0.15s ease;
}

.save-btn:hover {
  background: #059669;
}

.settings-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 18px;
}

@media (max-width: 1024px) {
  .settings-grid {
    grid-template-columns: 1fr;
  }
}

.settings-card {
  padding: 22px 24px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-title {
  font-size: 14px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 6px 0;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: #334155;
}

.form-input {
  padding: 10px 14px;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  font-size: 13px;
  color: #0f172a;
  outline: none;
  transition: border-color 0.15s ease;
}

.form-input:focus {
  border-color: #10b981;
}

.form-hint {
  font-size: 11px;
  color: #94a3b8;
}

.toggle-group {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid #f1f5f9;
}

.toggle-group:last-child {
  border-bottom: none;
}

.toggle-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
  max-width: 85%;
}

.toggle-title {
  font-size: 13px;
  font-weight: 600;
  color: #1e293b;
}

.toggle-desc {
  font-size: 11px;
  color: #64748b;
}

.toggle-switch {
  width: 18px;
  height: 18px;
  accent-color: #10b981;
  cursor: pointer;
}

.system-admin-callout {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
}

.callout-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.callout-icon {
  font-size: 24px;
}

.callout-title {
  font-size: 14px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 4px 0;
}

.callout-desc {
  font-size: 12px;
  color: #64748b;
  margin: 0;
}

.open-system-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  background: #0f172a;
  color: #ffffff;
  border: none;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.15s ease;
}

.open-system-btn:hover {
  background: #1e293b;
}
</style>
