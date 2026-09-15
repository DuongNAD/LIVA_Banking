<script setup lang="ts">
/**
 * BackupRestoreTab.vue — P106 Sao Lưu Dữ Liệu & Phục Hồi Thảm Họa (DR)
 * =====================================================================
 * Quản lý tính toàn vẹn của chuỗi băm Merkle Tree, sao lưu Snapshot định kỳ
 * và thực thi kiểm toán phục hồi thảm họa (Disaster Recovery Drill) định kỳ.
 */
import { ref } from 'vue';
import { useSettingsStore } from '../../../stores/settingsStore';

const store = useSettingsStore();

const backupMessage = ref('');
const drMessage = ref('');

function runBackup() {
  const hash = store.triggerBackupNow();
  backupMessage.value = `Đã tạo bản sao lưu ${hash} thành công! ✓`;
  setTimeout(() => {
    backupMessage.value = '';
  }, 3000);
}

function runDrDrill() {
  const res = store.verifyDisasterRecovery();
  drMessage.value = `Kết quả diễn tập DR: ${res} ✓`;
  setTimeout(() => {
    drMessage.value = '';
  }, 4000);
}
</script>

<template>
  <div class="settings-card">
    <div class="card-header">
      <div>
        <div class="header-tags">
          <span class="p-tag">P106 SAO LƯU & DR</span>
          <span class="status-pill ok">Merkle Tree Tamper-Evident</span>
        </div>
        <h3 class="card-title">Sao Lưu Cơ Sở Dữ Liệu & Diễn Tập Phục Hồi Thảm Họa (DR)</h3>
        <p class="card-desc">
          Bảo vệ dữ liệu giao dịch đối soát ngân hàng theo mô hình bảo chứng mật mã SHA-256 bất biến.
        </p>
      </div>
    </div>

    <!-- Merkle Audit Ledger Card -->
    <div class="merkle-panel">
      <div class="merkle-top">
        <div class="merkle-title">Gốc Băm Merkle Tree Toàn Hệ Thống (Audit Root)</div>
        <span class="hash-badge">SHA-256 IMMUTABLE</span>
      </div>
      <code class="merkle-hash font-mono">{{ store.backup.merkleRootHash }}</code>
      <div class="merkle-note">
        Mọi bản ghi đối soát, lệnh thanh toán và bút toán ERP đều được liên kết mật mã vào gốc Merkle này.
        Bất kỳ hành vi sửa đổi dữ liệu trái phép nào đều ngay lập tức phá vỡ tính hợp lệ của chuỗi băm.
      </div>
    </div>

    <!-- DR Controls Grid -->
    <div class="dr-grid">
      <!-- Backup Box -->
      <div class="dr-box">
        <div class="box-header">
          <h4 class="box-title">Sao Lưu Snapshot Cơ Sở Dữ Liệu</h4>
          <span class="box-sub">Tần suất: Tự động mỗi 60 phút</span>
        </div>
        <div class="box-content">
          <div class="row-info">
            <span>Bản sao lưu gần nhất:</span>
            <strong class="font-mono text-accent">{{ store.backup.lastBackupAt }}</strong>
          </div>
          <div class="row-info">
            <span>Thời gian lưu trữ:</span>
            <strong>{{ store.backup.backupRetentionDays }} ngày (Tuân thủ luật lưu trữ)</strong>
          </div>
        </div>
        <div class="box-footer">
          <span v-if="backupMessage" class="feedback-msg">{{ backupMessage }}</span>
          <button class="dr-btn backup" @click="runBackup">
            Tạo Bản Sao Lưu Tức Thời
          </button>
        </div>
      </div>

      <!-- Disaster Recovery Box -->
      <div class="dr-box">
        <div class="box-header">
          <h4 class="box-title">Diễn Tập Phục Hồi Thảm Họa (DR Drill)</h4>
          <span class="box-sub">Cam kết: RTO &lt; 30s, RPO &lt; 60s</span>
        </div>
        <div class="box-content">
          <div class="row-info">
            <span>Kiểm toán DR gần nhất:</span>
            <strong class="text-success">{{ store.backup.lastDrVerification }}</strong>
          </div>
          <div class="row-info">
            <span>Mô hình dự phòng:</span>
            <strong>Active-Passive Local Mirror (Zero Egress)</strong>
          </div>
        </div>
        <div class="box-footer">
          <span v-if="drMessage" class="feedback-msg text-success">{{ drMessage }}</span>
          <button class="dr-btn drill" @click="runDrDrill">
            Chạy Diễn Tập Khôi Phục DR
          </button>
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

.merkle-panel {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.merkle-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.merkle-title {
  font-size: 13px;
  font-weight: 700;
  color: #f1f5f9;
}

.hash-badge {
  font-size: 10px;
  font-weight: 700;
  color: #10b981;
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
  padding: 2px 6px;
  border-radius: 4px;
}

.merkle-hash {
  background: #090d16;
  border: 1px solid rgba(255, 255, 255, 0.04);
  padding: 10px;
  border-radius: 6px;
  font-size: 12px;
  color: #38bdf8;
  word-break: break-all;
}

.merkle-note {
  font-size: 11px;
  color: #64748b;
  line-height: 1.5;
}

.dr-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 14px;
}

@media (max-width: 900px) {
  .dr-grid {
    grid-template-columns: 1fr;
  }
}

.dr-box {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 14px;
}

.box-header {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.box-title {
  font-size: 14px;
  font-weight: 700;
  color: #f1f5f9;
  margin: 0;
}

.box-sub {
  font-size: 11px;
  color: #64748b;
}

.box-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 12px;
}

.row-info {
  display: flex;
  justify-content: space-between;
  color: #94a3b8;
}

.box-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  padding-top: 10px;
}

.feedback-msg {
  font-size: 11px;
  font-weight: 600;
  color: #10b981;
}

.dr-btn {
  padding: 8px 14px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.dr-btn.backup {
  background: rgba(56, 189, 248, 0.15);
  border-color: rgba(56, 189, 248, 0.3);
  color: #38bdf8;
}

.dr-btn.backup:hover {
  background: rgba(56, 189, 248, 0.25);
}

.dr-btn.drill {
  background: rgba(16, 185, 129, 0.15);
  border-color: rgba(16, 185, 129, 0.3);
  color: #10b981;
}

.dr-btn.drill:hover {
  background: rgba(16, 185, 129, 0.25);
}

.text-accent {
  color: #38bdf8;
}

.text-success {
  color: #10b981;
}
</style>
