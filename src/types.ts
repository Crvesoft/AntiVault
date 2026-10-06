export interface AccountInfo {
  id: string;
  email: string;
  display_name: string | null;
  avatar_url: string | null;
  subscription_type: string | null;
  status: string;
  is_current: boolean;
  created_at: number;
  updated_at: number;
  last_used_at: number | null;
  last_quota_update_at: number | null;
  sort_order?: number;
}

export interface AddAccountRequest {
  email: string;
  display_name?: string | null;
  refresh_token: string;
  access_token?: string | null;
  subscription_type?: string | null;
}

export interface QuotaRecord {
  id: string;
  account_id: string;
  model_name: string;
  remaining_percent: number | null;
  remaining_value: number | null;
  limit_value: number | null;
  reset_at: number | null;
  status: string;
  updated_at: number;
}

export interface AntigravityStatus {
  is_running: boolean;
  executable_path: string | null;
  db_path: string | null;
  running_pids: number[];
}

export interface SwitchResult {
  success: boolean;
  was_running: boolean;
  restarted: boolean;
  message: string | null;
}

export interface QuotaRefreshOutcome {
  quotas: QuotaRecord[];
  warning: string | null;
}

export interface RefreshFailure {
  account_id: string;
  email: string;
  error: string;
}

export interface RefreshAllResult {
  accounts: AccountInfo[];
  succeeded: string[];
  failed: RefreshFailure[];
}

export interface ToastMessage {
  id: string;
  type: 'success' | 'error' | 'info' | 'warning';
  title: string;
  description?: string;
}

export type QuotaChartType = "ring" | "bar";

export interface AppSettings {
  close_to_tray: boolean;
  auto_check_update: boolean;
  quota_chart_type?: QuotaChartType;
}

export interface UpdateCheckResult {
  has_update: boolean;
  current_version: string;
  latest_version: string;
  release_notes: string | null;
  release_url: string | null;
  published_at: string | null;
  message: string | null;
}

