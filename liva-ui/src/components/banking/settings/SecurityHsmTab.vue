<script setup lang="ts">
/**
 * SecurityHsmTab.vue — P105 Quản Lý Chứng Thư Số & Ký Số HSM / USB Token
 * =========================================================================
 * Kiểm soát khóa bảo mật X.509, phần cứng ký số HSM và thực thi
 * chính sách Zero Cloud Egress theo Nghị định 13/2023 và Thông tư 09/2020.
 */
import { useSettingsStore } from '../../../stores/settingsStore';

const store = useSettingsStore();
</script>

<template>
  <div class="settings-card">
    <div class="card-header">
      <div>
        <div class="header-tags">
          <span class="p-tag">P105 BẢO MẬT & CHỮ KÝ SỐ</span>
          <span class="status-pill ok">Zero Cloud Egress Active</span>
        </div>
        <h3 class="card-title">Quản Trị Khóa Mật Mã & Thiết Bị Ký Số (HSM / USB Token)</h3>
        <p class="card-desc">
          Bảo chứng tính pháp lý cho các quyết định duyệt chi của Kế toán trưởng và Giám đốc tài chính.
        </p>
      </div>
    </div>

    <div class="security-grid">
      <!-- 1. Zero Cloud Egress Card -->
      <div class="sec-item">
        <div class="sec-icon-wrap">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
            <path d="M7 11V7a5 5 0 0 1 10 0v4" />
          </svg>
        </div>
        <div class="sec-content">
          <div class="sec-title-row">
            <h4 class="sec-title">Cô Lập Dữ Liệu On-Premise (Zero Cloud Egress)</h4>
            <span class="badge-lock">KHÓA TUYỆT ĐỐI</span>
          </div>
          <p class="sec-desc">
            Toàn bộ socket mạng ngoài (non-loopback WAN) đều bị ngăn chặn ở cấp kernel (nftables / eBPF).
            Không có bất kỳ dữ liệu sao kê tài chính nào bị rò rỉ ra môi trường đám mây công cộng.
          </p>
          <div class="sec-meta">
            <span>Tiêu chuẩn: <strong>Nghị định 13/2023/NĐ-CP</strong></span>
            <span>Trạng thái: <strong class="text-success">Hoạt động (100% On-Premise)</strong></span>
          </div>
        </div>
      </div>

      <!-- 2. Hardware Security Module (HSM) -->
      <div class="sec-item">
        <div class="sec-icon-wrap">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#38bdf8" stroke-width="2">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            <circle cx="12" cy="11" r="3" />
          </svg>
        </div>
        <div class="sec-content">
          <div class="sec-title-row">
            <h4 class="sec-title">Nhà Cung Cấp Dịch Vụ Chứng Thực Chữ Ký Số (CA/HSM)</h4>
            <span class="badge-hsm">FIPS 140-2 LEVEL 3</span>
          </div>
          <p class="sec-desc">
            {{ store.security.hsmProvider }}. Cung cấp chứng thư số ký điện tử phê duyệt thanh toán Maker-Checker.
          </p>
          <div class="sec-meta">
            <span>Số sê-ri chứng thư: <strong class="font-mono text-accent">{{ store.security.certSerial }}</strong></span>
            <span>Hiệu lực đến: <strong class="text-muted">{{ store.security.certValidUntil }}</strong></span>
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

.security-grid {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.sec-item {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 16px;
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.sec-icon-wrap {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.sec-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
}

.sec-title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.sec-title {
  font-size: 14px;
  font-weight: 700;
  color: #f1f5f9;
  margin: 0;
}

.badge-lock {
  font-size: 10px;
  font-weight: 700;
  color: #10b981;
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
  padding: 2px 6px;
  border-radius: 4px;
}

.badge-hsm {
  font-size: 10px;
  font-weight: 700;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.12);
  border: 1px solid rgba(56, 189, 248, 0.3);
  padding: 2px 6px;
  border-radius: 4px;
}

.sec-desc {
  font-size: 12px;
  color: #94a3b8;
  margin: 0;
  line-height: 1.5;
}

.sec-meta {
  display: flex;
  gap: 20px;
  font-size: 12px;
  color: #64748b;
  border-top: 1px solid rgba(255, 255, 255, 0.04);
  padding-top: 8px;
}

.sec-meta strong {
  color: #e2e8f0;
}

.text-accent {
  color: #38bdf8;
}

.text-success {
  color: #10b981;
}

.text-muted {
  color: #94a3b8;
}
</style>
