import { Calculator, Settings, HelpCircle } from "lucide-react";
import { useStore } from "../store/useStore";

export function TitleBar() {
  const toggleSettings = useStore((s) => s.toggleSettings);
  const toggleHelp = useStore((s) => s.toggleHelp);

  return (
    <header className="h-10 glass-strong border-b border-white/[0.06] flex items-center justify-between px-4 drag-region">
      <div className="flex items-center gap-2 no-drag">
        <div className="w-6 h-6 rounded-lg bg-gradient-to-br from-blue-500 via-violet-500 to-emerald-500 flex items-center justify-center">
          <Calculator size={12} className="text-white" />
        </div>
        <span className="text-xs font-bold text-text-primary gradient-text">
          Finansal Hesaplayıcı
        </span>
        <span className="text-[9px] text-text-ghost ml-1">by Arda M. Ekiz</span>
      </div>

      <div className="flex items-center gap-1 no-drag">
        <button
          onClick={toggleHelp}
          className="p-1.5 rounded-lg text-text-ghost hover:text-violet-400 hover:bg-violet-500/10 transition-all"
          title="Yardım (F1)"
        >
          <HelpCircle size={14} />
        </button>
        <button
          onClick={toggleSettings}
          className="p-1.5 rounded-lg text-text-ghost hover:text-brand-400 hover:bg-brand-500/10 transition-all"
          title="Ayarlar (F2)"
        >
          <Settings size={14} />
        </button>
      </div>
    </header>
  );
}
