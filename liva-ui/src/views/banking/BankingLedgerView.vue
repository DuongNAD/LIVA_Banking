<script setup lang="ts">
/**
 * BankingLedgerView.vue — P50–P53 Accounting Ledgers & Automated ERP Posting View
 * =================================================================================
 * Master View integrating:
 * - P50 Journal Entry Proposals
 * - P51 MISA AMIS / FAST Payload Inspector Modal
 * - P52 Sync Monitor & Batch Posting Queue
 * - P53 Bank vs GL Balance Reconciliation Certificate
 */
import { ref } from 'vue';
import { useLedgerStore } from '../../stores/ledgerStore';
import JournalVoucherTable from '../../components/banking/ledger/JournalVoucherTable.vue';
import BalanceCertificateCard from '../../components/banking/ledger/BalanceCertificateCard.vue';
import ErpPayloadModal from '../../components/banking/ledger/ErpPayloadModal.vue';

const store = useLedgerStore();
const activeSubTab = ref<'VOUCHERS' | 'CERTIFICATE'>('VOUCHERS');

function formatVnd(amount: number): string {
  return `${amount.toLocaleString('vi-VN')} ₫`;
}
</script>

<template>
  <div class="banking-ledger-view">
    <!-- Top Header -->
    <header class="ledger-topbar">
      <div class="topbar-left">
        <div class="title-cluster">
          <span class="view-tag">P50–P53</span>
          <h1 class="view-title">Sổ Cái & Bút Toán Đối Ứng ERP</h1>
        </div>
        <div class="reg-tags">
          <span class="reg-pill">TT 200/2014/TT-BTC</span>
          <span class="reg-pill">MISA AMIS & FAST Online</span>
          <span class="reg-pill">Nghị định 123/2020</span>
        </div>
      </div>

      <!-- Sub-Tab Navigation -->
      <div class="topbar-right">
        <div class="sub-tab-group">
          <button
            class="sub-tab-btn"
            :class="{ active: activeSubTab === 'VOUCHERS' }"
            @click="activeSubTab = 'VOUCHERS'"
          >
            📋 Bút Toán Hạch Toán ({{ store.vouchers.length }})
          </button>
          <button
            class="sub-tab-btn"
            :class="{ active: activeSubTab === 'CERTIFICATE' }"
            @click="activeSubTab = 'CERTIFICATE'"
          >
            🛡️ Biên Bản Đối Chiếu (P53)
          </button>
        </div>
      </div>
    </header>

    <!-- KPI Metric Cards Bar -->
    <section class="kpi-metrics-bar">
      <div class="metric-card">
        <span class="metric-lbl">Tổng chứng từ đề xuất:</span>
        <span class="metric-val num">{{ store.voucherStats.total }}</span>
        <span class="metric-sub">Từ kết quả đối soát P42 & P44</span>
      </div>

      <div class="metric-card">
        <span class="metric-lbl">Tổng giá trị hạch toán (Nợ):</span>
        <span class="metric-val currency">{{ formatVnd(store.voucherStats.totalAmount) }}</span>
        <span class="metric-sub">Bảo toàn số học nguyên i64</span>
      </div>

      <div class="metric-card">
        <span class="metric-lbl">Đã đồng bộ thành công:</span>
        <span class="metric-val success">{{ store.voucherStats.posted }} / {{ store.voucherStats.total }}</span>
        <span class="metric-sub">Tỷ lệ đồng bộ: {{ store.voucherStats.syncRate }}%</span>
      </div>

      <div class="metric-card">
        <span class="metric-lbl">Chứng nhận số dư P53:</span>
        <span class="metric-val verified">
          {{ store.certificate.isCertified ? 'ĐÃ CHỨNG THỰC (Δ=0₫)' : 'CHƯA CHỨNG THỰC' }}
        </span>
        <span class="metric-sub">Sổ phụ NH ≡ Sổ cái TK 1121</span>
      </div>
    </section>

    <!-- Main Content Area -->
    <main class="ledger-main">
      <JournalVoucherTable v-if="activeSubTab === 'VOUCHERS'" />
      <BalanceCertificateCard v-else-if="activeSubTab === 'CERTIFICATE'" />
    </main>

    <!-- Modals -->
    <ErpPayloadModal />
  </div>
</template>

<style scoped>
.banking-ledger-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #090d16;
  color: #f8fafc;
  overflow: hidden;
}

.ledger-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 24px;
  background: #111827;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  flex-shrink: 0;
}

.topbar-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.title-cluster {
  display: flex;
  align-items: center;
  gap: 10px;
}

.view-tag {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 800;
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
  border: 1px solid rgba(16, 185, 129, 0.35);
  padding: 2px 6px;
  border-radius: 4px;
  letter-spacing: 0.05em;
}

.view-title {
  font-size: 16px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0;
}

.reg-tags {
  display: flex;
  align-items: center;
  gap: 6px;
}

.reg-pill {
  font-size: 10px;
  font-weight: 600;
  color: #94a3b8;
  background: rgba(255, 255, 255, 0.04);
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.sub-tab-group {
  display: flex;
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 3px;
  gap: 4px;
}

.sub-tab-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.sub-tab-btn:hover {
  color: #f8fafc;
}

.sub-tab-btn.active {
  background: #1e293b;
  color: #38bdf8;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

.kpi-metrics-bar {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  padding: 12px 24px;
  background: #0d131f;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.metric-card {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 10px 14px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.metric-lbl {
  font-size: 11px;
  color: #94a3b8;
  font-weight: 600;
}

.metric-val {
  font-size: 15px;
  font-weight: 700;
}

.metric-val.num {
  font-family: 'JetBrains Mono', monospace;
  color: #f8fafc;
}

.metric-val.currency {
  font-family: 'JetBrains Mono', monospace;
  color: #38bdf8;
}

.metric-val.success {
  font-family: 'JetBrains Mono', monospace;
  color: #10b981;
}

.metric-val.verified {
  font-size: 12px;
  color: #34d399;
  letter-spacing: 0.02em;
}

.metric-sub {
  font-size: 10px;
  color: #64748b;
}

.ledger-main {
  flex: 1;
  overflow: hidden;
}
</style>
