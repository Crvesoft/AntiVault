import React from "react";
import { CheckCircle2, AlertCircle, Info, AlertTriangle, X } from "lucide-react";
import { useVaultStore } from "../stores/useVaultStore";

export const ToastContainer: React.FC = () => {
  const { toasts, removeToast } = useVaultStore();

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-14 right-5 z-50 flex flex-col space-y-2.5 pointer-events-none max-w-sm w-full">
      {toasts.map((toast) => {
        let Icon = Info;
        let iconColor = "text-sky-600";
        let borderColor = "border-sky-200";
        let shadowColor = "shadow-sky-500/5";

        if (toast.type === "success") {
          Icon = CheckCircle2;
          iconColor = "text-emerald-600";
          borderColor = "border-emerald-200";
          shadowColor = "shadow-emerald-500/5";
        } else if (toast.type === "error") {
          Icon = AlertCircle;
          iconColor = "text-rose-600";
          borderColor = "border-rose-200";
          shadowColor = "shadow-rose-500/5";
        } else if (toast.type === "warning") {
          Icon = AlertTriangle;
          iconColor = "text-amber-600";
          borderColor = "border-amber-200";
          shadowColor = "shadow-amber-500/5";
        }

        return (
          <div
            key={toast.id}
            className={`pointer-events-auto flex items-start space-x-3 p-3.5 rounded-xl border bg-white shadow-lg ${borderColor} ${shadowColor} transition-all duration-300 animate-in fade-in slide-in-from-bottom-2`}
          >
            <Icon className={`w-5 h-5 shrink-0 mt-0.5 ${iconColor}`} />
            <div className="flex-1 min-w-0">
              <div className="text-sm font-semibold text-slate-800">{toast.title}</div>
              {toast.description && (
                <div className="text-xs text-slate-600 mt-0.5 break-words leading-relaxed">
                  {toast.description}
                </div>
              )}
            </div>
            <button
              onClick={() => removeToast(toast.id)}
              className="text-slate-400 hover:text-slate-600 p-1 rounded-lg hover:bg-slate-100 transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        );
      })}
    </div>
  );
};

