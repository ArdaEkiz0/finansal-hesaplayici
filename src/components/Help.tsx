import { useStore } from "../store/useStore";
import { X } from "lucide-react";

const shortcuts = [
  { keys: ["0-9"], action: "Rakam gir" },
  { keys: ["."], action: "Ondalık nokta" },
  { keys: ["⌫"], action: "Son karakteri sil" },
  { keys: ["Tab"], action: "Alan değiştir (1→2)" },
  { keys: ["Esc"], action: "Her şeyi temizle" },
  { keys: ["Enter"], action: "Sonucu kaydet" },
  { keys: ["="], action: "Enter ile aynı" },
  { keys: ["F1"], action: "Bu yardımı aç/kapat" },
  { keys: ["F2"], action: "Ayarları aç/kapat" },
];

export function Help() {
  const show = useStore((s) => s.showHelp);
  const toggle = useStore((s) => s.toggleHelp);
  if (!show) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-md anim-fade-in" onClick={toggle}>
      <div className="glass-strong rounded-3xl shadow-2xl w-full max-w-xs mx-4 overflow-hidden anim-scale-in" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between p-4 border-b border-white/[0.06]">
          <h3 className="text-sm font-bold text-text-primary">Klavye Kısayolları</h3>
          <button onClick={toggle} className="p-1.5 rounded-lg glass hover:bg-white/[0.06] text-text-ghost">
            <X size={16} />
          </button>
        </div>
        <div className="p-4 space-y-1.5">
          {shortcuts.map((s, i) => (
            <div key={i} className="flex items-center justify-between py-1.5">
              <div className="flex gap-1">
                {s.keys.map((k) => (
                  <kbd key={k} className="px-2 py-0.5 rounded-lg glass-strong text-[10px] font-mono font-bold text-text-secondary">{k}</kbd>
                ))}
              </div>
              <span className="text-[11px] text-text-secondary">{s.action}</span>
            </div>
          ))}
        </div>
        <div className="px-4 py-2.5 border-t border-white/[0.06] text-center">
          <p className="text-[10px] text-text-ghost">Fiziksel numpad desteklenir</p>
        </div>
      </div>
    </div>
  );
}
