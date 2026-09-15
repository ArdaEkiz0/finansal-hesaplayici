import { useStore } from "../store/useStore";
import { X } from "lucide-react";

export function Settings() {
  const settings = useStore((s) => s.settings);
  const updateSettings = useStore((s) => s.updateSettings);
  const toggleSettings = useStore((s) => s.toggleSettings);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-md anim-fade-in" onClick={toggleSettings}>
      <div className="glass-strong rounded-3xl shadow-2xl w-full max-w-sm mx-4 overflow-hidden anim-scale-in" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between p-5 border-b border-white/[0.06]">
          <h3 className="text-sm font-bold text-text-primary">Ayarlar</h3>
          <button onClick={toggleSettings} className="p-1.5 rounded-lg glass hover:bg-white/[0.06] text-text-ghost">
            <X size={16} />
          </button>
        </div>
        <div className="p-5 space-y-4">
          {/* Currency Symbol */}
          <div className="flex items-center justify-between">
            <span className="text-xs text-text-secondary">₺ Sembolü</span>
            <button onClick={() => updateSettings({ showCurrencySymbol: !settings.showCurrencySymbol })}
              className={`w-10 h-5 rounded-full transition-all duration-200 ${settings.showCurrencySymbol ? "bg-blue-500" : "bg-white/10"}`}>
              <div className={`w-4 h-4 rounded-full bg-white shadow transition-transform duration-200 ${settings.showCurrencySymbol ? "translate-x-5.5" : "translate-x-0.5"}`} />
            </button>
          </div>
          {/* Precision */}
          <div className="space-y-1.5">
            <span className="text-xs text-text-secondary">Basamak Hassasiyeti</span>
            <div className="flex gap-1.5">
              {[0, 2, 4, 6].map((p) => (
                <button key={p} onClick={() => updateSettings({ precision: p })}
                  className={`flex-1 py-1.5 rounded-xl text-xs font-bold transition-all ${settings.precision === p ? "bg-blue-500 text-white" : "glass text-text-secondary hover:text-text-primary"}`}>
                  {p}
                </button>
              ))}
            </div>
          </div>
        </div>
        <div className="px-5 py-3 border-t border-white/[0.06] text-center">
          <p className="text-[10px] text-text-ghost">decimal.js — 30 basamak hassasiyet</p>
        </div>
      </div>
    </div>
  );
}
