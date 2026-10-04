import React, { useEffect, useMemo, useState } from "react";
import { 
  Plus, 
  Search, 
  RotateCw,
  X
} from "lucide-react";
import { useVaultStore } from "./stores/useVaultStore";
import { TitleBar } from "./components/TitleBar";
import { AccountCard } from "./components/AccountCard";
import { AddAccountModal } from "./components/AddAccountModal";
import { QuotaDetailModal } from "./components/QuotaDetailModal";
import { DeleteConfirmModal } from "./components/DeleteConfirmModal";
import { ToastContainer } from "./components/Toast";
import { StatusBar } from "./components/StatusBar";
import { AntiVaultLogo } from "./components/AntiVaultLogo";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export const App: React.FC = () => {
  const { 
    accounts, 
    loadInitialData, 
    fetchStatus, 
    setIsAddModalOpen, 
    searchQuery, 
    setSearchQuery,
    isLoading,
    isRefreshingAll,
    refreshAllQuotas,
    theme
  } = useVaultStore();

  const [tierFilter, setTierFilter] = useState<string>("ALL");

  useEffect(() => {
    if (theme === "dark") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }, [theme]);

  useEffect(() => {
    loadInitialData();

    // Poll IDE & Antigravity 2.0 running status every 4 seconds
    const interval = setInterval(() => {
      fetchStatus();
    }, 4000);

    // Auto silent refresh all quotas every 1 minute
    const silentRefreshInterval = setInterval(() => {
      refreshAllQuotas(true);
    }, 60000);

    // Smoothly reveal window once React has mounted and painted
    if (isTauri()) {
      (async () => {
        try {
          const win = getCurrentWindow();
          await win.show();
          await win.setFocus();
        } catch {
          // ignore
        }
      })();
    }

    return () => {
      clearInterval(interval);
      clearInterval(silentRefreshInterval);
    };
  }, []);

  const counts = useMemo(() => {
    let pro = 0;
    let ultra = 0;
    let free = 0;
    accounts.forEach(a => {
      const t = (a.subscription_type || "FREE").toUpperCase();
      if (t.includes("ULTRA")) ultra++;
      else if (t.includes("PRO")) pro++;
      else free++;
    });
    return { all: accounts.length, pro, ultra, free };
  }, [accounts]);

  const filteredAccounts = useMemo(() => {
    return accounts.filter((a) => {
      // Tier filter
      if (tierFilter !== "ALL") {
        const t = (a.subscription_type || "FREE").toUpperCase();
        if (tierFilter === "PRO" && !t.includes("PRO")) return false;
        if (tierFilter === "ULTRA" && !t.includes("ULTRA")) return false;
        if (tierFilter === "FREE" && (t.includes("PRO") || t.includes("ULTRA"))) return false;
      }

      // Search query
      if (!searchQuery.trim()) return true;
      const q = searchQuery.toLowerCase();
      return (
        a.email.toLowerCase().includes(q) ||
        (a.display_name && a.display_name.toLowerCase().includes(q)) ||
        (a.subscription_type && a.subscription_type.toLowerCase().includes(q))
      );
    });
  }, [accounts, searchQuery, tierFilter]);

  return (
    <div className="flex flex-col h-screen w-screen bg-slate-50 dark:bg-[#181b20] text-slate-800 dark:text-slate-100 select-none overflow-hidden font-sans transition-colors duration-200">
      {/* Title Bar */}
      <TitleBar />

      {/* Main Content Area */}
      <main className="flex-1 overflow-y-auto px-4 py-3 sm:px-5 sm:py-3.5 flex flex-col space-y-3">
        
        {/* Top Control Bar: Single-row Compact Toolbar */}
        <div className="flex items-center gap-2">
          {/* Search Box */}
          <div className="relative flex-1 min-w-0">
            <Search className="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 dark:text-slate-500" />
            <input
              type="text"
              placeholder="搜索账号..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-8 pr-7 py-1.5 rounded-xl bg-white dark:bg-[#20242b] border border-slate-200 dark:border-[#2c323f] text-xs text-slate-800 dark:text-slate-100 placeholder-slate-400 dark:placeholder-slate-500 focus:outline-none focus:border-emerald-500 shadow-2xs transition-all"
            />
            {searchQuery && (
              <button
                onClick={() => setSearchQuery("")}
                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-300"
              >
                <X className="w-3 h-3" />
              </button>
            )}
          </div>

          {/* Action Buttons: Unified in single row */}
          <button
            onClick={() => refreshAllQuotas()}
            disabled={isRefreshingAll}
            title="刷新全部账号最新配额"
            className="flex items-center space-x-1 px-2.5 py-1.5 rounded-xl bg-white dark:bg-[#20242b] hover:bg-slate-50 dark:hover:bg-[#282d38] border border-slate-200 dark:border-[#2c323f] text-slate-700 dark:text-slate-200 text-xs font-medium transition-all shadow-2xs active:scale-95 disabled:opacity-50 shrink-0"
          >
            <RotateCw className={`w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400 ${isRefreshingAll ? "animate-spin" : ""}`} />
            <span>刷新</span>
          </button>

          <button
            onClick={() => setIsAddModalOpen(true)}
            className="flex items-center space-x-1 px-3 py-1.5 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white font-medium text-xs transition-all shadow-xs shadow-emerald-700/20 active:scale-95 shrink-0"
          >
            <Plus className="w-3.5 h-3.5 stroke-[2.5]" />
            <span>添加</span>
          </button>
        </div>

        {/* Compact Filter Pills */}
        {accounts.length > 0 && (
          <div className="flex items-center justify-between text-xs pt-0.5">
            <div className="flex items-center space-x-1">
              <button
                onClick={() => setTierFilter("ALL")}
                className={`px-2.5 py-0.5 rounded-lg text-xs font-medium transition-all ${
                  tierFilter === "ALL"
                    ? "bg-slate-200 text-slate-800 dark:bg-slate-700 dark:text-slate-100 font-semibold shadow-2xs border border-slate-300/70 dark:border-slate-600/70"
                    : "text-slate-500 dark:text-slate-400 hover:bg-slate-200/50 dark:hover:bg-[#252a34] border border-transparent"
                }`}
              >
                全部 {counts.all}
              </button>
              {counts.pro > 0 && (
                <button
                  onClick={() => setTierFilter("PRO")}
                  className={`px-2.5 py-0.5 rounded-lg text-xs font-medium transition-all ${
                    tierFilter === "PRO"
                      ? "bg-emerald-600 text-white font-semibold shadow-2xs border border-emerald-600"
                      : "text-slate-500 dark:text-slate-400 hover:bg-slate-200/50 dark:hover:bg-[#252a34] border border-transparent"
                  }`}
                >
                  PRO {counts.pro}
                </button>
              )}
              {counts.ultra > 0 && (
                <button
                  onClick={() => setTierFilter("ULTRA")}
                  className={`px-2.5 py-0.5 rounded-lg text-xs font-medium transition-all ${
                    tierFilter === "ULTRA"
                      ? "bg-purple-600 text-white font-semibold shadow-2xs border border-purple-600"
                      : "text-slate-500 dark:text-slate-400 hover:bg-slate-200/50 dark:hover:bg-[#252a34] border border-transparent"
                  }`}
                >
                  ULTRA {counts.ultra}
                </button>
              )}
            </div>

            <span className="text-[11px] text-slate-400 dark:text-slate-500 font-mono">
              {filteredAccounts.length} / {accounts.length}
            </span>
          </div>
        )}

        {/* Loading Spinner */}
        {isLoading && accounts.length === 0 ? (
          <div className="flex-1 flex flex-col items-center justify-center space-y-2 py-12 text-slate-400">
            <RotateCw className="w-6 h-6 animate-spin text-emerald-600" />
            <span className="text-xs">加载账号与配额数据...</span>
          </div>
        ) : filteredAccounts.length === 0 ? (
          /* Empty State */
          <div className="flex-1 flex flex-col items-center justify-center py-12 px-6 text-center space-y-4 border border-dashed border-slate-200 dark:border-[#2c323f] rounded-2xl bg-white dark:bg-[#20242b]">
            <AntiVaultLogo size={32} />
            <div className="space-y-1">
              <h3 className="text-sm font-bold text-slate-800 dark:text-slate-200">
                {searchQuery || tierFilter !== "ALL" ? "未找到账号" : "暂无账号"}
              </h3>
              <p className="text-xs text-slate-500 dark:text-slate-400 max-w-xs">
                {searchQuery || tierFilter !== "ALL"
                  ? "请尝试清除搜索关键词"
                  : "点击下方按钮授权添加账号，快捷管理多账号配额。"}
              </p>
            </div>
            {!searchQuery && tierFilter === "ALL" && (
              <div className="flex items-center gap-2 pt-1">
                <button
                  onClick={() => setIsAddModalOpen(true)}
                  className="px-3.5 py-1.5 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white text-xs font-medium transition-colors shadow-xs flex items-center space-x-1"
                >
                  <Plus className="w-3.5 h-3.5" />
                  <span>添加账号</span>
                </button>
              </div>
            )}
          </div>
        ) : (
          <div className="account-cards-grid">
            {filteredAccounts.map((account) => (
              <AccountCard key={account.id} account={account} />
            ))}
          </div>
        )}
      </main>

      {/* Modals & Overlays */}
      <AddAccountModal />
      <ErrorBoundary fallbackTitle="配额详情加载错误">
        <QuotaDetailModal />
      </ErrorBoundary>
      <DeleteConfirmModal />
      <ToastContainer />

      {/* Status Bar */}
      <StatusBar />
    </div>
  );
};

export default App;
