<script setup lang="ts">
/**
 * UserManagementTab.vue — P101 Quản Trị Người Dùng & Phân Quyền RBAC
 * ====================================================================
 * Quản lý danh sách tài khoản, vai trò và bảo đảm Quy tắc Phân nhiệm
 * Segregation of Duties (SoD) theo Thông tư 09/2020/TT-NHNN.
 */
import { useSettingsStore, type AppUserRole } from '../../../stores/settingsStore';

const store = useSettingsStore();

function getRoleBadgeClass(role: AppUserRole): string {
  switch (role) {
    case 'ADMIN': return 'role-admin';
    case 'MAKER': return 'role-maker';
    case 'CHECKER': return 'role-checker';
    case 'COMPLIANCE': return 'role-compliance';
    case 'AUDITOR': return 'role-auditor';
    default: return '';
  }
}
</script>

<template>
  <div class="settings-card">
    <div class="card-header">
      <div>
        <div class="header-tags">
          <span class="p-tag">P101 THÔNG TƯ 09/2020/TT-NHNN</span>
          <span v-if="!store.hasSodViolation" class="status-pill ok">
            ✓ Tuân Thủ Phân Nhiệm (SoD Compliant)
          </span>
          <span v-else class="status-pill warn">
            ⚠ Vi Phạm Phân Nhiệm (SoD Violation)
          </span>
        </div>
        <h3 class="card-title">Quản Trị Người Dùng & Ma Trận Phân Quyền RBAC</h3>
        <p class="card-desc">
          Phân định ranh giới nghiêm ngặt giữa Người Lập Lệnh (Maker) và Người Duyệt Lệnh (Checker).
        </p>
      </div>
    </div>

    <!-- Users Table -->
    <div class="table-wrap">
      <table class="settings-table">
        <thead>
          <tr>
            <th>Mã NV</th>
            <th>Tên Đăng Nhập</th>
            <th>Họ Và Tên</th>
            <th>Vai Trò Nghiệp Vụ</th>
            <th>Chứng Thư Số (CA)</th>
            <th>Lần Đăng Nhập Cuối</th>
            <th>Trạng Thái</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="u in store.users" :key="u.id">
            <td class="code-cell font-mono">{{ u.id }}</td>
            <td class="font-mono text-accent">{{ u.username }}</td>
            <td>{{ u.fullName }}</td>
            <td>
              <span class="role-badge" :class="getRoleBadgeClass(u.role)">
                {{ u.role }}
              </span>
            </td>
            <td class="font-mono text-muted">{{ u.certSerial || '-' }}</td>
            <td class="text-muted">{{ u.lastLogin }}</td>
            <td>
              <span class="active-badge" :class="{ active: u.isActive }">
                {{ u.isActive ? 'Hoạt động' : 'Tạm khóa' }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- SoD Footnote -->
    <div class="sod-guideline-card">
      <div class="guide-title">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
          <path d="m9 12 2 2 4-4" />
        </svg>
        <span>Nguyên Tắc Bất Biến Về Phân Quyền Kiểm Soát Kép:</span>
      </div>
      <p class="guide-text">
        Hệ thống tự động chặn mọi nỗ lực gán đồng thời vai trò <code>MAKER</code> và <code>CHECKER</code> cho cùng một tài khoản.
        Mọi đề xuất đối soát từ hàng đợi cách ly P44 và lệnh chi ngân quỹ P62 bắt buộc phải được ký duyệt bởi 2 cá nhân hoàn toàn độc lập (<code>maker_user_id &ne; checker_user_id</code>).
      </p>
    </div>
  </div>
</template>

<style scoped>
.settings-card {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.header-tags {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.p-tag {
  font-size: 11px;
  font-weight: 700;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.12);
  border: 1px solid rgba(56, 189, 248, 0.3);
  padding: 2px 8px;
  border-radius: 4px;
}

.status-pill {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 4px;
}

.status-pill.ok {
  color: #10b981;
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.status-pill.warn {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.12);
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.card-title {
  font-size: 16px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0 0 4px 0;
}

.card-desc {
  font-size: 12px;
  color: #94a3b8;
  margin: 0;
}

.table-wrap {
  overflow-x: auto;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
}

.settings-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
  color: #e2e8f0;
}

.settings-table th,
.settings-table td {
  padding: 10px 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.settings-table th {
  background: #0f172a;
  color: #94a3b8;
  font-weight: 600;
  text-align: left;
}

.code-cell {
  color: #64748b;
}

.text-accent {
  color: #38bdf8;
}

.text-muted {
  color: #64748b;
  font-size: 11px;
}

.role-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 4px;
}

.role-admin {
  background: rgba(147, 51, 234, 0.15);
  color: #c084fc;
  border: 1px solid rgba(147, 51, 234, 0.3);
}

.role-maker {
  background: rgba(56, 189, 248, 0.15);
  color: #38bdf8;
  border: 1px solid rgba(56, 189, 248, 0.3);
}

.role-checker {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.role-compliance {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.role-auditor {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.active-badge {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(100, 116, 139, 0.2);
  color: #94a3b8;
}

.active-badge.active {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
}

.sod-guideline-card {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 14px;
}

.guide-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 700;
  color: #f1f5f9;
  margin-bottom: 6px;
}

.guide-text {
  font-size: 12px;
  color: #94a3b8;
  margin: 0;
  line-height: 1.5;
}

.guide-text code {
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.1);
  padding: 1px 4px;
  border-radius: 3px;
}
</style>
