import { useState, useEffect } from "react";
import { X, Minus, Calculator, Copy, Check, Eraser } from "lucide-react";
import { calculateKDV, formatTurkishNumber, parseAmount, numberToTurkishWords } from "../core/engine";

/** Ayri always-on-top BrowserWindow icinde calisan mini hesaplayici (?widget=1). */
export function WidgetWindow() {
  const [amount, setAmount] = useState("");
  const [rate, setRate] = useState<1 | 10 | 20>(20);
  const [extract, setExtract] = useState(false);
  const [copied, setCopied] = useState(false);
  const [theme, setTheme] = useState("dark");

  useEffect(() => {
    const apply = () => {
      let th: "dark" | "light" = "dark";
      try {
        const raw = localStorage.getItem("finansal-hesaplayici-v2");
        if (raw && JSON.parse(raw).theme === "light") th = "light";
      } catch { /* yok */ }
      setTheme(th);
      document.documentElement.setAttribute("data-theme", th);
    };
    apply();
    window.addEventListener("storage", apply);
    return () => window.removeEventListener("storage", apply);
  }, []);

  let result = "";
  let kdvCiktisi = "";
  let matrah = "";
  let words = "";
  try {
    const pv = amount ? parseAmount(amount) : null;
    if (pv) {
      const r = calculateKDV(pv, rate, extract);
      const v = extract ? r.netAmount : r.total;
      const k = extract ? pv.minus(r.netAmount) : r.total.minus(pv);
      result = formatTurkishNumber(v, 2);
      kdvCiktisi = formatTurkishNumber(k, 2);
      matrah = formatTurkishNumber(extract ? r.netAmount : pv, 2);
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
    <div className="h-screen w-screen flex flex-col overflow-hidden select-none relative" style={{ background: "var(--color-bg-base)", color: "var(--color-text-primary)" }} data-theme={theme}>
      <div aria-hidden className="ambient-glow" style={{ width: 260, height: 260, top: -130, right: -90, background: "radial-gradient(circle, rgba(47,123,255,.12), transparent 70%)" }} />

      <div className="drag-region flex items-center gap-2 px-3 h-10 shrink-0 relative" style={{ borderBottom: "1px solid var(--color-glass-border)", background: "rgba(148,175,255,.03)" }}>
        <div className="w-5 h-5 rounded-md flex items-center justify-center shrink-0" style={{ background: "linear-gradient(135deg, #2f7bff, #38bdf8)" }}>
          <Calculator size={11} className="text-white" />
        </div>
        <span className="text-[12px] font-bold truncate">Hızlı KDV</span>
        <span className="text-[9px] px-1.5 py-0.5 rounded-full font-bold shrink-0" style={{ background: "rgba(16,185,129,.14)", color: "var(--color-success)" }}>%{rate}</span>
        <div className="ml-auto flex items-center shrink-0">
          <button title="Temizle (C)" onClick={() => setAmount("")} disabled={!amount} className="no-drag p-1.5 rounded-lg btn-press" style={{ color: amount ? "var(--color-text-secondary)" : "var(--color-text-ghost)", opacity: amount ? 1 : 0.4 }}>
            <Eraser size={13} />
          </button>
          <button title="Alta al (gizle)" onClick={close} className="no-drag p-1.5 rounded-lg btn-press" style={{ color: "var(--color-text-secondary)" }}>
            <Minus size={13} strokeWidth={2.5} />
          </button>
          <button title="Kapat (Alt+Shift+K ile geri açılır)" onClick={close} className="no-drag p-1.5 rounded-lg btn-press win-close" style={{ color: "var(--color-text-ghost)" }}>
            <X size={13} />
          </button>
        </div>
      </div>

      <div className="flex-1 p-3 space-y-2.5 overflow-y-auto min-h-0 relative z-[1] mx-auto w-full max-w-[440px]">
        <div className="relative">
          <span className="absolute left-3 top-1/2 -translate-y-1/2 text-[13px] font-bold pointer-events-none" style={{ color: "var(--color-text-ghost)" }}>₺</span>
          <input value={amount} onChange={(e) => setAmount(e.target.value.replace(/[^0-9.,+\-*/()]/g, "").replace(/,/g, "."))}
            placeholder="Tutar — örn: 100+50" inputMode="decimal" autoFocus spellCheck={false}
            className="num-display w-full pl-7 pr-3 py-2.5 rounded-xl text-right font-bold focus-ring"
            style={{ background: "var(--color-glass)", border: amount ? "1px solid rgba(47,123,255,.45)" : "1px solid var(--color-glass-border)", color: "var(--color-text-primary)", fontSize: "clamp(18px, 5vw, 28px)" }} />
        </div>

        <div className="flex gap-1 rounded-xl p-1" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
          {([1, 10, 20] as const).map((r) => (
            <button key={r} onClick={() => setRate(r)} className="flex-1 py-1.5 rounded-lg text-[13px] font-bold btn-press focus-ring"
              style={rate === r
                ? { background: "var(--color-bg-elevated)", color: "var(--color-primary-light)", boxShadow: "0 2px 8px rgba(2,6,18,.25)" }
                : { background: "transparent", color: "var(--color-text-tertiary)" }}>
              %{r}
            </button>
          ))}
        </div>

        <div className="flex gap-1 rounded-xl p-1" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
          {([{ v: false, l: "+KDV ekle" }, { v: true, l: "−KDV çıkar" }] as const).map((o) => (
            <button key={o.l} onClick={() => setExtract(o.v)} className="flex-1 py-1.5 rounded-lg text-[11px] font-semibold btn-press"
              style={extract === o.v
                ? { background: "var(--color-bg-elevated)", color: "var(--color-primary-light)", boxShadow: "0 2px 8px rgba(2,6,18,.25)" }
                : { background: "transparent", color: "var(--color-text-tertiary)" }}>
              {o.l}
            </button>
          ))}
        </div>

        {result ? (
          <>
            <div className="relative rounded-2xl px-4 pt-4 pb-3.5 overflow-hidden text-center" style={{ background: "linear-gradient(160deg, var(--color-glass-hover), var(--color-glass) 45%)", border: "1px solid var(--color-glass-border)" }}>
              <div aria-hidden className="ambient-glow" style={{ width: 150, height: 150, top: -80, right: -30, background: "radial-gradient(circle, rgba(56,189,248,.14), transparent 70%)" }} />
              <span className="relative text-[9px] uppercase tracking-[0.1em] block" style={{ color: "var(--color-text-ghost)" }}>
                {extract ? "Net tutar" : "KDV dahil toplam"} • tıkla / Enter ile kopyala
              </span>
              <button onClick={copy} className="relative num-display w-full font-bold gradient-text-result mt-1 flex items-center justify-center gap-1.5 focus-ring btn-press"
                style={{ fontSize: "clamp(24px, 9vw, 44px)" }}>
                {result} ₺ {copied ? <Check size={14} /> : <Copy size={14} />}
              </button>
              {words && <span className="relative block text-[10px] italic px-2 leading-snug mt-1" style={{ color: "var(--color-text-tertiary)" }}>{words}</span>}
            </div>

            <div className="grid grid-cols-2 gap-2">
              <div className="rounded-xl px-3 py-2" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
                <span className="text-[9px] uppercase tracking-wider block" style={{ color: "var(--color-text-ghost)" }}>{extract ? "Net (matrah)" : "Matrah"}</span>
                <span className="num-display text-[13px] font-bold block mt-0.5" style={{ color: "var(--color-text-secondary)" }}>{matrah} ₺</span>
              </div>
              <div className="rounded-xl px-3 py-2" style={{ background: "rgba(47,123,255,.10)", border: "1px solid rgba(47,123,255,.28)" }}>
                <span className="text-[9px] uppercase tracking-wider block" style={{ color: "var(--color-primary-light)", opacity: 0.8 }}>KDV (%{rate})</span>
                <span className="num-display text-[13px] font-bold block mt-0.5" style={{ color: "var(--color-primary-light)" }}>{kdvCiktisi} ₺</span>
              </div>
            </div>
          </>
        ) : (
          <div className="rounded-2xl p-4 text-center" style={{ border: "1px dashed var(--color-glass-border)" }}>
            <p className="text-[12px] font-semibold" style={{ color: "var(--color-text-secondary)" }}>Yazdıkça sonuç gelir</p>
            <p className="text-[10px] mt-1" style={{ color: "var(--color-text-ghost)" }}>Aritmetik de olur: 100+50, 250*4</p>
          </div>
        )}

        <p className="text-center text-[10px] leading-relaxed pb-1" style={{ color: "var(--color-text-ghost)" }}>Enter: kopyala • Esc: kapat • Alt+Shift+K: göster/gizle</p>
      </div>
    </div>
  );
}
