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

  const handleMinimize = (e: React.MouseEvent) => {
    e.stopPropagation();
    window.electronAPI?.minimize();
  };

  const handleMaximize = (e: React.MouseEvent) => {
    e.stopPropagation();
    window.electronAPI?.maximize();
  };

  const handleClose = (e: React.MouseEvent) => {
    e.stopPropagation();
    window.electronAPI?.close();
  };

  return (
    <header className="h-10 glass-strong border-b border-glass-border flex items-center justify-between drag-region" style={{ WebkitAppRegion: "drag" } as any}>
      {/* Left - App info */}
      <div className="flex items-center gap-2 pl-4 no-drag" style={{ WebkitAppRegion: "no-drag" } as any}>
        <div className="w-6 h-6 rounded-lg bg-gradient-to-br from-blue-500 via-violet-500 to-emerald-500 flex items-center justify-center">
          <Calculator size={12} className="text-white" />
        </div>
        <span className="text-xs font-bold text-text-primary gradient-text">
          Finansal Hesaplayıcı
        </span>
        <span className="text-[9px] text-text-ghost ml-1">by Arda M. Ekiz</span>
      </div>

      {/* Center - Tool buttons */}
      <div className="flex items-center gap-1 no-drag" style={{ WebkitAppRegion: "no-drag" } as any}>
        <button
          onClick={toggleWidget}
          className={`p-1.5 rounded-lg transition-all duration-200 active:scale-90 ${showWidget ? "text-brand-400 bg-brand-500/10" : "text-text-ghost hover:text-brand-400 hover:bg-brand-500/10"}`}
          title="Widget"
        >
          <AppWindow size={14} />
        </button>
        <button
          onClick={toggleHelp}
          className="p-1.5 rounded-lg text-text-ghost hover:text-violet-400 hover:bg-violet-500/10 transition-all duration-200 active:scale-90"
          title="Yardım (F1)"
        >
          <HelpCircle size={14} />
        </button>
        <button
          onClick={toggleSettings}
          className="p-1.5 rounded-lg text-text-ghost hover:text-brand-400 hover:bg-brand-500/10 transition-all duration-200 active:scale-90"
          title="Ayarlar (F2)"
        >
          <Settings size={14} />
        </button>
      </div>

      {/* Right - Window controls */}
      <div className="flex items-center h-full no-drag" style={{ WebkitAppRegion: "no-drag" } as any}>
        <button
          onClick={handleMinimize}
          className="w-11 h-10 flex items-center justify-center text-text-ghost hover:text-white hover:bg-white/[0.08] transition-all duration-150 active:scale-90 active:bg-white/[0.12]"
          title="Kucult"
          style={{ WebkitAppRegion: "no-drag" } as any}
        >
          <Minus size={14} strokeWidth={2.5} />
        </button>
        <button
          onClick={handleMaximize}
          className="w-11 h-10 flex items-center justify-center text-text-ghost hover:text-white hover:bg-white/[0.08] transition-all duration-150 active:scale-90 active:bg-white/[0.12]"
          title={maximized ? "Pencere Boyutlandir" : "Tam Ekran"}
          style={{ WebkitAppRegion: "no-drag" } as any}
        >
          {maximized ? <Maximize2 size={13} strokeWidth={2.5} /> : <Square size={12} strokeWidth={2.5} />}
        </button>
        <button
          onClick={handleClose}
          className="w-11 h-10 flex items-center justify-center text-text-ghost hover:text-white hover:bg-red-500 transition-all duration-150 active:scale-90"
          title="Kapat"
          style={{ WebkitAppRegion: "no-drag" } as any}
        >
          <X size={14} strokeWidth={2.5} />
        </button>
      </div>
    </header>
  );
}
