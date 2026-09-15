<script setup lang="ts">
/**
 * BankingTreasuryView.vue — Màn hình Quản trị Ngân quỹ Tập trung
 * Giám sát vị thế số dư trên tất cả các tài khoản ngân hàng thương mại (VCB, TCB, BIDV),
 * tỷ lệ tập trung vốn và bảng điều phối Treasury MiniDock.
 */
import { onMounted } from 'vue';
import { useBankingStore } from '../../stores/bankingStore';
import BankCard from '../../components/banking/BankCard.vue';
import TreasuryMiniDock from '../../components/banking/TreasuryMiniDock.vue';

const bankingStore = useBankingStore();

onMounted(async () => {
  await bankingStore.fetchOverview();
});

function formatVnd(val: number): string {
  return new Intl.NumberFormat('vi-VN').format(val) + ' VND';
}
</script>

<template>
  <div class="banking-view-container">
    <!-- Header Summary -->
    <div class="treasury-header-banner">
      <div>
        <h2 class="banner-title">Quản Trị Ngân Quỹ & Tập Trung Vốn</h2>
        <p class="banner-desc">
          Theo dõi số dư thanh toán đa ngân hàng theo thời gian thực. Tổng tiền mặt khả dụng:
          <strong>{{ formatVnd(bankingStore.totalBalanceAll) }}</strong>.
        </p>
      </div>
      <TreasuryMiniDock />
    </div>

    <!-- Multi-bank Accounts Grid -->
    <div class="section-title">Danh Sách Tài Khoản Thanh Toán Doanh Nghiệp</div>
    <section class="dashboard-row row-bank-cards">
      <BankCard
        v-for="acc in bankingStore.accounts"
        :key="acc.id"
        :bank-code="acc.bankCode"
        :bank-name="acc.bankName"
        :account-number="acc.accountNumber"
        :opening-balance="acc.openingBalance"
        :closing-balance="acc.closingBalance"
        :balance="acc.totalBalance"
        :reconciled="acc.reconciledAmount"
        :unreconciled="acc.unreconciledAmount"
        :discrepancy="acc.discrepancy"
        :last-sync="acc.lastSync"
        class="card-col"
      />
    </section>

    <!-- Capital Concentration Summary -->
    <div class="treasury-breakdown-card">
      <h3 class="breakdown-title">Cơ cấu phân bổ số dư khả dụng</h3>
      <div class="allocation-bars">
        <div
          v-for="acc in bankingStore.accounts"
          :key="acc.id"
          class="allocation-item"
        >
          <div class="item-header">
            <span class="item-name">{{ acc.bankName }} ({{ acc.bankCode }})</span>
            <span class="item-val">
              {{ formatVnd(acc.totalBalance) }}
              ({{ ((acc.totalBalance / (bankingStore.totalBalanceAll || 1)) * 100).toFixed(1) }}%)
            </span>
          </div>
          <div class="progress-track">
            <div
              class="progress-fill"
              :style="{
                width: `${((acc.totalBalance / (bankingStore.totalBalanceAll || 1)) * 100).toFixed(1)}%`,
                backgroundColor: acc.themeColor
              }"
            />
          </div>
        </div>
      </div>
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

.treasury-header-banner {
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

.banner-desc strong {
  color: #10b981;
}

.section-title {
  font-size: 13px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #475569;
}

.dashboard-row {
  display: flex;
  gap: 18px;
  width: 100%;
}

.row-bank-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 18px;
}

.treasury-breakdown-card {
  padding: 20px 24px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.breakdown-title {
  font-size: 14px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 16px 0;
}

.allocation-bars {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.allocation-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.item-header {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  font-weight: 600;
  color: #334155;
}

.item-val {
  color: #64748b;
}

.progress-track {
  height: 8px;
  background: #f1f5f9;
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.3s ease;
}
</style>
