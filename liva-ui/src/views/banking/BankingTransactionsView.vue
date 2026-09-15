<script setup lang="ts">
/**
 * BankingTransactionsView.vue — Màn hình Sổ cái Giao dịch thời gian thực
 * Cung cấp không gian làm việc rộng rãi cho kế toán viên: tra cứu, lọc trạng thái, xem chi tiết chứng từ đối ứng.
 */
import { onMounted } from 'vue';
import { useBankingStore } from '../../stores/bankingStore';
import { useReconciliationStore } from '../../stores/reconciliationStore';
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
    <!-- Transactions Overview Stats Bar -->
    <div class="transactions-stats-bar">
      <div class="stat-pill total">
        <span class="stat-label">Tổng giao dịch</span>
        <span class="stat-value">{{ bankingStore.summary.totalCount.toLocaleString() }}</span>
      </div>
      <div class="stat-pill matched">
        <span class="stat-label">Khớp tự động</span>
        <span class="stat-value">{{ bankingStore.summary.reconciledCount.toLocaleString() }} ({{ bankingStore.summary.reconciledRate }}%)</span>
      </div>
      <div class="stat-pill pending">
        <span class="stat-label">Chờ duyệt Maker-Checker</span>
        <span class="stat-value">{{ reconciliationStore.transactionCounts.pending }} GD</span>
      </div>
      <div class="stat-pill unmatched">
        <span class="stat-label">Chênh lệch / Chưa khớp</span>
        <span class="stat-value">{{ bankingStore.summary.unmatchedCount }} GD ({{ bankingStore.summary.unmatchedRate }}%)</span>
      </div>
    </div>

    <!-- Main Full-Width Ledger -->
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

.transactions-stats-bar {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 14px;
}

@media (max-width: 1024px) {
  .transactions-stats-bar {
    grid-template-columns: repeat(2, 1fr);
  }
}

.stat-pill {
  display: flex;
  flex-direction: column;
  padding: 14px 18px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.stat-label {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
}

.stat-value {
  font-size: 18px;
  font-weight: 700;
  color: #0f172a;
}

.stat-pill.matched {
  border-left: 4px solid #10b981;
}
.stat-pill.matched .stat-value {
  color: #059669;
}

.stat-pill.pending {
  border-left: 4px solid #f59e0b;
}
.stat-pill.pending .stat-value {
  color: #d97706;
}

.stat-pill.unmatched {
  border-left: 4px solid #ef4444;
}
.stat-pill.unmatched .stat-value {
  color: #dc2626;
}

.stat-pill.total {
  border-left: 4px solid #3b82f6;
}

.dashboard-row {
  display: flex;
  gap: 18px;
  width: 100%;
}

.row-ledger {
  display: flex;
  flex-direction: column;
}
</style>
