import { useState, useRef, useCallback, useEffect } from "react";
import { useStore } from "../store/useStore";
import { X, Calculator, Copy, Check } from "lucide-react";
import { calculateKDV, formatTurkishNumber, parseAmount } from "../core/engine";

export function Widget() {
  const showWidget = useStore((s) => s.showWidget);
  const toggleWidget = useStore((s) => s.toggleWidget);
  const notify = useStore((s) => s.notify);
  const [amount, setAmount] = useState("");
  const [rate, setRate] = useState<1 | 10 | 20>(20);
  const [extract, setExtract] = useState(false);
  const [copied, setCopied] = useState(false);
  const [pos, setPos] = useState(() => ({ x: Math.max(0, window.innerWidth - 300), y: 80 }));
  const drag = useRef<{ dx: number; dy: number; active: boolean }>({ dx: 0, dy: 0, active: false });

  useEffect(() => {
    try {
      const raw = localStorage.getItem("fh-widget-pos");
      if (raw) {
        const p = JSON.parse(raw);
        if (typeof p.x === "number" && typeof p.y === "number") setPos({ x: Math.min(Math.max(0, p.x), window.innerWidth - 280), y: Math.min(Math.max(0, p.y), window.innerHeight - 200) });
      }
    } catch { /* yok */ }
  }, []);

  useEffect(() => {
    try { localStorage.setItem("fh-widget-pos", JSON.stringify(pos)); } catch { /* yok */ }
  }, [pos]);

  const onHeaderDown = useCallback((e: React.PointerEvent<HTMLDivElement>) => {
    drag.current = { dx: e.clientX - pos.x, dy: e.clientY - pos.y, active: true };
    e.currentTarget.setPointerCapture(e.pointerId);
  }, [pos]);

  const onHeaderMove = useCallback((e: React.PointerEvent<HTMLDivElement>) => {
    if (!drag.current.active) return;
    setPos({ x: e.clientX - drag.current.dx, y: e.clientY - drag.current.dy });
  }, []);

  const onHeaderUp = useCallback(() => { drag.current.active = false; }, []);

  let result = "";
  try {
    const pv = amount ? parseAmount(amount) : null;
    if (pv) {
      const r = calculateKDV(pv, rate, extract);
      result = formatTurkishNumber(extract ? r.netAmount : r.total, 2);
    }
  } catch { result = ""; }

  if (!showWidget) return null;

  return (
    <div className="fixed z-[9999] select-none anim-scale-in" style={{ left: pos.x, top: pos.y, width: 264 }}>
      <div className="rounded-2xl overflow-hidden shadow-2xl" style={{ background: "var(--color-bg-surface)", border: "1px solid var(--color-glass-border)" }}>
        <div className="flex items-center gap-2 px-3 py-2 cursor-grab active:cursor-grabbing"
          style={{ borderBottom: "1px solid var(--color-glass-border)", touchAction: "none" }}
          onPointerDown={onHeaderDown} onPointerMove={onHeaderMove} onPointerUp={onHeaderUp}>
          <div className="w-5 h-5 rounded-md flex items-center justify-center shrink-0" style={{ background: "linear-gradient(135deg, #2f7bff, #7aa8ff)" }}>
            <Calculator size={11} className="text-white" />
          </div>
          <span className="text-[11px] font-bold" style={{ color: "var(--color-text-secondary)" }}>Hızlı KDV</span>
          <button onClick={toggleWidget} className="no-drag ml-auto p-1 rounded-md btn-press" style={{ color: "var(--color-text-ghost)" }} title="Kapat">
            <X size={12} />
          </button>
        </div>

        <div className="p-3 space-y-2">
          <input value={amount} onChange={(e) => setAmount(e.target.value.replace(/[^0-9.,]/g, "").replace(/,/g, "."))}
            placeholder="₺ Tutar..." inputMode="decimal" autoFocus
            className="num-display w-full px-3 py-2 rounded-xl text-right font-bold text-[16px] focus-ring"
            style={{ background: "var(--color-glass)", border: amount ? "1px solid rgba(47,123,255,.45)" : "1px solid var(--color-glass-border)", color: "var(--color-text-primary)", touchAction: "auto" }} />
          <div className="flex gap-1 rounded-xl p-1" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
            {([1, 10, 20] as const).map((r) => (
              <button key={r} onClick={() => setRate(r)} className="flex-1 py-1.5 rounded-lg text-[11px] font-bold btn-press"
                style={rate === r
                  ? { background: "var(--color-bg-elevated)", color: "var(--color-primary-light)", boxShadow: "0 2px 8px rgba(2,6,18,.25)" }
                  : { background: "transparent", color: "var(--color-text-tertiary)" }}>
                %{r}
              </button>
            ))}
            <button onClick={() => setExtract(!extract)} title="Yön değiştir"
              className="flex-1 py-1.5 rounded-lg text-[11px] font-bold btn-press"
              style={extract
                ? { background: "var(--color-bg-elevated)", color: "var(--color-accent)", boxShadow: "0 2px 8px rgba(2,6,18,.25)" }
                : { background: "transparent", color: "var(--color-text-tertiary)" }}>
              {extract ? "−KDV çıkar" : "+KDV ekle"}
            </button>
          </div>
          {result && (
            <div className="rounded-xl px-3 py-2 text-center" style={{ background: "linear-gradient(160deg, var(--color-glass-hover), var(--color-glass) 45%)", border: "1px solid var(--color-glass-border)" }}>
              <button onClick={async () => { try { await navigator.clipboard.writeText(result); } catch {} setCopied(true); notify("Kopyalandı", "success"); setTimeout(() => setCopied(false), 1200); }}
                className="w-full flex items-center justify-center gap-1 btn-press focus-ring">
                <span className="num-display text-[18px] font-bold gradient-text-result">{result} ₺</span>
                {copied ? <Check size={12} /> : <Copy size={12} />}
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
