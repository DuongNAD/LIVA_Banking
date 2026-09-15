<script setup lang="ts">
/**
 * BankingDashboardView.vue
 * 2D Banking & Treasury UI Dashboard matching user mockup 1:1.
 * Integrates:
 * - Row 1: Bank Cards (VCB, TCB) & Auto-Reconciliation Gauge (99.8% Target)
 * - Row 2: Charts (Reconciliation Line Trend & Status Allocation Donut)
 * - Row 3: Rolling Cashflow Sentinel (30-90 Days Forecast & Deficit Warning)
 * - Row 4: Real-time Transaction Ledger & Hot-Folder Ingestion
 */
import { onMounted } from 'vue';
import { useBankingStore } from '../stores/bankingStore';
import { useReconciliationStore } from '../stores/reconciliationStore';
import BankCard from '../components/banking/BankCard.vue';
import AutoReconciliationGauge from '../components/banking/AutoReconciliationGauge.vue';
import ReconciliationTrendChart from '../components/banking/ReconciliationTrendChart.vue';
import StatusAllocationDonut from '../components/banking/StatusAllocationDonut.vue';
import RollingCashflowSentinel from '../components/banking/RollingCashflowSentinel.vue';
import TransactionLedger from '../components/banking/TransactionLedger.vue';

const bankingStore = useBankingStore();
const reconciliationStore = useReconciliationStore();

onMounted(async () => {
  // Real Tauri IPC initialization
  await bankingStore.fetchOverview();
  await reconciliationStore.fetchMatrix();
});
</script>

<template>
  <div class="banking-dashboard-container">
    <!-- Row 1: Bank Cards & Auto-Reconciliation Gauge -->
    <section class="dashboard-row row-bank-cards">
      <!-- Vietcombank Card -->
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

      <!-- Techcombank Card -->
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

      <!-- Auto-Reconciliation Gauge Card -->
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
    </section>

    <!-- Row 2: Charts (Line Trend & Status Donut) -->
    <section class="dashboard-row row-charts">
      <ReconciliationTrendChart
        :data="bankingStore.trendDays"
        filter-label="Last lướt 30 ngày"
      />
      <StatusAllocationDonut
        :matched-rate="bankingStore.summary.reconciledRate"
        :unmatched-rate="bankingStore.summary.unmatchedRate"
      />
    </section>

    <!-- Row 3: Rolling Cashflow Sentinel (30-90 Days Forecast) -->
    <section class="dashboard-row row-sentinel">
      <RollingCashflowSentinel />
    </section>

    <!-- Row 4: Real-time Transaction Ledger & Hot-Folder -->
    <section class="dashboard-row row-ledger">
      <TransactionLedger />
    </section>
  </div>
</template>

<style scoped>
.banking-dashboard-container {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 20px 24px 40px 24px;
  background-color: #f8fafc;
  min-height: 100%;
  overflow-y: auto;
}

.dashboard-row {
  display: flex;
  gap: 18px;
  width: 100%;
}

.row-bank-cards {
  display: grid;
  grid-template-columns: 1fr 1fr 1.05fr;
  gap: 18px;
}

@media (max-width: 1024px) {
  .row-bank-cards {
    grid-template-columns: 1fr;
  }
}

.row-charts {
  display: flex;
  flex-wrap: wrap;
  gap: 18px;
}

.row-sentinel {
  display: flex;
  flex-direction: column;
}

.row-ledger {
  display: flex;
  flex-direction: column;
}
</style>
