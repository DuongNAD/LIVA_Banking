<script setup lang="ts">
/**
 * BankingCashflowView.vue — Màn hình Quản trị Thu Chi & Dự báo Dòng tiền
 * Tích hợp Tháp canh giám sát (Rolling Cashflow Sentinel) 30–90 ngày, cảnh báo thâm hụt sớm và điều phối thanh khoản.
 */
import { onMounted } from 'vue';
import { useBankingStore } from '../../stores/bankingStore';
import RollingCashflowSentinelCard from '../../components/banking/treasury/RollingCashflowSentinelCard.vue';

const bankingStore = useBankingStore();

onMounted(async () => {
  await bankingStore.fetchOverview();
});
</script>

<template>
  <div class="banking-view-container">
    <!-- Header Strategy Banner -->
    <div class="cashflow-header-banner">
      <div>
        <h2 class="banner-title">Tháp Canh Giám Sát & Dự Báo Dòng Tiền (Rolling Cashflow Sentinel)</h2>
        <p class="banner-desc">
          Mô hình hóa chu kỳ luân chuyển vốn 30–90 ngày. Cảnh báo sớm nguy cơ mất thanh khoản dưới ngưỡng đệm an toàn 500M VND.
        </p>
      </div>
      <div class="liquidity-health-badge">
        <span class="health-dot" />
        <span>Trạng thái: An Toàn Thanh Khoản</span>
      </div>
    </div>

    <!-- Main Sentinel Component -->
    <section class="dashboard-row row-sentinel">
      <RollingCashflowSentinelCard />
    </section>

    <!-- Capital Transfer Recommendation Box -->
    <div class="capital-transfer-box">
      <div class="box-icon">💡</div>
      <div class="box-content">
        <h4 class="box-title">Khuyến nghị điều phối thanh khoản tự động:</h4>
        <p class="box-text">
          Dự kiến ngày 18/10 có nghĩa vụ thanh toán nhà cung cấp và nộp thuế VAT. 
          Hệ thống khuyến nghị duy trì số dư khả dụng tại Vietcombank trên 1.2 tỷ VND bằng cách điều chuyển 500 triệu VND từ tài khoản thanh toán Techcombank sang trước 14:00 ngày 17/10.
        </p>
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

.cashflow-header-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
  border-radius: 12px;
  color: #ffffff;
  border-left: 4px solid #0284c7;
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

.liquidity-health-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(14, 165, 233, 0.15);
  border: 1px solid rgba(14, 165, 233, 0.3);
  padding: 6px 14px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  color: #38bdf8;
}

.health-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #0284c7;
  box-shadow: 0 0 6px #0284c7;
}

.dashboard-row {
  display: flex;
  gap: 18px;
  width: 100%;
}

.row-sentinel {
  display: flex;
  flex-direction: column;
}

.capital-transfer-box {
  display: flex;
  gap: 14px;
  padding: 16px 20px;
  background: #f0f9ff;
  border: 1px solid #bae6fd;
  border-radius: 10px;
}

.box-icon {
  font-size: 20px;
}

.box-title {
  font-size: 13px;
  font-weight: 700;
  color: #0369a1;
  margin: 0 0 4px 0;
}

.box-text {
  font-size: 12px;
  color: #0c4a6e;
  line-height: 1.5;
  margin: 0;
}
</style>
