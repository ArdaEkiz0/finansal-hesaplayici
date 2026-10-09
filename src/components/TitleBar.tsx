import { useStore } from "../store/useStore";
import { Minus, Square, X, Settings, HelpCircle, Calculator, ReceiptText } from "lucide-react";
import { useState, useEffect } from "react";

export function TitleBar() {
  const toggleWidget = useStore((s) => s.toggleWidget);
  const toggleHelp = useStore((s) => s.toggleHelp);
  const toggleSettings = useStore((s) => s.toggleSettings);
  const toggleInvoice = useStore((s) => s.toggleInvoice);
  const result = useStore((s) => s.result);
  const [isMax, setIsMax] = useState(false);

  useEffect(() => {
    const api = (window as unknown as { electronAPI?: { onMaximizeChange?: (cb: (_e: unknown, m: boolean) => void) => void; isMaximized?: () => Promise<boolean> } }).electronAPI;
    api?.onMaximizeChange?.((_e, m) => setIsMax(m));
    api?.isMaximized?.().then(setIsMax).catch(() => {});
  }, []);

  const api = (window as unknown as { electronAPI?: { minimize?: () => void; maximize?: () => void; close?: () => void } }).electronAPI;

  const iconBtn = "no-drag p-2 rounded-lg btn-press focus-ring transition-colors";
  const iconColor = "var(--color-text-tertiary)";

  return (
    <div className="drag-region flex items-center h-11 px-3 shrink-0 gap-2" style={{ background: "var(--color-bg-base)", borderBottom: "1px solid var(--color-glass-border)" }}>
      <div className="flex items-center gap-2.5 min-w-0">
        <div className="w-7 h-7 rounded-[10px] flex items-center justify-center shrink-0" style={{ background: "linear-gradient(135deg, #2f7bff, #7aa8ff)" }}>
          <Calculator size={14} className="text-white" />
        </div>
        <div className="leading-tight min-w-0">
          <div className="text-[13px] font-bold truncate">Finansal Hesaplayıcı</div>
          <div className="text-[10px] truncate" style={{ color: "var(--color-text-ghost)" }}>KDV • Stopaj • Tevkifat • Kâr</div>
        </div>
      </div>

      <div className="ml-auto flex items-center gap-1">
        <button title="Hızlı hesap" onClick={toggleWidget} className={iconBtn} style={{ color: iconColor }}>
          <Calculator size={15} />
        </button>
        <button title="Fatura önizleme" onClick={toggleInvoice} disabled={!result} className={iconBtn} style={{ color: iconColor, opacity: result ? 1 : 0.35 }}>
          <ReceiptText size={15} />
        </button>
        <button title="Yardım" onClick={toggleHelp} className={iconBtn} style={{ color: iconColor }}>
          <HelpCircle size={15} />
        </button>
        <button title="Ayarlar" onClick={toggleSettings} className={iconBtn} style={{ color: iconColor }}>
          <Settings size={15} />
        </button>
        <div className="w-px h-4 mx-1" style={{ background: "var(--color-glass-border)" }} />
        <button title="Küçült" onClick={() => api?.minimize?.()} className={iconBtn} style={{ color: iconColor }}>
          <Minus size={13} />
        </button>
        <button title={isMax ? "Pencereden çık" : "Büyüt"} onClick={() => api?.maximize?.()} className={iconBtn} style={{ color: iconColor }}>
          <Square size={12} />
        </button>
        <button title="Kapat" onClick={() => api?.close?.()} className={`${iconBtn} hover:!text-white`} onMouseEnter={(e) => (e.currentTarget.style.background = "#e5484d")} onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}>
          <X size={13} />
        </button>
      </div>
    </div>
  );
}
