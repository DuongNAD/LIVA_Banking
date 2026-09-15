<script setup lang="ts">
/**
 * BankingSettingsView.vue — P100–P106 Trung Tâm Cài Đặt & Quản Trị Hệ Thống
 * =========================================================================
 * Trung tâm điều hành cấu hình LIVA Banking Harness:
 * - P101: Phân quyền RBAC & Tuân thủ Phân nhiệm (SoD) TT 09/2020
 * - P102: Cổng kết nối Open Banking API (VCB, TCB, BIDV, MBB)
 * - P103: Cầu nối đồng bộ ERP (MISA AMIS, FAST Accounting, SAP B1)
 * - P104: Tham số dung sai đối soát (Tier 1/2/3, TTL)
 * - P105: Bảo mật chữ ký số HSM & Khóa Zero Cloud Egress
 * - P106: Sao lưu dữ liệu Merkle Tree & Diễn tập DR
 */
import { useSettingsStore, type SettingsTab } from '../../stores/settingsStore';
import UserManagementTab from '../../components/banking/settings/UserManagementTab.vue';
import BankConnectorTab from '../../components/banking/settings/BankConnectorTab.vue';
import ErpBridgeTab from '../../components/banking/settings/ErpBridgeTab.vue';
import ThresholdConfigTab from '../../components/banking/settings/ThresholdConfigTab.vue';
import SecurityHsmTab from '../../components/banking/settings/SecurityHsmTab.vue';
import BackupRestoreTab from '../../components/banking/settings/BackupRestoreTab.vue';

const store = useSettingsStore();

const tabs: { id: SettingsTab; label: string; code: string }[] = [
  { id: 'USERS', label: 'Quản Trị Người Dùng & RBAC', code: 'P101' },
  { id: 'BANK_API', label: 'Open Banking B2B', code: 'P102' },
  { id: 'ERP_BRIDGE', label: 'Cầu Nối Đồng Bộ ERP', code: 'P103' },
  { id: 'THRESHOLDS', label: 'Tham Số Dung Sai Đối Soát', code: 'P104' },
  { id: 'SECURITY', label: 'Bảo Mật & Chữ Ký Số HSM', code: 'P105' },
  { id: 'BACKUP', label: 'Sao Lưu & Phục Hồi DR', code: 'P106' },
];
</script>

<template>
  <div class="banking-view-container">
    <!-- Header Banner -->
    <div class="settings-header-banner">
      <div class="banner-left">
        <div class="header-tags">
          <span class="p-title-pill">P100–P106 HỆ THỐNG & BẢO MẬT</span>
          <span class="integrity-badge">
            ✓ Thông tư 09/2020/TT-NHNN & NĐ 13/2023/NĐ-CP
          </span>
        </div>
        <h2 class="banner-title">Cài Đặt Cấu Hình Hệ Thống & Quản Trị Bảo Mật</h2>
        <p class="banner-desc">
          Quản lý tập trung phân quyền người dùng, hạ tầng kết nối ngân hàng/ERP và chính sách an toàn thông tin.
        </p>
      </div>

      <div class="banner-stats">
        <div class="stat-pill">
          <span class="label">Ngân hàng B2B:</span>
          <strong>{{ store.connectedBankCount }}/{{ store.bankConfigs.length }}</strong>
        </div>
        <div class="stat-pill">
          <span class="label">ERP Bridges:</span>
          <strong>{{ store.activeErpCount }}/{{ store.erpConfigs.length }}</strong>
        </div>
      </div>
    </div>

    <!-- Navigation Sub-tabs -->
    <div class="settings-subtabs">
      <button
        v-for="t in tabs"
        :key="t.id"
        class="subtab-btn"
        :class="{ active: store.activeTab === t.id }"
        @click="store.setActiveTab(t.id)"
      >
        <span class="tab-code">{{ t.code }}</span>
        <span>{{ t.label }}</span>
      </button>
    </div>

    <!-- Dynamic Subtab Content -->
    <div class="settings-content-wrap">
      <UserManagementTab v-if="store.activeTab === 'USERS'" />
      <BankConnectorTab v-else-if="store.activeTab === 'BANK_API'" />
      <ErpBridgeTab v-else-if="store.activeTab === 'ERP_BRIDGE'" />
      <ThresholdConfigTab v-else-if="store.activeTab === 'THRESHOLDS'" />
      <SecurityHsmTab v-else-if="store.activeTab === 'SECURITY'" />
      <BackupRestoreTab v-else-if="store.activeTab === 'BACKUP'" />
    </div>
  </div>
</template>

<style scoped>
.banking-view-container {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 24px 32px;
  max-width: 1600px;
  margin: 0 auto;
  width: 100%;
  box-sizing: border-box;
}

.settings-header-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 20px 24px;
  gap: 16px;
  flex-wrap: wrap;
}

.banner-left {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.header-tags {
  display: flex;
  align-items: center;
  gap: 10px;
}

.p-title-pill {
  font-size: 11px;
  font-weight: 700;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.12);
  border: 1px solid rgba(56, 189, 248, 0.3);
  padding: 2px 8px;
  border-radius: 4px;
}

.integrity-badge {
  font-size: 11px;
  font-weight: 700;
  color: #10b981;
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
  padding: 2px 8px;
  border-radius: 4px;
}

.banner-title {
  font-size: 20px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0;
}

.banner-desc {
  font-size: 13px;
  color: #94a3b8;
  margin: 0;
}

.banner-stats {
  display: flex;
  gap: 12px;
}

.stat-pill {
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.08);
  padding: 6px 12px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #94a3b8;
}

.stat-pill strong {
  color: #38bdf8;
  font-size: 13px;
}

.settings-subtabs {
  display: flex;
  gap: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  padding-bottom: 12px;
  overflow-x: auto;
}

.subtab-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.06);
  color: #94a3b8;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s;
}

.subtab-btn:hover {
  background: rgba(255, 255, 255, 0.06);
  color: #e2e8f0;
}

.subtab-btn.active {
  background: rgba(56, 189, 248, 0.15);
  border-color: rgba(56, 189, 248, 0.4);
  color: #f8fafc;
  font-weight: 600;
}

.tab-code {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 5px;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.08);
  color: #cbd5e1;
  font-family: monospace;
}

.subtab-btn.active .tab-code {
  background: #38bdf8;
  color: #0f172a;
}

.settings-content-wrap {
  display: flex;
  flex-direction: column;
  gap: 20px;
}
</style>
