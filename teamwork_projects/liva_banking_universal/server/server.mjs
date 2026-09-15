/**
 * LIVA Banking Universal — Centralized Backend API Server
 * Provides JWT Authentication, Multi-Machine Data Sync, and Centralized Treasury State
 * Pure Node.js Standard Library (Zero External Dependencies)
 */

import http from 'node:http';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PORT = process.env.PORT || 3001;
const JWT_SECRET = process.env.JWT_SECRET || 'liva-enterprise-super-secret-key-2026-circular-09';

// ==========================================
// 1. JWT Implementation (HMAC-SHA256)
// ==========================================
function base64UrlEncode(str) {
  return Buffer.from(str).toString('base64url');
}

function base64UrlDecode(str) {
  return Buffer.from(str, 'base64url').toString('utf-8');
}

function signJwt(payload, expiresInSeconds = 14400) {
  const header = { alg: 'HS256', typ: 'JWT' };
  const now = Math.floor(Date.now() / 1000);
  const fullPayload = {
    ...payload,
    iat: now,
    exp: now + expiresInSeconds,
  };

  const headerB64 = base64UrlEncode(JSON.stringify(header));
  const payloadB64 = base64UrlEncode(JSON.stringify(fullPayload));
  const data = `${headerB64}.${payloadB64}`;

  const signature = crypto
    .createHmac('sha256', JWT_SECRET)
    .update(data)
    .digest('base64url');

  return `${data}.${signature}`;
}

function verifyJwt(token) {
  if (!token || typeof token !== 'string') return null;
  const parts = token.split('.');
  if (parts.length !== 3) return null;

  const [headerB64, payloadB64, signature] = parts;
  const data = `${headerB64}.${payloadB64}`;

  const expectedSignature = crypto
    .createHmac('sha256', JWT_SECRET)
    .update(data)
    .digest('base64url');

  const sigBuf = Buffer.from(signature);
  const expectedSigBuf = Buffer.from(expectedSignature);
  if (sigBuf.length !== expectedSigBuf.length || !crypto.timingSafeEqual(sigBuf, expectedSigBuf)) {
    return null;
  }

  try {
    const payload = JSON.parse(base64UrlDecode(payloadB64));
    const now = Math.floor(Date.now() / 1000);
    if (payload.exp !== undefined && payload.exp < now) {
      return null; // Token expired
    }
    return payload;
  } catch (err) {
    return null;
  }
}

// ==========================================
// 2. Pre-seeded Users (RBAC) & SSE Channels
// ==========================================
const USERS = [
  {
    id: 'usr_maker_01',
    username: 'maker_nam',
    email: 'maker@livabanking.vn',
    password: 'LivaMaker@2026',
    altPasswords: ['maker123'],
    aliases: ['maker_nam', 'opr-77092', 'opr77092', 'maker@livabanking.vn', 'maker_nam@livabanking.vn', 'maker'],
    fullName: 'Nguyễn Văn Kế Toán',
    role: 'MAKER',
    officerId: 'OPR-77092',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Cán Bộ Vận Hành & Đối Soát (Maker)',
    avatar: '👤',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_maker_02',
    username: 'maker_mai',
    email: 'maker_mai@livabanking.vn',
    password: 'LivaMaker@2026',
    altPasswords: ['maker123'],
    aliases: ['maker_mai', 'opr-77093', 'opr77093'],
    fullName: 'Lê Phương Mai',
    role: 'MAKER',
    officerId: 'OPR-77093',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Kế toán viên Thanh toán (Maker)',
    avatar: '👤',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_checker_01',
    username: 'checker_tri',
    email: 'checker@livabanking.vn',
    password: 'LivaChecker@2026',
    altPasswords: ['checker123'],
    aliases: ['checker_tri', 'sup-88214', 'sup88214', 'checker@livabanking.vn', 'checker_tri@livabanking.vn', 'checker'],
    fullName: 'Trần Thị Giám Đốc',
    role: 'CHECKER',
    officerId: 'SUP-88214',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Kiểm Soát Viên Phê Duyệt (Checker)',
    avatar: '🛡️',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_checker_02',
    username: 'checker_huong',
    email: 'checker_huong@livabanking.vn',
    password: 'LivaChecker@2026',
    altPasswords: ['checker123'],
    aliases: ['checker_huong', 'sup-88215', 'sup88215'],
    fullName: 'Đỗ Lan Hương',
    role: 'CHECKER',
    officerId: 'SUP-88215',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Phó phòng Kế toán / Kiểm Soát Viên (Checker)',
    avatar: '🛡️',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_aml_01',
    username: 'auditor_lan',
    email: 'aml@livabanking.vn',
    password: 'LivaAudit@2026',
    altPasswords: ['aml123', 'auditor123'],
    aliases: ['auditor_lan', 'aml_lan', 'cmp-99015', 'cmp99015', 'aml@livabanking.vn', 'auditor@livabanking.vn', 'aml', 'auditor'],
    fullName: 'Lê Hoàng Thanh Tra',
    role: 'AML',
    officerId: 'CMP-99015',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Cán Bộ Giám Sát Tuân Thủ & PCRT',
    avatar: '⚖️',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_treasury_01',
    username: 'cfo_hoang',
    email: 'treasury@livabanking.vn',
    password: 'LivaCfo@2026',
    altPasswords: ['treasury123'],
    aliases: ['cfo_hoang', 'trz-55038', 'trz55038', 'treasury@livabanking.vn', 'treasury'],
    fullName: 'Đặng Đình Bảo',
    role: 'TREASURY',
    officerId: 'TRZ-55038',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Cán Bộ Quản Trị Thanh Khoản & Vốn',
    avatar: '🏦',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
];

// Connected SSE clients for real-time synchronization
const sseClients = new Set();

function broadcastEvent(event, data) {
  const payload = `event: ${event}\ndata: ${JSON.stringify(data)}\n\n`;
  for (const client of sseClients) {
    try {
      client.write(payload);
    } catch (err) {
      sseClients.delete(client);
    }
  }
}

// ==========================================
// 3. Centralized Shared Database (In-Memory + State)
// ==========================================
let DB = {
  accounts: [
    {
      id: 'acc_vcb_01',
      bankCode: 'VCB',
      bankName: 'Ngân hàng TMCP Ngoại thương Việt Nam (Vietcombank)',
      accountNumber: '0071001234567',
      accountName: 'CONG TY TNHH LIVA SOLUTIONS',
      balance: 3450000000,
      currency: 'VND',
      inflows: 1710000000,
      outflows: 514500000,
      status: 'ACTIVE',
      lastReconciledAt: '2026-08-31T17:00:00Z',
    },
    {
      id: 'acc_tcb_01',
      bankCode: 'TCB',
      bankName: 'Ngân hàng TMCP Kỹ thương Việt Nam (Techcombank)',
      accountNumber: '19034567890123',
      accountName: 'CONG TY TNHH LIVA SOLUTIONS',
      balance: 1250000000,
      currency: 'VND',
      inflows: 800000000,
      outflows: 2200000,
      status: 'ACTIVE',
      lastReconciledAt: '2026-08-31T17:00:00Z',
    },
    {
      id: 'acc_bidv_01',
      bankCode: 'BIDV',
      bankName: 'Ngân hàng TMCP Đầu tư & Phát triển VN (BIDV)',
      accountNumber: '12410001234567',
      accountName: 'CONG TY TNHH LIVA SOLUTIONS',
      balance: 486130000,
      currency: 'VND',
      inflows: 420000000,
      outflows: 419500000,
      status: 'ACTIVE',
      lastReconciledAt: '2026-08-31T17:00:00Z',
    },
  ],
  vouchers: [
    {
      id: 'vch_init_01',
      voucherNumber: 'VCH-2026-08-001',
      sourceAccount: '0071001234567 (VCB)',
      targetAccount: '020012345678',
      targetBeneficiary: 'CONG TY CP THIET BI MAY BOM DONG LUC',
      targetBank: 'BIDV',
      amount: 450000000,
      currency: 'VND',
      description: 'Thanh toan may bom cong nghiep theo HD-88',
      purpose: 'VẬT TƯ & THIẾT BỊ',
      status: 'PENDING_APPROVAL',
      makerId: 'usr_maker_01',
      makerName: 'Nguyễn Văn Kế Toán (Maker)',
      createdAt: new Date().toISOString(),
      approvedAt: null,
      checkerId: null,
      checkerName: null,
      merkleLeafHash: null,
    },
  ],
  merkleLedger: {
    genesisHash: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    currentRootHash: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    leaves: [],
    history: [],
  },
};

// ==========================================
// 4. HTTP Server & Route Handler
// ==========================================
const server = http.createServer((req, res) => {
  // CORS Headers
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');

  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  const url = new URL(req.url, `http://${req.headers.host}`);
  const pathname = url.pathname;

  // ------------------------------------------
  // Route: SSE Real-Time Sync Channel
  // ------------------------------------------
  if (req.method === 'GET' && pathname === '/api/sync/events') {
    res.writeHead(200, {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      'Connection': 'keep-alive',
      'Access-Control-Allow-Origin': '*',
    });

    res.write(': connected\n\n');
    sseClients.add(res);

    const keepAliveTimer = setInterval(() => {
      try {
        res.write(': keepalive\n\n');
      } catch (err) {
        clearInterval(keepAliveTimer);
        sseClients.delete(res);
      }
    }, 15000);

    req.on('close', () => {
      clearInterval(keepAliveTimer);
      sseClients.delete(res);
    });
    return;
  }

  // Body Parsing Helper
  let rawBody = '';
  req.on('data', (chunk) => {
    rawBody += chunk;
  });

  req.on('end', () => {
    let body = {};
    if (rawBody) {
      try {
        body = JSON.parse(rawBody);
      } catch (e) {
        // ignore parse error
      }
    }

    // Helper Send JSON
    const sendJson = (statusCode, data) => {
      res.writeHead(statusCode, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(data));
    };

    // Helper Extract Auth User
    const getAuthUser = () => {
      const authHeader = req.headers['authorization'];
      if (!authHeader || !authHeader.startsWith('Bearer ')) return null;
      const token = authHeader.split(' ')[1];
      return verifyJwt(token);
    };

    // ------------------------------------------
    // Route 1: Health Check
    // ------------------------------------------
    if (req.method === 'GET' && (pathname === '/api/health' || pathname === '/health')) {
      return sendJson(200, {
        status: 'OK',
        service: 'LIVA Banking Universal Centralized API Server',
        timestamp: new Date().toISOString(),
        activeUsersCount: USERS.length,
        vouchersCount: DB.vouchers.length,
        merkleRoot: DB.merkleLedger.currentRootHash,
      });
    }

    // ------------------------------------------
    // Route 2: Auth Login
    // ------------------------------------------
    if (req.method === 'POST' && pathname === '/api/auth/login') {
      const { email, password, role: preferredRole } = body;
      const inputIdent = (email || '').trim().toLowerCase();

      const user = USERS.find((u) =>
        u.email.toLowerCase() === inputIdent ||
        (u.username && u.username.toLowerCase() === inputIdent) ||
        u.officerId.toLowerCase() === inputIdent ||
        (u.aliases && u.aliases.some((a) => a.toLowerCase() === inputIdent))
      );

      if (!user) {
        return sendJson(401, {
          success: false,
          error: 'Email hoặc mật khẩu không chính xác.',
        });
      }

      const validPasswords = [user.password, ...(user.altPasswords || [])];
      if (!validPasswords.includes(password)) {
        return sendJson(401, {
          success: false,
          error: 'Email hoặc mật khẩu không chính xác.',
        });
      }

      // Enforce Segregation of Duties (SoD - Thông tư 09/2020/TT-NHNN)
      if (preferredRole) {
        const isAmlAuditorMatch =
          (preferredRole === 'AML' && user.role === 'AUDITOR') ||
          (preferredRole === 'AUDITOR' && user.role === 'AML');

        if (user.role !== preferredRole && !isAmlAuditorMatch) {
          const roleLabels = {
            MAKER: 'Kế toán (Maker)',
            CHECKER: 'Kiểm soát (Checker)',
            AML: 'Giám sát AML',
            AUDITOR: 'Kiểm toán (Auditor)',
            TREASURY: 'Quản trị Vốn (Treasury)',
          };
          const userRoleLabel = roleLabels[user.role] || user.role;
          const requestedRoleLabel = roleLabels[preferredRole] || preferredRole;

          return sendJson(403, {
            success: false,
            error: `Vi phạm Tách bạch trách nhiệm (SoD - Thông tư 09/2020/TT-NHNN): Cán bộ ${user.fullName} được định danh vai trò [${userRoleLabel}], không có thẩm quyền truy cập phân hệ [${requestedRoleLabel}].`,
          });
        }
      }

      const token = signJwt({
        sub: user.id,
        id: user.id,
        email: user.email,
        name: user.fullName,
        fullName: user.fullName,
        role: user.role,
        officerId: user.officerId,
        branchCode: user.branchCode,
        branchName: user.branchName,
        terminalId: user.terminalId,
        title: user.title,
        avatar: user.avatar,
        bankAccess: user.bankAccess,
      });

      return sendJson(200, {
        success: true,
        token,
        user: {
          id: user.id,
          email: user.email,
          fullName: user.fullName,
          name: user.fullName,
          role: user.role,
          officerId: user.officerId,
          branchCode: user.branchCode,
          branchName: user.branchName,
          terminalId: user.terminalId,
          title: user.title,
          avatar: user.avatar,
          bankAccess: user.bankAccess,
        },
      });
    }

    // ------------------------------------------
    // Route 3: Auth Me (Verify Token)
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/auth/me') {
      const authUser = getAuthUser();
      if (!authUser) {
        return sendJson(401, { success: false, error: 'Token không hợp lệ hoặc đã hết hạn.' });
      }
      return sendJson(200, { success: true, user: authUser });
    }

    // ------------------------------------------
    // Route 4: Get Bank Accounts
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/banking/accounts') {
      const authUser = getAuthUser();
      if (!authUser) {
        return sendJson(401, { success: false, error: 'Yêu cầu đăng nhập để xem tài khoản.' });
      }
      return sendJson(200, { success: true, accounts: DB.accounts });
    }

    // ------------------------------------------
    // Route 5: Get Vouchers
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/treasury/vouchers') {
      const authUser = getAuthUser();
      if (!authUser) {
        return sendJson(401, { success: false, error: 'Yêu cầu đăng nhập.' });
      }
      return sendJson(200, {
        success: true,
        vouchers: DB.vouchers,
        pendingCount: DB.vouchers.filter((v) => v.status === 'PENDING_APPROVAL').length,
      });
    }

    // ------------------------------------------
    // Route 6: Create Voucher (Maker / Treasury / Checker)
    // ------------------------------------------
    if (req.method === 'POST' && pathname === '/api/treasury/vouchers') {
      const authUser = getAuthUser();
      if (!authUser) return sendJson(401, { success: false, error: 'Chưa đăng nhập.' });
      if (authUser.role !== 'MAKER' && authUser.role !== 'CHECKER' && authUser.role !== 'TREASURY') {
        return sendJson(403, { success: false, error: 'Chỉ Maker/Treasury mới có quyền tạo lệnh chi.' });
      }

      const rawAmount = body.amount !== undefined ? body.amount : body.amountVnd;
      const amount = Number(rawAmount);
      if (!Number.isFinite(amount) || amount <= 0) {
        return sendJson(400, {
          success: false,
          error: 'Số tiền lệnh chi phải là số dương hợp lệ.',
        });
      }

      const voucherId = body.id || body.voucherId || `vch_${Date.now()}`;
      const newVoucher = {
        id: voucherId,
        voucherId: voucherId,
        voucherNumber: body.voucherNumber || `VCH-2026-${String(DB.vouchers.length + 1).padStart(3, '0')}`,
        sourceAccount: body.sourceAccount || '0071001234567 (VCB)',
        targetAccount: body.targetAccount || body.beneficiaryAccount || '020012345678',
        beneficiaryAccount: body.beneficiaryAccount || body.targetAccount || '020012345678',
        targetBeneficiary: body.targetBeneficiary || body.beneficiaryName || 'Đơn vị thụ hưởng',
        beneficiaryName: body.beneficiaryName || body.targetBeneficiary || 'Đơn vị thụ hưởng',
        targetBank: body.targetBank || body.beneficiaryBank || 'CITAD_SBV',
        beneficiaryBank: body.beneficiaryBank || body.targetBank || 'CITAD_SBV',
        amount: amount,
        amountVnd: amount,
        currency: 'VND',
        description: body.description || body.purpose || 'Chi thanh toán liên ngân hàng',
        purpose: body.purpose || body.description || 'Chi thanh toán liên ngân hàng',
        status: 'PENDING_APPROVAL',
        makerId: authUser.id || authUser.sub,
        makerName: authUser.name || authUser.fullName,
        createdAt: new Date().toISOString(),
        approvedAt: null,
        checkerId: null,
        checkerName: null,
        merkleLeafHash: null,
        hitlToken: body.hitlToken || null,
      };

      DB.vouchers.unshift(newVoucher);
      broadcastEvent('voucher:created', { voucher: newVoucher });

      return sendJson(201, {
        success: true,
        message: 'Lệnh chi đã được tạo và gửi lên Server chờ Checker phê duyệt.',
        voucher: newVoucher,
      });
    }

    // ------------------------------------------
    // Route 7: Approve Voucher (Checker Only)
    // ------------------------------------------
    if (req.method === 'POST' && pathname.startsWith('/api/treasury/vouchers/') && pathname.endsWith('/approve')) {
      const authUser = getAuthUser();
      if (!authUser) return sendJson(401, { success: false, error: 'Chưa đăng nhập.' });
      if (authUser.role !== 'CHECKER') {
        return sendJson(403, {
          success: false,
          error: 'Quy tắc Maker-Checker (TT 09/2020): Chỉ Giám đốc (Checker) mới có quyền phê duyệt.',
        });
      }

      const parts = pathname.split('/');
      const voucherId = parts[parts.length - 2];
      const voucher = DB.vouchers.find((v) => v.id === voucherId || v.voucherId === voucherId);

      if (!voucher) {
        return sendJson(404, { success: false, error: 'Không tìm thấy lệnh chi.' });
      }

      const currentUserId = authUser.id || authUser.sub;
      if (voucher.makerId === currentUserId) {
        return sendJson(403, {
          success: false,
          error: 'Kiểm soát viên không được tự phê duyệt lệnh chi do chính mình tạo lập (Thông tư 09/2020/TT-NHNN).',
        });
      }

      if (voucher.status !== 'PENDING_APPROVAL') {
        return sendJson(400, { success: false, error: 'Lệnh chi không ở trạng thái chờ phê duyệt.' });
      }

      // Compute Cryptographic Merkle Leaf Hash
      const leafPayload = `${voucher.id || voucher.voucherId}|${voucher.amount || voucher.amountVnd}|${voucher.targetAccount || voucher.beneficiaryAccount}|${Date.now()}`;
      const leafHash = crypto.createHash('sha256').update(leafPayload).digest('hex');

      // Update Voucher State
      voucher.status = 'APPROVED';
      voucher.approvedAt = new Date().toISOString();
      voucher.checkerId = authUser.id || authUser.sub;
      voucher.checkerName = authUser.name || authUser.fullName;
      voucher.merkleLeafHash = leafHash;

      // Update Merkle Ledger
      DB.merkleLedger.leaves.push(leafHash);
      const combinedHash = crypto
        .createHash('sha256')
        .update(DB.merkleLedger.currentRootHash + leafHash)
        .digest('hex');
      DB.merkleLedger.currentRootHash = combinedHash;
      DB.merkleLedger.history.push({
        voucherId: voucher.id || voucher.voucherId,
        leafHash,
        rootHash: combinedHash,
        timestamp: voucher.approvedAt,
        approvedBy: voucher.checkerName,
      });

      broadcastEvent('voucher:approved', {
        voucher,
        voucherId: voucher.voucherId || voucher.id,
        checkerId: voucher.checkerId,
        merkleHash: leafHash,
        merkleRoot: combinedHash,
      });

      return sendJson(200, {
        success: true,
        message: 'Lệnh chi đã được phê duyệt và ghi nhận vào Merkle Audit Ledger bất biến.',
        voucher,
        merkleRoot: combinedHash,
      });
    }

    // ------------------------------------------
    // Route 8: Reject Voucher (Checker Only)
    // ------------------------------------------
    if (req.method === 'POST' && pathname.startsWith('/api/treasury/vouchers/') && pathname.endsWith('/reject')) {
      const authUser = getAuthUser();
      if (!authUser) return sendJson(401, { success: false, error: 'Chưa đăng nhập.' });
      if (authUser.role !== 'CHECKER') {
        return sendJson(403, { success: false, error: 'Chỉ Checker mới có quyền từ chối lệnh chi.' });
      }

      const parts = pathname.split('/');
      const voucherId = parts[parts.length - 2];
      const voucher = DB.vouchers.find((v) => v.id === voucherId || v.voucherId === voucherId);

      if (!voucher) return sendJson(404, { success: false, error: 'Không tìm thấy lệnh chi.' });

      if (voucher.status !== 'PENDING_APPROVAL') {
        return sendJson(400, { success: false, error: 'Lệnh chi không ở trạng thái chờ phê duyệt.' });
      }

      voucher.status = 'REJECTED';
      voucher.approvedAt = new Date().toISOString();
      voucher.checkerId = authUser.id || authUser.sub;
      voucher.checkerName = authUser.name || authUser.fullName;
      voucher.rejectionReason = body.reason || body.remarks || 'Không đạt điều kiện thẩm định thanh toán.';

      broadcastEvent('voucher:rejected', {
        voucher,
        voucherId: voucher.voucherId || voucher.id,
        checkerId: voucher.checkerId,
        remarks: voucher.rejectionReason,
        reason: voucher.rejectionReason,
      });

      return sendJson(200, {
        success: true,
        message: 'Lệnh chi đã bị từ chối.',
        voucher,
      });
    }

    // ------------------------------------------
    // Route 9: Merkle Audit Ledger
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/treasury/merkle-ledger') {
      return sendJson(200, {
        success: true,
        ledger: DB.merkleLedger,
      });
    }

    // ------------------------------------------
    // 404 Not Found
    // ------------------------------------------
    return sendJson(404, { error: `Endpoint ${pathname} không tồn tại.` });
  });
});

// Direct execution check to prevent port conflicts in test runners
const isDirectRun = Boolean(
  process.argv[1] && (
    fileURLToPath(import.meta.url) === path.resolve(process.argv[1]) ||
    process.argv[1].endsWith('server.mjs')
  )
);

if (isDirectRun || process.env.RUN_SERVER === 'true') {
  server.listen(PORT, () => {
    console.log(`[LIVA Backend Server] Online at http://localhost:${PORT}`);
    console.log(`[LIVA Backend Server] JWT Auth ready. Presets: maker@livabanking.vn, checker@livabanking.vn, aml@livabanking.vn, treasury@livabanking.vn`);
    console.log(`[LIVA Backend Server] SSE Real-time Sync active at /api/sync/events`);
  });
}

export { server, USERS, DB, signJwt, verifyJwt, broadcastEvent, sseClients, PORT, base64UrlEncode, base64UrlDecode };
export default server;
