import React from "react";
import { AlertTriangle, Trash2, X, Loader2 } from "lucide-react";
import { useVaultStore } from "../stores/useVaultStore";

/**
 * In-app replacement for `window.confirm` when removing an account.
 *
 * The native dialog is unreliable inside the Tauri webview — when it is suppressed
 * the click looks like it did nothing at all — so removal is confirmed here instead.
 */
export const DeleteConfirmModal: React.FC = () => {
  const {
    deleteTargetAccount,
    setDeleteTargetAccount,
    deleteAccount,
    isDeletingAccount,
  } = useVaultStore();

  if (!deleteTargetAccount) return null;

  const account = deleteTargetAccount;

  const handleConfirm = async () => {
    await deleteAccount(account.id);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/40 dark:bg-black/70 animate-in fade-in duration-200"
      onClick={() => {
        if (!isDeletingAccount) setDeleteTargetAccount(null);
      }}
    >
      <div
        className="w-full max-w-md bg-white dark:bg-[#1e222a] border border-slate-200 dark:border-[#2c323f] rounded-2xl shadow-2xl overflow-hidden flex flex-col animate-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="px-6 py-4 border-b border-rose-100 dark:border-rose-950/50 bg-rose-50/70 dark:bg-rose-950/20 flex items-center justify-between">
          <div className="flex items-center space-x-2.5">
            <AlertTriangle className="w-5 h-5 shrink-0 text-rose-600 dark:text-rose-400" />
            <span className="text-base font-semibold text-rose-700 dark:text-rose-300">移除账号</span>
          </div>
          <button
            type="button"
            onClick={() => setDeleteTargetAccount(null)}
            disabled={isDeletingAccount}
            className="p-1.5 rounded-lg hover:bg-slate-200/50 dark:hover:bg-[#282d38] text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 transition-colors disabled:opacity-40"
            aria-label="关闭"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-4">
          <p className="text-sm text-slate-600 dark:text-slate-300 leading-relaxed">
            确定要从 AntiVault 中移除以下账号吗？该账号保存在本地密钥环中的凭据会一并删除。
          </p>

          <div className="p-3.5 rounded-xl bg-slate-50 dark:bg-[#181b20] border border-slate-200/80 dark:border-[#282c37] space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400 font-medium">即将移除的账号：</div>
            <div className="text-sm font-semibold text-slate-800 dark:text-slate-100 break-all">
              {account.display_name ? `${account.display_name} · ` : ""}
              {account.email}
            </div>
          </div>

          <p className="text-xs text-slate-400 dark:text-slate-500 leading-relaxed">
            此操作不会影响 Antigravity 客户端中已登录的账号，您随时可以重新添加。
          </p>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-slate-100 dark:border-[#282c37] bg-slate-50/60 dark:bg-[#181b20] flex justify-end space-x-3">
          <button
            type="button"
            onClick={() => setDeleteTargetAccount(null)}
            disabled={isDeletingAccount}
            className="px-4 py-2 rounded-xl bg-slate-200/80 dark:bg-[#282d38] hover:bg-slate-200 dark:hover:bg-[#323947] text-slate-700 dark:text-slate-300 text-sm font-medium transition-colors disabled:opacity-50"
          >
            取消
          </button>
          <button
            type="button"
            onClick={handleConfirm}
            disabled={isDeletingAccount}
            className="px-5 py-2 rounded-xl bg-rose-600 hover:bg-rose-700 text-white text-sm font-medium flex items-center space-x-2 shadow-sm shadow-rose-700/20 transition-all active:scale-95 disabled:opacity-50"
          >
            {isDeletingAccount ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                <span>正在移除...</span>
              </>
            ) : (
              <>
                <Trash2 className="w-4 h-4" />
                <span>确认移除</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};
