<script setup lang="ts">
/**
 * BankingComplianceView.vue — Màn hình Giám Sát Tuân Thủ & PCRT (P80–P85)
 * =========================================================================
 * Trung tâm kiểm soát tuân thủ Nghị định 13/2023/NĐ-CP (Zero Cloud Egress),
 * rà soát rửa tiền (AML/STR) theo Quyết định 11/2023/QĐ-TTg,
 * và bảo chứng sổ cái mật mã Merkle Audit Tree.
 */
import { ref } from 'vue';
import { useComplianceStore } from '../../stores/complianceStore';
import AmlAlertList from '../../components/banking/compliance/AmlAlertList.vue';
import MerkleAuditCard from '../../components/banking/compliance/MerkleAuditCard.vue';
import PiiSanitizerPreviewCard from '../../components/banking/compliance/PiiSanitizerPreviewCard.vue';
import StrReportModal from '../../components/banking/compliance/StrReportModal.vue';

const store = useComplianceStore();
const activeSubTab = ref<'ALERTS' | 'MERKLE' | 'PII'>('ALERTS');
</script>

<template>
  <div class="banking-view-container">
    <!-- Header Compliance Banner -->
    <div class="compliance-header-banner">
      <div class="banner-left">
        <div class="title-with-pill">
          <span class="p-tag">P80–P85</span>
          <h2 class="banner-title">Giám Sát Tuân Thủ AML / CTF & Bảo Vệ Dữ Liệu Cá Nhân</h2>
        </div>
        <p class="banner-desc">
          Thực thi Nghị định 13/2023/NĐ-CP, Quyết định 11/2023/QĐ-TTg và bảo chứng sổ cái bất biến Merkle Tree.
        </p>
      </div>

      <div class="banner-badges">
        <span class="badge egress-badge">
          <span class="dot-pulse" />
          100% Cục Bộ (Zero Cloud Egress)
        </span>
        <span class="badge authority-badge">
          Cục PCRT — NHNN
        </span>
      </div>
    </div>

    <!-- Sub-tab Navigation Switcher -->
    <div class="tab-switcher">
      <button
        class="tab-btn"
        :class="{ active: activeSubTab === 'ALERTS' }"
        @click="activeSubTab = 'ALERTS'"
      >
        <span>⚠️ Cảnh Báo AML & Lập Mẫu STR</span>
        <span class="tab-count">{{ store.alerts.length }}</span>
      </button>

      <button
        class="tab-btn"
        :class="{ active: activeSubTab === 'MERKLE' }"
        @click="activeSubTab = 'MERKLE'"
      >
        <span>🛡️ Bảo Chứng Mật Mã (Merkle Tree)</span>
      </button>

      <button
        class="tab-btn"
        :class="{ active: activeSubTab === 'PII' }"
        @click="activeSubTab = 'PII'"
      >
        <span>🔒 Bộ Lọc Nghị Định 13 (PII Sanitizer)</span>
      </button>
    </div>

    <!-- Tab 1: AML Alerts -->
    <div v-if="activeSubTab === 'ALERTS'" class="tab-content">
      <AmlAlertList />
    </div>

    <!-- Tab 2: Merkle Audit -->
    <div v-else-if="activeSubTab === 'MERKLE'" class="tab-content">
      <MerkleAuditCard />
    </div>

    <!-- Tab 3: PII Sanitizer -->
    <div v-else-if="activeSubTab === 'PII'" class="tab-content">
      <PiiSanitizerPreviewCard />
    </div>

    <!-- Modal Form STR -->
    <StrReportModal
      :is-open="!!store.activeStrModal"
      @close="store.closeStrModal()"
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

.compliance-header-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 18px 24px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
  flex-wrap: wrap;
  gap: 14px;
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

.banner-badges {
  display: flex;
  align-items: center;
  gap: 10px;
}

.badge {
  font-size: 11px;
  font-weight: 700;
  padding: 6px 12px;
  border-radius: 20px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.egress-badge {
  background: #dcfce7;
  color: #15803d;
  border: 1px solid #86efac;
}

.authority-badge {
  background: #eff6ff;
  color: #1d4ed8;
  border: 1px solid #bfdbfe;
}

.dot-pulse {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #10b981;
  box-shadow: 0 0 6px #10b981;
}

.tab-switcher {
  display: flex;
  gap: 10px;
  border-bottom: 1px solid #e2e8f0;
  padding-bottom: 4px;
}

.tab-btn {
  background: transparent;
  border: none;
  font-size: 13px;
  font-weight: 600;
  color: #64748b;
  padding: 8px 16px;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s ease;
}

.tab-btn:hover {
  background: #f1f5f9;
  color: #1e293b;
}

.tab-btn.active {
  background: #0f172a;
  color: #ffffff;
}

.tab-count {
  background: #ef4444;
  color: #ffffff;
  font-size: 10px;
  font-weight: 800;
  padding: 1px 6px;
  border-radius: 10px;
}

.tab-btn.active .tab-count {
  background: rgba(255, 255, 255, 0.2);
}

.tab-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
</style>
