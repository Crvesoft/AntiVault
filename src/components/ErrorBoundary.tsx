import { Component, ErrorInfo, ReactNode } from "react";
import { AlertCircle, RotateCw } from "lucide-react";

interface Props {
  children: ReactNode;
  fallbackTitle?: string;
  onReset?: () => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("AntiVault Caught Error in ErrorBoundary:", error, errorInfo);
  }

  private handleReset = () => {
    this.setState({ hasError: false, error: null });
    if (this.props.onReset) {
      this.props.onReset();
    }
  };

  public render() {
    if (this.state.hasError) {
      return (
        <div className="p-4 m-3 rounded-2xl bg-rose-50/90 border border-rose-200 text-rose-800 text-xs space-y-2 animate-in fade-in">
          <div className="flex items-center space-x-2 font-bold text-rose-900">
            <AlertCircle className="w-4 h-4 text-rose-600 shrink-0" />
            <span>{this.props.fallbackTitle || "组件渲染异常"}</span>
          </div>
          <p className="text-slate-600 font-mono text-[11px] break-all">
            {this.state.error?.message || "未知渲染错误"}
          </p>
          <button
            onClick={this.handleReset}
            className="inline-flex items-center space-x-1 px-3 py-1 bg-white hover:bg-rose-100/50 border border-rose-200 rounded-lg text-rose-700 font-medium transition-colors shadow-2xs"
          >
            <RotateCw className="w-3 h-3" />
            <span>重试恢复</span>
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
