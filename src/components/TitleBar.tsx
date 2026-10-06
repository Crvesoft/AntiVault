import React, { useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { isTauri } from "@tauri-apps/api/core";
import { Minus, Square, X } from "lucide-react";
import { useVaultStore } from "../stores/useVaultStore";
import { AntiVaultLogo } from "./AntiVaultLogo";

export const TitleBar: React.FC = () => {
  const { addToast, closeToTray } = useVaultStore();
  const [busy, setBusy] = useState<"minimize" | "maximize" | "close" | null>(null);

  const runWindowAction = async (
    kind: "minimize" | "maximize" | "close",
    action: () => Promise<void>,
  ) => {
    if (!isTauri()) {
      addToast({
        type: "info",
        title: "浏览器预览模式",
        description: "窗口控制仅在桌面应用中可用。",
      });
      return;
    }

    setBusy(kind);
    try {
      await action();
    } catch (err: any) {
      addToast({
        type: "error",
        title:
          kind === "close"
            ? "关闭窗口失败"
            : kind === "minimize"
              ? "最小化失败"
              : "最大化失败",
        description: err?.message || String(err),
      });
    } finally {
      setBusy(null);
    }
  };

  const handleMinimize = () =>
    runWindowAction("minimize", () => getCurrentWindow().minimize());

  const handleMaximize = () =>
    runWindowAction("maximize", () => getCurrentWindow().toggleMaximize());

  const handleClose = () =>
    runWindowAction("close", async () => {
      if (closeToTray) {
        await getCurrentWindow().hide();
      } else {
        await getCurrentWindow().close();
      }
    });

  return (
    <header
      data-tauri-drag-region="deep"
      className="h-11 flex items-center justify-between px-3.5 bg-white dark:bg-[#1c1f26] border-b border-slate-200 dark:border-[#282c37] select-none z-50 sticky top-0 transition-colors duration-200"
    >
      {/* Brand */}
      <div className="flex items-center space-x-2 pointer-events-none">
        <AntiVaultLogo size={22} showText={true} />
      </div>

      {/* Window Controls */}
      <div className="flex items-center space-x-0.5">
        <button
          type="button"
          onClick={handleMinimize}
          disabled={busy !== null}
          className="w-8 h-8 flex items-center justify-center rounded-lg text-slate-400 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#282d38] transition-colors disabled:opacity-40"
          title="最小化"
          aria-label="最小化窗口"
        >
          <Minus className="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onClick={handleMaximize}
          disabled={busy !== null}
          className="w-8 h-8 flex items-center justify-center rounded-lg text-slate-400 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#282d38] transition-colors disabled:opacity-40"
          title="最大化 / 还原"
          aria-label="最大化或还原窗口"
        >
          <Square className="w-3 h-3" />
        </button>
        <button
          type="button"
          onClick={handleClose}
          disabled={busy !== null}
          className="w-8 h-8 flex items-center justify-center rounded-lg text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 transition-colors disabled:opacity-40"
          title={closeToTray ? "关闭到系统托盘（后台常驻）" : "关闭程序"}
          aria-label="关闭窗口"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
};
