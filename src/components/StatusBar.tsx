import React from "react";
import { Play } from "lucide-react";
import { useVaultStore } from "../stores/useVaultStore";

export const StatusBar: React.FC = () => {
  const { 
    accounts, 
    antigravityStatus, 
    launchAntigravity 
  } = useVaultStore();

  const isRunning = antigravityStatus?.is_running ?? false;

  return (
    <footer className="h-9 px-4 bg-white dark:bg-[#1c1f26] border-t border-slate-200 dark:border-[#282c37] flex items-center justify-between text-xs text-slate-500 dark:text-slate-400 select-none sticky bottom-0 z-40 transition-colors duration-200">
      {/* Left: Accounts Count & Status */}
      <div className="flex items-center space-x-2 text-[11px] text-slate-500 dark:text-slate-400">
        <span>共 <strong className="text-slate-800 dark:text-slate-200 font-semibold">{accounts.length}</strong> 个账号</span>
        <span className="text-slate-300 dark:text-slate-600">·</span>
        <span>凭证库已连接</span>
      </div>

      {/* Right: Actions */}
      <div className="flex items-center space-x-2">
        {!isRunning && (
          <button
            onClick={() => launchAntigravity()}
            className="flex items-center space-x-1 px-2.5 py-0.5 rounded-md text-[11px] font-medium bg-emerald-50 dark:bg-emerald-950/40 hover:bg-emerald-100 dark:hover:bg-emerald-900/60 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800/60 transition-colors"
            title="启动 Antigravity 客户端"
          >
            <Play className="w-2.5 h-2.5 fill-current" />
            <span>启动客户端</span>
          </button>
        )}
      </div>
    </footer>
  );
};
