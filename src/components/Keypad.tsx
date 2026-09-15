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

  function actionBtn() {
    if (isDiscount) {
      return (
        <button onClick={addDiscount}
          className="h-14 min-h-[56px] rounded-2xl glass flex items-center justify-center gap-1.5 text-emerald-400 font-bold text-xs uppercase tracking-wider hover:bg-emerald-500/10 active:scale-95 transition-all touch-manipulation">
          <Plus size={14} /> Ekle
        </button>
      );
    }
    if (isTwo) {
      if (activeField === "first" && firstFilled) {
        return (
          <button onClick={switchField}
            className="h-14 min-h-[56px] rounded-2xl glass flex items-center justify-center gap-1.5 text-cyan-400 font-bold text-xs uppercase tracking-wider hover:bg-cyan-500/10 active:scale-95 transition-all touch-manipulation">
            <ArrowRight size={14} /> Sonraki
          </button>
        );
      }
      if (activeField === "second" && firstFilled && secondFilled) {
        return (
          <button onClick={confirmInput}
            className="h-14 min-h-[56px] rounded-2xl glass flex items-center justify-center gap-1.5 text-violet-400 font-bold text-xs uppercase tracking-wider hover:bg-violet-500/10 active:scale-95 transition-all touch-manipulation">
            <CornerDownLeft size={14} /> Kaydet
          </button>
        );
      }
      return (
        <button disabled
          className="h-14 min-h-[56px] rounded-2xl glass flex items-center justify-center gap-1.5 text-text-ghost cursor-not-allowed opacity-40 font-bold text-xs uppercase tracking-wider">
          <ArrowRight size={14} /> Sonraki
        </button>
      );
    }
    return (
      <button onClick={confirmInput} disabled={!hasResult}
        className={`h-14 min-h-[56px] rounded-2xl glass flex items-center justify-center gap-1.5 font-bold text-xs uppercase tracking-wider active:scale-95 transition-all touch-manipulation ${hasResult ? "text-violet-400 hover:bg-violet-500/10" : "text-text-ghost cursor-not-allowed opacity-40"}`}>
        <CornerDownLeft size={14} /> Kaydet
      </button>
    );
  }

  return (
    <div className="space-y-3">
      <div className={`grid gap-2 ${isDiscount ? "grid-cols-4" : "grid-cols-3"}`}>
        <button onClick={clearAll}
          className="h-14 min-h-[56px] rounded-2xl glass flex items-center justify-center gap-1.5 text-rose-400 font-bold text-xs uppercase tracking-wider hover:bg-rose-500/10 active:scale-95 transition-all touch-manipulation">
          <RotateCcw size={14} /> Sıfırla
        </button>
        <button onClick={deleteLast}
          className="h-14 min-h-[56px] rounded-2xl glass flex items-center justify-center text-amber-400 hover:bg-amber-500/10 active:scale-95 transition-all touch-manipulation">
          <Delete size={20} />
        </button>
        {actionBtn()}
        {isDiscount && (
          <button onClick={confirmInput} disabled={!hasResult}
            className={`h-14 min-h-[56px] rounded-2xl glass flex items-center justify-center gap-1.5 font-bold text-xs uppercase tracking-wider active:scale-95 transition-all touch-manipulation ${hasResult ? "text-violet-400 hover:bg-violet-500/10" : "text-text-ghost cursor-not-allowed opacity-40"}`}>
            <CornerDownLeft size={14} /> Kaydet
          </button>
        )}
      </div>

      {isTwo && (
        <div className="flex items-center justify-center gap-2 py-0.5">
          <div className={`flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold transition-all ${activeField === "first" ? "bg-cyan-500/15 text-cyan-400 border border-cyan-500/30" : "glass text-text-ghost"}`}>
            <span className="w-1.5 h-1.5 rounded-full bg-current" /> 1. Alan
          </div>
          <div className={`w-5 h-[1px] ${activeField === "second" ? "bg-cyan-500/40" : "bg-white/10"}`} />
          <div className={`flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold transition-all ${activeField === "second" ? "bg-cyan-500/15 text-cyan-400 border border-cyan-500/30" : "glass text-text-ghost"}`}>
            <span className="w-1.5 h-1.5 rounded-full bg-current" /> 2. Alan
          </div>
        </div>
      )}

      <div className="glass-strong rounded-3xl p-2.5">
        <div className="grid grid-cols-3 gap-1.5">
          {digits.flat().map((d) => {
            const isDel = d === "⌫";
            return (
              <button key={d}
                onClick={() => isDel ? deleteLast() : appendDigit(d)}
                className={`group h-16 min-h-[64px] rounded-2xl flex items-center justify-center font-bold text-xl transition-all duration-150 active:scale-90 active:duration-75 num-display overflow-hidden relative touch-manipulation ${
                  isDel ? "glass text-text-secondary hover:text-text-primary hover:bg-white/[0.06]" : "glass-strong text-text-primary hover:bg-white/[0.08] hover:shadow-lg"
                }`}>
                <span className="relative z-10 transition-transform duration-100 group-active:scale-90">{d}</span>
                <span className="absolute inset-0 bg-white/[0.08] scale-0 group-active:scale-100 rounded-2xl transition-transform duration-200 origin-center" />
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
