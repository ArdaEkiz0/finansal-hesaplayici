import { useStore } from "../store/useStore";
import { Minus, Square, X, Settings, HelpCircle, Calculator } from "lucide-react";
import { useState, useEffect } from "react";

export function TitleBar() {
  const toggleWidget = useStore((s) => s.toggleWidget);
  const toggleHelp = useStore((s) => s.toggleHelp);
  const toggleSettings = useStore((s) => s.toggleSettings);
  const [isMax, setIsMax] = useState(false);

  useEffect(() => {
    const api = (window as any).electronAPI;
    if (api?.onMaximizeChange) api.onMaximizeChange((_e: any, m: boolean) => setIsMax(m));
    if (api?.isMaximized) api.isMaximized().then(setIsMax);
  }, []);

  const minimize = () => (window as any).electronAPI?.minimize?.();
  const maximize = () => (window as any).electronAPI?.maximize?.();
  const close = () => (window as any).electronAPI?.close?.();

  return (
    <div className="drag-region flex items-center h-9 px-3 shrink-0" style={{ background: "var(--color-bg-base)", borderBottom: "1px solid var(--color-glass-border)" }}>
      <div className="flex items-center gap-2.5">
        <div className="w-5 h-5 rounded-md flex items-center justify-center" style={{ background: "linear-gradient(135deg, #3b82f6, #8b5cf6)" }}>
          <Calculator size={11} className="text-white" />
        </div>
        <span className="text-xs font-semibold tracking-tight" style={{ color: "var(--color-text-primary)" }}>Finansal Hesaplayici</span>
      </div>

      <div className="ml-auto flex items-center gap-0.5">
        <button onClick={toggleWidget} className="no-drag p-1.5 rounded-md transition-colors" style={{ color: "var(--color-text-tertiary)" }}
          onMouseEnter={(e) => (e.currentTarget.style.background = "var(--color-glass-hover)")}
          onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}>
          <Calculator size={14} />
        </button>
        <button onClick={toggleHelp} className="no-drag p-1.5 rounded-md transition-colors" style={{ color: "var(--color-text-tertiary)" }}
          onMouseEnter={(e) => (e.currentTarget.style.background = "var(--color-glass-hover)")}
          onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}>
          <HelpCircle size={14} />
        </button>
        <button onClick={toggleSettings} className="no-drag p-1.5 rounded-md transition-colors" style={{ color: "var(--color-text-tertiary)" }}
          onMouseEnter={(e) => (e.currentTarget.style.background = "var(--color-glass-hover)")}
          onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}>
          <Settings size={14} />
        </button>

        <div className="w-px h-3.5 mx-1" style={{ background: "var(--color-glass-border)" }} />

        <button onClick={minimize} className="no-drag p-1.5 rounded-md transition-colors" style={{ color: "var(--color-text-tertiary)" }}
          onMouseEnter={(e) => (e.currentTarget.style.background = "var(--color-glass-hover)")}
          onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}>
          <Minus size={12} />
        </button>
        <button onClick={maximize} className="no-drag p-1.5 rounded-md transition-colors" style={{ color: "var(--color-text-tertiary)" }}
          onMouseEnter={(e) => (e.currentTarget.style.background = "var(--color-glass-hover)")}
          onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}>
          <Square size={11} />
        </button>
        <button onClick={close} className="no-drag p-1.5 rounded-md transition-colors" style={{ color: "var(--color-text-tertiary)" }}
          onMouseEnter={(e) => { e.currentTarget.style.background = "#ef4444"; e.currentTarget.style.color = "#fff"; }}
          onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = "var(--color-text-tertiary)"; }}>
          <X size={12} />
        </button>
      </div>
    </div>
  );
}
