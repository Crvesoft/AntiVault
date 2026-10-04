import React, { useState } from "react";
import { X, Globe, KeyRound, ArrowRight, Loader2, ShieldCheck } from "lucide-react";
import { useVaultStore } from "../stores/useVaultStore";
import { api } from "../services/api";

export const AddAccountModal: React.FC = () => {
  const { 
    isAddModalOpen, 
    setIsAddModalOpen, 
    startGoogleLogin, 
    completeGoogleLogin,
    cancelGoogleLogin,
    fetchAccounts,
    addToast
  } = useVaultStore();

  const [activeTab, setActiveTab] = useState<"oauth" | "manual">("oauth");
  const [loading, setLoading] = useState(false);

  // 手动完成授权（浏览器回调无法到达本机时的兜底）
  const [showManualFallback, setShowManualFallback] = useState(false);
  const [manualCode, setManualCode] = useState("");
  const [manualLoading, setManualLoading] = useState(false);

  // 手动录入表单状态
  const [email, setEmail] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [refreshToken, setRefreshToken] = useState("");

  if (!isAddModalOpen) return null;

  const handleOAuthLogin = async () => {
    setLoading(true);
    // 授权等待期间用户可能已经拿到授权码，直接展开兜底输入框
    setShowManualFallback(true);
    try {
      const ok = await startGoogleLogin();
      if (ok) setIsAddModalOpen(false);
    } finally {
      setLoading(false);
    }
  };

  const handleManualCodeSubmit = async () => {
    setManualLoading(true);
    try {
      const ok = await completeGoogleLogin(manualCode);
      if (ok) {
        setManualCode("");
        setIsAddModalOpen(false);
      }
    } finally {
      setManualLoading(false);
    }
  };

  // 关闭弹窗不取消授权：让浏览器里的登录继续在后台等待回调，否则用户刚在
  // 浏览器里完成授权，应用这边却已把监听器关掉，导致"登录成功但没反应"。
  const handleClose = () => {
    if (loading && activeTab === "oauth") {
      addToast({
        type: "info",
        title: "登录仍在后台进行",
        description: "在浏览器中完成授权后会直接添加账号，无需再点任何按钮。",
      });
    }
    setIsAddModalOpen(false);
  };

  const handleTabChange = (tab: "oauth" | "manual") => {
    setActiveTab(tab);
  };

  const handleManualSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email.trim() || !refreshToken.trim()) {
      addToast({
        type: "warning",
        title: "必填信息未完善",
        description: "请提供有效的谷歌邮箱及 Refresh Token。",
      });
      return;
    }

    setLoading(true);
    try {
      await api.addAccount({
        email: email.trim(),
        display_name: displayName.trim() || null,
        refresh_token: refreshToken.trim(),
      });
      await fetchAccounts();
      addToast({
        type: "success",
        title: "账号添加成功",
        description: `已成功保存账号「${email}」`,
      });
      setIsAddModalOpen(false);
    } catch (err: any) {
      addToast({
        type: "error",
        title: "保存账号失败",
        description: err?.message || String(err),
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/40 dark:bg-black/70 animate-in fade-in duration-200"
      onClick={handleClose}
    >
      <div 
        className="w-full max-w-lg bg-white dark:bg-[#1e222a] border border-slate-200 dark:border-[#2c323f] rounded-3xl shadow-2xl overflow-hidden flex flex-col animate-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        {/* 顶部标题栏 */}
        <div className="px-6 py-4.5 border-b border-slate-100 dark:border-[#282c37] flex items-center justify-between bg-slate-50/70 dark:bg-[#181b20]">
          <div className="flex items-center space-x-2">
            <span className="text-base font-bold text-slate-900 dark:text-slate-100">添加 Antigravity 账号</span>
          </div>
          <button
            onClick={handleClose}
            className="p-2 rounded-xl hover:bg-slate-200/60 dark:hover:bg-[#282d38] text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* 标签栏 */}
        <div className="flex border-b border-slate-200 dark:border-[#282c37] bg-slate-50/50 dark:bg-[#181b20] p-1.5 gap-1">
          <button
            onClick={() => handleTabChange("oauth")}
            className={`flex-1 flex items-center justify-center space-x-2 py-2.5 text-xs sm:text-sm font-semibold rounded-xl transition-all ${
              activeTab === "oauth"
                ? "bg-white dark:bg-[#282d38] text-emerald-700 dark:text-emerald-400 shadow-sm"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100"
            }`}
          >
            <Globe className="w-4 h-4" />
            <span>Google 授权</span>
          </button>
          <button
            onClick={() => handleTabChange("manual")}
            className={`flex-1 flex items-center justify-center space-x-2 py-2.5 text-xs sm:text-sm font-semibold rounded-xl transition-all ${
              activeTab === "manual"
                ? "bg-white dark:bg-[#282d38] text-emerald-700 dark:text-emerald-400 shadow-sm"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100"
            }`}
          >
            <KeyRound className="w-4 h-4" />
            <span>手动录入 Token</span>
          </button>
        </div>

        {/* 表单内容 */}
        <div className="p-6">
          {activeTab === "oauth" && (
            <div className="text-center py-4 space-y-4">
              <div className="w-14 h-14 rounded-2xl bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800/60 text-emerald-600 dark:text-emerald-400 flex items-center justify-center mx-auto shadow-sm">
                <Globe className="w-7 h-7" />
              </div>
              <div className="space-y-1.5">
                <h3 className="text-base font-bold text-slate-900 dark:text-slate-100">通过浏览器一键登录 Google</h3>
                <p className="text-xs sm:text-sm text-slate-500 dark:text-slate-400 max-w-sm mx-auto leading-relaxed">
                  点击后将唤起系统浏览器进入 Google 授权页。授权完成后，凭据将自动回调并加密存入 Windows 本地密钥环。
                </p>
              </div>

              <div className="pt-3">
                <button
                  onClick={handleOAuthLogin}
                  disabled={loading}
                  className="w-full py-3 px-5 rounded-2xl bg-emerald-600 hover:bg-emerald-500 text-white font-semibold text-sm flex items-center justify-center space-x-2 shadow-md shadow-emerald-700/20 transition-all active:scale-[0.98] disabled:opacity-50"
                >
                  {loading ? (
                    <>
                      <Loader2 className="w-4 h-4 animate-spin" />
                      <span>正在等待浏览器授权...</span>
                    </>
                  ) : (
                    <>
                      <span>打开浏览器进行授权登录</span>
                      <ArrowRight className="w-4 h-4" />
                    </>
                  )}
                </button>
              </div>

              {/* 等待中显式提供取消入口：关闭弹窗/切换标签页不再取消授权，
                  让浏览器里的登录可以继续等待回调 */}
              {loading && (
                <div className="pt-2 text-center">
                  <button
                    type="button"
                    onClick={() => {
                      void cancelGoogleLogin();
                      setLoading(false);
                    }}
                    className="text-xs text-slate-400 dark:text-slate-500 hover:text-red-500 dark:hover:text-red-400 transition-colors"
                  >
                    取消本次授权
                  </button>
                </div>
              )}

              {/* 手动兜底：浏览器回调无法到达本机时使用 */}
              <div className="pt-1 text-left">
                <button
                  type="button"
                  onClick={() => setShowManualFallback((v) => !v)}
                  className="w-full flex items-center justify-center space-x-1.5 text-xs text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200 transition-colors py-1"
                >
                  <span>浏览器授权后没有自动跳回？点此手动完成</span>
                </button>

                {showManualFallback && (
                  <div className="mt-2 p-3.5 rounded-2xl bg-slate-50 dark:bg-[#181b20] border border-slate-200 dark:border-[#282c37] space-y-2.5">
                    <p className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed">
                      授权成功后浏览器地址栏会停留在{" "}
                      <code className="font-mono text-[11px] bg-white dark:bg-[#20242b] px-1 py-0.5 rounded border border-slate-200 dark:border-[#2c323f] text-slate-800 dark:text-slate-200">
                        http://localhost:.../oauth-callback?code=...
                      </code>
                      。请把整条链接（或 Google 返回的授权码）粘贴到下方。
                    </p>
                    <textarea
                      value={manualCode}
                      onChange={(e) => setManualCode(e.target.value)}
                      rows={3}
                      placeholder="粘贴完整回调链接，或 4/0Axxxxxxxx 形式的授权码"
                      className="w-full text-xs font-mono p-2.5 rounded-xl border border-slate-200 dark:border-[#2c323f] bg-white dark:bg-[#181b20] text-slate-800 dark:text-slate-100 placeholder:text-slate-400 dark:placeholder:text-slate-500 focus:outline-hidden focus:ring-2 focus:ring-emerald-500/30 focus:border-emerald-400 resize-none select-text"
                    />
                    <button
                      type="button"
                      onClick={handleManualCodeSubmit}
                      disabled={manualLoading || !manualCode.trim()}
                      className="w-full py-2.5 px-4 rounded-xl bg-slate-900 dark:bg-emerald-600 hover:bg-slate-800 dark:hover:bg-emerald-500 text-white font-semibold text-xs flex items-center justify-center space-x-2 transition-all active:scale-[0.98] disabled:opacity-50"
                    >
                      {manualLoading ? (
                        <>
                          <Loader2 className="w-3.5 h-3.5 animate-spin" />
                          <span>正在校验授权码...</span>
                        </>
                      ) : (
                        <span>提交授权码完成登录</span>
                      )}
                    </button>
                  </div>
                )}
              </div>
            </div>
          )}


          {activeTab === "manual" && (
            <form onSubmit={handleManualSubmit} className="space-y-4">
              <div>
                <label className="block text-xs sm:text-sm font-semibold text-slate-700 dark:text-slate-300 mb-1.5">
                  谷歌账号邮箱 *
                </label>
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="developer@gmail.com"
                  className="w-full px-3.5 py-2.5 rounded-xl border border-slate-200 dark:border-[#2c323f] bg-white dark:bg-[#181b20] text-sm text-slate-900 dark:text-slate-100 placeholder-slate-400 dark:placeholder-slate-500 focus:outline-hidden focus:ring-2 focus:ring-emerald-500/30 focus:border-emerald-400"
                  required
                />
              </div>

              <div>
                <label className="block text-xs sm:text-sm font-semibold text-slate-700 dark:text-slate-300 mb-1.5">
                  账号别名备注 (选填)
                </label>
                <input
                  type="text"
                  value={displayName}
                  onChange={(e) => setDisplayName(e.target.value)}
                  placeholder="例如：工作主账号 / 备用高配账号"
                  className="w-full px-3.5 py-2.5 rounded-xl border border-slate-200 dark:border-[#2c323f] bg-white dark:bg-[#181b20] text-sm text-slate-900 dark:text-slate-100 placeholder-slate-400 dark:placeholder-slate-500 focus:outline-hidden focus:ring-2 focus:ring-emerald-500/30 focus:border-emerald-400"
                />
              </div>

              <div>
                <label className="block text-xs sm:text-sm font-semibold text-slate-700 dark:text-slate-300 mb-1.5">
                  OAuth Refresh Token *
                </label>
                <textarea
                  value={refreshToken}
                  onChange={(e) => setRefreshToken(e.target.value)}
                  placeholder="1//04..."
                  rows={3}
                  className="w-full px-3.5 py-2.5 rounded-xl border border-slate-200 dark:border-[#2c323f] bg-white dark:bg-[#181b20] text-sm font-mono text-slate-900 dark:text-slate-100 placeholder-slate-400 dark:placeholder-slate-500 resize-none focus:outline-hidden focus:ring-2 focus:ring-emerald-500/30 focus:border-emerald-400"
                  required
                />
              </div>

              <div className="pt-2">
                <button
                  type="submit"
                  disabled={loading}
                  className="w-full py-3 px-5 rounded-2xl bg-emerald-600 hover:bg-emerald-500 text-white font-semibold text-sm flex items-center justify-center space-x-2 shadow-md shadow-emerald-700/20 transition-all active:scale-[0.98] disabled:opacity-50"
                >
                  {loading ? (
                    <>
                      <Loader2 className="w-4 h-4 animate-spin" />
                      <span>正在保存账号...</span>
                    </>
                  ) : (
                    <span>保存并添加账号</span>
                  )}
                </button>
              </div>
            </form>
          )}
        </div>

        {/* 底部隐私加密说明 */}
        <div className="px-6 py-3.5 border-t border-slate-100 dark:border-[#282c37] bg-slate-50/70 dark:bg-[#181b20] flex items-center space-x-2 text-xs text-slate-500 dark:text-slate-400">
          <ShieldCheck className="w-4 h-4 text-emerald-600 shrink-0" />
          <span>采用 Windows 凭据管理器系统级加密，数据仅存本地，绝无云端泄露。</span>
        </div>
      </div>
    </div>
  );
};
