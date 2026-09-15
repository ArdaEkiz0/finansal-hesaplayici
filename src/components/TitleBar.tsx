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
      {/* App Icon and Title */}
      <div className="flex items-center gap-2.5">
        <div className="w-6 h-6 rounded-xl flex items-center justify-center" style={{ background: "linear-gradient(135deg, var(--color-primary), var(--color-primary-light))" }}>
          <Calculator size={12} className="text-white" />
        </div>
        <span className="text-xs font-semibold tracking-tight gradient-text" style={{ background: "linear-gradient(135deg, var(--color-primary), var(--color-primary-light))", WebkitBackgroundClip: "text", WebkitTextFillColor: "transparent" }}>
          Finansal Hesaplayici
        </span>
      </div>

      {/* Right Controls */}
      <div className="ml-auto flex items-center gap-0.5">
        <button onClick={toggleWidget} className="no-drag p-1.5 rounded-xl hover-lift press-scale transition-all duration-150" style={{ color: "var(--color-text-tertiary)", background: "transparent" }}
          onMouseEnter={(e) => { e.currentTarget.style.background = "var(--color-glass)"; e.currentTarget.style.color = "var(--color-text-secondary)"; }}
          onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = "var(--color-text-tertiary)"; }}>
          <Calculator size={14} />
        </button>
        <button onClick={toggleHelp} className="no-drag p-1.5 rounded-xl hover-lift press-scale transition-all duration-150" style={{ color: "var(--color-text-tertiary)", background: "transparent" }}
          onMouseEnter={(e) => { e.currentTarget.style.background = "var(--color-glass)"; e.currentTarget.style.color = "var(--color-text-secondary)"; }}
          onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = "var(--color-text-tertiary)"; }}>
          <HelpCircle size={14} />
        </button>
        <button onClick={toggleSettings} className="no-drag p-1.5 rounded-xl hover-lift press-scale transition-all duration-150" style={{ color: "var(--color-text-tertiary)", background: "transparent" }}
          onMouseEnter={(e) => { e.currentTarget.style.background = "var(--color-glass)"; e.currentTarget.style.color = "var(--color-text-secondary)"; }}
          onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = "var(--color-text-tertiary)"; }}>
          <Settings size={14} />
        </button>

        <div className="w-px h-3.5 mx-1" style={{ background: "var(--color-glass-border)" }} />

        <button onClick={minimize} className="no-drag p-1.5 rounded-xl hover-lift press-scale transition-all duration-150" style={{ color: "var(--color-text-tertiary)", background: "transparent" }}
          onMouseEnter={(e) => { e.currentTarget.style.background = "var(--color-glass)"; e.currentTarget.style.color = "var(--color-text-secondary)"; }}
          onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = "var(--color-text-tertiary)"; }}>
          <Minus size={12} />
        </button>
        <button onClick={maximize} className="no-drag p-1.5 rounded-xl hover-lift press-scale transition-all duration-150" style={{ color: "var(--color-text-tertiary)", background: "transparent" }}
          onMouseEnter={(e) => { e.currentTarget.style.background = "var(--color-glass)"; e.currentTarget.style.color = "var(--color-text-secondary)"; }}
          onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = "var(--color-text-tertiary)"; }}>
          <Square size={11} />
        </button>
        <button onClick={close} className="no-drag p-1.5 rounded-xl hover-lift press-scale transition-all duration-150" style={{ color: "var(--color-text-tertiary)", background: "transparent" }}
          onMouseEnter={(e) => { e.currentTarget.style.background = "var(--color-error)"; e.currentTarget.style.color = "#fff"; }}
          onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = "var(--color-text-tertiary)"; }}>
          <X size={12} />
        </button>
      </div>
    </div>
  );
}