import { useState, useEffect } from "react";
import { X, Minus, Calculator, Copy, Check } from "lucide-react";
import { calculateKDV, formatTurkishNumber, parseAmount, numberToTurkishWords } from "../core/engine";

/** Ayri always-on-top BrowserWindow icinde calisan mini hesaplayici (?widget=1). */
export function WidgetWindow() {
  const [amount, setAmount] = useState("");
  const [rate, setRate] = useState<1 | 10 | 20>(20);
  const [extract, setExtract] = useState(false);
  const [copied, setCopied] = useState(false);
  const [theme, setTheme] = useState("dark");

  useEffect(() => {
    try {
      const t = localStorage.getItem("fh-theme");
      const th = t === "light" ? "light" : "dark";
      setTheme(th);
      document.documentElement.setAttribute("data-theme", th);
    } catch { /* yok */ }
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") close();
      if (e.key === "Enter" && result) copy();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  });

  let result = "";
  let words = "";
  try {
    const pv = amount ? parseAmount(amount) : null;
    if (pv) {
      const r = calculateKDV(pv, rate, extract);
      const v = extract ? r.netAmount : r.total;
      result = formatTurkishNumber(v, 2);
      words = numberToTurkishWords(v);
    }
  } catch { result = ""; }

  const close = () => {
    if (window.electronAPI?.widgetClose) window.electronAPI.widgetClose();
  };

  const copy = async () => {
    if (!result) return;
    try {
      await navigator.clipboard.writeText(result);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = result;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      ta.remove();
    }
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  return (
    <div className="h-screen w-screen flex flex-col overflow-hidden select-none" style={{ background: "var(--color-bg-surface)", color: "var(--color-text-primary)" }} data-theme={theme}>
      <div className="drag-region flex items-center gap-2 px-3 h-10 shrink-0" style={{ borderBottom: "1px solid var(--color-glass-border)" }}>
        <div className="w-5 h-5 rounded-md flex items-center justify-center shrink-0" style={{ background: "linear-gradient(135deg, #2f7bff, #7aa8ff)" }}>
          <Calculator size={11} className="text-white" />
        </div>
        <span className="text-[12px] font-bold">Hızlı KDV</span>
        <span className="text-[9px] px-1.5 py-0.5 rounded font-bold" style={{ background: "rgba(22,185,129,.14)", color: "var(--color-success)" }}>CANLI</span>
        <div className="ml-auto flex items-center">
          <button title="Alta al (gizle)" onClick={close} className="no-drag p-1.5 rounded-lg btn-press" style={{ color: "var(--color-text-secondary)" }}>
            <Minus size={13} strokeWidth={2.5} />
          </button>
          <button title="Kapat (Alt+Shift+K ile geri açılır)" onClick={close} className="no-drag p-1.5 rounded-lg btn-press" style={{ color: "var(--color-text-ghost)" }}
            onMouseEnter={(e) => { e.currentTarget.style.background = "#e5484d"; e.currentTarget.style.color = "#fff"; }}
            onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = "var(--color-text-ghost)"; }}>
            <X size={13} />
          </button>
        </div>
      </div>

      <div className="flex-1 p-3 space-y-2.5 overflow-hidden">
        <input value={amount} onChange={(e) => setAmount(e.target.value.replace(/[^0-9.,+\-*/()]/g, "").replace(/,/g, "."))}
          placeholder="Tutar veya işlem (100+50)..." inputMode="decimal" autoFocus
          className="num-display w-full px-3 py-2.5 rounded-xl text-right font-bold text-[20px] focus-ring"
          style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-primary)" }} />

        <div className="flex gap-1.5">
          {([1, 10, 20] as const).map((r) => (
            <button key={r} onClick={() => setRate(r)} className="flex-1 py-2 rounded-xl text-[13px] font-bold btn-press focus-ring"
              style={rate === r ? { background: "var(--color-primary)", color: "#fff" } : { background: "var(--color-glass)", color: "var(--color-text-secondary)", border: "1px solid var(--color-glass-border)" }}>
              %{r}
            </button>
          ))}
        </div>

        <div className="flex gap-1.5">
          {([{ v: false, l: "+KDV (hariçten)" }, { v: true, l: "−KDV (dahilden)" }] as const).map((o) => (
            <button key={o.l} onClick={() => setExtract(o.v)} className="flex-1 py-1.5 rounded-lg text-[11px] font-semibold btn-press"
              style={extract === o.v
                ? { background: "rgba(47,123,255,.16)", color: "var(--color-primary-light)", border: "1px solid var(--color-primary)" }
                : { background: "transparent", color: "var(--color-text-ghost)", border: "1px solid var(--color-glass-border)" }}>
              {o.l}
            </button>
          ))}
        </div>

        {result ? (
          <button onClick={copy} className="w-full py-2.5 rounded-xl btn-press focus-ring" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
            <span className="text-[9px] uppercase tracking-widest block" style={{ color: "var(--color-text-ghost)" }}>
              {extract ? "Net tutar" : "KDV dahil toplam"} • kopyalamak için tıkla / Enter
            </span>
            <span className="num-display text-[22px] font-bold gradient-text-result flex items-center justify-center gap-1.5">
              {result} ₺ {copied ? <Check size={14} /> : <Copy size={14} />}
            </span>
            {words && <span className="block text-[10px] italic px-2 leading-snug" style={{ color: "var(--color-text-tertiary)" }}>{words}</span>}
          </button>
        ) : (
          <p className="text-center text-[12px] py-4" style={{ color: "var(--color-text-ghost)" }}>Yazdıkça sonuç gelir — buton yok.</p>
        )}

        <p className="text-center text-[9px]" style={{ color: "var(--color-text-ghost)" }}>Alt+Shift+K: göster/gizle • Esc: kapat</p>
      </div>
    </div>
  );
}
