import { Calculator, Settings, HelpCircle, AppWindow, Minus, Square, X, Maximize2 } from "lucide-react";
import { useStore } from "../store/useStore";
import { useState, useEffect } from "react";

export function TitleBar() {
  const toggleSettings = useStore((s) => s.toggleSettings);
  const toggleHelp = useStore((s) => s.toggleHelp);
  const toggleWidget = useStore((s) => s.toggleWidget);
  const showWidget = useStore((s) => s.showWidget);
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    window.electronAPI?.isMaximized().then(setMaximized);
    window.electronAPI?.onMaximizeChange(setMaximized);
  }, []);

  const handleMinimize = () => window.electronAPI?.minimize();
  const handleMaximize = () => window.electronAPI?.maximize();
  const handleClose = () => window.electronAPI?.close();

  return (
    <header className="h-10 glass-strong border-b border-glass-border flex items-center justify-between drag-region">
      {/* Left - App info */}
      <div className="flex items-center gap-2 pl-4 no-drag">
        <div className="w-6 h-6 rounded-lg bg-gradient-to-br from-blue-500 via-violet-500 to-emerald-500 flex items-center justify-center">
          <Calculator size={12} className="text-white" />
        </div>
        <span className="text-xs font-bold text-text-primary gradient-text">
          Finansal Hesaplayıcı
        </span>
        <span className="text-[9px] text-text-ghost ml-1">by Arda M. Ekiz</span>
      </div>

      {/* Center - Tool buttons */}
      <div className="flex items-center gap-1 no-drag">
        <button
          onClick={toggleWidget}
          className={`p-1.5 rounded-lg transition-all ${showWidget ? "text-brand-400 bg-brand-500/10" : "text-text-ghost hover:text-brand-400 hover:bg-brand-500/10"}`}
          title="Widget"
        >
          <AppWindow size={14} />
        </button>
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

      {/* Right - Window controls */}
      <div className="flex items-center no-drag">
        <button
          onClick={handleMinimize}
          className="w-11 h-10 flex items-center justify-center text-text-ghost hover:text-text-primary hover:bg-white/[0.06] transition-all"
          title="Kucult"
        >
          <Minus size={14} />
        </button>
        <button
          onClick={handleMaximize}
          className="w-11 h-10 flex items-center justify-center text-text-ghost hover:text-text-primary hover:bg-white/[0.06] transition-all"
          title={maximized ? "Pencere Boyutlandir" : "Tam Ekran"}
        >
          {maximized ? <Maximize2 size={13} /> : <Square size={12} />}
        </button>
        <button
          onClick={handleClose}
          className="w-11 h-10 flex items-center justify-center text-text-ghost hover:text-red-400 hover:bg-red-500/10 transition-all"
          title="Kapat"
        >
          <X size={14} />
        </button>
      </div>
    </header>
  );
}
