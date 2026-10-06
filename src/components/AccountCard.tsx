import React, { useState, useMemo } from "react";
import { 
  Check, 
  RotateCw, 
  Trash2, 
  ArrowRightLeft,
  ChevronRight,
  Copy,
  CheckCheck,
  GripVertical
} from "lucide-react";
import { AccountInfo } from "../types";
import { useVaultStore } from "../stores/useVaultStore";
import { 
  formatResetCountdown, 
  getProgressColor, 
  getPercentTextColor, 
  getMergedIndividualModels 
} from "../utils/quotaFormat";
import { QuotaRing } from "./QuotaRing";

interface AccountCardProps {
  account: AccountInfo;
}

let globalDraggedId: string | null = null;

export const AccountCard: React.FC<AccountCardProps> = ({ account }) => {
  const { 
    quotas, 
    switchingAccountId, 
    fetchQuota, 
    switchAccount,
    antigravityStatus,
    openAccountDetail,
    setDeleteTargetAccount,
    reorderAccounts,
    addToast,
    quotaChartType,
  } = useVaultStore();

  const [copied, setCopied] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const [isDragOver, setIsDragOver] = useState(false);

  const accountQuotas = Array.isArray(quotas[account?.id]) ? quotas[account.id] : [];
  const isSwitching = switchingAccountId === account?.id;

  // Extract core 5h & Weekly buckets with full null-safety
  const claude5h = accountQuotas.find(q => q?.model_name === "Claude (5h)") || 
    accountQuotas.find(q => typeof q?.model_name === "string" && q.model_name.toLowerCase().includes("claude"));
  const claudeWeekly = accountQuotas.find(q => q?.model_name === "Claude (Weekly)");

  const gemini5h = accountQuotas.find(q => q?.model_name === "Gemini (5h)") || 
    accountQuotas.find(q => typeof q?.model_name === "string" && q.model_name.toLowerCase().includes("gemini"));
  const geminiWeekly = accountQuotas.find(q => q?.model_name === "Gemini (Weekly)");

  const aiCredits = accountQuotas.find(q => q?.model_name === "AI Credits");

  // Individual detailed models count (merged without difficulty tiers)
  const individualModels = useMemo(() => {
    return getMergedIndividualModels(accountQuotas);
  }, [accountQuotas]);

  const getTierBadge = (tier: string | null) => {
    const t = (tier || "FREE").toUpperCase();
    if (t.includes("ULTRA")) {
      return (
        <span className="px-1.5 py-0.2 text-[10px] font-bold rounded bg-purple-50 text-purple-700 border border-purple-200/70">
          ULTRA
        </span>
      );
    }
    if (t.includes("PRO")) {
      return (
        <span className="px-1.5 py-0.2 text-[10px] font-bold rounded bg-emerald-50 text-emerald-700 border border-emerald-200/70">
          PRO
        </span>
      );
    }
    return (
      <span className="px-1.5 py-0.2 text-[10px] font-medium rounded bg-slate-100 text-slate-500 border border-slate-200/60">
        FREE
      </span>
    );
  };

  const handleSwitchClick = () => {
    if (account.is_current || isSwitching) return;
    switchAccount(account.id, antigravityStatus?.is_running ?? false);
  };

  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setDeleteTargetAccount(account);
  };

  const handleCopyEmail = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(account.email);
    setCopied(true);
    addToast({
      type: "info",
      title: "已复制邮箱",
      description: account.email,
    });
    setTimeout(() => setCopied(false), 2000);
  };

  const handleRefresh = async (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsRefreshing(true);
    try {
      await fetchQuota(account.id);
    } finally {
      setIsRefreshing(false);
    }
  };

  const initial = (account.display_name || account.email)[0].toUpperCase();

  const renderQuotaRingItem = (label: string, quota: any) => {
    const percent = quota?.remaining_percent;
    const cd = formatResetCountdown(quota?.reset_at);

    return (
      <div 
        className="flex items-center space-x-2.5 py-1"
        title={cd.dateStr ? `${label} 重置: ${cd.dateStr} (${cd.timeStr})` : undefined}
      >
        <QuotaRing
          percent={percent}
          size={48}
          strokeWidth={3.8}
        />
        <div className="min-w-0 flex-1">
          <div className="flex items-center justify-between leading-snug">
            <span className="font-bold text-xs text-slate-800 dark:text-slate-200">
              {label}
            </span>
            <span className="font-mono font-semibold text-xs text-slate-700 dark:text-slate-300">
              {cd.timeStr}
            </span>
          </div>
          {cd.dateStr && (
            <div className="font-mono text-[11px] font-medium leading-snug text-slate-400 dark:text-slate-500 text-right mt-0.5">
              ({cd.dateStr})
            </div>
          )}
        </div>
      </div>
    );
  };

  const renderQuotaBarItem = (label: string, quota: any) => {
    const percent = quota?.remaining_percent;
    const hasPercent = percent !== null && percent !== undefined;
    const val = hasPercent ? Math.min(100, Math.max(0, percent)) : 100;
    const cd = formatResetCountdown(quota?.reset_at);

    return (
      <div 
        className="space-y-1 py-0.5"
        title={cd.dateStr ? `${label} 重置: ${cd.dateStr} (${cd.timeStr})` : undefined}
      >
        <div className="flex items-center justify-between text-xs">
          <span className="font-bold text-xs text-slate-800 dark:text-slate-200">{label}</span>
          <span className={`font-mono font-bold text-xs ${getPercentTextColor(percent ?? null)}`}>
            {hasPercent ? `${Math.round(percent)}%` : "100%"}
          </span>
        </div>
        <div 
          className="w-full h-1.5 rounded-full bg-slate-200/70 dark:bg-[#282d38] overflow-hidden"
        >
          <div
            className={`h-full rounded-full transition-all duration-300 ${getProgressColor(hasPercent ? percent : 100)}`}
            style={{ width: `${val}%` }}
          />
        </div>
        <div className="flex items-center justify-between text-[11px] text-slate-400 dark:text-slate-500 font-mono">
          <span>{cd.timeStr}</span>
          {cd.dateStr && (
            <span>({cd.dateStr})</span>
          )}
        </div>
      </div>
    );
  };

  const renderQuotaItem = (label: string, quota: any) => {
    if (quotaChartType === "bar") {
      return renderQuotaBarItem(label, quota);
    }
    return renderQuotaRingItem(label, quota);
  };

  const handleDragStart = (e: React.DragEvent) => {
    globalDraggedId = account.id;
    e.dataTransfer.setData("text/plain", account.id);
    e.dataTransfer.effectAllowed = "move";
    setTimeout(() => {
      setIsDragging(true);
    }, 0);
  };

  const handleDragEnd = () => {
    globalDraggedId = null;
    setIsDragging(false);
    setIsDragOver(false);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    if (!isDragOver) {
      setIsDragOver(true);
    }
  };

  const handleDragLeave = (e: React.DragEvent) => {
    if (!e.currentTarget.contains(e.relatedTarget as Node)) {
      setIsDragOver(false);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    setIsDragging(false);
    const sourceId = e.dataTransfer.getData("text/plain") || globalDraggedId;
    globalDraggedId = null;
    if (sourceId && sourceId !== account.id) {
      reorderAccounts(sourceId, account.id);
    }
  };

  return (
    <div
      draggable
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      className={`group relative rounded-2xl border transition-all duration-200 shadow-2xs hover:shadow-xs w-full ${
        isDragging
          ? "opacity-35 scale-[0.98] border-dashed border-emerald-500"
          : isDragOver
          ? "ring-2 ring-emerald-500 border-emerald-500 scale-[1.01] shadow-md"
          : account.is_current 
          ? "border-emerald-500/80 dark:border-emerald-500/70 ring-1 ring-emerald-500/20 bg-gradient-to-b from-emerald-50/15 to-white dark:from-[#1b2723] dark:to-[#20242b]" 
          : "bg-white dark:bg-[#20242b] border-slate-200/80 dark:border-[#2c323f] hover:border-slate-300 dark:hover:border-[#3b4354]"
      } p-3.5 sm:p-4 flex flex-col justify-between`}
    >
      {/* 顶部行：头像、名称、邮箱、当前生效或一键切换 */}
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center space-x-2 min-w-0">
          {/* 拖动抓手手柄 */}
          <div
            className="text-slate-300 dark:text-slate-600 hover:text-slate-500 dark:hover:text-slate-400 cursor-grab active:cursor-grabbing p-1 rounded transition-colors shrink-0"
            title="按住拖拽调整卡片排序"
          >
            <GripVertical className="w-3.5 h-3.5" />
          </div>

          <div className="relative shrink-0">
            <div className={`w-9 h-9 rounded-lg flex items-center justify-center font-bold text-xs shadow-2xs ${
              account.is_current
                ? "bg-emerald-600 text-white"
                : "bg-slate-100 dark:bg-[#282d38] border border-slate-200/80 dark:border-[#333947] text-slate-700 dark:text-slate-200"
            }`}>
              {account.avatar_url ? (
                <img src={account.avatar_url} alt="" draggable={false} className="w-full h-full rounded-lg object-cover pointer-events-none select-none" />
              ) : (
                initial
              )}
            </div>
            {account.is_current && (
              <span className="absolute -bottom-0.5 -right-0.5 w-3.5 h-3.5 bg-emerald-500 rounded-full border-2 border-white dark:border-slate-900 flex items-center justify-center">
                <Check className="w-2 h-2 text-white stroke-[3]" />
              </span>
            )}
          </div>

          <div className="min-w-0 flex-1">
            <div className="flex items-center space-x-2">
              <span className="text-sm font-bold text-slate-900 dark:text-slate-100 truncate">
                {account.display_name || account.email.split("@")[0]}
              </span>
              {getTierBadge(account.subscription_type)}
            </div>
            <div className="flex items-center space-x-1 mt-0.5">
              <span className="text-xs text-slate-400 dark:text-slate-500 truncate font-mono">
                {account.email}
              </span>
              <button
                type="button"
                onClick={handleCopyEmail}
                className="text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 p-0.5 rounded transition-colors"
                title="复制邮箱"
              >
                {copied ? <CheckCheck className="w-3 h-3 text-emerald-600" /> : <Copy className="w-3 h-3" />}
              </button>
            </div>
          </div>
        </div>

        {/* 状态或切换按钮 */}
        {account.is_current ? (
          <div className="flex items-center space-x-1 px-2.5 py-1 rounded-full bg-emerald-50 dark:bg-emerald-950/50 border border-emerald-200/70 dark:border-emerald-800/70 text-emerald-700 dark:text-emerald-400 text-xs font-semibold shrink-0">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
            <span>生效中</span>
          </div>
        ) : (
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleSwitchClick();
            }}
            disabled={isSwitching}
            className="flex items-center space-x-1.5 px-3 py-1.5 rounded-xl bg-slate-100 dark:bg-[#282d38] hover:bg-emerald-600 dark:hover:bg-emerald-600 border border-slate-200/90 dark:border-[#333947] text-slate-700 dark:text-slate-200 hover:text-white dark:hover:text-white text-xs font-medium transition-all active:scale-95 disabled:opacity-50 shrink-0 shadow-2xs"
          >
            <ArrowRightLeft className={`w-3 h-3 ${isSwitching ? "animate-spin" : ""}`} />
            <span>{isSwitching ? "切换中" : "切换"}</span>
          </button>
        )}
      </div>

      {/* 核心额度看板：Gemini 与 Claude 对称和谐展示 */}
      <div className="mt-3 p-3 rounded-2xl bg-slate-50/70 dark:bg-[#181b20] border border-slate-200/70 dark:border-[#282c37]">
        <div className="grid grid-cols-2 gap-3 divide-x divide-slate-200/70 dark:divide-[#282c37]">
          
          {/* Gemini 列 */}
          <div className="space-y-1.5 pr-1.5">
            <div className="flex items-center space-x-1.5 pb-0.5">
              <span className="w-2 h-2 rounded-full bg-blue-500 shadow-2xs" />
              <span className="font-bold text-xs text-slate-800 dark:text-slate-200">Gemini</span>
            </div>
            {renderQuotaItem("5h", gemini5h)}
            {renderQuotaItem("Weekly", geminiWeekly)}
          </div>

          {/* Claude 列 */}
          <div className="space-y-1.5 pl-3">
            <div className="flex items-center space-x-1.5 pb-0.5">
              <span className="w-2 h-2 rounded-full bg-amber-500 shadow-2xs" />
              <span className="font-bold text-xs text-slate-800 dark:text-slate-200">Claude</span>
            </div>
            {renderQuotaItem("5h", claude5h)}
            {renderQuotaItem("Weekly", claudeWeekly)}
          </div>

        </div>

        {/* 可用 AI 积分 */}
        {aiCredits && (
          <div className="mt-2.5 pt-2 border-t border-slate-200/50 dark:border-slate-800/50 flex items-center justify-between text-xs text-slate-500 dark:text-slate-400">
            <span>可用 AI 积分: <strong className="font-mono text-slate-700 dark:text-slate-200 font-semibold">{aiCredits.remaining_value ?? 0}</strong></span>
            {accountQuotas.length === 0 && (
              <button
                onClick={handleRefresh}
                className="text-emerald-600 hover:text-emerald-700 font-medium"
              >
                获取配额
              </button>
            )}
          </div>
        )}
      </div>

      {/* 底栏：模型总数入口与快捷操作 */}
      <div className="mt-2.5 pt-1.5 flex items-center justify-between text-xs text-slate-400 dark:text-slate-500">
        <button
          onClick={(e) => {
            e.stopPropagation();
            openAccountDetail(account, "all");
          }}
          className="flex items-center space-x-1 text-slate-500 dark:text-slate-400 hover:text-emerald-600 dark:hover:text-emerald-400 text-xs font-medium transition-colors"
        >
          <span>全部模型 ({individualModels.length > 0 ? individualModels.length : accountQuotas.length})</span>
          <ChevronRight className="w-3.5 h-3.5" />
        </button>

        <div className="flex items-center space-x-1" onClick={(e) => e.stopPropagation()}>
          <button
            onClick={handleRefresh}
            disabled={isRefreshing}
            title="刷新此账号"
            className="w-7 h-7 flex items-center justify-center rounded-lg hover:bg-slate-100 dark:hover:bg-[#282d38] text-slate-400 hover:text-emerald-600 dark:hover:text-emerald-400 transition-colors disabled:opacity-50"
          >
            <RotateCw className={`w-3.5 h-3.5 ${isRefreshing ? "animate-spin text-emerald-600" : ""}`} />
          </button>
          <button
            onClick={handleDeleteClick}
            title="移除账号"
            className="w-7 h-7 flex items-center justify-center rounded-lg hover:bg-rose-50 dark:hover:bg-rose-950/40 text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 transition-colors"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>
  );
};
