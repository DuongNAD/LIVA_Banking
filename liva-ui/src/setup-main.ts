import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

function getEl<T extends HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) {
    throw new Error(`Element #${id} not found`);
  }
  return el as T;
}

const co = (n: number): string => {
  if (n >= 1024 ** 3) return (n / 1024 ** 3).toFixed(2) + ' GB';
  if (n >= 1024 ** 2) return (n / 1024 ** 2).toFixed(1) + ' MB';
  return (n / 1024).toFixed(0) + ' KB';
};

interface GroupReport {
  key: string;
  name: string;
  required: boolean;
  ready: boolean;
  broken?: string;
  note?: string;
}

interface FileReport {
  group: string;
  dest: string;
  state: string;
  bytes: number;
  downloadable: boolean;
  manual?: string | null;
}

interface SetupStatus {
  groups: GroupReport[];
  missing: FileReport[];
  blocking: boolean;
}

interface SetupPaths {
  resourceRoot: string;
  llmDir: string;
  dataDir: string;
  configFile: string;
}

interface ProgressData {
  progress?: {
    overall_total?: number;
    overall_downloaded?: number;
    index?: number;
    total_files?: number;
    dest?: string;
  };
}

interface StreamEventPayload {
  data?: ProgressData;
}

interface SetupFetchResult {
  failed?: string[];
}

const goi = <T>(command: string, payload: Record<string, unknown> = {}): Promise<T> =>
  invoke<T>('native_ipc_call', { command, payload });

let dangTai = false;

async function veTrangThai(): Promise<void> {
  let st: SetupStatus;
  try {
    st = await goi<SetupStatus>('setup:status');
  } catch (e) {
    getEl('mota').innerHTML = '<span class="loi">Không đọc được danh sách model: ' + e + '</span>';
    return;
  }

  getEl('danhsach').innerHTML = st.groups
    .map((g) => {
      const lop = g.ready ? 'ok' : g.required ? 'thieu' : 'tuychon';
      const nhan = g.ready ? 'sẵn sàng' : g.required ? 'CHƯA CÓ' : 'chưa bật';
      const hong = !g.ready && g.broken ? '<div class="hong">' + g.broken + '</div>' : '';
      return (
        '<div class="nhom"><span class="cham ' + lop + '"></span>' +
        '<span class="ten">' + g.name + '</span>' +
        '<span class="trangthai">' + nhan + '</span></div>' + hong
      );
    })
    .join('');

  const taiDuoc = st.missing.filter((f) => f.downloadable);
  const soByte = taiDuoc.reduce((s, f) => s + f.bytes, 0);

  const motaEl = getEl('mota');
  const taiBtn = getEl<HTMLButtonElement>('tai');
  const boBtn = getEl<HTMLButtonElement>('bo');

  if (!st.missing.length) {
    motaEl.textContent = 'Đã đủ model. LIVA dùng được ngay.';
    taiBtn.disabled = true;
    boBtn.textContent = 'Đóng';
  } else if (st.blocking) {
    motaEl.textContent =
      'LIVA còn thiếu model bắt buộc. Ứng dụng vẫn mở để bạn tải lại; các năng lực ghi ở nhóm thiếu chưa dùng được.';
  } else {
    motaEl.textContent = 'Model bắt buộc đã đủ. Phần còn thiếu chỉ là tính năng tuỳ chọn.';
  }

  if (taiDuoc.length && !dangTai) {
    taiBtn.disabled = false;
    taiBtn.textContent = 'Tải ' + taiDuoc.length + ' file (~' + co(soByte) + ')';
  }

  const tuTay = st.missing.filter((f) => !f.downloadable);
  getEl('ghichu').textContent = tuTay.length
    ? tuTay.length + ' file phải tự chuẩn bị (không có nguồn tải công khai).'
    : '';

  try {
    const p = await goi<SetupPaths>('setup:paths');
    getEl('duongdan').textContent =
      'Model     : ' + p.resourceRoot + '\n' +
      'Model LLM : ' + p.llmDir + '\n' +
      'Dữ liệu   : ' + p.dataDir + '\n' +
      'Cấu hình  : ' + p.configFile;
  } catch {
    /* không quan trọng bằng phần trên */
  }
}

async function tai(): Promise<void> {
  dangTai = true;
  getEl<HTMLButtonElement>('tai').disabled = true;
  getEl('khoitai').hidden = false;
  getEl('mota').textContent = 'Đang tải. Có thể đóng cửa sổ và làm việc khác — lần sau mở lại sẽ tải tiếp phần dở.';

  const reqId = 'setup-' + Date.now();
  const un = await listen<StreamEventPayload>('ipc-stream:' + reqId, (ev) => {
    const d = ev.payload && ev.payload.data;
    if (!d || !d.progress) return;
    const p = d.progress;
    const total = p.overall_total ?? 0;
    const downloaded = p.overall_downloaded ?? 0;
    const pt = total ? (downloaded / total) * 100 : 0;
    getEl<HTMLProgressElement>('thanh').value = pt;
    getEl('tiendo').textContent =
      '[' + (p.index ?? 0) + '/' + (p.total_files ?? 0) + '] ' + (p.dest ?? '') +
      ' — ' + co(downloaded) + ' / ' + co(total);
  });

  try {
    const kq = await invoke<SetupFetchResult>('native_ipc_call_stream', {
      command: 'setup:fetch',
      payload: {},
      reqId,
    });
    getEl('tiendo').textContent = '';
    if (kq.failed && kq.failed.length) {
      getEl('mota').innerHTML =
        '<span class="loi">' + kq.failed.length +
        ' file tải hỏng. Bấm lại để tải tiếp — phần đã tải được giữ lại.</span>';
      getEl('tiendo').innerHTML = '<pre class="loi">' + kq.failed.join('\n') + '</pre>';
    } else {
      getEl('mota').textContent = 'Tải xong.';
    }
  } catch (e) {
    getEl('mota').innerHTML = '<span class="loi">Tải hỏng: ' + e + '</span>';
  } finally {
    un();
    dangTai = false;
    getEl<HTMLProgressElement>('thanh').value = 100;
    await veTrangThai();
  }
}

getEl<HTMLButtonElement>('tai').addEventListener('click', () => {
  void tai();
});

getEl<HTMLButtonElement>('bo').addEventListener('click', async () => {
  try {
    await invoke('open_dashboard');
  } catch {
    // Ignore error if dashboard fails to open
  }
  try {
    await getCurrentWindow().close();
  } catch {
    // Ignore error if close fails
  }
});

void veTrangThai();
