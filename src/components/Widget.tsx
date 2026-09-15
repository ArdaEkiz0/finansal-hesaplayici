import { useState, useEffect, useRef } from "react";
import { useStore } from "../store/useStore";
import { X, Minus, Calculator } from "lucide-react";
import { calculateKDV, formatTurkishNumber, toDecimal } from "../core/engine";

export function Widget() {
  const showWidget = useStore((s) => s.showWidget);
  const toggleWidget = useStore((s) => s.toggleWidget);
  const [amount, setAmount] = useState("");
  const [rate, setRate] = useState<1 | 10 | 20>(20);
  const [result, setResult] = useState("");
  const [pos, setPos] = useState({ x: 100, y: 100 });
  const [dragging, setDragging] = useState(false);
  const dragRef = useRef({ startX: 0, startY: 0, startPosX: 0, startPosY: 0 });

  useEffect(() => {
    if (!amount) { setResult(""); return; }
    try {
      const r = calculateKDV(toDecimal(amount), rate, false);
      setResult(formatTurkishNumber(r.total, 2));
    } catch { setResult(""); }
  }, [amount, rate]);

  useEffect(() => {
    const handleMove = (e: MouseEvent) => {
      if (!dragging) return;
      setPos({
        x: dragRef.current.startPosX + (e.clientX - dragRef.current.startX),
        y: dragRef.current.startPosY + (e.clientY - dragRef.current.startY),
      });
    };
    const handleUp = () => setDragging(false);
    if (dragging) {
      window.addEventListener("mousemove", handleMove);
      window.addEventListener("mouseup", handleUp);
    }
    return () => {
      window.removeEventListener("mousemove", handleMove);
      window.removeEventListener("mouseup", handleUp);
    };
  }, [dragging]);

  if (!showWidget) return null;

  return (
    <div className="fixed z-[9999]" style={{ left: pos.x, top: pos.y }}>
      <div className="w-64 glass-strong rounded-2xl shadow-2xl border border-glass-border overflow-hidden anim-scale-in">
        {/* Header */}
        <div
          className="flex items-center justify-between px-3 py-2 border-b border-glass-border cursor-move bg-glass"
          onMouseDown={(e) => {
            setDragging(true);
            dragRef.current = { startX: e.clientX, startY: e.clientY, startPosX: pos.x, startPosY: pos.y };
          }}
        >
          <div className="flex items-center gap-1.5">
            <Calculator size={12} className="text-brand-400" />
            <span className="text-[11px] font-bold text-text-primary">Hesaplaci</span>
          </div>
          <button onClick={toggleWidget} className="p-1 rounded hover:bg-glass-hover text-text-ghost">
            <X size={12} />
          </button>
        </div>

        {/* Input */}
        <div className="p-3 space-y-2">
          <input type="text" inputMode="decimal" placeholder="Tutar" value={amount}
            onChange={(e) => setAmount(e.target.value.replace(",", "."))}
            className="w-full px-3 py-2 rounded-xl bg-glass border border-glass-border text-sm font-mono text-text-primary placeholder:text-text-ghost focus:border-brand-500 focus:outline-none" autoFocus />

          {/* Rate selector */}
          <div className="flex gap-1">
            {([1, 10, 20] as const).map((r) => (
              <button key={r} onClick={() => setRate(r)}
                className={`flex-1 py-1.5 rounded-lg text-[10px] font-bold transition-all ${
                  rate === r ? "bg-brand-500 text-white" : "glass text-text-secondary hover:text-text-primary"
                }`}>
                %{r} KDV
              </button>
            ))}
          </div>

          {/* Result */}
          {result && (
            <div className="text-center py-2 rounded-xl bg-glass border border-glass-border">
              <p className="text-lg font-bold font-mono text-brand-400">{result} ₺</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
