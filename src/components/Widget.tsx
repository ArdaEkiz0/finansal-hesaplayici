import { useState, useRef, useCallback } from "react";
import { useStore } from "../store/useStore";
import { X, Calculator, GripVertical } from "lucide-react";
import { calculateKDV, formatTurkishNumber, toDecimal } from "../core/engine";

export function Widget() {
  const showWidget = useStore((s) => s.showWidget);
  const toggleWidget = useStore((s) => s.toggleWidget);
  const [amount, setAmount] = useState("");
  const [rate, setRate] = useState<1 | 10 | 20>(20);
  const [pos, setPos] = useState({ x: window.innerWidth - 300, y: 60 });
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
    <div
      className="fixed z-[9999] select-none"
      style={{ left: pos.x, top: pos.y, touchAction: "none" }}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
    >
      <div className="w-72 rounded-2xl shadow-2xl overflow-hidden" style={{ background: "rgba(15,23,42,0.95)", border: "1px solid rgba(255,255,255,0.08)", backdropFilter: "blur(20px)" }}>
        {/* Header - draggable */}
        <div
          className="flex items-center justify-between px-3 py-2.5 cursor-grab active:cursor-grabbing"
          style={{ borderBottom: "1px solid rgba(255,255,255,0.06)" }}
          onPointerDown={onPointerDown}
        >
          <div className="flex items-center gap-2">
            <div className="w-5 h-5 rounded-md flex items-center justify-center" style={{ background: "linear-gradient(135deg, #3b82f6, #8b5cf6)" }}>
              <Calculator size={11} className="text-white" />
            </div>
            <span className="text-[11px] font-bold text-white/90">Hızlı Hesap</span>
          </div>
          <button onClick={toggleWidget} className="p-1 rounded-md hover:bg-white/10 text-white/30 hover:text-white/70 transition-colors">
            <X size={12} />
          </button>
        </div>

        {/* Body */}
        <div className="p-3 space-y-2.5">
          <input
            type="text"
            inputMode="decimal"
            placeholder="Tutar girin..."
            value={amount}
            onChange={(e) => setAmount(e.target.value.replace(",", "."))}
            className="w-full px-3 py-2.5 rounded-xl text-sm font-mono text-white placeholder:text-white/20 focus:outline-none focus:ring-1 focus:ring-blue-500/50"
            style={{ background: "rgba(255,255,255,0.06)", border: "1px solid rgba(255,255,255,0.08)" }}
          />

          <div className="flex gap-1">
            {([1, 10, 20] as const).map((r) => (
              <button key={r} onClick={() => setRate(r)}
                className="flex-1 py-2 rounded-xl text-[11px] font-bold transition-all duration-150"
                style={{
                  background: rate === r ? "linear-gradient(135deg, #3b82f6, #6366f1)" : "rgba(255,255,255,0.04)",
                  color: rate === r ? "#fff" : "rgba(255,255,255,0.4)",
                  border: rate === r ? "none" : "1px solid rgba(255,255,255,0.06)",
                }}>
                %{r} KDV
              </button>
            ))}
          </div>

          {result && (
            <div className="text-center py-3 rounded-xl" style={{ background: "rgba(255,255,255,0.04)", border: "1px solid rgba(255,255,255,0.06)" }}>
              <p className="text-[10px] text-white/30 uppercase tracking-widest mb-1">Toplam</p>
              <p className="text-lg font-black font-mono" style={{ background: "linear-gradient(135deg, #60a5fa, #a78bfa)", WebkitBackgroundClip: "text", WebkitTextFillColor: "transparent" }}>
                {result} ₺
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
