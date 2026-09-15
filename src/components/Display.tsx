import { useStore } from "../store/useStore";
import { Copy, Check } from "lucide-react";
import { useState, useCallback } from "react";

export function Display() {
  const mode = useStore((s) => s.mode);
  const input = useStore((s) => s.input);
  const secondInput = useStore((s) => s.secondInput);
  const activeField = useStore((s) => s.activeField);
  const result = useStore((s) => s.result);
  const kdvRate = useStore((s) => s.kdvRate);
  const setKdvRate = useStore((s) => s.setKdvRate);
  const stopajRate = useStore((s) => s.stopajRate);
  const setStopajRate = useStore((s) => s.setStopajRate);
  const notify = useStore((s) => s.notify);

  const [copied, setCopied] = useState(false);

  const TWO = ["margin", "markup", "percent", "compound"];
  const isTwo = TWO.includes(mode);

  const copyResult = useCallback(() => {
    if (result?.display) {
      navigator.clipboard.writeText(result.display);
      setCopied(true);
      notify("Kopyalandı", "success");
      setTimeout(() => setCopied(false), 1500);
    }
  }, [result, notify]);

  let displayValue = input;
  if (!input && !isTwo) displayValue = "0";

  return (
    <div className="space-y-4">
      {/* Rate Selector */}
      {mode === "kdv" && (
        <div className="flex gap-1.5">
          {([1, 10, 20] as const).map((r) => (
            <button key={r} onClick={() => setKdvRate(r)}
              className="flex-1 py-2 rounded-lg text-xs font-bold transition-all duration-150"
              style={{
                background: kdvRate === r ? "#3b82f6" : "var(--color-glass)",
                color: kdvRate === r ? "#fff" : "var(--color-text-tertiary)",
                border: `1px solid ${kdvRate === r ? "#3b82f6" : "var(--color-glass-border)"}`,
              }}>
              %{r}
            </button>
          ))}
        </div>
      )}

      {mode === "stopaj" && (
        <div className="flex flex-wrap gap-1.5">
          {([1, 3, 5, 7, 10, 15, 20] as const).map((r) => (
            <button key={r} onClick={() => setStopajRate(r)}
              className="px-3 py-2 rounded-lg text-xs font-bold transition-all duration-150"
              style={{
                background: stopajRate === r ? "#10b981" : "var(--color-glass)",
                color: stopajRate === r ? "#fff" : "var(--color-text-tertiary)",
                border: `1px solid ${stopajRate === r ? "#10b981" : "var(--color-glass-border)"}`,
              }}>
              %{r}
            </button>
          ))}
        </div>
      )}

      {/* Input Display */}
      <div className="rounded-xl p-4" style={{ background: "var(--color-bg-surface)", border: "1px solid var(--color-glass-border)" }}>
        {isTwo ? (
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <span className="text-[10px] font-bold uppercase tracking-wider" style={{ color: "var(--color-text-ghost)" }}>1. Alan</span>
              <div className="flex-1 h-px" style={{ background: "var(--color-glass-border)" }} />
            </div>
            <div className="text-right">
              <span className="num-display text-xl font-bold" style={{ color: activeField === "first" ? "var(--color-text-primary)" : "var(--color-text-ghost)" }}>
                {input || "0"}
              </span>
            </div>
            <div className="flex items-center gap-2 mt-2">
              <span className="text-[10px] font-bold uppercase tracking-wider" style={{ color: "var(--color-text-ghost)" }}>2. Alan</span>
              <div className="flex-1 h-px" style={{ background: "var(--color-glass-border)" }} />
            </div>
            <div className="text-right">
              <span className="num-display text-xl font-bold" style={{ color: activeField === "second" ? "var(--color-text-primary)" : "var(--color-text-ghost)" }}>
                {secondInput || "0"}
              </span>
            </div>
          </div>
        ) : (
          <div className="text-right">
            <span className="num-display text-2xl font-bold" style={{ color: "var(--color-text-primary)" }}>{displayValue}</span>
            <span className="text-sm font-semibold ml-1" style={{ color: "var(--color-text-ghost)" }}>₺</span>
          </div>
        )}
      </div>

      {/* Result */}
      {result && (
        <div className="rounded-xl p-4" style={{ background: "var(--color-bg-surface)", border: "1px solid var(--color-glass-border)" }}>
          <div className="flex items-center justify-between mb-2">
            <span className="text-[10px] font-bold uppercase tracking-wider" style={{ color: "var(--color-text-ghost)" }}>{result.detail}</span>
            <button onClick={copyResult} className="p-1 rounded-md transition-colors" style={{ color: copied ? "#10b981" : "var(--color-text-ghost)" }}>
              {copied ? <Check size={12} /> : <Copy size={12} />}
            </button>
          </div>
          <div className="text-right">
            <span className="num-display text-2xl font-bold gradient-text-result">{result.display}</span>
          </div>
        </div>
      )}

      {/* Detail Rows */}
      {result?.rows && result.rows.length > 0 && (
        <div className="rounded-xl p-3 space-y-1" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
          {result.rows.map((d: { label: string; value: string; accent?: boolean }, i: number) => (
            <div key={i} className="flex items-center justify-between py-1"
              style={{ borderBottom: i < result.rows!.length - 1 ? "1px solid var(--color-glass-border)" : "none" }}>
              <span className="text-[11px] font-medium" style={{ color: "var(--color-text-tertiary)" }}>{d.label}</span>
              <span className="num-display text-xs font-bold"
                style={{ color: d.accent ? "var(--color-text-primary)" : "var(--color-text-secondary)" }}>{d.value}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
