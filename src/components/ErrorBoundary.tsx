import { Component, type ReactNode } from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";

interface Props { children: ReactNode; }
interface State { hasError: boolean; error: Error | null; }

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="h-screen bg-bg-deep flex items-center justify-center p-6">
          <div className="glass-strong rounded-3xl p-8 max-w-md w-full text-center space-y-6">
            <div className="w-16 h-16 mx-auto rounded-2xl bg-rose-500/15 flex items-center justify-center">
              <AlertTriangle size={28} className="text-rose-400" />
            </div>
            <div className="space-y-2">
              <h2 className="text-xl font-bold text-text-primary">Bir Hata Oluştu</h2>
              <p className="text-sm text-text-secondary">Hesaplayıcı beklenmeyen bir hata ile karşılaştı.</p>
              {this.state.error && (
                <p className="text-xs text-text-ghost font-mono bg-white/[0.03] rounded-xl p-3 text-left overflow-auto max-h-32">
                  {this.state.error.message}
                </p>
              )}
            </div>
            <button onClick={() => window.location.reload()}
              className="flex items-center gap-2 mx-auto px-6 py-3 rounded-2xl bg-blue-500/15 text-blue-400 border border-blue-500/30 font-bold text-sm hover:bg-blue-500/25 transition-all active:scale-95">
              <RefreshCw size={16} /> Yeniden Dene
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
