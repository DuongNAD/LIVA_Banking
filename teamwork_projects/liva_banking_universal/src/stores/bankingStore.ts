/**
 * Pinia Store for Internal Commercial Bank Liquidity & Interbank Settlement Channels
 * Complies with Circular 09/2020/TT-NHNN, Circular 09/2023/TT-NHNN & Universal Banking Operations.
 */

import { defineStore } from 'pinia';
import type { BankCode, LiquidityChannelProfile } from '../types/banking';

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
    dailyAverageBurnVnd: 85_000_000, // For 30-45 day runway computation
    rebalanceLogs: [
      {
        id: 'REBAL-INIT-01',
        timestamp: '2026-08-30T16:45:00.000Z',
        epochSeconds: 1788108300,
        fromChannel: 'CITAD',
        toChannel: 'NAPAS',
        amountVnd: 200_000_000,
        purpose: 'Bổ sung hạn mức thanh toán bù trừ đa phương cuối ngày NAPAS 24/7',
        status: 'COMPLETED',
        executedBy: 'CB-CHECKER-01',
        note: 'Đã hoàn tất quyết toán điện tử',
      },
    ] as LiquidityRebalanceLog[],
  }),

  getters: {
    totalLiquidVnd: (state) =>
      state.accounts.reduce((sum, acc) => sum + acc.balanceVnd, 0),

    liquidityRunwayDays: (state) => {
      const total = state.accounts.reduce((sum, acc) => sum + acc.balanceVnd, 0);
      return Math.round(total / (state.dailyAverageBurnVnd || 1));
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
  },
});
