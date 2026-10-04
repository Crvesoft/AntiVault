import React, { useState } from "react";
import { AlertTriangle, ArrowRightLeft, X, Loader2 } from "lucide-react";
import { useVaultStore } from "../stores/useVaultStore";

export const SwitchConfirmModal: React.FC = () => {
  const { 
    switchTargetAccount, 
    setSwitchTargetAccount, 
    switchAccount, 
    switchingAccountId,
    antigravityStatus
  } = useVaultStore();

  const isRunning = antigravityStatus?.is_running ?? false;
  const [autoRestart, setAutoRestart] = useState(true);

  if (!switchTargetAccount || !isRunning) return null;

  const handleConfirm = async () => {
    await switchAccount(switchTargetAccount.id, autoRestart);
  };

  const isSwitching = switchingAccountId === switchTargetAccount.id;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/40 dark:bg-black/70 animate-in fade-in duration-200"
      onClick={() => {
        if (!isSwitching) setSwitchTargetAccount(null);
      }}
    >
      <div 
        className="w-full max-w-md bg-white dark:bg-[#1e222a] border border-slate-200 dark:border-[#2c323f] rounded-2xl shadow-2xl overflow-hidden flex flex-col animate-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className={`px-6 py-4 border-b flex items-center justify-between ${
          isRunning 
            ? "border-amber-100 dark:border-amber-950/40 bg-amber-50/70 dark:bg-amber-950/20 text-amber-700 dark:text-amber-300" 
            : "border-emerald-100 dark:border-emerald-950/40 bg-emerald-50/70 dark:bg-emerald-950/20 text-emerald-800 dark:text-emerald-300"
        }`}>
          <div className="flex items-center space-x-2.5">
            {isRunning ? (
              <AlertTriangle className="w-5 h-5 shrink-0 text-amber-600 dark:text-amber-400" />
            ) : (
              <ArrowRightLeft className="w-5 h-5 shrink-0 text-emerald-600 dark:text-emerald-400" />
            )}
            <span className="text-base font-semibold">
              {isRunning ? "Antigravity 正在运行" : "确认切换账号"}
            </span>
          </div>
          <button
            onClick={() => setSwitchTargetAccount(null)}
            className="p-1.5 rounded-lg hover:bg-slate-200/50 dark:hover:bg-[#282d38] text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-4">
          <p className="text-sm text-slate-600 dark:text-slate-300 leading-relaxed">
            {isRunning
              ? "检测到 Antigravity 客户端正在运行。AntiVault 将同步更新 Windows 凭据管理器（Antigravity 2.0 / CLI）及本地存储配置。"
              : "即将为您切换生效目标账号凭证，并同步更新 Windows 凭据管理器及本地 IDE 配置。"}
          </p>

          <div className="p-3.5 rounded-xl bg-slate-50 dark:bg-[#181b20] border border-slate-200/80 dark:border-[#282c37] space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400 font-medium">即将切换的目标账号：</div>
            <div className="text-sm font-semibold text-slate-800 dark:text-slate-100 break-all">{switchTargetAccount.email}</div>
          </div>

          <label className="flex items-start space-x-3 cursor-pointer text-sm text-slate-700 dark:text-slate-300 select-none pt-1">
            <input
              type="checkbox"
              checked={autoRestart}
              onChange={(e) => setAutoRestart(e.target.checked)}
              className="w-4 h-4 mt-0.5 rounded border-slate-300 dark:border-slate-600 text-emerald-600 focus:ring-emerald-500/30 accent-emerald-600"
            />
            <div className="space-y-0.5">
              <span className="font-medium text-slate-800 dark:text-slate-200">
                {isRunning ? "切换后自动重启客户端以立即生效" : "切换完成后自动唤起启动 Antigravity 客户端"}
              </span>
              <p className="text-xs text-slate-500 dark:text-slate-400">
                {isRunning
                  ? autoRestart
                    ? "AntiVault 将平稳重启客户端以立即载入新账号。"
                    : "取消勾选将静默更新凭据，不打扰当前工作，您可稍后手动重载窗口生效。"
                  : "便捷启动客户端并直接连接至当前选定账号。"}
              </p>
            </div>
          </label>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-slate-100 dark:border-[#282c37] bg-slate-50/60 dark:bg-[#181b20] flex justify-end space-x-3">
          <button
            onClick={() => setSwitchTargetAccount(null)}
            disabled={isSwitching}
            className="px-4 py-2 rounded-xl bg-slate-200/80 dark:bg-[#282d38] hover:bg-slate-200 dark:hover:bg-[#323947] text-slate-700 dark:text-slate-300 text-sm font-medium transition-colors"
          >
            取消
          </button>
          <button
            onClick={handleConfirm}
            disabled={isSwitching}
            className="px-5 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white text-sm font-medium flex items-center space-x-2 shadow-sm shadow-emerald-700/20 transition-all active:scale-95 disabled:opacity-50"
          >
            {isSwitching ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                <span>{isRunning && autoRestart ? "正在重启并切换..." : "正在写入凭据..."}</span>
              </>
            ) : (
              <>
                <ArrowRightLeft className="w-4 h-4" />
                <span>确认切换账号</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};

