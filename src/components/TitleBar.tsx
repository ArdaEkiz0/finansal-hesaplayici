import { useStore } from "../store/useStore";
import { Minus, Square, Copy, X, Settings, HelpCircle, Calculator, ReceiptText, BookOpen } from "lucide-react";
import { useState, useEffect } from "react";

export function TitleBar() {
  const toggleWidget = useStore((s) => s.toggleWidget);
  const toggleHelp = useStore((s) => s.toggleHelp);
  const toggleSettings = useStore((s) => s.toggleSettings);
  const toggleInvoice = useStore((s) => s.toggleInvoice);
  const toggleRates = useStore((s) => s.toggleRates);
  const result = useStore((s) => s.result);
  const [isMax, setIsMax] = useState(false);
  const [noBridge, setNoBridge] = useState(false);

  useEffect(() => {
    const api = (window as unknown as { electronAPI?: { onMaximizeChange?: (cb: (_e: unknown, m: boolean) => void) => void; isMaximized?: () => Promise<boolean> } }).electronAPI;
    if (!api) {
      setNoBridge(true);
      return;
    }
    api.onMaximizeChange?.((_e, m) => setIsMax(m));
    api.isMaximized?.().then(setIsMax).catch(() => {});
  }, []);

  const api = (window as unknown as { electronAPI?: { minimize?: () => void; maximize?: () => void; close?: () => void } }).electronAPI;

  const toolBtn =
    "no-drag flex items-center gap-1.5 px-2.5 h-9 rounded-xl btn-press focus-ring transition-colors text-[12px] font-semibold";
  const winBtn =
    "no-drag flex items-center justify-center w-12 h-9 btn-press focus-ring transition-colors";

  return (
    <div
      className="drag-region flex items-center h-12 px-3 shrink-0 gap-2 select-none"
      style={{ background: "var(--color-bg-base)", borderBottom: "1px solid var(--color-glass-border)" }}
      onDoubleClick={() => api?.maximize?.()}
    >
      <div className="flex items-center gap-2.5 min-w-0">
        <div className="w-8 h-8 rounded-xl flex items-center justify-center shrink-0" style={{ background: "linear-gradient(135deg, #2f7bff, #7aa8ff)" }}>
          <Calculator size={16} className="text-white" />
        </div>
        <div className="leading-tight min-w-0">
          <div className="text-[14px] font-bold truncate">Finansal Hesaplayıcı</div>
          <div className="text-[10px] truncate" style={{ color: "var(--color-text-ghost)" }}>KDV • Stopaj • Tevkifat • Kâr</div>
        </div>
      </div>

      <div className="ml-auto flex items-center gap-1.5">
        <button title="Hızlı hesap (Alt+Shift+K)" onClick={toggleWidget} className={toolBtn}
          style={{ color: "var(--color-text-secondary)", background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
          <Calculator size={15} /> <span className="hidden md:inline">Hızlı KDV</span>
        </button>
        <button title="Fatura önizleme" onClick={toggleInvoice} disabled={!result} className={toolBtn}
          style={{ color: "var(--color-text-secondary)", background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", opacity: result ? 1 : 0.4 }}>
          <ReceiptText size={15} /> <span className="hidden md:inline">Fatura</span>
        </button>
        <button title="2026 pratik oranlar" onClick={toggleRates} className={toolBtn}
          style={{ color: "var(--color-text-secondary)", background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
          <BookOpen size={15} /> <span className="hidden md:inline">Oranlar</span>
        </button>
        <button title="Yardım (F1)" onClick={toggleHelp} className={toolBtn}
          style={{ color: "var(--color-text-secondary)", background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
          <HelpCircle size={15} />
        </button>
        <button title="Ayarlar (F2)" onClick={toggleSettings} className={toolBtn}
          style={{ color: "var(--color-text-secondary)", background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
          <Settings size={15} />
        </button>

        <div className="flex items-center rounded-xl overflow-hidden ml-1" style={{ border: "1px solid var(--color-glass-border)", background: "var(--color-glass)" }} title={noBridge ? "Masaüstü köprüsü yok" : ""}>
          <button title="Alta al (küçült)" aria-label="Alta al"
            onClick={() => api?.minimize?.()}
            className={`${winBtn}`} style={{ color: "var(--color-text-primary)" }}
            onMouseEnter={(e) => (e.currentTarget.style.background = "var(--color-glass-hover)")}
            onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}>
            <Minus size={17} strokeWidth={2.5} />
          </button>
          <button title={isMax ? "Pencereden çık" : "Tam ekran yap"} aria-label="Büyüt"
            onClick={() => api?.maximize?.()}
            className={`${winBtn}`} style={{ color: "var(--color-text-primary)", borderLeft: "1px solid var(--color-glass-border)", borderRight: "1px solid var(--color-glass-border)" }}
            onMouseEnter={(e) => (e.currentTarget.style.background = "var(--color-glass-hover)")}
            onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}>
            {isMax ? <Copy size={14} strokeWidth={2.5} /> : <Square size={14} strokeWidth={2.5} />}
          </button>
          <button title="Kapat" aria-label="Kapat"
            onClick={() => api?.close?.()}
            className={`${winBtn}`} style={{ color: "var(--color-text-primary)" }}
            onMouseEnter={(e) => { e.currentTarget.style.background = "#e5484d"; e.currentTarget.style.color = "#fff"; }}
            onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = "var(--color-text-primary)"; }}>
            <X size={17} strokeWidth={2.5} />
          </button>
        </div>
      </div>
    </div>
  );
}
