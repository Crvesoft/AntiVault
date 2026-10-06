import React from "react";
import {
  X,
  Settings as SettingsIcon,
  RotateCw,
  ExternalLink,
  ShieldCheck,
  CheckCircle2,
  AlertCircle,
  Sparkles,
  Layers,
  CircleDot,
  Sliders,
  PieChart,
} from "lucide-react";
import { isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useVaultStore } from "../stores/useVaultStore";
import { AntiVaultLogo } from "./AntiVaultLogo";

const GithubIcon: React.FC<{ className?: string }> = ({ className = "w-4 h-4" }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor">
    <path
      fillRule="evenodd"
      clipRule="evenodd"
      d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
    />
  </svg>
);

export const SettingsModal: React.FC = () => {
  const {
    isSettingsModalOpen,
    setIsSettingsModalOpen,
    closeToTray,
    setCloseToTray,
    autoCheckUpdate,
    setAutoCheckUpdate,
    quotaChartType,
    setQuotaChartType,
    checkForUpdates,
    isCheckingUpdate,
    updateResult,
  } = useVaultStore();

  if (!isSettingsModalOpen) return null;

  const handleOpenUrl = async (url: string) => {
    if (isTauri()) {
      try {
        await openUrl(url);
        return;
      } catch (err) {
        console.error("Failed to open URL via plugin-opener:", err);
      }
    }
    window.open(url, "_blank", "noopener,noreferrer");
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-modal-title"
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/60 dark:bg-black/70 backdrop-blur-xs animate-in fade-in duration-150"
    >
      <div
        className="w-full max-w-md bg-white dark:bg-[#20242b] border border-slate-200 dark:border-[#2c323f] rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh] animate-in zoom-in-95 duration-150"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-slate-200 dark:border-[#2c323f] bg-slate-50/70 dark:bg-[#1a1d24]/60">
          <div className="flex items-center space-x-2.5">
            <div className="w-8 h-8 rounded-xl bg-emerald-50 dark:bg-emerald-950/50 flex items-center justify-center text-emerald-600 dark:text-emerald-400 border border-emerald-200/60 dark:border-emerald-800/40">
              <SettingsIcon className="w-4 h-4" />
            </div>
            <div>
              <h2 id="settings-modal-title" className="text-sm font-semibold text-slate-800 dark:text-slate-100">
                应用设置
              </h2>
              <p className="text-[11px] text-slate-500 dark:text-slate-400">
                运行模式、自动更新与项目信息
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={() => setIsSettingsModalOpen(false)}
            className="w-7 h-7 rounded-lg text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-[#2a303c] transition-colors flex items-center justify-center"
            title="关闭设置"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Content Body */}
        <div className="flex-1 overflow-y-auto px-5 py-4 space-y-5 text-xs text-slate-700 dark:text-slate-200">
          
          {/* Section 1: Background & Residency */}
          <section className="space-y-3">
            <div className="flex items-center space-x-1.5 text-slate-400 dark:text-slate-500 font-semibold tracking-wider uppercase text-[10px]">
              <Layers className="w-3 h-3" />
              <span>运行模式</span>
            </div>

            <div className="p-3.5 rounded-xl bg-slate-50 dark:bg-[#191c22] border border-slate-200/70 dark:border-[#2a303c] space-y-3">
              <div className="flex items-start justify-between gap-3">
                <div className="space-y-0.5">
                  <span className="font-medium text-slate-800 dark:text-slate-200 block">
                    后台常驻运行（关闭到托盘）
                  </span>
                  <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed">
                    点击窗口关闭按钮时最小化到系统托盘，右键任务栏托盘图标可秒级快速切换账号。
                  </p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer shrink-0 mt-0.5">
                  <input
                    type="checkbox"
                    checked={closeToTray}
                    onChange={(e) => setCloseToTray(e.target.checked)}
                    className="sr-only peer"
                  />
                  <div className="w-9 h-5 bg-slate-300 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-emerald-600"></div>
                </label>
              </div>

              {!closeToTray && (
                <div className="flex items-center space-x-1.5 p-2 rounded-lg bg-amber-50 dark:bg-amber-950/30 text-amber-700 dark:text-amber-400 border border-amber-200 dark:border-amber-800/40 text-[11px]">
                  <AlertCircle className="w-3.5 h-3.5 shrink-0" />
                  <span>已停用后台常驻，关闭窗口后将直接完全退出 AntiVault。</span>
                </div>
              )}
            </div>
          </section>

          {/* Section: Quota Chart Type */}
          <section className="space-y-3">
            <div className="flex items-center space-x-1.5 text-slate-400 dark:text-slate-500 font-semibold tracking-wider uppercase text-[10px]">
              <PieChart className="w-3 h-3" />
              <span>配额图表展现</span>
            </div>

            <div className="p-3.5 rounded-xl bg-slate-50 dark:bg-[#191c22] border border-slate-200/70 dark:border-[#2a303c] space-y-3">
              <div className="space-y-0.5">
                <span className="font-medium text-slate-800 dark:text-slate-200 block">
                  图表展现样式
                </span>
                <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed">
                  选择账号卡片及详情弹窗中模型配额的展示方式。
                </p>
              </div>

              <div className="grid grid-cols-2 gap-2.5 pt-1">
                <button
                  type="button"
                  onClick={() => setQuotaChartType("ring")}
                  className={`p-3 rounded-xl border text-left transition-all cursor-pointer flex flex-col justify-between ${
                    quotaChartType === "ring"
                      ? "border-emerald-500 bg-white dark:bg-[#20252e] ring-1 ring-emerald-500/40 text-emerald-950 dark:text-emerald-100 shadow-2xs"
                      : "border-slate-200/80 dark:border-[#2a303c] bg-white/70 dark:bg-[#1a1d24] text-slate-700 dark:text-slate-300 hover:border-slate-300 dark:hover:border-slate-600"
                  }`}
                >
                  <div className="flex items-center justify-between mb-1.5">
                    <span className="font-semibold text-xs flex items-center gap-1.5">
                      <CircleDot className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" />
                      <span>环形进度图</span>
                    </span>
                    {quotaChartType === "ring" && (
                      <span className="w-2 h-2 rounded-full bg-emerald-500 shadow-2xs" />
                    )}
                  </div>
                  <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed">
                    圆环进度与居中百分比数字，现代大方。
                  </p>
                </button>

                <button
                  type="button"
                  onClick={() => setQuotaChartType("bar")}
                  className={`p-3 rounded-xl border text-left transition-all cursor-pointer flex flex-col justify-between ${
                    quotaChartType === "bar"
                      ? "border-emerald-500 bg-white dark:bg-[#20252e] ring-1 ring-emerald-500/40 text-emerald-950 dark:text-emerald-100 shadow-2xs"
                      : "border-slate-200/80 dark:border-[#2a303c] bg-white/70 dark:bg-[#1a1d24] text-slate-700 dark:text-slate-300 hover:border-slate-300 dark:hover:border-slate-600"
                  }`}
                >
                  <div className="flex items-center justify-between mb-1.5">
                    <span className="font-semibold text-xs flex items-center gap-1.5">
                      <Sliders className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" />
                      <span>条形进度条</span>
                    </span>
                    {quotaChartType === "bar" && (
                      <span className="w-2 h-2 rounded-full bg-emerald-500 shadow-2xs" />
                    )}
                  </div>
                  <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed">
                    经典横向长条，直观线性百分比展示。
                  </p>
                </button>
              </div>
            </div>
          </section>

          {/* Section 2: Updates */}
          <section className="space-y-3">
            <div className="flex items-center space-x-1.5 text-slate-400 dark:text-slate-500 font-semibold tracking-wider uppercase text-[10px]">
              <Sparkles className="w-3 h-3" />
              <span>更新检测</span>
            </div>

            <div className="p-3.5 rounded-xl bg-slate-50 dark:bg-[#191c22] border border-slate-200/70 dark:border-[#2a303c] space-y-3.5">
              <div className="flex items-start justify-between gap-3">
                <div className="space-y-0.5">
                  <span className="font-medium text-slate-800 dark:text-slate-200 block">
                    启动时自动检测新版本
                  </span>
                  <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed">
                    每次启动 AntiVault 时，自动静默查询 GitHub Releases 是否有更新。
                  </p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer shrink-0 mt-0.5">
                  <input
                    type="checkbox"
                    checked={autoCheckUpdate}
                    onChange={(e) => setAutoCheckUpdate(e.target.checked)}
                    className="sr-only peer"
                  />
                  <div className="w-9 h-5 bg-slate-300 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-emerald-600"></div>
                </label>
              </div>

              {/* Version & Manual Check Action */}
              <div className="pt-2 border-t border-slate-200 dark:border-[#2a303c] flex items-center justify-between gap-2">
                <div className="flex items-center space-x-2">
                  <span className="text-[11px] text-slate-500 dark:text-slate-400">当前版本</span>
                  <span className="px-2 py-0.5 rounded-md font-mono text-[11px] font-semibold bg-emerald-100 dark:bg-emerald-950/60 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800/50">
                    {updateResult?.current_version || "v0.1.0"}
                  </span>
                </div>

                <button
                  type="button"
                  onClick={() => checkForUpdates(false)}
                  disabled={isCheckingUpdate}
                  className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-white dark:bg-[#20242b] hover:bg-slate-100 dark:hover:bg-[#282d38] border border-slate-200 dark:border-[#2c323f] text-slate-700 dark:text-slate-200 text-xs font-medium transition-all shadow-2xs active:scale-95 disabled:opacity-50"
                >
                  <RotateCw className={`w-3 h-3 text-emerald-600 dark:text-emerald-400 ${isCheckingUpdate ? "animate-spin" : ""}`} />
                  <span>{isCheckingUpdate ? "正在检测..." : "检查更新"}</span>
                </button>
              </div>

              {/* Update Result Feedback */}
              {updateResult && (
                <div className="pt-1 animate-in fade-in duration-200">
                  {updateResult.has_update ? (
                    <div className="p-3 rounded-lg bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800/50 space-y-2">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center space-x-1.5 text-emerald-700 dark:text-emerald-300 font-semibold text-xs">
                          <CheckCircle2 className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" />
                          <span>发现新版本 {updateResult.latest_version}</span>
                        </div>
                        <button
                          type="button"
                          onClick={() => handleOpenUrl(updateResult.release_url || "https://github.com/Crvesoft/AntiVault/releases")}
                          className="px-2.5 py-1 rounded-md bg-emerald-600 hover:bg-emerald-700 text-white font-medium text-[11px] flex items-center space-x-1 transition-colors shadow-2xs"
                        >
                          <span>前往下载</span>
                          <ExternalLink className="w-3 h-3" />
                        </button>
                      </div>
                      {updateResult.release_notes && (
                        <p className="text-[11px] text-slate-600 dark:text-slate-300 whitespace-pre-wrap max-h-24 overflow-y-auto bg-white/70 dark:bg-black/30 p-2 rounded border border-slate-200/50 dark:border-white/5 font-mono">
                          {updateResult.release_notes}
                        </p>
                      )}
                    </div>
                  ) : (
                    <div className="flex items-center space-x-1.5 text-slate-500 dark:text-slate-400 text-[11px]">
                      <CheckCircle2 className="w-3.5 h-3.5 text-emerald-500" />
                      <span>{updateResult.message || "当前已是最新版本，无需更新。"}</span>
                    </div>
                  )}
                </div>
              )}
            </div>
          </section>

          {/* Section 3: GitHub & Project Info */}
          <section className="space-y-3">
            <div className="flex items-center space-x-1.5 text-slate-400 dark:text-slate-500 font-semibold tracking-wider uppercase text-[10px]">
              <GithubIcon className="w-3 h-3" />
              <span>关于项目</span>
            </div>

            <div className="p-3.5 rounded-xl bg-slate-50 dark:bg-[#191c22] border border-slate-200/70 dark:border-[#2a303c] space-y-3">
              <div className="flex items-center space-x-3">
                <AntiVaultLogo size={36} showText={false} />
                <div className="space-y-0.5">
                  <div className="flex items-center space-x-2">
                    <h3 className="text-sm font-bold text-slate-800 dark:text-slate-100">
                      AntiVault
                    </h3>
                    <span className="px-1.5 py-0.2 rounded text-[10px] font-mono bg-slate-200 dark:bg-[#2c323f] text-slate-600 dark:text-slate-300 font-medium">
                      MIT License
                    </span>
                  </div>
                  <p className="text-[11px] text-slate-500 dark:text-slate-400">
                    Antigravity 多账号凭据管理与配额监控工具
                  </p>
                </div>
              </div>

              {/* GitHub Link Cards */}
              <div className="grid grid-cols-2 gap-2 pt-1">
                <button
                  type="button"
                  onClick={() => handleOpenUrl("https://github.com/Crvesoft/AntiVault")}
                  className="flex items-center justify-between p-2.5 rounded-lg bg-white dark:bg-[#20242b] hover:bg-slate-100 dark:hover:bg-[#282d38] border border-slate-200 dark:border-[#2c323f] text-slate-700 dark:text-slate-200 transition-colors text-left group"
                >
                  <div className="flex items-center space-x-2">
                    <GithubIcon className="w-4 h-4 text-slate-700 dark:text-slate-300 group-hover:text-emerald-600 transition-colors" />
                    <div>
                      <span className="font-semibold text-[11px] block">GitHub 仓库</span>
                      <span className="text-[10px] text-slate-400">Crvesoft/AntiVault</span>
                    </div>
                  </div>
                  <ExternalLink className="w-3 h-3 text-slate-400 group-hover:text-emerald-600 transition-colors" />
                </button>

                <button
                  type="button"
                  onClick={() => handleOpenUrl("https://github.com/Crvesoft/AntiVault/issues")}
                  className="flex items-center justify-between p-2.5 rounded-lg bg-white dark:bg-[#20242b] hover:bg-slate-100 dark:hover:bg-[#282d38] border border-slate-200 dark:border-[#2c323f] text-slate-700 dark:text-slate-200 transition-colors text-left group"
                >
                  <div className="flex items-center space-x-2">
                    <ShieldCheck className="w-4 h-4 text-slate-700 dark:text-slate-300 group-hover:text-emerald-600 transition-colors" />
                    <div>
                      <span className="font-semibold text-[11px] block">问题反馈</span>
                      <span className="text-[10px] text-slate-400">提交 Issue / 建议</span>
                    </div>
                  </div>
                  <ExternalLink className="w-3 h-3 text-slate-400 group-hover:text-emerald-600 transition-colors" />
                </button>
              </div>

              <div className="text-[10px] text-slate-400 dark:text-slate-500 pt-1 text-center">
                Built with Tauri 2 + Rust & React · © 2026 Crvesoft
              </div>
            </div>
          </section>

        </div>

        {/* Footer */}
        <div className="px-5 py-3 border-t border-slate-200 dark:border-[#2c323f] bg-slate-50/70 dark:bg-[#1a1d24]/60 flex items-center justify-end">
          <button
            type="button"
            onClick={() => setIsSettingsModalOpen(false)}
            className="px-4 py-1.5 rounded-xl bg-slate-200 hover:bg-slate-300 dark:bg-slate-700 dark:hover:bg-slate-600 text-slate-800 dark:text-slate-200 font-medium text-xs transition-colors shadow-2xs"
          >
            完成
          </button>
        </div>
      </div>
    </div>
  );
};
