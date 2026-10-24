/**
 * Pinia Store for Internal Commercial Bank Liquidity & Interbank Settlement Channels
 * Complies with Circular 09/2020/TT-NHNN, Circular 09/2023/TT-NHNN & Universal Banking Operations.
 */

import { defineStore } from 'pinia';
import type { BankCode, LiquidityChannelProfile } from '../types/banking';
import { apiRequest } from '../services/apiClient';

export type { LiquidityChannelProfile, BankAccountProfile } from '../types/banking';

export interface LiquidityChannelAccount extends LiquidityChannelProfile {
  minReserveVnd: number;
}

export interface LiquidityRebalanceLog {
  id: string;
  timestamp: string; // ISO string
  epochSeconds: number;
  fromChannel: BankCode;
  toChannel: BankCode;
  amountVnd: number;
  purpose: string;
  status: 'COMPLETED' | 'REJECTED';
  executedBy: string;
  note?: string;
}

export const useBankingStore = defineStore('banking', {
  state: () => ({
    selectedBankPersona: 'ALL' as 'ALL' | BankCode,
    accounts: [
      {
        channelCode: 'CITAD',
        bankCode: 'CITAD',
        channelName: 'Kênh Thanh toán Điện tử Liên ngân hàng CITAD (NHNN)',
        clearingAuthority: 'Ngân hàng Nhà nước Việt Nam (Sở Giao dịch NHNN)',
        accountNumber: '01-CITAD-SBV-VND',
        accountName: 'TÀI KHOẢN TIỀN GỬI THANH TOÁN TẠI SỞ GIAO DỊCH NHNN',
        currency: 'VND',
        balanceVnd: 3_250_000_000,
        availableBalanceVnd: 3_250_000_000,
        inflowMonthVnd: 1_850_000_000,
        outflowMonthVnd: 514_500_000,
        minReserveVnd: 1_000_000_000, // 1 tỷ VND ngưỡng dự trữ tối thiểu CITAD
        brandColor: '#005a3c',
        status: 'ACTIVE',
        lastReconciledDate: '2026-08-31',
        clearingMechanism: 'RTGS Thanh toán Điện tử Liên ngân hàng IBPS/CITAD Giá trị cao',
        cutoffTime: '16:30 (Phiên chiều NHNN)',
      },
      {
        channelCode: 'NAPAS',
        bankCode: 'NAPAS',
        channelName: 'Kênh Chuyển mạch Tài chính & Bù trừ Điện tử NAPAS 24/7',
        clearingAuthority: 'Công ty Cổ phần Thanh toán Quốc gia Việt Nam (NAPAS)',
        accountNumber: 'NAPAS-ACH-247-SETTLE',
        accountName: 'HẠN MỨC QUYẾT TOÁN BÙ TRỪ RÒNG ĐA PHƯƠNG NAPAS',
        currency: 'VND',
        balanceVnd: 1_680_000_000,
        availableBalanceVnd: 1_680_000_000,
        inflowMonthVnd: 920_000_000,
        outflowMonthVnd: 45_000_000,
        minReserveVnd: 500_000_000, // 500 triệu VND ngưỡng bù trừ ròng đa phương NAPAS
        brandColor: '#0284c7',
        status: 'ACTIVE',
        lastReconciledDate: '2026-08-31',
        clearingMechanism: 'Bù trừ Tự động ACH & Chuyển tiền nhanh 24/7 (VietQR)',
        cutoffTime: '24/7/365 (Thời gian thực)',
      },
      {
        channelCode: 'BILATERAL',
        bankCode: 'BILATERAL',
        channelName: 'Kênh Thanh toán Song phương & Tài khoản Vostro / Nostro',
        clearingAuthority: 'Hệ thống Ngân hàng Đại lý & Định chế Tài chính Thành viên',
        accountNumber: 'NOSTRO-VOSTRO-INTERBANK-01',
        accountName: 'TÀI KHOẢN THANH TOÁN SONG PHƯƠNG NOSTRO/VOSTRO',
        currency: 'VND',
        balanceVnd: 850_000_000,
        availableBalanceVnd: 850_000_000,
        inflowMonthVnd: 480_000_000,
        outflowMonthVnd: 420_000_000,
        minReserveVnd: 300_000_000, // 300 triệu VND ngưỡng dự trữ song phương
        brandColor: '#d97706',
        status: 'ACTIVE',
        lastReconciledDate: '2026-08-31',
        clearingMechanism: 'Kết nối Kênh Thanh toán Song phương Trực tiếp & Điều chuyển Vốn',
        cutoffTime: '17:00 (Hàng ngày)',
      },
      {
        channelCode: 'SWIFT',
        bankCode: 'SWIFT',
        channelName: 'Kênh Chuyển tiền & Điện báo Quốc tế SWIFT',
        clearingAuthority: 'Hiệp hội Viễn thông Tài chính Liên ngân hàng Quốc tế (SWIFT)',
        accountNumber: 'SWIFT-NOSTRO-USD-EUR',
        accountName: 'TÀI KHOẢN QUYẾT TOÁN THANH TOÁN QUỐC TẾ SWIFT ALLIANCE',
        currency: 'VND',
        balanceVnd: 420_000_000,
        availableBalanceVnd: 420_000_000,
        inflowMonthVnd: 350_000_000,
        outflowMonthVnd: 180_000_000,
        minReserveVnd: 200_000_000, // 200 triệu VND ngưỡng dự trữ Nostro SWIFT
        brandColor: '#7c3aed',
        status: 'ACTIVE',
        lastReconciledDate: '2026-08-31',
        clearingMechanism: 'Điện báo ISO 20022 MX (pacs.008/009) & MT103/MT202',
        cutoffTime: '23:00 (Theo giờ chuẩn GMT)',
      },
    ] as LiquidityChannelAccount[],
    syncedDailyBurnVnd: 0 as number,
    rebalanceLogs: [] as LiquidityRebalanceLog[],
  }),

  getters: {
    totalLiquidVnd: (state) =>
      state.accounts.reduce((sum, acc) => sum + acc.balanceVnd, 0),

    dailyAverageBurnVnd: (state): number => {
      if (state.syncedDailyBurnVnd > 0) return state.syncedDailyBurnVnd;
      const totalOutflow = state.accounts.reduce((sum, acc) => sum + (acc.outflowMonthVnd || 0), 0);
      return totalOutflow > 0 ? Math.round(totalOutflow / 30) : 0;
    },

    liquidityRunwayDays: (state): number => {
      const total = state.accounts.reduce((sum, acc) => sum + acc.balanceVnd, 0);
      const burn = state.syncedDailyBurnVnd > 0
        ? state.syncedDailyBurnVnd
        : Math.round(state.accounts.reduce((sum, acc) => sum + (acc.outflowMonthVnd || 0), 0) / 30);
      if (burn <= 0) return total > 0 ? 999 : 0;
      return Math.round(total / burn);
    },

    totalInflowMonthVnd: (state) =>
      state.accounts.reduce((sum, acc) => sum + (acc.inflowMonthVnd || 0), 0),

    totalOutflowMonthVnd: (state) =>
      state.accounts.reduce((sum, acc) => sum + (acc.outflowMonthVnd || 0), 0),

    netFlowMonthVnd: (state) =>
      state.accounts.reduce(
        (sum, acc) => sum + ((acc.inflowMonthVnd || 0) - (acc.outflowMonthVnd || 0)),
        0
      ),

    activeAccounts: (state) => state.accounts.filter((a) => a.status === 'ACTIVE'),

    channels: (state) => state.accounts,

    breachedChannels: (state) =>
      state.accounts.filter(
        (a) => a.minReserveVnd !== undefined && a.balanceVnd < a.minReserveVnd
      ),

    hasReserveBreach: (state) =>
      state.accounts.some(
        (a) => a.minReserveVnd !== undefined && a.balanceVnd < a.minReserveVnd
      ),

    displayedAccounts: (state) => {
      if (state.selectedBankPersona === 'ALL') {
        return state.accounts;
      }
      return state.accounts.filter(
        (a) => a.bankCode === state.selectedBankPersona || a.channelCode === state.selectedBankPersona
      );
    },

    currentPersonaAccount: (state) => {
      if (state.selectedBankPersona === 'ALL') return null;
      return (
        state.accounts.find(
          (a) => a.bankCode === state.selectedBankPersona || a.channelCode === state.selectedBankPersona
        ) || null
      );
    },
  },

  actions: {
    setBankPersona(persona: 'ALL' | BankCode) {
      this.selectedBankPersona = persona;
    },

    setChannel(channel: 'ALL' | BankCode) {
      this.selectedBankPersona = channel;
    },

    updateBalance(bankCode: BankCode, newBalance: number) {
      const acc = this.accounts.find(
        (a) => a.bankCode === bankCode || a.channelCode === bankCode
      );
      if (acc) {
        acc.balanceVnd = newBalance;
        acc.availableBalanceVnd = newBalance;
      }
    },

    updateMinReserve(bankCode: BankCode, minReserveVnd: number) {
      const acc = this.accounts.find(
        (a) => a.bankCode === bankCode || a.channelCode === bankCode
      );
      if (acc && minReserveVnd >= 0) {
        acc.minReserveVnd = minReserveVnd;
      }
    },

    /**
     * Intraday / Overnight Cash Rebalancing between interbank channels
     */
    rebalanceLiquidity(
      fromChannel: BankCode,
      toChannel: BankCode,
      amountVnd: number,
      purpose: string,
      executedBy: string = 'CB-TREASURY-01'
    ): { success: boolean; message?: string; logId?: string; error?: string } {
      if (amountVnd <= 0) {
        return { success: false, error: 'Số tiền điều chuyển phải lớn hơn 0 VND.' };
      }
      if (fromChannel === toChannel) {
        return { success: false, error: 'Kênh nguồn và kênh đích phải khác nhau.' };
      }

      const fromAcc = this.accounts.find(
        (a) => a.bankCode === fromChannel || a.channelCode === fromChannel
      );
      const toAcc = this.accounts.find(
        (a) => a.bankCode === toChannel || a.channelCode === toChannel
      );

      if (!fromAcc || !toAcc) {
        return { success: false, error: 'Không tìm thấy thông tin kênh thanh toán tương ứng.' };
      }

      if (fromAcc.balanceVnd < amountVnd) {
        const rejectedLog: LiquidityRebalanceLog = {
          id: `REBAL-REJ-${Date.now()}`,
          timestamp: new Date().toISOString(),
          epochSeconds: Math.floor(Date.now() / 1000),
          fromChannel,
          toChannel,
          amountVnd,
          purpose,
          status: 'REJECTED',
          executedBy,
          note: `Từ chối: Số dư kênh ${fromChannel} (${fromAcc.balanceVnd.toLocaleString('vi-VN')} VND) không đủ để điều chuyển.`,
        };
        this.rebalanceLogs.unshift(rejectedLog);
        return {
          success: false,
          error: `Số dư kênh ${fromChannel} không đủ để thực hiện lệnh chuyển ${amountVnd.toLocaleString('vi-VN')} VND.`,
        };
      }

      // Execute transfer
      fromAcc.balanceVnd -= amountVnd;
      fromAcc.availableBalanceVnd -= amountVnd;
      fromAcc.outflowMonthVnd += amountVnd;

      toAcc.balanceVnd += amountVnd;
      toAcc.availableBalanceVnd += amountVnd;
      toAcc.inflowMonthVnd += amountVnd;

      const logId = `REBAL-${Date.now().toString().slice(-6)}`;
      const log: LiquidityRebalanceLog = {
        id: logId,
        timestamp: new Date().toISOString(),
        epochSeconds: Math.floor(Date.now() / 1000),
        fromChannel,
        toChannel,
        amountVnd,
        purpose: purpose || 'Điều chuyển vốn thanh khoản nội bộ liên ngân hàng',
        status: 'COMPLETED',
        executedBy,
        note: `Điều chuyển thành công từ ${fromChannel} sang ${toChannel}`,
      };

      this.rebalanceLogs.unshift(log);

      return {
        success: true,
        logId,
        message: `Đã điều chuyển thành công ${amountVnd.toLocaleString('vi-VN')} VND từ ${fromChannel} sang ${toChannel}.`,
      };
    },

    applyAccountUpdated(payload: {
      accountId?: string;
      channelCode?: string;
      bankCode?: BankCode;
      balanceVnd?: number;
      balance?: number;
      availableBalanceVnd?: number;
      inflowMonthVnd?: number;
      outflowMonthVnd?: number;
      lastUpdated?: string;
    }) {
      const target = payload.channelCode || payload.bankCode || payload.accountId;
      const acc = this.accounts.find(
        (a) =>
          (payload.channelCode && (a.channelCode === payload.channelCode || a.bankCode === payload.channelCode)) ||
          (payload.bankCode && (a.bankCode === payload.bankCode || a.channelCode === payload.bankCode)) ||
          (payload.bankCode === 'VCB' && (a.channelCode === 'CITAD' || a.bankCode === 'CITAD')) ||
          (payload.bankCode === 'TCB' && (a.channelCode === 'NAPAS' || a.bankCode === 'NAPAS')) ||
          (payload.bankCode === 'BIDV' && (a.channelCode === 'BILATERAL' || a.bankCode === 'BILATERAL')) ||
          (payload.accountId && a.accountNumber && a.accountNumber.includes(payload.accountId)) ||
          (target && (a.channelCode === target || a.bankCode === target || a.accountNumber === target))
      );
      if (acc) {
        const bal = payload.balanceVnd !== undefined ? payload.balanceVnd : payload.balance;
        if (bal !== undefined) {
          acc.balanceVnd = bal;
          acc.availableBalanceVnd = payload.availableBalanceVnd ?? bal;
        }
        if (payload.inflowMonthVnd !== undefined) acc.inflowMonthVnd = payload.inflowMonthVnd;
        if (payload.outflowMonthVnd !== undefined) acc.outflowMonthVnd = payload.outflowMonthVnd;
        if (payload.lastUpdated) acc.lastReconciledDate = payload.lastUpdated.split('T')[0];
      }
    },

    applyStateSnapshot(snapshot: { accounts?: any[]; dailyBurnVnd?: number; dailyAverageBurnVnd?: number }) {
      if (snapshot.accounts && Array.isArray(snapshot.accounts) && snapshot.accounts.length > 0) {
        for (const snapAcc of snapshot.accounts) {
          const local = this.accounts.find(
            (a) =>
              (snapAcc.channelCode && (a.channelCode === snapAcc.channelCode || a.bankCode === snapAcc.channelCode)) ||
              (snapAcc.bankCode && (a.bankCode === snapAcc.bankCode || a.channelCode === snapAcc.bankCode)) ||
              (snapAcc.bankCode === 'VCB' && a.channelCode === 'CITAD') ||
              (snapAcc.bankCode === 'TCB' && a.channelCode === 'NAPAS') ||
              (snapAcc.bankCode === 'BIDV' && a.channelCode === 'BILATERAL') ||
              (snapAcc.accountNumber && a.accountNumber === snapAcc.accountNumber)
          );
          const bal = snapAcc.balanceVnd !== undefined ? snapAcc.balanceVnd : snapAcc.balance;
          if (local && bal !== undefined) {
            local.balanceVnd = bal;
            local.availableBalanceVnd = snapAcc.availableBalanceVnd ?? bal;
            if (snapAcc.inflowMonthVnd !== undefined || snapAcc.inflows !== undefined) {
              local.inflowMonthVnd = snapAcc.inflowMonthVnd ?? snapAcc.inflows;
            }
            if (snapAcc.outflowMonthVnd !== undefined || snapAcc.outflows !== undefined) {
              local.outflowMonthVnd = snapAcc.outflowMonthVnd ?? snapAcc.outflows;
            }
            if (snapAcc.status) local.status = snapAcc.status;
          }
        }
      }
      const burn = snapshot.dailyAverageBurnVnd !== undefined ? snapshot.dailyAverageBurnVnd : snapshot.dailyBurnVnd;
      if (burn !== undefined && burn > 0) {
        this.syncedDailyBurnVnd = burn;
      }
    },

    async fetchAccountsFromServer(): Promise<boolean> {
      try {
        const res = await apiRequest<{ accounts: any[]; dailyAverageBurnVnd?: number }>('/banking/accounts');
        if (res.success && res.data?.accounts) {
          this.applyStateSnapshot(res.data);
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },
  },
});
