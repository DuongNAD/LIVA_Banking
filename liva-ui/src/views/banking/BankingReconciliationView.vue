<script setup lang="ts">
/**
 * BankingReconciliationView.vue — Màn hình Đối soát chuyên sâu
 * Tập trung vào: Đồng hồ đo đối soát 99.8%, Xử lý chênh lệch, Duyệt HITL Maker-Checker và Import sao kê.
 */
import { onMounted } from 'vue';
import { useBankingStore } from '../../stores/bankingStore';
import { useReconciliationStore } from '../../stores/reconciliationStore';
import AutoReconciliationGauge from '../../components/banking/AutoReconciliationGauge.vue';
import BankCard from '../../components/banking/BankCard.vue';
import TransactionLedger from '../../components/banking/TransactionLedger.vue';

const bankingStore = useBankingStore();
const reconciliationStore = useReconciliationStore();

onMounted(async () => {
  await bankingStore.fetchOverview();
  await reconciliationStore.fetchMatrix('ALL');
});
</script>

<template>
  <div class="banking-view-container">
    <!-- Header banner -->
    <div class="reconcile-header-banner">
      <div class="banner-info">
        <h2 class="banner-title">Trung Tâm Điều Hành Đối Soát Tự Động</h2>
        <p class="banner-desc">
          Công cụ đối soát 3 tầng xác định (1:1 Hash, Mờ Jaro-Winkler, Tách gộp Subset-Sum u64).
          Tuân thủ Nghị định 13/2023/NĐ-CP và Thông tư 09/2020/TT-NHNN Maker-Checker.
        </p>
      </div>
      <div class="banner-badge">
        <span class="badge-dot" />
        <span>Mục tiêu: 99.8% Tự Động</span>
      </div>
    </div>

    <!-- Top KPI Row -->
    <section class="dashboard-row row-reconcile-summary">
      <AutoReconciliationGauge
        :reconciled-count="bankingStore.summary.reconciledCount"
        :total-count="bankingStore.summary.totalCount"
        :reconciled-rate="bankingStore.summary.reconciledRate"
        :auto-rate="bankingStore.summary.autoMatchedRate"
        :manual-rate="bankingStore.summary.manualMatchedRate"
        :unmatched-count="bankingStore.summary.unmatchedCount"
        :unmatched-rate="bankingStore.summary.unmatchedRate"
        :target-rate="99.8"
        class="card-col gauge-card-col"
      />

      <!-- VCB Account Card with discrepancy -->
      <BankCard
        bank-code="VCB"
        :bank-name="bankingStore.accounts[0].bankName"
        :account-number="bankingStore.accounts[0].accountNumber"
        :opening-balance="bankingStore.accounts[0].openingBalance"
        :closing-balance="bankingStore.accounts[0].closingBalance"
        :balance="bankingStore.accounts[0].totalBalance"
        :reconciled="bankingStore.accounts[0].reconciledAmount"
        :unreconciled="bankingStore.accounts[0].unreconciledAmount"
        :discrepancy="bankingStore.accounts[0].discrepancy"
        :last-sync="bankingStore.accounts[0].lastSync"
        class="card-col"
      />

      <!-- TCB Account Card with discrepancy -->
      <BankCard
        bank-code="TCB"
        :bank-name="bankingStore.accounts[1].bankName"
        :account-number="bankingStore.accounts[1].accountNumber"
        :opening-balance="bankingStore.accounts[1].openingBalance"
        :closing-balance="bankingStore.accounts[1].closingBalance"
        :balance="bankingStore.accounts[1].totalBalance"
        :reconciled="bankingStore.accounts[1].reconciledAmount"
        :unreconciled="bankingStore.accounts[1].unreconciledAmount"
        :discrepancy="bankingStore.accounts[1].discrepancy"
        :last-sync="bankingStore.accounts[1].lastSync"
        class="card-col"
      />
    </section>

    <!-- Main Reconciliation Ledger with Dropzone -->
    <section class="dashboard-row row-ledger">
      <TransactionLedger />
    </section>
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

.reconcile-header-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
  border-radius: 12px;
  color: #ffffff;
  border-left: 4px solid #10b981;
}

.banner-title {
  font-size: 16px;
  font-weight: 700;
  margin: 0 0 4px 0;
}

.banner-desc {
  font-size: 12px;
  color: #94a3b8;
  margin: 0;
}

.banner-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(16, 185, 129, 0.15);
  border: 1px solid rgba(16, 185, 129, 0.3);
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  color: #34d399;
}

.badge-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #10b981;
  box-shadow: 0 0 6px #10b981;
}

.dashboard-row {
  display: flex;
  gap: 18px;
  width: 100%;
}

.row-reconcile-summary {
  display: grid;
  grid-template-columns: 1.1fr 1fr 1fr;
  gap: 18px;
}

@media (max-width: 1024px) {
  .row-reconcile-summary {
    grid-template-columns: 1fr;
  }
}

.row-ledger {
  display: flex;
  flex-direction: column;
}
</style>
