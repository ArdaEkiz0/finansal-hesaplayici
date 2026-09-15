import { useState, useRef, useCallback } from "react";
import { useStore } from "../store/useStore";
import { X, Calculator } from "lucide-react";
import { calculateKDV, formatTurkishNumber, toDecimal } from "../core/engine";

export function Widget() {
  const showWidget = useStore((s) => s.showWidget);
  const toggleWidget = useStore((s) => s.toggleWidget);
  const [amount, setAmount] = useState("");
  const [rate, setRate] = useState<1 | 10 | 20>(20);
  const [pos, setPos] = useState({ x: 0, y: 80 });
  const [dragging, setDragging] = useState(false);
  const offset = useRef({ x: 0, y: 0 });

  const onPointerDown = useCallback((e: React.PointerEvent) => {
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
    offset.current = { x: e.clientX - pos.x, y: e.clientY - pos.y };
    setDragging(true);
  }, [pos]);

  const onPointerMove = useCallback((e: React.PointerEvent) => {
    if (!dragging) return;
    setPos({ x: e.clientX - offset.current.x, y: e.clientY - offset.current.y });
  }, [dragging]);

  const onPointerUp = useCallback(() => setDragging(false), []);

  let result = "";
  try {
    if (amount) {
      const r = calculateKDV(toDecimal(amount), rate, false);
      result = formatTurkishNumber(r.total, 2);
    }
  } catch {}

  if (!showWidget) return null;

  return (
    <div className="fixed z-[9999] select-none" style={{ left: pos.x || window.innerWidth - 300, top: pos.y, touchAction: "none" }}
      onPointerMove={onPointerMove} onPointerUp={onPointerUp}>
      <div className="w-64 rounded-xl overflow-hidden shadow-2xl" style={{ background: "var(--color-bg-base)", border: "1px solid var(--color-glass-border)" }}>
        {/* Header */}
        <div className="flex items-center justify-between px-3 py-2 cursor-grab active:cursor-grabbing" style={{ borderBottom: "1px solid var(--color-glass-border)" }}
          onPointerDown={onPointerDown}>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded flex items-center justify-center" style={{ background: "linear-gradient(135deg, #3b82f6, #8b5cf6)" }}>
              <Calculator size={9} className="text-white" />
            </div>
            <span className="text-[10px] font-bold" style={{ color: "var(--color-text-secondary)" }}>Hızlı Hesap</span>
          </div>
          <button onClick={toggleWidget} className="p-0.5 rounded transition-colors" style={{ color: "var(--color-text-ghost)" }}
            onMouseEnter={(e) => e.currentTarget.style.color = "var(--color-text-secondary)"}
            onMouseLeave={(e) => e.currentTarget.style.color = "var(--color-text-ghost)"}>
            <X size={11} />
          </button>
        </div>

        {/* Body */}
        <div className="p-3 space-y-2">
          <input type="text" inputMode="decimal" placeholder="Tutar..." value={amount}
            onChange={(e) => setAmount(e.target.value.replace(",", "."))}
            className="w-full px-3 py-2 rounded-lg text-sm font-mono focus:outline-none focus:ring-1 transition-all"
            style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-primary)" }}
            onFocus={(e) => e.currentTarget.style.borderColor = "#3b82f640"}
            onBlur={(e) => e.currentTarget.style.borderColor = "var(--color-glass-border)"}
          />

          <div className="flex gap-1">
            {([1, 10, 20] as const).map((r) => (
              <button key={r} onClick={() => setRate(r)}
                className="flex-1 py-1.5 rounded-lg text-[10px] font-bold transition-all"
                style={{
                  background: rate === r ? "#3b82f6" : "var(--color-glass)",
                  color: rate === r ? "#fff" : "var(--color-text-ghost)",
                  border: `1px solid ${rate === r ? "#3b82f6" : "var(--color-glass-border)"}`,
                }}>
                %{r}
              </button>
            ))}
          </div>

          {result && (
            <div className="text-center py-2 rounded-lg" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
              <p className="text-[9px] uppercase tracking-widest mb-0.5" style={{ color: "var(--color-text-ghost)" }}>Toplam</p>
              <p className="num-display text-base font-bold gradient-text-result">{result} ₺</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
