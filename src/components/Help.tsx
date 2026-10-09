import { useStore } from "../store/useStore";
import { X } from "lucide-react";

const SHORTCUTS = [
  { k: "0-9 .", v: "Rakam yaz (canlı hesap)" },
  { k: "Tab", v: "Alan değiştir" },
  { k: "Enter", v: "Sonraki alan / Kaydet" },
  { k: "Backspace", v: "Sil" },
  { k: "Esc", v: "Temizle" },
];

export function Help() {
  const show = useStore((s) => s.showHelp);
  const toggle = useStore((s) => s.toggleHelp);
  if (!show) return null;
  return (
    <div className="fixed inset-0 z-[9998] flex items-center justify-center p-4" style={{ background: "rgba(0,0,0,.5)", backdropFilter: "blur(4px)" }} onClick={toggle}>
      <div className="w-[320px] rounded-2xl p-5 space-y-3 anim-scale-in" style={{ background: "var(--color-bg-surface)", border: "1px solid var(--color-glass-border)" }} onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between">
          <span className="text-[14px] font-bold">Klavye</span>
          <button onClick={toggle} className="p-1.5 rounded-lg btn-press" style={{ color: "var(--color-text-ghost)" }}><X size={14} /></button>
        </div>
        {SHORTCUTS.map((s, i) => (
          <div key={i} className="flex items-center justify-between py-1.5" style={{ borderBottom: i < SHORTCUTS.length - 1 ? "1px solid var(--color-glass-border)" : "none" }}>
            <kbd className="px-2 py-0.5 rounded-lg text-[11px] font-mono font-bold" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-secondary)" }}>{s.k}</kbd>
            <span className="text-[12px]" style={{ color: "var(--color-text-tertiary)" }}>{s.v}</span>
          </div>
        ))}
        <p className="text-[10px] text-center" style={{ color: "var(--color-text-ghost)" }}>Sonuç satırına tıklayarak kopyalayabilirsiniz</p>
      </div>
    </div>
  );
}
