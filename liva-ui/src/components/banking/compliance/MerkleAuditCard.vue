<script setup lang="ts">
/**
 * MerkleAuditCard.vue — P83 Bảo Chứng Sổ Cái Mật Mã (Merkle Audit Tree)
 * ======================================================================
 * Hiển thị gốc băm Merkle Tree chứng thực tính bất biến của dữ liệu sổ cái,
 * ngăn chặn gian lận và chứng thực môi trường Zero Cloud Egress (Nghị định 13/2023).
 */
import { ref } from 'vue';
import { useComplianceStore } from '../../../stores/complianceStore';

const store = useComplianceStore();
const copyStatus = ref('Sao chép hash');

async function copyRootHash() {
  try {
    await navigator.clipboard.writeText(store.merkleRootHash);
    copyStatus.value = 'Đã sao chép! ✓';
    setTimeout(() => {
      copyStatus.value = 'Sao chép hash';
    }, 2000);
  } catch {
    copyStatus.value = 'Lỗi copy';
  }
}
</script>

<template>
  <div class="merkle-card">
    <div class="merkle-header">
      <div class="header-left">
        <span class="p-tag">P83 MERKLE AUDIT</span>
        <div>
          <h3 class="card-title">Bảo Chứng Sổ Cái Mật Mã (Merkle Audit Tree)</h3>
          <p class="card-desc">Chuỗi khối kiểm toán bất biến & Chứng thực Zero Cloud Egress nội bộ</p>
        </div>
      </div>

      <div class="status-badge">
        <span class="dot pulse" />
        <span>Toàn Vẹn 100% (Tamper-Evident)</span>
      </div>
    </div>

    <!-- Cryptographic Root Hash Box -->
    <div class="hash-box">
      <div class="hash-meta-row">
        <span class="meta-label">Gốc Cây Merkle Hiện Tại (Current Root Hash - SHA-256):</span>
        <button class="btn-copy font-mono" @click="copyRootHash">
          {{ copyStatus }}
        </button>
      </div>
      <div class="hash-content font-mono select-all">
        {{ store.merkleRootHash }}
      </div>
    </div>

    <!-- Regulatory Compliance Pillars -->
    <div class="pillars-grid">
      <div class="pillar-item green">
        <div class="p-icon">🛡️</div>
        <div class="p-text">
          <strong>Zero Cloud Egress</strong>
          <span>100% Xử lý nội bộ trên máy trạm (On-premise air-gapped)</span>
        </div>
      </div>

      <div class="pillar-item blue">
        <div class="p-icon">🔒</div>
        <div class="p-text">
          <strong>Nghị Định 13/2023</strong>
          <span>Bảo vệ & Ẩn danh hóa dữ liệu cá nhân tự động</span>
        </div>
      </div>

      <div class="pillar-item purple">
        <div class="p-icon">⚖️</div>
        <div class="p-text">
          <strong>Thông Tư 09/2020</strong>
          <span>Bảo đảm an toàn hệ thống thông tin & kiểm toán lưu vết</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.merkle-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 18px 22px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.merkle-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
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

.status-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #dcfce7;
  color: #15803d;
  font-size: 11px;
  font-weight: 700;
  padding: 4px 10px;
  border-radius: 20px;
  border: 1px solid #86efac;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #10b981;
}

.dot.pulse {
  box-shadow: 0 0 6px #10b981;
}

.hash-box {
  background: #0f172a;
  border-radius: 8px;
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.hash-meta-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  color: #94a3b8;
}

.btn-copy {
  background: transparent;
  border: none;
  color: #38bdf8;
  font-size: 11px;
  cursor: pointer;
}

.btn-copy:hover {
  text-decoration: underline;
}

.hash-content {
  font-size: 12px;
  color: #34d399;
  letter-spacing: 0.5px;
  word-break: break-all;
  user-select: all;
}

.pillars-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 10px;
}

.pillar-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-radius: 8px;
  border: 1px solid transparent;
}

.pillar-item.green {
  background: #f0fdf4;
  border-color: #bbf7d0;
}

.pillar-item.blue {
  background: #eff6ff;
  border-color: #bfdbfe;
}

.pillar-item.purple {
  background: #faf5ff;
  border-color: #e9d5ff;
}

.p-icon {
  font-size: 20px;
}

.p-text {
  display: flex;
  flex-direction: column;
}

.p-text strong {
  font-size: 12px;
  color: #0f172a;
}

.p-text span {
  font-size: 10px;
  color: #64748b;
}

.font-mono {
  font-family: 'JetBrains Mono', monospace;
}
</style>
