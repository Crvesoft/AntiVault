import { invoke, isTauri } from "@tauri-apps/api/core";
import {
  AccountInfo,
  AddAccountRequest,
  AntigravityStatus,
  QuotaRecord,
  QuotaRefreshOutcome,
  RefreshAllResult,
  SwitchResult,
} from "../types";

export const api = {
  async listAccounts(): Promise<AccountInfo[]> {
    if (!isTauri()) return mockAccounts;
    return await invoke<AccountInfo[]>("list_accounts");
  },

  async getAccount(id: string): Promise<AccountInfo> {
    if (!isTauri()) return mockAccounts.find(a => a.id === id) || mockAccounts[0];
    return await invoke<AccountInfo>("get_account", { id });
  },

  async getCurrentAccount(): Promise<AccountInfo | null> {
    if (!isTauri()) return mockAccounts.find(a => a.is_current) || null;
    return await invoke<AccountInfo | null>("get_current_account");
  },

  async addAccount(request: AddAccountRequest): Promise<AccountInfo> {
    if (!isTauri()) {
      const newAcc: AccountInfo = {
        id: `mock-${Date.now()}`,
        email: request.email,
        display_name: request.display_name || null,
        avatar_url: null,
        subscription_type: request.subscription_type || "FREE",
        status: "active",
        is_current: false,
        created_at: Math.floor(Date.now() / 1000),
        updated_at: Math.floor(Date.now() / 1000),
        last_used_at: null,
        last_quota_update_at: null,
      };
      mockAccounts.push(newAcc);
      return newAcc;
    }
    return await invoke<AccountInfo>("add_account", { request });
  },

  async deleteAccount(id: string): Promise<void> {
    if (!isTauri()) {
      const idx = mockAccounts.findIndex(a => a.id === id);
      if (idx !== -1) mockAccounts.splice(idx, 1);
      return;
    }
    await invoke<void>("delete_account", { id });
  },

  async setCurrentAccount(id: string): Promise<void> {
    if (!isTauri()) {
      mockAccounts.forEach(a => a.is_current = (a.id === id));
      return;
    }
    await invoke<void>("set_current_account", { id });
  },

  async reorderAccounts(accountIds: string[]): Promise<void> {
    if (!isTauri()) {
      mockAccounts.sort((a, b) => accountIds.indexOf(a.id) - accountIds.indexOf(b.id));
      return;
    }
    await invoke<void>("reorder_accounts", { accountIds });
  },

  async importLocalAccounts(): Promise<AccountInfo[]> {
    if (!isTauri()) return [];
    return await invoke<AccountInfo[]>("import_local_accounts");
  },

  async refreshQuota(accountId: string): Promise<QuotaRefreshOutcome> {
    if (!isTauri()) return { quotas: mockQuotas(accountId), warning: null };
    return await invoke<QuotaRefreshOutcome>("refresh_quota", { accountId });
  },

  async getQuotas(accountId: string): Promise<QuotaRecord[]> {
    if (!isTauri()) return mockQuotas(accountId);
    return await invoke<QuotaRecord[]>("get_quotas", { accountId });
  },

  async refreshAllQuotas(): Promise<RefreshAllResult> {
    if (!isTauri()) {
      return {
        accounts: mockAccounts,
        succeeded: mockAccounts.map((a) => a.id),
        failed: [],
      };
    }
    return await invoke<RefreshAllResult>("refresh_all_quotas");
  },

  async getAntigravityStatus(): Promise<AntigravityStatus> {
    if (!isTauri()) {
      return {
        is_running: true,
        executable_path: "C:\\Program Files\\Antigravity\\Antigravity.exe",
        db_path: "C:\\Users\\User\\AppData\\Roaming\\Antigravity IDE\\User\\globalStorage\\state.vscdb",
        running_pids: [12345],
      };
    }
    return await invoke<AntigravityStatus>("get_antigravity_status");
  },

  async closeAntigravityProcess(): Promise<boolean> {
    if (!isTauri()) return true;
    return await invoke<boolean>("close_antigravity_process");
  },

  async launchAntigravityApp(exePath?: string): Promise<void> {
    if (!isTauri()) return;
    await invoke<void>("launch_antigravity_app", { exePath });
  },

  async switchAccount(accountId: string, autoRestart?: boolean): Promise<SwitchResult> {
    if (!isTauri()) {
      mockAccounts.forEach(a => a.is_current = (a.id === accountId));
      return {
        success: true,
        was_running: true,
        restarted: autoRestart ?? true,
        message: "Switched mock account",
      };
    }
    return await invoke<SwitchResult>("switch_account", { accountId, autoRestart });
  },

  async startGoogleLogin(): Promise<AccountInfo> {
    if (!isTauri()) throw new Error("Google OAuth only available in desktop app");
    return await invoke<AccountInfo>("start_google_login");
  },

  async completeGoogleLogin(codeOrUrl: string): Promise<AccountInfo> {
    if (!isTauri()) throw new Error("Google OAuth only available in desktop app");
    return await invoke<AccountInfo>("complete_google_login", { codeOrUrl });
  },

  async cancelGoogleLogin(): Promise<void> {
    if (!isTauri()) return;
    await invoke<void>("cancel_google_login");
  },

  async saveWindowSize(width: number, height: number, isMaximized: boolean): Promise<void> {
    if (!isTauri()) return;
    await invoke<void>("save_window_size", { width, height, isMaximized });
  },

  async getSavedWindowSize(): Promise<{ width: number; height: number; is_maximized: boolean } | null> {
    if (!isTauri()) return null;
    return await invoke<{ width: number; height: number; is_maximized: boolean } | null>("get_saved_window_size");
  },
};

// Mock data for preview/fallback testing
const mockAccounts: AccountInfo[] = [
  {
    id: "acc-1",
    email: "dev.primary@gmail.com",
    display_name: "Primary Dev",
    avatar_url: null,
    subscription_type: "PRO",
    status: "active",
    is_current: true,
    created_at: 1727800000,
    updated_at: 1727900000,
    last_used_at: 1727900000,
    last_quota_update_at: 1727900000,
  },
  {
    id: "acc-2",
    email: "backup.coder@gmail.com",
    display_name: "Backup Coder",
    avatar_url: null,
    subscription_type: "FREE",
    status: "active",
    is_current: false,
    created_at: 1727700000,
    updated_at: 1727850000,
    last_used_at: 1727800000,
    last_quota_update_at: 1727850000,
  },
];

function mockQuotas(accountId: string): QuotaRecord[] {
  const now = Math.floor(Date.now() / 1000);
  const isAcc1 = accountId === "acc-1";
  return [
    {
      id: `${accountId}_claude_5h`,
      account_id: accountId,
      model_name: "Claude (5h)",
      remaining_percent: 100,
      remaining_value: null,
      limit_value: null,
      reset_at: now + (isAcc1 ? 18000 : 17460),
      status: "active",
      updated_at: now,
    },
    {
      id: `${accountId}_claude_weekly`,
      account_id: accountId,
      model_name: "Claude (Weekly)",
      remaining_percent: isAcc1 ? 100 : 47,
      remaining_value: null,
      limit_value: null,
      reset_at: now + 561600,
      status: "active",
      updated_at: now,
    },
    {
      id: `${accountId}_gemini_5h`,
      account_id: accountId,
      model_name: "Gemini (5h)",
      remaining_percent: isAcc1 ? 90 : 100,
      remaining_value: null,
      limit_value: null,
      reset_at: now + 16740,
      status: "active",
      updated_at: now,
    },
    {
      id: `${accountId}_gemini_weekly`,
      account_id: accountId,
      model_name: "Gemini (Weekly)",
      remaining_percent: isAcc1 ? 89 : 99,
      remaining_value: null,
      limit_value: null,
      reset_at: now + 544000,
      status: "active",
      updated_at: now,
    },
    {
      id: `${accountId}_ai_credits`,
      account_id: accountId,
      model_name: "AI Credits",
      remaining_percent: null,
      remaining_value: isAcc1 ? 0 : 50,
      limit_value: null,
      reset_at: null,
      status: "active",
      updated_at: now,
    },
    {
      id: `${accountId}_claude-3-7-sonnet`,
      account_id: accountId,
      model_name: "Claude Sonnet 4.6 (Thinking)",
      remaining_percent: isAcc1 ? 100 : 47,
      remaining_value: null,
      limit_value: null,
      reset_at: now + 18000,
      status: "active",
      updated_at: now,
    },
    {
      id: `${accountId}_gemini-2-5-pro`,
      account_id: accountId,
      model_name: "Gemini 2.5 Pro",
      remaining_percent: isAcc1 ? 90 : 100,
      remaining_value: null,
      limit_value: null,
      reset_at: now + 16740,
      status: "active",
      updated_at: now,
    },
  ];
}
