import { create } from "zustand";
import { AccountInfo, AntigravityStatus, QuotaRecord, ToastMessage } from "../types";
import { api } from "../services/api";

interface VaultState {
  accounts: AccountInfo[];
  quotas: Record<string, QuotaRecord[]>;
  antigravityStatus: AntigravityStatus | null;
  isLoading: boolean;
  isRefreshingAll: boolean;
  isScanningLocal: boolean;
  isLoggingIn: boolean;
  /// Set after a status/account poll failed, so the repeated poll reports only the
  /// first failure of a streak instead of spamming toasts every few seconds.
  statusPollFailed: boolean;
  accountsLoadFailed: boolean;
  switchingAccountId: string | null;
  selectedAccountForDetail: AccountInfo | null;
  detailModalTab: "summary" | "all";
  isAddModalOpen: boolean;
  switchTargetAccount: AccountInfo | null;
  /// Account awaiting removal confirmation (in-app modal instead of window.confirm).
  deleteTargetAccount: AccountInfo | null;
  isDeletingAccount: boolean;
  searchQuery: string;
  toasts: ToastMessage[];

  // Actions
  loadInitialData: () => Promise<void>;
  fetchAccounts: () => Promise<void>;
  fetchStatus: () => Promise<void>;
  fetchQuota: (accountId: string) => Promise<void>;
  refreshAllQuotas: (silent?: boolean) => Promise<void>;
  switchAccount: (accountId: string, autoRestart?: boolean) => Promise<boolean>;
  deleteAccount: (accountId: string) => Promise<void>;
  startGoogleLogin: () => Promise<boolean>;
  completeGoogleLogin: (codeOrUrl: string) => Promise<boolean>;
  cancelGoogleLogin: () => Promise<void>;
  importLocalAccounts: () => Promise<number>;
  launchAntigravity: () => Promise<void>;
  reorderAccounts: (sourceId: string, targetId: string) => Promise<void>;
  
  // UI helpers
  setSearchQuery: (query: string) => void;
  setSelectedAccountForDetail: (account: AccountInfo | null) => void;
  setDetailModalTab: (tab: "summary" | "all") => void;
  openAccountDetail: (account: AccountInfo, tab?: "summary" | "all") => void;
  setIsAddModalOpen: (open: boolean) => void;
  setSwitchTargetAccount: (account: AccountInfo | null) => void;
  setDeleteTargetAccount: (account: AccountInfo | null) => void;
  addToast: (toast: Omit<ToastMessage, "id">) => void;
  removeToast: (id: string) => void;

  // Theme
  theme: "dark" | "light";
  toggleTheme: () => void;
  setTheme: (theme: "dark" | "light") => void;
}

export const useVaultStore = create<VaultState>((set, get) => ({
  accounts: [],
  quotas: {},
  antigravityStatus: null,
  isLoading: false,
  isRefreshingAll: false,
  isScanningLocal: false,
  isLoggingIn: false,
  statusPollFailed: false,
  accountsLoadFailed: false,
  switchingAccountId: null,
  selectedAccountForDetail: null,
  detailModalTab: "summary",
  isAddModalOpen: false,
  switchTargetAccount: null,
  deleteTargetAccount: null,
  isDeletingAccount: false,
  searchQuery: "",
  toasts: [],
  theme: (localStorage.getItem("antivault_theme") as "dark" | "light") || "dark",

  toggleTheme: () => {
    const next = get().theme === "dark" ? "light" : "dark";
    localStorage.setItem("antivault_theme", next);
    if (next === "dark") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
    set({ theme: next });
  },

  setTheme: (theme: "dark" | "light") => {
    localStorage.setItem("antivault_theme", theme);
    if (theme === "dark") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
    set({ theme });
  },

  addToast: (toast) => {
    const id = `toast-${Date.now()}-${Math.random().toString(36).substring(2, 6)}`;
    const newToast: ToastMessage = { ...toast, id };
    set((state) => ({ toasts: [...state.toasts, newToast] }));
    setTimeout(() => {
      get().removeToast(id);
    }, 4000);
  },

  removeToast: (id) => {
    set((state) => ({ toasts: state.toasts.filter((t) => t.id !== id) }));
  },

  setSearchQuery: (searchQuery) => set({ searchQuery }),
  setSelectedAccountForDetail: (selectedAccountForDetail) => set({ selectedAccountForDetail }),
  setDetailModalTab: (detailModalTab) => set({ detailModalTab }),
  openAccountDetail: (account, tab = "all") => set({ selectedAccountForDetail: account, detailModalTab: tab }),
  setIsAddModalOpen: (isAddModalOpen) => set({ isAddModalOpen }),
  setSwitchTargetAccount: (switchTargetAccount) => set({ switchTargetAccount }),
  setDeleteTargetAccount: (deleteTargetAccount) => set({ deleteTargetAccount }),

  loadInitialData: async () => {
    set({ isLoading: true });
    try {
      await Promise.all([get().fetchAccounts(), get().fetchStatus()]);
      
      // Load cached quotas for accounts
      const accounts = get().accounts;
      for (const acc of accounts) {
        try {
          const quotaList = await api.getQuotas(acc.id);
          if (quotaList.length > 0) {
            set((state) => ({
              quotas: { ...state.quotas, [acc.id]: quotaList },
            }));
          }
        } catch {
          // ignore individual quota cache misses
        }
      }
    } catch (err: any) {
      get().addToast({
        type: "error",
        title: "初始化加载失败",
        description: err?.message || String(err),
      });
    } finally {
      set({ isLoading: false });
    }
  },

  fetchAccounts: async () => {
    try {
      const accounts = await api.listAccounts();
      set({ accounts, accountsLoadFailed: false });
    } catch (err: any) {
      console.error("Failed to fetch accounts:", err);
      if (!get().accountsLoadFailed) {
        set({ accountsLoadFailed: true });
        get().addToast({
          type: "error",
          title: "读取账号列表失败",
          description: err?.message || String(err),
        });
      }
    }
  },

  fetchStatus: async () => {
    try {
      const status = await api.getAntigravityStatus();
      set({ antigravityStatus: status, statusPollFailed: false });
    } catch (err: any) {
      console.error("Failed to check Antigravity status:", err);
      // The status poll runs every few seconds; report the first failure of a streak
      // instead of staying silent, then keep quiet until it recovers.
      if (!get().statusPollFailed) {
        set({ statusPollFailed: true });
        get().addToast({
          type: "warning",
          title: "无法读取 Antigravity 状态",
          description: err?.message || String(err),
        });
      }
    }
  },

  fetchQuota: async (accountId: string) => {
    try {
      const outcome = await api.refreshQuota(accountId);
      set((state) => ({
        quotas: { ...state.quotas, [accountId]: outcome.quotas },
      }));
      await get().fetchAccounts();

      if (outcome.warning) {
        get().addToast({
          type: "warning",
          title: "额度同步不完整",
          description: `账号凭证有效，但模型额度接口未返回数据：${outcome.warning}`,
        });
      } else if (outcome.quotas.length === 0) {
        get().addToast({
          type: "info",
          title: "暂无可用模型数据",
          description: "该账号当前没有返回任何模型额度信息。",
        });
      } else {
        get().addToast({
          type: "success",
          title: "模型额度已更新",
          description: `已同步 ${outcome.quotas.length} 个模型的剩余额度`,
        });
      }
    } catch (err: any) {
      get().addToast({
        type: "error",
        title: "额度刷新失败",
        description: err?.message || String(err),
      });
    }
  },

  refreshAllQuotas: async (silent = false) => {
    // If switching account or logging in, skip silent auto-refresh to prevent race conditions
    if (silent && (get().switchingAccountId || get().isLoggingIn)) {
      return;
    }
    if (get().isRefreshingAll) return;
    if (!silent) {
      set({ isRefreshingAll: true });
    }
    try {
      const result = await api.refreshAllQuotas();
      set({ accounts: result.accounts });

      // Reload quotas for all accounts
      for (const acc of result.accounts) {
        try {
          const qList = await api.getQuotas(acc.id);
          set((state) => ({
            quotas: { ...state.quotas, [acc.id]: qList },
          }));
        } catch {
          // keep previously cached quotas for this account
        }
      }

      if (!silent) {
        if (result.accounts.length === 0) {
          get().addToast({
            type: "info",
            title: "没有可刷新的账号",
            description: "请先添加一个 Antigravity 账号。",
          });
        } else if (result.failed.length === 0) {
          get().addToast({
            type: "success",
            title: "全部配额刷新成功",
            description: `已成功同步更新 ${result.succeeded.length} 个账号的最新额度`,
          });
        } else if (result.succeeded.length === 0) {
          get().addToast({
            type: "error",
            title: "全部账号刷新失败",
            description: result.failed
              .map((f) => `${f.email}: ${f.error}`)
              .join("；"),
          });
        } else {
          get().addToast({
            type: "warning",
            title: "部分账号刷新失败",
            description: `成功 ${result.succeeded.length} 个，失败 ${result.failed.length} 个：${result.failed
              .map((f) => `${f.email}: ${f.error}`)
              .join("；")}`,
          });
        }
      }
    } catch (err: any) {
      if (!silent) {
        get().addToast({
          type: "error",
          title: "批量刷新配额失败",
          description: err?.message || String(err),
        });
      } else {
        console.warn("Silent quota refresh error:", err);
      }
    } finally {
      if (!silent) {
        set({ isRefreshingAll: false });
      }
    }
  },

  switchAccount: async (accountId: string, autoRestart = true) => {
    set({ switchingAccountId: accountId });
    try {
      const result = await api.switchAccount(accountId, autoRestart);
      if (result.success) {
        // Optimistically update is_current in state so the card state flips instantly without position jumping
        set((state) => ({
          accounts: state.accounts.map((a) => ({
            ...a,
            is_current: a.id === accountId,
          })),
        }));
        await Promise.all([get().fetchAccounts(), get().fetchStatus()]);
        const desc = result.restarted
          ? "已成功切换账号并重新启动 Antigravity 客户端"
          : result.was_running
          ? "凭据已安全写入生效！您可在 IDE 中重载窗口（Reload Window）应用新凭据"
          : "Antigravity 2.0 及 IDE 当前凭据已切换生效";
        get().addToast({
          type: "success",
          title: "账号切换成功",
          description: desc,
        });
        return true;
      } else {
        get().addToast({
          type: "error",
          title: "切换失败",
          description: result.message || "切换过程中发生异常",
        });
        return false;
      }
    } catch (err: any) {
      get().addToast({
        type: "error",
        title: "账号切换异常",
        description: err?.message || String(err),
      });
      return false;
    } finally {
      set({ switchingAccountId: null, switchTargetAccount: null });
    }
  },

  reorderAccounts: async (sourceId: string, targetId: string) => {
    if (sourceId === targetId) return;
    const { accounts } = get();
    const sourceIndex = accounts.findIndex((a) => a.id === sourceId);
    const targetIndex = accounts.findIndex((a) => a.id === targetId);
    if (sourceIndex === -1 || targetIndex === -1) return;

    const newAccounts = [...accounts];
    const [moved] = newAccounts.splice(sourceIndex, 1);
    newAccounts.splice(targetIndex, 0, moved);

    // Optimistic local state update for zero latency
    set({ accounts: newAccounts });

    try {
      await api.reorderAccounts(newAccounts.map((a) => a.id));
    } catch (err: any) {
      console.error("Failed to persist account order:", err);
      await get().fetchAccounts();
    }
  },

  deleteAccount: async (accountId: string) => {
    set({ isDeletingAccount: true });
    try {
      await api.deleteAccount(accountId);
      set((state) => ({
        accounts: state.accounts.filter((a) => a.id !== accountId),
        quotas: Object.fromEntries(
          Object.entries(state.quotas).filter(([k]) => k !== accountId)
        ),
      }));
      get().addToast({
        type: "info",
        title: "账号已移除",
        description: "该账号凭证已从本地安全存储中移除",
      });
    } catch (err: any) {
      get().addToast({
        type: "error",
        title: "移除失败",
        description: err?.message || String(err),
      });
    } finally {
      set({ isDeletingAccount: false, deleteTargetAccount: null });
    }
  },

  startGoogleLogin: async () => {
    if (get().isLoggingIn) {
      get().addToast({
        type: "warning",
        title: "授权正在进行中",
        description: "请先在上一个浏览器窗口中完成授权，或等待其超时。",
      });
      return false;
    }

    set({ isLoggingIn: true });
    try {
      get().addToast({
        type: "info",
        title: "正在发起 Google 授权",
        description: "请在弹出的系统浏览器窗口中完成账号登录与授权（最长等待 10 分钟）",
      });
      const newAccount = await api.startGoogleLogin();
      await get().fetchAccounts();
      if (newAccount.id) {
        try {
          const qList = await api.getQuotas(newAccount.id);
          set((state) => ({
            quotas: { ...state.quotas, [newAccount.id]: qList },
          }));
        } catch {
          // quota cache is optional here; the account itself was saved
        }
      }
      get().addToast({
        type: "success",
        title: "Google 账号已连接",
        description: `已成功授权添加：${newAccount.email}`,
      });
      return true;
    } catch (err: any) {
      const message = err?.message || String(err);
      // The waiting browser flow is unwound when the user finishes the login by
      // pasting a code (already reported) or dismisses the modal (intentional).
      if (
        message.includes("__ANTIVAULT_MANUAL_COMPLETION__") ||
        message.includes("__ANTIVAULT_LOGIN_CANCELLED__")
      ) {
        return false;
      }
      get().addToast({
        type: "error",
        title: "授权登录失败",
        description: message,
      });
      return false;
    } finally {
      set({ isLoggingIn: false });
    }
  },

  completeGoogleLogin: async (codeOrUrl: string) => {
    if (!codeOrUrl.trim()) {
      get().addToast({
        type: "warning",
        title: "请输入授权码",
        description: "请粘贴浏览器地址栏中的完整回调链接，或 Google 返回的授权码。",
      });
      return false;
    }

    // `isLoggingIn` is owned by `startGoogleLogin`: the waiting browser call is
    // still in flight while this manual path finishes it, so this must not clear it.
    try {
      const newAccount = await api.completeGoogleLogin(codeOrUrl.trim());
      await get().fetchAccounts();
      try {
        const qList = await api.getQuotas(newAccount.id);
        set((state) => ({
          quotas: { ...state.quotas, [newAccount.id]: qList },
        }));
      } catch {
        // quota cache is optional here
      }
      get().addToast({
        type: "success",
        title: "Google 账号已连接",
        description: `已通过授权码添加：${newAccount.email}`,
      });
      return true;
    } catch (err: any) {
      get().addToast({
        type: "error",
        title: "授权码校验失败",
        description: err?.message || String(err),
      });
      return false;
    }
  },

  cancelGoogleLogin: async () => {
    try {
      await api.cancelGoogleLogin();
    } catch {
      // nothing to do — the flow may already be gone
    }
  },

  importLocalAccounts: async () => {
    if (get().isScanningLocal) {
      get().addToast({
        type: "warning",
        title: "扫描正在进行中",
        description: "请等待当前扫描结束后再试。",
      });
      return 0;
    }

    set({ isScanningLocal: true });
    get().addToast({
      type: "info",
      title: "正在扫描本地凭据",
      description: "正在读取 Windows 凭据管理器与 Antigravity 本地数据库，请稍候…",
    });

    try {
      const discovered = await api.importLocalAccounts();
      if (discovered.length > 0) {
        await get().fetchAccounts();
        for (const acc of discovered) {
          await get().fetchQuota(acc.id);
        }
        get().addToast({
          type: "success",
          title: "本地凭据扫描完成",
          description: `成功发现并导入 ${discovered.length} 个 Antigravity 账号凭证`,
        });
        return discovered.length;
      } else {
        get().addToast({
          type: "info",
          title: "未发现新账号",
          description:
            "未在 Windows 凭据管理器（gemini:antigravity）或 Antigravity 本地数据库中找到新的凭据。",
        });
        return 0;
      }
    } catch (err: any) {
      get().addToast({
        type: "error",
        title: "本地扫描失败",
        description: err?.message || String(err),
      });
      return 0;
    } finally {
      set({ isScanningLocal: false });
    }
  },

  launchAntigravity: async () => {
    try {
      await api.launchAntigravityApp();
      get().addToast({
        type: "info",
        title: "正在启动客户端",
        description: "已向系统发送 Antigravity 启动请求",
      });
      setTimeout(() => {
        get().fetchStatus();
      }, 1500);
    } catch (err: any) {
      get().addToast({
        type: "error",
        title: "启动失败",
        description: err?.message || String(err),
      });
    }
  },
}));
