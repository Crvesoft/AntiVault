import React, { useState, useMemo, useEffect } from "react";
import { X, RotateCw, Clock, Search, Layers, ShieldCheck, Sparkles, CheckCircle2 } from "lucide-react";
import { useVaultStore } from "../stores/useVaultStore";
import { formatResetCountdown, getProgressColor, getPercentTextColor, getMergedIndividualModels } from "../utils/quotaFormat";
import { ErrorBoundary } from "./ErrorBoundary";
import { QuotaRing } from "./QuotaRing";

export const QuotaDetailModal: React.FC = () => {
  const { 
    selectedAccountForDetail, 
    setSelectedAccountForDetail, 
    detailModalTab,
    setDetailModalTab,
    quotas, 
    fetchQuota,
    accounts,
    quotaChartType,
  } = useVaultStore();

  const [modelSearch, setModelSearch] = useState("");
  const [isRefreshing, setIsRefreshing] = useState(false);

  // Close on Escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && selectedAccountForDetail) {
        setSelectedAccountForDetail(null);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [selectedAccountForDetail, setSelectedAccountForDetail]);

  // Safely resolve account and quota list unconditionally
  const account = selectedAccountForDetail 
    ? (accounts.find(a => a.id === selectedAccountForDetail.id) || selectedAccountForDetail) 
    : null;
  const quotaList = account && Array.isArray(quotas[account.id]) ? quotas[account.id] : [];

  // Individual detailed models (merged without difficulty tiers) - executed unconditionally for React hook rules
  const individualModels = useMemo(() => {
    if (!account) return [];
    const merged = getMergedIndividualModels(quotaList);
    const s = (modelSearch || "").trim().toLowerCase();
    if (!s) return merged;
    return merged.filter(m => (m?.model_name || "").toLowerCase().includes(s));
  }, [account, quotaList, modelSearch]);

  if (!selectedAccountForDetail || !account) return null;

  // Summary buckets with comprehensive null-safety
  const claude5h = quotaList.find(q => q?.model_name === "Claude (5h)") || 
    quotaList.find(q => typeof q?.model_name === "string" && q.model_name.toLowerCase().includes("claude"));
  const claudeWeekly = quotaList.find(q => q?.model_name === "Claude (Weekly)");

  const gemini5h = quotaList.find(q => q?.model_name === "Gemini (5h)") || 
    quotaList.find(q => typeof q?.model_name === "string" && q.model_name.toLowerCase().includes("gemini"));
  const geminiWeekly = quotaList.find(q => q?.model_name === "Gemini (Weekly)");

  const aiCredits = quotaList.find(q => q?.model_name === "AI Credits");

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await fetchQuota(account.id);
    } finally {
      setIsRefreshing(false);
    }
  };

  const email = account.email || "";
  const displayName = account.display_name || (email.includes("@") ? email.split("@")[0] : email) || "账号";
  const tier = (account.subscription_type || "FREE").toUpperCase();

  const getPercentNum = (val: number | string | null | undefined): number | null => {
    if (val === null || val === undefined) return null;
    const n = Number(val);
    return isNaN(n) ? null : Math.round(n);
  };

  const renderModalCountdown = (resetAt: number | string | null | undefined) => {
    const cd = formatResetCountdown(resetAt);
    if (cd.timeStr === "--" || cd.fullStr === "无限制") {
      return <div className="text-xs text-slate-400 font-mono">无限制</div>;
    }
    if (cd.timeStr === "已就绪") {
      return (
        <div className="text-xs font-semibold flex items-center gap-1.5 font-mono pt-0.5 text-emerald-600 dark:text-emerald-400">
          <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 shrink-0" />
          <span>已就绪</span>
          {cd.dateStr && <span className="text-slate-400 dark:text-slate-500 font-normal">({cd.dateStr})</span>}
        </div>
      );
    }
    return (
      <div className="flex flex-col text-xs font-mono pt-0.5">
        <span className="font-semibold text-slate-800 dark:text-slate-200">{cd.timeStr}</span>
        {cd.dateStr && (
          <span className="text-[11px] text-slate-500 dark:text-slate-400 font-medium">({cd.dateStr})</span>
        )}
      </div>
    );
  };

  const renderModalQuotaItem = (label: string, quota: any) => {
    const percent = quota?.remaining_percent;
    const pNum = getPercentNum(percent);

    if (quotaChartType === "bar") {
      return (
        <div className="p-3 rounded-xl bg-white dark:bg-[#20242b] border border-slate-200/80 dark:border-[#2c323f] space-y-1.5 shadow-2xs">
          <div className="flex items-center justify-between text-xs">
            <span className="font-bold text-slate-800 dark:text-slate-200">{label}</span>
            <span className={`font-mono font-bold ${getPercentTextColor(pNum)}`}>
              {pNum !== null ? `${pNum}%` : "100%"}
            </span>
          </div>
          <div className="w-full h-1.5 rounded-full bg-slate-100 dark:bg-[#282d38] overflow-hidden">
            <div
              className={`h-full rounded-full transition-all duration-300 ${getProgressColor(pNum ?? 100)}`}
              style={{ width: `${Math.min(100, Math.max(0, pNum ?? 100))}%` }}
            />
          </div>
          {renderModalCountdown(quota?.reset_at)}
        </div>
      );
    }

    return (
      <div className="p-3 rounded-xl bg-white dark:bg-[#20242b] border border-slate-200/80 dark:border-[#2c323f] flex items-center space-x-3 shadow-2xs">
        <QuotaRing
          percent={pNum}
          size={48}
          strokeWidth={3.8}
        />
        <div className="min-w-0 flex-1">
          <div className="text-xs font-bold text-slate-800 dark:text-slate-200">{label}</div>
          {renderModalCountdown(quota?.reset_at)}
        </div>
      </div>
    );
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/40 dark:bg-black/70 animate-in fade-in duration-150"
      onClick={() => setSelectedAccountForDetail(null)}
    >
      <div 
        className="w-full max-w-lg bg-white dark:bg-[#1e222a] border border-slate-200/90 dark:border-[#2c323f] rounded-3xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh] animate-in zoom-in-95 duration-150"
        onClick={(e) => e.stopPropagation()}
      >
        <ErrorBoundary 
          fallbackTitle="模型额度面板遇到异常" 
          onReset={() => setSelectedAccountForDetail(null)}
        >
          {/* 顶部标题栏 */}
          <div className="px-5 py-3.5 border-b border-slate-100 dark:border-[#282c37] flex items-center justify-between bg-slate-50/80 dark:bg-[#181b20]">
            <div className="flex items-center space-x-3 min-w-0">
              <div className="w-8 h-8 rounded-xl bg-emerald-100/80 dark:bg-emerald-950/60 border border-emerald-200/80 dark:border-emerald-800/60 flex items-center justify-center text-emerald-700 dark:text-emerald-400 font-bold shrink-0 shadow-2xs">
                <ShieldCheck className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
              </div>
              <div className="min-w-0">
                <div className="text-sm font-bold text-slate-900 dark:text-slate-100 flex items-center space-x-1.5 truncate">
                  <span className="truncate">{displayName}</span>
                  <span className="px-1.5 py-0.2 text-[10px] font-semibold rounded bg-emerald-50 dark:bg-emerald-950/50 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800/70 shrink-0">
                    {tier}
                  </span>
                </div>
                <div className="text-xs text-slate-500 dark:text-slate-400 font-mono truncate">{email}</div>
              </div>
            </div>

            <div className="flex items-center space-x-1 shrink-0">
              <button
                onClick={handleRefresh}
                disabled={isRefreshing}
                className="p-1.5 rounded-lg hover:bg-slate-200/70 dark:hover:bg-[#282d38] text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200 transition-colors disabled:opacity-50"
                title="刷新此账号配额"
              >
                <RotateCw className={`w-3.5 h-3.5 ${isRefreshing ? "animate-spin text-emerald-600" : ""}`} />
              </button>
              <button
                onClick={() => setSelectedAccountForDetail(null)}
                className="p-1.5 rounded-lg hover:bg-slate-200/70 dark:hover:bg-[#282d38] text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200 transition-colors"
                title="关闭"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
          </div>

          {/* 切换 Tab */}
          <div className="px-5 pt-2.5 border-b border-slate-100 dark:border-[#282c37] flex items-center space-x-4 bg-white dark:bg-[#181b20] text-xs font-medium">
            <button
              onClick={() => setDetailModalTab("all")}
              className={`pb-2 border-b-2 font-semibold transition-all flex items-center space-x-1.5 ${
                detailModalTab === "all"
                  ? "border-emerald-600 text-emerald-700 dark:text-emerald-400"
                  : "border-transparent text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200"
              }`}
            >
              <Layers className="w-3.5 h-3.5" />
              <span>全部模型</span>
              <span className="px-1.5 py-0.2 rounded-full bg-slate-100 dark:bg-[#282d38] text-slate-600 dark:text-slate-300 text-[10px] font-mono">
                {individualModels.length > 0 ? individualModels.length : quotaList.length}
              </span>
            </button>

            <button
              onClick={() => setDetailModalTab("summary")}
              className={`pb-2 border-b-2 font-semibold transition-all flex items-center space-x-1.5 ${
                detailModalTab === "summary"
                  ? "border-emerald-600 text-emerald-700 dark:text-emerald-400"
                  : "border-transparent text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200"
              }`}
            >
              <Sparkles className="w-3.5 h-3.5 text-amber-500" />
              <span>5h & 周限额</span>
            </button>
          </div>

          {/* 主内容区域 */}
          <div className="flex-1 overflow-y-auto p-5 space-y-3">
            {detailModalTab === "all" ? (
              /* 全部细分模型列表 */
              <div className="space-y-3">
                {/* 搜索过滤框 */}
                <div className="relative">
                  <Search className="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 dark:text-slate-500" />
                  <input
                    type="text"
                    placeholder="按模型名称搜索..."
                    value={modelSearch}
                    onChange={(e) => setModelSearch(e.target.value)}
                    className="w-full pl-8 pr-7 py-1.5 text-xs rounded-xl bg-slate-50 dark:bg-[#181b20] border border-slate-200 dark:border-[#2c323f] text-slate-800 dark:text-slate-100 placeholder-slate-400 dark:placeholder-slate-500 focus:outline-none focus:border-emerald-500 transition-colors"
                  />
                  {modelSearch && (
                    <button
                      onClick={() => setModelSearch("")}
                      className="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-300"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  )}
                </div>

                {individualModels.length === 0 ? (
                  <div className="text-center py-10 text-xs text-slate-400 dark:text-slate-500 bg-slate-50/50 dark:bg-[#181b20] rounded-2xl border border-dashed border-slate-200 dark:border-[#2c323f]">
                    {modelSearch ? "未搜索到匹配的模型" : "暂无细分模型额度数据"}
                  </div>
                ) : (
                  <div className="space-y-2">
                    {individualModels.map((m, idx) => {
                      const pNum = getPercentNum(m?.remaining_percent);
                      const countdown = formatResetCountdown(m?.reset_at);
                      const safeName = m?.model_name || "未知模型";

                      if (quotaChartType === "bar") {
                        return (
                          <div
                            key={`${m.id || safeName}-${idx}`}
                            className="p-3 rounded-2xl bg-white dark:bg-[#20242b] border border-slate-200/90 dark:border-[#2c323f] shadow-2xs hover:border-slate-300 dark:hover:border-[#3b4354] transition-colors"
                          >
                            <div className="flex items-center justify-between">
                              <span className="text-xs font-bold text-slate-800 dark:text-slate-200 truncate pr-2">
                                {safeName}
                              </span>
                              <span className={`font-mono text-xs font-bold shrink-0 ${getPercentTextColor(pNum)}`}>
                                {pNum !== null ? `${pNum}%` : "100%"}
                              </span>
                            </div>
                            <div className="w-full h-1.5 rounded-full bg-slate-100 dark:bg-[#282d38] mt-2 overflow-hidden">
                              <div
                                className={`h-full rounded-full transition-all duration-300 ${getProgressColor(pNum ?? 100)}`}
                                style={{ width: `${Math.min(100, Math.max(0, pNum ?? 100))}%` }}
                              />
                            </div>
                            <div className="flex items-center space-x-1.5 text-xs text-slate-500 dark:text-slate-400 mt-2 font-mono">
                              <Clock className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                              <span className="font-semibold text-slate-800 dark:text-slate-200">{countdown.timeStr}</span>
                              {countdown.dateStr && (
                                <span className="text-[11px] text-slate-500 dark:text-slate-400 font-medium">({countdown.dateStr})</span>
                              )}
                            </div>
                          </div>
                        );
                      }

                      return (
                        <div
                          key={`${m.id || safeName}-${idx}`}
                          className="p-3 rounded-2xl bg-white dark:bg-[#20242b] border border-slate-200/90 dark:border-[#2c323f] shadow-2xs hover:border-slate-300 dark:hover:border-[#3b4354] transition-colors flex items-center space-x-3.5"
                        >
                          <QuotaRing
                            percent={pNum}
                            size={44}
                            strokeWidth={3.6}
                          />
                          <div className="min-w-0 flex-1">
                            <div className="text-xs font-bold text-slate-800 dark:text-slate-200 truncate">
                              {safeName}
                            </div>
                            <div className="flex items-center space-x-1.5 text-xs text-slate-500 dark:text-slate-400 mt-1 font-mono">
                              <Clock className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                              <span className="font-semibold text-slate-700 dark:text-slate-300">{countdown.timeStr}</span>
                              {countdown.dateStr && (
                                <span className="text-[11px] text-slate-400 dark:text-slate-500 font-medium">({countdown.dateStr})</span>
                              )}
                            </div>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            ) : (
              /* 核心额度看板 (Gemini & Claude 对称和谐) */
              <div className="space-y-3">
                {/* Gemini 卡片 */}
                <div className="p-3.5 rounded-2xl bg-slate-50/70 dark:bg-[#181b20] border border-slate-200/80 dark:border-[#282c37] space-y-2.5">
                  <div className="flex items-center space-x-1.5">
                    <span className="w-2 h-2 rounded-full bg-blue-500 shadow-2xs" />
                    <span className="text-xs font-bold text-slate-900 dark:text-slate-100">Gemini 模型组</span>
                  </div>

                  <div className="grid grid-cols-2 gap-2.5 pt-0.5">
                    {renderModalQuotaItem("5h 限额", gemini5h)}
                    {renderModalQuotaItem("周限额", geminiWeekly)}
                  </div>
                </div>

                {/* Claude 卡片 */}
                <div className="p-3.5 rounded-2xl bg-slate-50/70 dark:bg-[#181b20] border border-slate-200/80 dark:border-[#282c37] space-y-2.5">
                  <div className="flex items-center space-x-1.5">
                    <span className="w-2 h-2 rounded-full bg-amber-500 shadow-2xs" />
                    <span className="text-xs font-bold text-slate-900 dark:text-slate-100">Claude 模型组</span>
                  </div>

                  <div className="grid grid-cols-2 gap-2.5 pt-0.5">
                    {renderModalQuotaItem("5h 限额", claude5h)}
                    {renderModalQuotaItem("周限额", claudeWeekly)}
                  </div>
                </div>



                {/* 可用 AI 积分说明卡 */}
                <div className="p-3 rounded-xl bg-slate-50 dark:bg-[#20242b] border border-slate-200 dark:border-[#2c323f] flex items-center justify-between text-xs">
                  <div className="flex items-center space-x-1.5">
                    <Sparkles className="w-3.5 h-3.5 text-amber-500" />
                    <span className="text-slate-600 dark:text-slate-400 font-medium">可用 AI 积分</span>
                  </div>
                  <span className="font-mono font-bold text-slate-900 dark:text-slate-100">
                    {typeof aiCredits?.remaining_value === "number" ? aiCredits.remaining_value : 0}
                  </span>
                </div>
              </div>
            )}
          </div>

          {/* 底栏 */}
          <div className="px-5 py-3 border-t border-slate-100 dark:border-[#282c37] bg-slate-50/60 dark:bg-[#181b20] flex justify-between items-center text-[11px] text-slate-400 dark:text-slate-500">
            <span className="flex items-center space-x-1">
              <CheckCircle2 className="w-3 h-3 text-emerald-500" />
              <span>Google Cloud Code</span>
            </span>
            <button
              onClick={() => setSelectedAccountForDetail(null)}
              className="px-3.5 py-1 rounded-lg bg-slate-900 dark:bg-emerald-600 hover:bg-slate-800 dark:hover:bg-emerald-500 text-white text-xs font-medium transition-colors"
            >
              完成
            </button>
          </div>
        </ErrorBoundary>
      </div>
    </div>
  );
};
