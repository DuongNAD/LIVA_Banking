import fs from 'node:fs';
import path from 'node:path';

const wbsPath = path.resolve('teamwork_projects/liva_banking_harness/WBS_PHASE_0_1.md');
const content = fs.readFileSync(wbsPath, 'utf-8');

console.log('═══════════════════════════════════════════════════════');
console.log('         LIVA WBS PHASE 0 & 1 EMPIRICAL AUDIT          ');
console.log('═══════════════════════════════════════════════════════\n');

// Split into task blocks
const taskBlocks = content.split(/#### \[(WBS \d+\.\d+\.\d+)\]/);
const validRoles = new Set([
  'Solution Architect (SA)',
  'Lead Rust Engineer (RE1)',
  'Senior Rust Systems Engineer (RE2)',
  'AI & SLM Engineer (AIE)',
  'Full-stack Engineer (FSE)',
  'Security Engineer (SE)',
  'QA / SDET Engineer (QAE)',
  'Kế toán trưởng (KTT / SME)',
  'Tư vấn Pháp lý (TVPL / LC)'
]);

const tasks = new Map();
const roleLeadMd = {};
const roleTaskCount = {};
for (const r of validRoles) {
  roleLeadMd[r] = { GĐ0: 0, GĐ1: 0, total: 0 };
  roleTaskCount[r] = { GĐ0: 0, GĐ1: 0, total: 0 };
}

let fieldErrors = [];
let invalidRoleErrors = [];
let mdRangeErrors = [];

for (let i = 1; i < taskBlocks.length; i += 2) {
  const tid = taskBlocks[i].trim();
  const block = taskBlocks[i + 1];

  const nameMatch = block.match(/^\s*([^\n]+)/);
  const primaryMatch = block.match(/\*\*Vai trò phụ trách chính\*\*:\s*([^\n]+)/);
  const coordMatch = block.match(/\*\*Vai trò phối hợp\*\*:\s*([^\n]+)/);
  const mdMatch = block.match(/\*\*Ước lượng ngày công\*\*:\s*(\d+)\s*Man-days/);
  const depMatch = block.match(/\*\*Task phụ thuộc\*\*:\s*([^\n]+)/);
  const delivMatch = block.match(/\*\*Sản phẩm bàn giao\*\*:\s*([^\n]+)/);
  const criteriaMatch = block.match(/\*\*Tiêu chí nghiệm thu kỹ thuật\*\*:\s*([\s\S]+?)(?=\n---|\n####|\n##|$)/);

  const name = nameMatch ? nameMatch[1].trim() : '';
  const primary = primaryMatch ? primaryMatch[1].trim() : '';
  const coord = coordMatch ? coordMatch[1].trim() : '';
  const md = mdMatch ? parseInt(mdMatch[1], 10) : 0;
  const rawDeps = depMatch ? depMatch[1].trim() : '';
  const deps = Array.from(rawDeps.matchAll(/WBS \d+\.\d+\.\d+/g)).map(m => m[0]);
  const hasDeliverable = Boolean(delivMatch);
  const hasCriteria = Boolean(criteriaMatch);

  // Field completeness
  const missing = [];
  if (!name) missing.push('Tên task');
  if (!primary) missing.push('Vai trò phụ trách chính');
  if (!coord) missing.push('Vai trò phối hợp');
  if (!mdMatch) missing.push('Ước lượng ngày công');
  if (!depMatch) missing.push('Task phụ thuộc');
  if (!hasDeliverable) missing.push('Sản phẩm bàn giao');
  if (!hasCriteria) missing.push('Tiêu chí nghiệm thu kỹ thuật');

  if (missing.length > 0) {
    fieldErrors.push({ tid, missing });
  }

  // Role validation
  if (!validRoles.has(primary)) {
    invalidRoleErrors.push({ tid, primary });
  }

  // Man-days bounds
  if (md < 1 || md > 5) {
    mdRangeErrors.push({ tid, md });
  }

  const phase = tid.startsWith('WBS 0.') ? 'GĐ0' : 'GĐ1';
  if (validRoles.has(primary)) {
    roleLeadMd[primary][phase] += md;
    roleLeadMd[primary].total += md;
    roleTaskCount[primary][phase] += 1;
    roleTaskCount[primary].total += 1;
  }

  tasks.set(tid, {
    id: tid,
    name,
    primary,
    coord,
    md,
    rawDeps,
    deps,
    phase,
    block
  });
}

console.log(`[1] TỔNG QUAN CÁC TASK:`);
console.log(`- Tổng số task phát hiện: ${tasks.size} (Kỳ vọng: 49)`);
const gd0Count = Array.from(tasks.values()).filter(t => t.phase === 'GĐ0').length;
const gd1Count = Array.from(tasks.values()).filter(t => t.phase === 'GĐ1').length;
console.log(`  * GĐ0: ${gd0Count} tasks (Kỳ vọng: 29)`);
console.log(`  * GĐ1: ${gd1Count} tasks (Kỳ vọng: 20)`);
console.log(`- Lỗi thiếu trường thông tin bắt buộc: ${fieldErrors.length}`);
console.log(`- Lỗi vai trò không hợp lệ: ${invalidRoleErrors.length}`);
console.log(`- Lỗi ngày công ngoài khoảng [1..5]: ${mdRangeErrors.length}\n`);

// 2. Dependency Analysis: Dangling & Cycles
console.log(`[2] PHÂN TÍCH ĐỒ THỊ PHỤ THUỘC (DEPENDENCY GRAPH AUDIT):`);
const dangling = [];
for (const [tid, t] of tasks) {
  for (const dep of t.deps) {
    if (!tasks.has(dep)) {
      dangling.push({ tid, dep });
    }
  }
}
console.log(`- Phụ thuộc treo (Dangling Dependencies): ${dangling.length}`);
if (dangling.length > 0) {
  for (const d of dangling) {
    console.log(`  ! ${d.tid} phụ thuộc vào ${d.dep} (không tồn tại)`);
  }
}

// Cycle check
const visited = new Map(); // 0: unvisited, 1: visiting, 2: visited
const cycles = [];
function dfs(u, path) {
  visited.set(u, 1);
  const t = tasks.get(u);
  if (t) {
    for (const v of t.deps) {
      if (!tasks.has(v)) continue;
      const vState = visited.get(v) || 0;
      if (vState === 1) {
        const idx = path.indexOf(v);
        cycles.push([...path.slice(idx), v]);
      } else if (vState === 0) {
        dfs(v, [...path, v]);
      }
    }
  }
  visited.set(u, 2);
}

for (const tid of tasks.keys()) {
  if ((visited.get(tid) || 0) === 0) {
    dfs(tid, [tid]);
  }
}
console.log(`- Phụ thuộc vòng (Circular Dependencies / Cycles): ${cycles.length}`);
if (cycles.length > 0) {
  for (const c of cycles) {
    console.log(`  ! Vòng: ${c.join(' -> ')}`);
  }
}

// Cross-phase dependencies
console.log(`\n[3] PHÂN TÍCH PHỤ THUỘC LIÊN GIAI ĐOẠN (CROSS-PHASE DEPENDENCIES):`);
const crossPhase = [];
for (const [tid, t] of tasks) {
  if (t.phase === 'GĐ1') {
    const p0Deps = t.deps.filter(d => d.startsWith('WBS 0.'));
    if (p0Deps.length > 0) {
      crossPhase.push({ tid, name: t.name, p0Deps });
    }
  }
}
for (const cp of crossPhase) {
  console.log(`- ${cp.tid} (${cp.name}) phụ thuộc vào GĐ0: [${cp.p0Deps.join(', ')}]`);
}

// 4. Man-Days comparison between Task calculation and Section 4.1 table
console.log(`\n[4] ĐỐI CHIẾU NGÀY CÔNG (MAN-DAYS ROLLUP RECONCILIATION):`);
let gd0LeadSum = 0;
let gd1LeadSum = 0;
for (const t of tasks.values()) {
  if (t.phase === 'GĐ0') gd0LeadSum += t.md;
  else gd1LeadSum += t.md;
}
console.log(`- Tổng Lead MD tính từ chi tiết task:`);
console.log(`  * GĐ0 Lead MD: ${gd0LeadSum} MD (Bảng 4.1 ghi: 94 MD) -> ${gd0LeadSum === 94 ? 'KHỚP' : 'LỆCH'}`);
console.log(`  * GĐ1 Lead MD: ${gd1LeadSum} MD (Bảng 4.1 ghi: 79 MD) -> ${gd1LeadSum === 79 ? 'KHỚP' : 'LỆCH (' + (gd1LeadSum - 79) + ' MD)'}`);

console.log(`\n- Chi tiết Lead MD theo từng vai trò (Chi tiết task vs Bảng 4.1):`);
const tableValues = {
  'Solution Architect (SA)': { GĐ0: 15, GĐ1: 3 },
  'Lead Rust Engineer (RE1)': { GĐ0: 3, GĐ1: 25 },
  'Senior Rust Systems Engineer (RE2)': { GĐ0: 17, GĐ1: 21 },
  'AI & SLM Engineer (AIE)': { GĐ0: 4, GĐ1: 0 },
  'Full-stack Engineer (FSE)': { GĐ0: 5, GĐ1: 13 },
  'Security Engineer (SE)': { GĐ0: 20, GĐ1: 7 },
  'QA / SDET Engineer (QAE)': { GĐ0: 0, GĐ1: 6 },
  'Kế toán trưởng (KTT / SME)': { GĐ0: 17, GĐ1: 4 },
  'Tư vấn Pháp lý (TVPL / LC)': { GĐ0: 13, GĐ1: 0 }
};

console.log('| Vai Trò Nhân Lực                  | GĐ0 Thực | GĐ0 Bảng | Lệch GĐ0 | GĐ1 Thực | GĐ1 Bảng | Lệch GĐ1 |');
console.log('|-----------------------------------|----------|----------|----------|----------|----------|----------|');
for (const r of validRoles) {
  const actual0 = roleLeadMd[r].GĐ0;
  const table0 = tableValues[r].GĐ0;
  const diff0 = actual0 - table0;
  const actual1 = roleLeadMd[r].GĐ1;
  const table1 = tableValues[r].GĐ1;
  const diff1 = actual1 - table1;
  console.log(`| ${r.padEnd(33)} | ${String(actual0).padStart(8)} | ${String(table0).padStart(8)} | ${String(diff0).padStart(8)} | ${String(actual1).padStart(8)} | ${String(table1).padStart(8)} | ${String(diff1).padStart(8)} |`);
}
console.log('|-----------------------------------|----------|----------|----------|----------|----------|----------|');
console.log(`| TỔNG CỘNG TOÀN BỘ                 | ${String(gd0LeadSum).padStart(8)} |       94 | ${String(gd0LeadSum - 94).padStart(8)} | ${String(gd1LeadSum).padStart(8)} |       79 | ${String(gd1LeadSum - 79).padStart(8)} |`);
