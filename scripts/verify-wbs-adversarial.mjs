import fs from 'node:fs';
import path from 'node:path';

const wbsPath = path.resolve('teamwork_projects/liva_banking_harness/WBS_PHASE_0_1.md');
const content = fs.readFileSync(wbsPath, 'utf-8');

console.log('═══════════════════════════════════════════════════════════════');
console.log('       EMPIRICAL CHALLENGER: ADVERSARIAL WBS VERIFICATION      ');
console.log('═══════════════════════════════════════════════════════════════\n');

let totalTests = 0;
let passedTests = 0;
let failedTests = 0;

function assert(condition, message, details = '') {
  totalTests++;
  if (condition) {
    passedTests++;
    console.log(`  [PASS] ${message}`);
  } else {
    failedTests++;
    console.error(`  [FAIL] ${message}`);
    if (details) console.error(`         Detail: ${details}`);
  }
}

// -------------------------------------------------------------
// SUITE 1: PARSE ALL 49 TASKS & FIELD BOUNDS
// -------------------------------------------------------------
console.log('[SUITE 1] Kiểm tra Chi tiết 49 Tasks & Tính toàn vẹn Trường dữ liệu');

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
const roleLeadMdFromTasks = {};
for (const r of validRoles) {
  roleLeadMdFromTasks[r] = { GĐ0: 0, GĐ1: 0, total: 0 };
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

  const missing = [];
  if (!name) missing.push('Tên task');
  if (!primary) missing.push('Vai trò phụ trách chính');
  if (!coord) missing.push('Vai trò phối hợp');
  if (!mdMatch) missing.push('Ước lượng ngày công');
  if (!depMatch) missing.push('Task phụ thuộc');
  if (!hasDeliverable) missing.push('Sản phẩm bàn giao');
  if (!hasCriteria) missing.push('Tiêu chí nghiệm thu kỹ thuật');

  if (missing.length > 0) fieldErrors.push({ tid, missing });
  if (!validRoles.has(primary)) invalidRoleErrors.push({ tid, primary });
  if (md < 1 || md > 5) mdRangeErrors.push({ tid, md });

  const phase = tid.startsWith('WBS 0.') ? 'GĐ0' : 'GĐ1';
  if (validRoles.has(primary)) {
    roleLeadMdFromTasks[primary][phase] += md;
    roleLeadMdFromTasks[primary].total += md;
  }

  tasks.set(tid, { id: tid, name, primary, coord, md, deps, phase, block });
}

assert(tasks.size === 49, `Tổng số task đúng 49 (thực tế: ${tasks.size})`);
const gd0Tasks = Array.from(tasks.values()).filter(t => t.phase === 'GĐ0');
const gd1Tasks = Array.from(tasks.values()).filter(t => t.phase === 'GĐ1');
assert(gd0Tasks.length === 29, `GĐ0 có 29 tasks (thực tế: ${gd0Tasks.length})`);
assert(gd1Tasks.length === 20, `GĐ1 có 20 tasks (thực tế: ${gd1Tasks.length})`);
assert(fieldErrors.length === 0, `0 lỗi thiếu trường bắt buộc (thực tế: ${fieldErrors.length})`);
assert(invalidRoleErrors.length === 0, `0 vai trò không hợp lệ (thực tế: ${invalidRoleErrors.length})`);
assert(mdRangeErrors.length === 0, `0 lỗi ngày công ngoài dải [1..5] (thực tế: ${mdRangeErrors.length})`);

// -------------------------------------------------------------
// SUITE 2: DAG DEPENDENCIES, CYCLES & SE SCHEDULING REALISM
// -------------------------------------------------------------
console.log('\n[SUITE 2] Phân tích Đồ thị Phụ thuộc (DAG) & Tính khả thi Lịch trình SE');

const dangling = [];
for (const [tid, t] of tasks) {
  for (const dep of t.deps) {
    if (!tasks.has(dep)) dangling.push({ tid, dep });
  }
}
assert(dangling.length === 0, `0 phụ thuộc treo (dangling dependencies) (thực tế: ${dangling.length})`);

const visited = new Map();
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
  if ((visited.get(tid) || 0) === 0) dfs(tid, [tid]);
}
assert(cycles.length === 0, `0 vòng lặp phụ thuộc (circular dependency) (thực tế: ${cycles.length})`);

// Stress check SE schedule:
// SE tasks in GĐ0:
const seGd0Tasks = gd0Tasks.filter(t => t.primary.includes('Security'));
const seGd0LeadMd = seGd0Tasks.reduce((s, t) => s + t.md, 0);
assert(seGd0LeadMd === 20, `SE Lead GĐ0 = 20 MD (4 tuần x 5 ngày = 100% công suất) (thực tế: ${seGd0LeadMd})`);

// Verify WBS 1.4.1 dependency on GĐ0 (WBS 0.6.3, WBS 0.7.1)
const t141 = tasks.get('WBS 1.4.1');
assert(t141 !== undefined, 'Task WBS 1.4.1 tồn tại');
assert(t141.deps.includes('WBS 0.6.3') && t141.deps.includes('WBS 0.7.1'),
  `WBS 1.4.1 phụ thuộc chuẩn WBS 0.6.3 và WBS 0.7.1 (thực tế: ${t141.deps.join(', ')})`);

// Verify text mentions SE starting Luồng 1.4 in Week 5
const seWeek5Check = content.includes('Luồng 1.4') && 
  (content.includes('bắt đầu từ Tuần 5') || content.includes('khởi động từ đầu Tuần 5'));
assert(seWeek5Check, 'Văn bản xác định rõ SE bắt đầu Luồng 1.4 từ Tuần 5 (tránh xung đột GĐ0)');

// -------------------------------------------------------------
// SUITE 3: PARSE & RECONCILE TABLE 4.1 (MAN-DAYS ROLLUP)
// -------------------------------------------------------------
console.log('\n[SUITE 3] Thẩm định Bảng 4.1: Tính toán Động & Khử Phantom Coordination');

// Extract Table 4.1 rows from markdown text
const t41Regex = /\|\s*([A-Za-zÀ-ỹ\s/\-]+?\(([A-Z0-9]+)\))\s*\|\s*([0-9.]+)\s*FTE\s*\|\s*(\d+)\s*MD\s*\/\s*(\d+)\s*MD\s*\/\s*(\d+)\s*\|\s*(\d+)\s*MD\s*\/\s*(\d+)\s*MD\s*\/\s*(\d+)\s*\|\s*(\d+)\s*Man-days\s*\/\s*(\d+)\s*MD\s*\/\s*(\d+)\s*MD\s*\|/g;

const table41Parsed = {};
let match;
while ((match = t41Regex.exec(content)) !== null) {
  const code = match[2]; // e.g. SA, RE1, RE2, AIE, FSE, SE, QAE, KTT, TVPL
  let fullRole = Array.from(validRoles).find(r => r.includes(`(${code}`) || r.includes(`/${code}`));
  table41Parsed[fullRole] = {
    code,
    fte: parseFloat(match[3]),
    gd0Lead: parseInt(match[4], 10),
    gd0Coord: parseInt(match[5], 10),
    gd0Total: parseInt(match[6], 10),
    gd1Lead: parseInt(match[7], 10),
    gd1Coord: parseInt(match[8], 10),
    gd1Total: parseInt(match[9], 10),
    totalLead: parseInt(match[10], 10),
    totalCoord: parseInt(match[11], 10),
    totalOverall: parseInt(match[12], 10)
  };
}

assert(Object.keys(table41Parsed).length === 9, `Bảng 4.1 trích xuất đủ 9 vai trò (thực tế: ${Object.keys(table41Parsed).length})`);

// Calculate task-level actuals
let actualGd0LeadSum = 0;
let actualGd1LeadSum = 0;
for (const [r, d] of Object.entries(roleLeadMdFromTasks)) {
  actualGd0LeadSum += d.GĐ0;
  actualGd1LeadSum += d.GĐ1;
}

assert(actualGd0LeadSum === 94, `Tổng Lead GĐ0 từ 29 tasks = 94 MD (thực tế: ${actualGd0LeadSum})`);
assert(actualGd1LeadSum === 73, `Tổng Lead GĐ1 từ 20 tasks = 73 MD (thực tế: ${actualGd1LeadSum})`);
assert(actualGd0LeadSum + actualGd1LeadSum === 167, `Tổng Lead 2 giai đoạn = 167 MD (thực tế: ${actualGd0LeadSum + actualGd1LeadSum})`);

// Check each role 1:1 match between Task actuals and Table 4.1
let allRolesMatch = true;
for (const r of validRoles) {
  const tRow = table41Parsed[r];
  const actual0 = roleLeadMdFromTasks[r].GĐ0;
  const actual1 = roleLeadMdFromTasks[r].GĐ1;
  const match0 = tRow && tRow.gd0Lead === actual0;
  const match1 = tRow && tRow.gd1Lead === actual1;
  const matchTotal = tRow && tRow.totalLead === (actual0 + actual1);
  if (!match0 || !match1 || !matchTotal) {
    allRolesMatch = false;
    console.error(`  Mismatch for ${r}: Task=(G0:${actual0}, G1:${actual1}), Table=(G0:${tRow?.gd0Lead}, G1:${tRow?.gd1Lead})`);
  }
}
assert(allRolesMatch, 'Mọi vai trò (9/9) có Lead MD khớp 1:1 tuyệt đối giữa Chi tiết Task và Bảng 4.1');

// Verify Table 4.1 Totals row
const totalRowMatch = content.match(/\|\s*TỔNG CỘNG TOÀN ĐỘI\s*\|\s*([0-9.]+)\s*FTE\s*\|\s*(\d+)\s*MD\s*\/\s*(\d+)\s*MD\s*\/\s*(\d+)\s*\|\s*(\d+)\s*MD\s*\/\s*(\d+)\s*MD\s*\/\s*(\d+)\s*\|\s*(\d+)\s*MD\s*\/\s*(\d+)\s*MD\s*\/\s*(\d+)\s*MD\s*\|/);
assert(Boolean(totalRowMatch), 'Tìm thấy dòng TỔNG CỘNG TOÀN ĐỘI trong Bảng 4.1');
if (totalRowMatch) {
  const [_, fte, g0L, g0C, g0T, g1L, g1C, g1T, tL, tC, tAll] = totalRowMatch;
  assert(parseInt(g0L, 10) === 94, `Bảng 4.1 Tổng Lead GĐ0 ghi 94 MD (thực tế: ${g0L})`);
  assert(parseInt(g0C, 10) === 46, `Bảng 4.1 Tổng Phối GĐ0 ghi 46 MD (thực tế: ${g0C})`);
  assert(parseInt(g0T, 10) === 140, `Bảng 4.1 Tổng GĐ0 ghi 140 MD (thực tế: ${g0T})`);
  assert(parseInt(g1L, 10) === 73, `Bảng 4.1 Tổng Lead GĐ1 ghi 73 MD (thực tế: ${g1L})`);
  assert(parseInt(g1C, 10) === 126, `Bảng 4.1 Tổng Phối GĐ1 ghi 126 MD (thực tế: ${g1C})`);
  assert(parseInt(g1T, 10) === 199, `Bảng 4.1 Tổng GĐ1 ghi 199 MD (thực tế: ${g1T})`);
  assert(parseInt(tL, 10) === 167, `Bảng 4.1 Tổng Lead cả dự án ghi 167 MD (thực tế: ${tL})`);
  assert(parseInt(tC, 10) === 172, `Bảng 4.1 Tổng Phối cả dự án ghi 172 MD (thực tế: ${tC})`);
  assert(parseInt(tAll, 10) === 339, `Bảng 4.1 Tổng Phân Bổ cả dự án ghi 339 MD (thực tế: ${tAll})`);
}

// Check phantom coordination for AIE in GĐ1
const aieRow = table41Parsed['AI & SLM Engineer (AIE)'];
assert(aieRow.gd1Lead === 0 && aieRow.gd1Coord === 0 && aieRow.gd1Total === 0,
  `AIE trong GĐ1 có 0 MD Lead, 0 MD Phối (Triệt tiêu hoàn toàn phantom coordination)`);

// Check all GĐ1 task blocks: ensure AIE is NOT mentioned in any coordination or primary role
let aieMentionedInGd1 = false;
for (const t of gd1Tasks) {
  const cleanBlock = t.block.split(/\n---|\n##/)[0];
  if (cleanBlock.includes('AI & SLM') || cleanBlock.includes('(AIE)')) {
    aieMentionedInGd1 = true;
    console.error(`  Warning: AIE mentioned in ${t.id}`);
  }
}
assert(!aieMentionedInGd1, 'Không có task nào trong 20 task GĐ1 gán AIE (0 phantom assignment)');

// -------------------------------------------------------------
// SUITE 4: PARSE & RECONCILE TABLE 4.2 (CAPACITY & CONTINGENCY MATH)
// -------------------------------------------------------------
console.log('\n[SUITE 4] Thẩm định Bảng 4.2: Toán học Công suất Lịch biểu & Quỹ Dự phòng');

// Table 4.2 regex
const t42Regex = /\|\s*\d+\.\s*([^|]+)\|\s*([0-9.]+)\s*FTE\s*\|\s*(\d+)\s*MD\s*\|\s*(\d+)\s*MD\s*\|\s*(\d+)\s*MD\s*\|\s*(\d+)\s*MD\s*\|\s*([0-9.]+)\s*%\s*\|\s*(\d+)\s*MD\s*\|/g;
const table42Rows = [];
let m42;
while ((m42 = t42Regex.exec(content)) !== null) {
  table42Rows.push({
    role: m42[1].trim(),
    fte: parseFloat(m42[2]),
    gd0: parseInt(m42[3], 10),
    gd1: parseInt(m42[4], 10),
    total: parseInt(m42[5], 10),
    cap: parseInt(m42[6], 10),
    util: parseFloat(m42[7]),
    buffer: parseInt(m42[8], 10)
  });
}

assert(table42Rows.length === 9, `Bảng 4.2 có đủ 9 vai trò (thực tế: ${table42Rows.length})`);

// Verify capacity math for each role in Table 4.2
let sumFte = 0;
let sumGd0 = 0;
let sumGd1 = 0;
let sumTotal = 0;
let sumCap = 0;
let sumBuffer = 0;

for (const row of table42Rows) {
  sumFte += row.fte;
  sumGd0 += row.gd0;
  sumGd1 += row.gd1;
  sumTotal += row.total;
  sumCap += row.cap;
  sumBuffer += row.buffer;

  // Individual role invariants
  assert(row.total === row.gd0 + row.gd1, `${row.role}: Total (${row.total}) === G0 (${row.gd0}) + G1 (${row.gd1})`);
  assert(row.cap === Math.round(row.fte * 50), `${row.role}: Cap (${row.cap}) === FTE (${row.fte}) * 50 days`);
  assert(row.buffer === row.cap - row.total, `${row.role}: Buffer (${row.buffer}) === Cap (${row.cap}) - Total (${row.total})`);
  const expectedUtil = (row.total / row.cap) * 100;
  assert(Math.abs(row.util - expectedUtil) < 0.15, `${row.role}: Util (${row.util}%) khớp tính toán (${expectedUtil.toFixed(1)}%)`);
}

// Verify summary numbers
assert(sumFte === 8.5, `Tổng FTE = 8.5 (thực tế: ${sumFte})`);
assert(sumGd0 === 140, `Tổng phân bổ GĐ0 = 140 MD (thực tế: ${sumGd0})`);
assert(sumGd1 === 199, `Tổng phân bổ GĐ1 = 199 MD (thực tế: ${sumGd1})`);
assert(sumTotal === 339, `Tổng phân bổ cả 2 GĐ = 339 MD (thực tế: ${sumTotal})`);
assert(sumCap === 425, `Tổng trần công suất 10 tuần x 5 ngày x 8.5 FTE = 425 MD (thực tế: ${sumCap})`);
assert(sumBuffer === 86, `Tổng quỹ dự phòng = 86 MD (425 - 339 = 86) (thực tế: ${sumBuffer})`);
assert(sumTotal + sumBuffer === sumCap, `Bảo toàn công suất: Đã phân bổ (${sumTotal}) + Dự phòng (${sumBuffer}) = Trần (${sumCap})`);

const teamUtil = (sumTotal / sumCap) * 100;
const bufferPercent = (sumBuffer / sumCap) * 100;
assert(Math.abs(teamUtil - 79.76) < 0.1, `Tỷ lệ sử dụng toàn đội = 79.8% (thực tế: ${teamUtil.toFixed(1)}%)`);
assert(Math.abs(bufferPercent - 20.24) < 0.1, `Tỷ lệ quỹ dự phòng an toàn = 20.2% (thực tế: ${bufferPercent.toFixed(1)}%)`);

// -------------------------------------------------------------
// SUITE 5: TRACEABILITY MATRIX & QUALITY GATES
// -------------------------------------------------------------
console.log('\n[SUITE 5] Kiểm tra Ma Trận Traceability Exit Criteria & Cổng Chất Lượng');

const exitCriteriaIds = ['GĐ0-EC1', 'GĐ0-EC2', 'GĐ0-EC3', 'GĐ0-EC4', 'GĐ1-EC1', 'GĐ1-EC2', 'GĐ1-EC3'];
for (const ec of exitCriteriaIds) {
  assert(content.includes(ec), `Exit Criteria ${ec} có mặt trong tài liệu`);
}

// Check deep technical requirements:
assert(content.includes('#![deny(clippy::float_arithmetic)]'), 'Có chỉ thị deny float clippy');
assert(content.includes('crates/liva-money') && content.includes('crates/liva-ledger'), 'Phạm vi clippy float giới hạn trong liva-money & liva-ledger');
assert(content.includes('1.000.000') || content.includes('1_000_000'), 'Có test suite 1 triệu proptest cases');
assert(content.includes('max_shrink_iters') && content.includes('10_000'), 'Cấu hình khống chế max_shrink_iters <= 10_000 cho proptest');
assert(content.includes('#[ignore]'), 'Có gắn cờ #[ignore] cho bài test 1M cases trong CI thường');
assert(content.includes('#[cfg(target_os = "linux")]'), 'Có cờ target_os = "linux" cho liva-netguard');
assert(content.includes('Windows Filtering Platform') || content.includes('WFP'), 'Có giải pháp Windows isolation WFP cho máy trạm kế toán');
assert(content.includes('Gate 0A') && content.includes('Gate 0B'), 'Có Staged Handshake Protocol Gate 0A & Gate 0B');

// -------------------------------------------------------------
// FINAL EMPIRICAL AUDIT SUMMARY
// -------------------------------------------------------------
console.log('\n═══════════════════════════════════════════════════════════════');
console.log(`KẾT QUẢ KIỂM TRA THỰC CHỨNG ĐỐI KHÁNG:`);
console.log(`- Tổng số tiêu chí kiểm định: ${totalTests}`);
console.log(`- Vượt qua (PASS): ${passedTests}`);
console.log(`- Thất bại (FAIL): ${failedTests}`);
console.log(`- Tỷ lệ thành công: ${((passedTests / totalTests) * 100).toFixed(1)}%`);
console.log('═══════════════════════════════════════════════════════════════\n');

process.exit(failedTests === 0 ? 0 : 1);
