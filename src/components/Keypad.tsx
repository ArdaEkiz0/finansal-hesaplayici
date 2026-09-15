import { useStore } from "../store/useStore";
import { Delete, CornerDownLeft, Plus, RotateCcw, ArrowRight } from "lucide-react";

const TWO = ["margin", "markup", "percent", "compound"];

export function Keypad() {
  const appendDigit = useStore((s) => s.appendDigit);
  const deleteLast = useStore((s) => s.deleteLast);
  const clearAll = useStore((s) => s.clearAll);
  const confirmInput = useStore((s) => s.confirmInput);
  const switchField = useStore((s) => s.switchField);
  const addDiscount = useStore((s) => s.addDiscount);
  const mode = useStore((s) => s.mode);
  const result = useStore((s) => s.result);
  const input = useStore((s) => s.input);
  const secondInput = useStore((s) => s.secondInput);
  const activeField = useStore((s) => s.activeField);

  const isDiscount = mode === "discount";
  const isTwo = TWO.includes(mode);
  const hasResult = result !== null;
  const firstFilled = input.length > 0;
  const secondFilled = secondInput.length > 0;

  const digits = [
    ["7", "8", "9"],
    ["4", "5", "6"],
    ["1", "2", "3"],
    [".", "0", "⌫"],
  ];

  const btnBase = "h-12 rounded-lg flex items-center justify-center font-semibold text-sm transition-all duration-100 active:scale-95 active:duration-75 touch-manipulation";

  function ActionBtn() {
    if (isDiscount) {
      return (
        <button onClick={addDiscount} className={`${btnBase} gap-1.5`}
          style={{ background: "var(--color-glass)", color: "#10b981", border: "1px solid var(--color-glass-border)" }}>
          <Plus size={13} /> Ekle
        </button>
      );
    }
    if (isTwo) {
      if (activeField === "first" && firstFilled) {
        return (
          <button onClick={switchField} className={`${btnBase} gap-1.5`}
            style={{ background: "var(--color-glass)", color: "#06b6d4", border: "1px solid var(--color-glass-border)" }}>
            <ArrowRight size={13} /> Sonraki
          </button>
        );
      }
      if (activeField === "second" && firstFilled && secondFilled) {
        return (
          <button onClick={confirmInput} className={`${btnBase} gap-1.5`}
            style={{ background: "var(--color-glass)", color: "#8b5cf6", border: "1px solid var(--color-glass-border)" }}>
            <CornerDownLeft size={13} /> Kaydet
          </button>
        );
      }
      return (
        <button disabled className={`${btnBase} gap-1.5 opacity-30 cursor-not-allowed`}
          style={{ background: "var(--color-glass)", color: "var(--color-text-ghost)", border: "1px solid var(--color-glass-border)" }}>
          <ArrowRight size={13} /> Sonraki
        </button>
      );
    }
    return (
      <button onClick={confirmInput} disabled={!hasResult} className={`${btnBase} gap-1.5`}
        style={{
          background: hasResult ? "var(--color-glass)" : "var(--color-glass)",
          color: hasResult ? "#8b5cf6" : "var(--color-text-ghost)",
          opacity: hasResult ? 1 : 0.3,
          border: "1px solid var(--color-glass-border)",
        }}>
        <CornerDownLeft size={13} /> Kaydet
      </button>
    );
  }

  return (
    <div className="space-y-2">
      {/* Top Row */}
      <div className={`grid gap-1.5 ${isDiscount ? "grid-cols-4" : "grid-cols-3"}`}>
        <button onClick={clearAll} className={`${btnBase} gap-1.5`}
          style={{ background: "var(--color-glass)", color: "#f43f5e", border: "1px solid var(--color-glass-border)" }}>
          <RotateCcw size={13} /> Sıfırla
        </button>
        <button onClick={deleteLast} className={btnBase}
          style={{ background: "var(--color-glass)", color: "var(--color-text-secondary)", border: "1px solid var(--color-glass-border)" }}>
          <Delete size={18} />
        </button>
        <ActionBtn />
        {isDiscount && (
          <button onClick={confirmInput} disabled={!hasResult} className={`${btnBase} gap-1.5`}
            style={{
              background: "var(--color-glass)",
              color: hasResult ? "#8b5cf6" : "var(--color-text-ghost)",
              opacity: hasResult ? 1 : 0.3,
              border: "1px solid var(--color-glass-border)",
            }}>
            <CornerDownLeft size={13} /> Kaydet
          </button>
        )}
      </div>

      {/* Field Indicator */}
      {isTwo && (
        <div className="flex items-center justify-center gap-2 py-0.5">
          <div className="flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-bold transition-all"
            style={{
              background: activeField === "first" ? "#06b6d415" : "transparent",
              color: activeField === "first" ? "#06b6d4" : "var(--color-text-ghost)",
              border: activeField === "first" ? "1px solid #06b6d425" : "1px solid transparent",
            }}>
            1. Alan
          </div>
          <div className="w-4 h-px" style={{ background: "var(--color-glass-border)" }} />
          <div className="flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-bold transition-all"
            style={{
              background: activeField === "second" ? "#06b6d415" : "transparent",
              color: activeField === "second" ? "#06b6d4" : "var(--color-text-ghost)",
              border: activeField === "second" ? "1px solid #06b6d425" : "1px solid transparent",
            }}>
            2. Alan
          </div>
        </div>
      )}

      {/* Numpad */}
      <div className="rounded-xl p-2" style={{ background: "var(--color-bg-surface)", border: "1px solid var(--color-glass-border)" }}>
        <div className="grid grid-cols-3 gap-1.5">
          {digits.flat().map((d) => {
            const isDel = d === "⌫";
            return (
              <button key={d} onClick={() => isDel ? deleteLast() : appendDigit(d)}
                className="group h-14 rounded-lg flex items-center justify-center font-bold text-lg transition-all duration-100 active:scale-90 active:duration-75 num-display touch-manipulation relative overflow-hidden"
                style={{
                  background: "var(--color-bg-elevated)",
                  color: "var(--color-text-primary)",
                  border: "1px solid var(--color-glass-border)",
                }}
                onMouseEnter={(e) => e.currentTarget.style.background = "var(--color-glass-hover)"}
                onMouseLeave={(e) => e.currentTarget.style.background = "var(--color-bg-elevated)"}>
                {d}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
