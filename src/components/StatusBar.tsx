import React from "react";
import { Play, Moon, Sun, Settings } from "lucide-react";
import { useVaultStore } from "../stores/useVaultStore";

export const StatusBar: React.FC = () => {
  const { 
    accounts, 
    antigravityStatus, 
    launchAntigravity,
    closeToTray,
    setIsSettingsModalOpen,
    theme,
    toggleTheme,
  } = useVaultStore();

  const isRunning = antigravityStatus?.is_running ?? false;

  return (
    <footer className="h-9 px-4 bg-white dark:bg-[#1c1f26] border-t border-slate-200 dark:border-[#282c37] flex items-center justify-between text-xs text-slate-500 dark:text-slate-400 select-none sticky bottom-0 z-40 transition-colors duration-200">
      {/* Left: Accounts Count & Status */}
      <div className="flex items-center space-x-2 text-[11px] text-slate-500 dark:text-slate-400">
        <span>共 <strong className="text-slate-800 dark:text-slate-200 font-semibold">{accounts.length}</strong> 个账号</span>
        <span className="text-slate-300 dark:text-slate-600">·</span>
        <button
          type="button"
          onClick={() => setIsSettingsModalOpen(true)}
          className="flex items-center space-x-1 hover:text-slate-700 dark:hover:text-slate-200 transition-colors cursor-pointer"
          title="点击调整后台常驻与更新设置"
        >
          <span className={`w-1.5 h-1.5 rounded-full inline-block ${closeToTray ? "bg-emerald-500" : "bg-slate-400"}`}></span>
          <span>{closeToTray ? "后台常驻开启" : "后台常驻关闭"}</span>
        </button>
      </div>

      {/* Right: Actions */}
      <div className="flex items-center space-x-1">
        {!isRunning && (
          <button
            onClick={() => launchAntigravity()}
            className="flex items-center space-x-1 px-2.5 py-0.5 rounded-md text-[11px] font-medium bg-emerald-50 dark:bg-emerald-950/40 hover:bg-emerald-100 dark:hover:bg-emerald-900/60 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800/60 transition-colors mr-1"
            title="启动 Antigravity 客户端"
          >
            <Play className="w-2.5 h-2.5 fill-current" />
            <span>启动客户端</span>
          </button>
        )}

        <button
          type="button"
          onClick={toggleTheme}
          className="w-7 h-7 flex items-center justify-center rounded-md text-slate-400 hover:text-amber-500 dark:hover:text-amber-400 hover:bg-slate-100 dark:hover:bg-[#282d38] transition-colors"
          title={theme === "dark" ? "切换为浅色模式" : "切换为深色模式"}
          aria-label="切换界面主题"
        >
          {theme === "dark" ? <Sun className="w-3.5 h-3.5" /> : <Moon className="w-3.5 h-3.5" />}
        </button>

        <button
          type="button"
          onClick={() => setIsSettingsModalOpen(true)}
          className="w-7 h-7 flex items-center justify-center rounded-md text-slate-400 hover:text-emerald-600 dark:hover:text-emerald-400 hover:bg-slate-100 dark:hover:bg-[#282d38] transition-colors"
          title="系统设置与项目信息"
          aria-label="打开设置"
        >
          <Settings className="w-3.5 h-3.5" />
        </button>
      </div>
    </footer>
  );
};
