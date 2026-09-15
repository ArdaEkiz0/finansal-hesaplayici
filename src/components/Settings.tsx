import { useStore } from "../store/useStore";
import { X, Sun, Moon, Eye, EyeOff, Cloud, RefreshCw } from "lucide-react";

export function Settings() {
  const showSettings = useStore((s) => s.showSettings);
  const toggleSettings = useStore((s) => s.toggleSettings);
  const theme = useStore((s) => s.theme);
  const setTheme = useStore((s) => s.setTheme);
  const showWidget = useStore((s) => s.showWidget);
  const toggleWidget = useStore((s) => s.toggleWidget);

  if (!showSettings) return null;

  return (
    <div className="fixed inset-0 z-[9998] flex items-center justify-center" style={{ background: "rgba(0,0,0,0.5)", backdropFilter: "blur(4px)" }}
      onClick={toggleSettings}>
      <div className="w-80 rounded-xl p-5 space-y-4 anim-scale-in" style={{ background: "var(--color-bg-base)", border: "1px solid var(--color-glass-border)" }}
        onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between">
          <span className="text-sm font-bold" style={{ color: "var(--color-text-primary)" }}>Ayarlar</span>
          <button onClick={toggleSettings} className="p-1 rounded-md transition-colors" style={{ color: "var(--color-text-ghost)" }}
            onMouseEnter={(e) => e.currentTarget.style.background = "var(--color-glass-hover)"}
            onMouseLeave={(e) => e.currentTarget.style.background = "transparent"}>
            <X size={14} />
          </button>
        </div>

        {/* Theme */}
        <div className="flex items-center justify-between py-2" style={{ borderTop: "1px solid var(--color-glass-border)" }}>
          <span className="text-xs font-medium" style={{ color: "var(--color-text-secondary)" }}>Tema</span>
          <button onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-[11px] font-semibold transition-all"
            style={{ background: "var(--color-glass)", color: "var(--color-text-secondary)", border: "1px solid var(--color-glass-border)" }}>
            {theme === "dark" ? <Moon size={12} /> : <Sun size={12} />}
            {theme === "dark" ? "Karanlık" : "Aydınlık"}
          </button>
        </div>

        {/* Widget */}
        <div className="flex items-center justify-between py-2" style={{ borderTop: "1px solid var(--color-glass-border)" }}>
          <span className="text-xs font-medium" style={{ color: "var(--color-text-secondary)" }}>Widget</span>
          <button onClick={toggleWidget}
            className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-[11px] font-semibold transition-all"
            style={{ background: showWidget ? "#3b82f615" : "var(--color-glass)", color: showWidget ? "#3b82f6" : "var(--color-text-secondary)", border: `1px solid ${showWidget ? "#3b82f625" : "var(--color-glass-border)"}` }}>
            {showWidget ? <Eye size={12} /> : <EyeOff size={12} />}
            {showWidget ? "Açık" : "Kapalı"}
          </button>
        </div>

        {/* Version */}
        <div className="py-2" style={{ borderTop: "1px solid var(--color-glass-border)" }}>
          <p className="text-[10px]" style={{ color: "var(--color-text-ghost)" }}>v2.2.2 · Developer: Arda M. Ekiz</p>
        </div>
      </div>
    </div>
  );
}
