<script setup lang="ts">
/**
 * BalanceCertificateCard.vue — P53 Bank vs GL Reconciliation Certificate
 * =========================================================================
 * Attestation and balance equation verification:
 * Closing Bank + In-Transit Deposits - In-Transit Withdrawals == Closing GL
 * Signed with SHA-256 Merkle root and Checker authorization.
 */
import { computed } from 'vue';
import { useLedgerStore } from '../../../stores/ledgerStore';

const store = useLedgerStore();
const cert = computed(() => store.certificate);

function formatVnd(amount: number): string {
  return `${amount.toLocaleString('vi-VN')} ₫`;
}
</script>

<template>
  <div class="certificate-container">
    <div class="certificate-card">
      <!-- Certificate Header -->
      <div class="cert-header">
        <div class="cert-title-group">
          <div class="cert-badge">P53 CERTIFICATION</div>
          <h2 class="cert-title">Biên Bản Chứng Nhận Đối Chiếu Số Dư Sổ Cái & Sao Kê</h2>
          <span class="cert-period">Kỳ đối soát: {{ cert.periodStart }} đến {{ cert.periodEnd }} · Tài khoản: {{ cert.bankAccount }}</span>
        </div>

        <div class="cert-status-badge" :class="{ verified: cert.isCertified }">
          <span class="cert-icon">{{ cert.isCertified ? '🛡️' : '⚠️' }}</span>
          <span class="cert-status-text">
            {{ cert.isCertified ? 'ĐÃ XÁC THỰC CÂN ĐỐI (Δ = 0 ₫)' : 'CHƯA ĐỐI SOÁT XONG' }}
          </span>
        </div>
      </div>

      <!-- Comparison Grid -->
      <div class="comparison-grid">
        <!-- Bank Statement Column -->
        <div class="balance-column bank-side">
          <div class="col-header">
            <span class="col-pill bank">SỔ PHỤ NGÂN HÀNG (BANK STATEMENT)</span>
            <span class="col-acc">{{ cert.bankAccount }}</span>
          </div>

          <div class="balance-rows">
            <div class="bal-row">
              <span class="bal-lbl">Số dư đầu kỳ:</span>
              <span class="bal-val">{{ formatVnd(cert.bankOpeningBalance) }}</span>
            </div>
            <div class="bal-row text-credit">
              <span class="bal-lbl">Tổng phát sinh Tiền vào (+):</span>
              <span class="bal-val">+{{ formatVnd(cert.bankTotalCredits) }}</span>
            </div>
            <div class="bal-row text-debit">
              <span class="bal-lbl">Tổng phát sinh Tiền ra (-):</span>
              <span class="bal-val">-{{ formatVnd(cert.bankTotalDebits) }}</span>
            </div>
            <div class="bal-row total-row">
              <span class="bal-lbl">Số dư cuối kỳ thực tế:</span>
              <span class="bal-val bold">{{ formatVnd(cert.bankClosingBalance) }}</span>
            </div>
          </div>
        </div>

        <!-- GL Account 1121 Column -->
        <div class="balance-column gl-side">
          <div class="col-header">
            <span class="col-pill gl">SỔ CÁI DOANH NGHIỆP (GL ACCOUNT)</span>
            <span class="col-acc">TK 1121 — MISA AMIS / FAST</span>
          </div>

          <div class="balance-rows">
            <div class="bal-row">
              <span class="bal-lbl">Số dư đầu kỳ:</span>
              <span class="bal-val">{{ formatVnd(cert.glOpeningBalance) }}</span>
            </div>
            <div class="bal-row text-credit">
              <span class="bal-lbl">Tổng phát sinh Nợ (+):</span>
              <span class="bal-val">+{{ formatVnd(cert.glTotalDebits) }}</span>
            </div>
            <div class="bal-row text-debit">
              <span class="bal-lbl">Tổng phát sinh Có (-):</span>
              <span class="bal-val">-{{ formatVnd(cert.glTotalCredits) }}</span>
            </div>
            <div class="bal-row total-row">
              <span class="bal-lbl">Số dư cuối kỳ sổ cái:</span>
              <span class="bal-val bold">{{ formatVnd(cert.glClosingBalance) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- In-Transit Reconciliation Bar -->
      <div class="transit-reconciliation-bar">
        <div class="transit-item">
          <span class="t-lbl">Tiền gửi đang chuyển (+):</span>
          <span class="t-val">{{ formatVnd(cert.inTransitDeposits) }}</span>
        </div>
        <div class="transit-item">
          <span class="t-lbl">Lệnh chi chưa khớp (-):</span>
          <span class="t-val">{{ formatVnd(cert.inTransitWithdrawals) }}</span>
        </div>
        <div class="transit-item highlight">
          <span class="t-lbl">Số dư ngân hàng đã điều chỉnh:</span>
          <span class="t-val">{{ formatVnd(cert.adjustedBankBalance) }}</span>
        </div>
        <div class="transit-item result">
          <span class="t-lbl">Chênh lệch đối soát (Variance):</span>
          <span class="t-val zero-diff">{{ formatVnd(cert.variance) }}</span>
        </div>
      </div>

      <!-- Cryptographic Audit & Sign-off Seal -->
      <div class="cert-footer">
        <div class="footer-left">
          <div class="hash-chain-row">
            <span class="chain-icon">🔗</span>
            <span class="hash-lbl">Merkle Root Hash:</span>
            <span class="hash-str">{{ cert.merkleRootHash }}</span>
          </div>
          <div class="standard-row">
            <span>Tiêu chuẩn: Thông tư 200/2014/TT-BTC · Chuẩn mực kiểm toán VSA 505</span>
          </div>
        </div>

        <div class="footer-right">
          <div class="sign-off-stamp">
            <span class="stamp-title">CHỨNG THỰC BỞI KIỂM SOÁT VIÊN</span>
            <span class="stamp-signer">{{ cert.certifiedByChecker || 'Kế toán trưởng (Checker)' }}</span>
            <span class="stamp-time">{{ cert.certifiedAt || '2026-09-15T18:30:00Z' }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.certificate-container {
  padding: 20px;
  overflow-y: auto;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.certificate-card {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  max-width: 980px;
  width: 100%;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
}

.cert-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  padding-bottom: 16px;
}

.cert-badge {
  display: inline-block;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 800;
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
  border: 1px solid rgba(16, 185, 129, 0.3);
  padding: 2px 8px;
  border-radius: 4px;
  margin-bottom: 6px;
}

.cert-title {
  font-size: 18px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0 0 6px 0;
}

.cert-period {
  font-size: 12px;
  color: #94a3b8;
}

.cert-status-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  border-radius: 8px;
  background: rgba(16, 185, 129, 0.15);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: #34d399;
  font-size: 12px;
  font-weight: 700;
}

.comparison-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.balance-column {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.col-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  padding-bottom: 10px;
}

.col-pill {
  font-size: 10px;
  font-weight: 800;
  padding: 2px 6px;
  border-radius: 4px;
}

.col-pill.bank {
  background: rgba(56, 189, 248, 0.2);
  color: #38bdf8;
}

.col-pill.gl {
  background: rgba(168, 85, 247, 0.2);
  color: #c084fc;
}

.col-acc {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  color: #94a3b8;
}

.balance-rows {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.bal-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 13px;
}

.bal-lbl {
  color: #94a3b8;
}

.bal-val {
  font-family: 'JetBrains Mono', monospace;
  font-weight: 600;
  color: #e2e8f0;
}

.bal-val.bold {
  font-size: 15px;
  font-weight: 700;
}

.text-credit .bal-val {
  color: #10b981;
}

.text-debit .bal-val {
  color: #f43f5e;
}

.total-row {
  border-top: 1px dashed rgba(255, 255, 255, 0.12);
  padding-top: 8px;
  margin-top: 4px;
}

.transit-reconciliation-bar {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 14px;
}

.transit-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.t-lbl {
  font-size: 11px;
  color: #64748b;
}

.t-val {
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  font-weight: 600;
  color: #e2e8f0;
}

.transit-item.highlight .t-val {
  color: #38bdf8;
  font-weight: 700;
}

.transit-item.result .t-val.zero-diff {
  color: #10b981;
  font-weight: 800;
}

.cert-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  padding-top: 16px;
}

.footer-left {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.hash-chain-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
}

.hash-lbl {
  color: #64748b;
  font-weight: 600;
}

.hash-str {
  font-family: 'JetBrains Mono', monospace;
  color: #94a3b8;
}

.standard-row {
  font-size: 11px;
  color: #64748b;
}

.sign-off-stamp {
  border: 2px dashed #10b981;
  border-radius: 8px;
  padding: 8px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  background: rgba(16, 185, 129, 0.05);
}

.stamp-title {
  font-size: 9px;
  font-weight: 800;
  color: #10b981;
  letter-spacing: 0.05em;
}

.stamp-signer {
  font-size: 13px;
  font-weight: 700;
  color: #f8fafc;
  margin: 2px 0;
}

.stamp-time {
  font-size: 10px;
  color: #64748b;
  font-family: 'JetBrains Mono', monospace;
}

@media (max-width: 768px) {
  .comparison-grid {
    grid-template-columns: 1fr;
  }
  .transit-reconciliation-bar {
    grid-template-columns: 1fr 1fr;
  }
}
</style>
