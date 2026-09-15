import { useStore } from "../store/useStore";
import { X } from "lucide-react";

const shortcuts = [
  { keys: "0-9", action: "Rakam gir" },
  { keys: ".", action: "Ondalık ayracı" },
  { keys: "Backspace", action: "Son karakteri sil" },
  { keys: "Escape", action: "Temizle" },
  { keys: "Enter", action: "Sonucu kaydet" },
  { keys: "Tab", action: "Alan değiştir" },
];

export function Help() {
  const showHelp = useStore((s) => s.showHelp);
  const toggleHelp = useStore((s) => s.toggleHelp);

  if (!showHelp) return null;

  return (
    <div className="fixed inset-0 z-[9998] flex items-center justify-center" style={{ background: "rgba(0,0,0,0.5)", backdropFilter: "blur(4px)" }}
      onClick={toggleHelp}>
      <div className="w-80 rounded-xl p-5 space-y-4 anim-scale-in" style={{ background: "var(--color-bg-base)", border: "1px solid var(--color-glass-border)" }}
        onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between">
          <span className="text-sm font-bold" style={{ color: "var(--color-text-primary)" }}>Klavye Kısayolları</span>
          <button onClick={toggleHelp} className="p-1 rounded-md transition-colors" style={{ color: "var(--color-text-ghost)" }}
            onMouseEnter={(e) => e.currentTarget.style.background = "var(--color-glass-hover)"}
            onMouseLeave={(e) => e.currentTarget.style.background = "transparent"}>
            <X size={14} />
          </button>
        </div>

        <div className="space-y-1">
          {shortcuts.map((s, i) => (
            <div key={i} className="flex items-center justify-between py-1.5" style={{ borderBottom: i < shortcuts.length - 1 ? "1px solid var(--color-glass-border)" : "none" }}>
              <kbd className="px-1.5 py-0.5 rounded text-[10px] font-mono font-bold" style={{ background: "var(--color-glass)", color: "var(--color-text-secondary)", border: "1px solid var(--color-glass-border)" }}>
                {s.keys}
              </kbd>
              <span className="text-[11px]" style={{ color: "var(--color-text-tertiary)" }}>{s.action}</span>
            </div>
          ))}
        </div>

        <p className="text-[10px] text-center" style={{ color: "var(--color-text-ghost)" }}>Developer: Arda M. Ekiz</p>
      </div>
    </div>
  );
}
