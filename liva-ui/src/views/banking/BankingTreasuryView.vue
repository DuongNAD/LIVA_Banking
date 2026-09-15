<script setup lang="ts">
/**
 * BankingTreasuryView.vue — Màn hình Quản trị Ngân quỹ Tập trung (P60 & P62)
 * Giám sát vị thế số dư trên tất cả các tài khoản ngân hàng thương mại (VCB, TCB, BIDV, MBB),
 * cơ chế quét vốn tự động (Sweeping) và cổng phê duyệt lệnh chi kép (Maker-Checker Dual Control).
 */
import { ref } from 'vue';
import { useTreasuryStore } from '../../stores/treasuryStore';
import CashConcentrationGrid from '../../components/banking/treasury/CashConcentrationGrid.vue';
import PaymentDisbursementModal from '../../components/banking/treasury/PaymentDisbursementModal.vue';

const treasuryStore = useTreasuryStore();
const isPaymentModalOpen = ref(false);

function formatVnd(val: number): string {
  return `${val.toLocaleString('vi-VN')} ₫`;
}
</script>

<template>
  <div class="banking-view-container">
    <!-- Header Summary -->
    <div class="treasury-header-banner">
      <div>
        <div class="title-with-pill">
          <span class="p-tag">P60–P62</span>
          <h2 class="banner-title">Quản Trị Ngân Quỹ & Tập Trung Vốn Đa Ngân Hàng</h2>
        </div>
        <p class="banner-desc">
          Theo dõi số dư thanh toán đa ngân hàng theo thời gian thực. Tổng tiền mặt khả dụng toàn hệ thống:
          <strong class="text-green">{{ formatVnd(treasuryStore.totalCashPosition) }}</strong>.
        </p>
      </div>

      <div class="header-actions">
        <!-- Open Payment Modal -->
        <button class="btn btn-payment-action" @click="isPaymentModalOpen = true">
          <span>✍️ Lệnh Chi Tiền & Dual Control ({{ treasuryStore.paymentOrders.length }})</span>
        </button>
      </div>
    </div>

    <!-- P60 Cash Concentration Multi-Bank Grid -->
    <div class="section-title">Vị Thế Số Dư & Quét Vốn Đa Ngân Hàng (Cash Concentration Pool)</div>
    <CashConcentrationGrid />

    <!-- Payment Orders Modal -->
    <PaymentDisbursementModal
      :is-open="isPaymentModalOpen"
      @close="isPaymentModalOpen = false"
    />
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

.title-with-pill {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 4px;
}

.p-tag {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 800;
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #bfdbfe;
  padding: 2px 6px;
  border-radius: 4px;
}

.banner-title {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.banner-desc {
  font-size: 13px;
  color: #64748b;
  margin: 0;
}

.text-green {
  color: #10b981;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.btn-payment-action {
  background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
  color: #ffffff;
  border: 1px solid rgba(255, 255, 255, 0.1);
  padding: 10px 18px;
  font-size: 13px;
  font-weight: 600;
  border-radius: 8px;
  cursor: pointer;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
  transition: all 0.2s ease;
}

.btn-payment-action:hover {
  background: #334155;
  transform: translateY(-1px);
}

.section-title {
  font-size: 13px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #475569;
}
</style>
