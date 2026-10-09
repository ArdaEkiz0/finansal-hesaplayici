import { useStore } from "../store/useStore";
import { Delete, CornerDownLeft, Plus, RotateCcw, ArrowRight, X, Search } from "lucide-react";
import { useState } from "react";
import { STOPAJ_PRESETS, TEVKIFAT_CODES, TEVKIFAT_LIMIT_2026, RATE_DISCLAIMER } from "../data/tax";
import { parseAmount } from "../core/engine";

const TWO_LABELS: Record<string, [string, string]> = {
  margin: ["Maliyet (₺)", "Hedef marj (%)"],
  markup: ["Maliyet (₺)", "Kâr oranı (%)"],
  percent: ["Yüzde (%)", "Tutar"],
  compound: ["Ana para (₺)", "Yıllık faiz (%)"],
};

export function CalcPanel() {
  const mode = useStore((s) => s.mode);
  const input = useStore((s) => s.input);
  const secondInput = useStore((s) => s.secondInput);
  const activeField = useStore((s) => s.activeField);
  const setInputDirect = useStore((s) => s.setInputDirect);
  const setSecondInputDirect = useStore((s) => s.setSecondInputDirect);
  const appendDigit = useStore((s) => s.appendDigit);
  const deleteLast = useStore((s) => s.deleteLast);
  const clearAll = useStore((s) => s.clearAll);
  const confirmInput = useStore((s) => s.confirmInput);
  const switchField = useStore((s) => s.switchField);
  const result = useStore((s) => s.result);
  const kdvRate = useStore((s) => s.kdvRate);
  const setKdvRate = useStore((s) => s.setKdvRate);
  const stopajRate = useStore((s) => s.stopajRate);
  const setStopajRate = useStore((s) => s.setStopajRate);
  const tevkifatPay = useStore((s) => s.tevkifatPay);
  const setTevkifatPay = useStore((s) => s.setTevkifatPay);
  const tevkifatCode = useStore((s) => s.tevkifatCode);
  const setTevkifatCode = useStore((s) => s.setTevkifatCode);
  const kdvExtract = useStore((s) => s.kdvExtract);
  const setKdvExtract = useStore((s) => s.setKdvExtract);
  const customKdvRate = useStore((s) => s.customKdvRate);
  const setCustomKdvRate = useStore((s) => s.setCustomKdvRate);
  const useCustomRate = useStore((s) => s.useCustomRate);
  const discountInputs = useStore((s) => s.discountInputs);
  const activeDiscountIndex = useStore((s) => s.activeDiscountIndex);
  const setActiveDiscountIndex = useStore((s) => s.setActiveDiscountIndex);
  const addDiscount = useStore((s) => s.addDiscount);
  const removeDiscount = useStore((s) => s.removeDiscount);
  const compoundYears = useStore((s) => s.compoundYears);
  const setCompoundYears = useStore((s) => s.setCompoundYears);
  const compoundFrequency = useStore((s) => s.compoundFrequency);
  const setCompoundFrequency = useStore((s) => s.setCompoundFrequency);

  const isTwo = ["margin", "markup", "percent", "compound"].includes(mode);
  const isDiscount = mode === "discount" || mode === "fiyat";
  const labels = TWO_LABELS[mode];
  const [codeQuery, setCodeQuery] = useState("");

  const tevkifatTotal = (() => {
    if (mode !== "tevkifat" || !input) return null;
    const m = parseAmount(input);
    if (!m) return null;
    return m.plus(m.mul(kdvRate).div(100)).toDP(2);
  })();
  const underLimit = tevkifatTotal !== null && tevkifatTotal.lt(TEVKIFAT_LIMIT_2026);

  const digits = ["7", "8", "9", "4", "5", "6", "1", "2", "3", ".", "0", "⌫"];
  const showOps = !(isTwo && activeField === "second") && !(isDiscount && activeField !== "first");
  const ops: { label: string; value: string }[] = [
    { label: "+", value: "+" },
    { label: "−", value: "-" },
    { label: "×", value: "*" },
    { label: "÷", value: "/" },
  ];

  return (
    <section className="card p-4 gap-4 min-w-0 h-full flex flex-col min-h-[540px]">
      {(mode === "kdv" || mode === "tevkifat" || mode === "kdvCompare" || mode === "fiyat") && (
        <div className="space-y-2">
          <div className="flex gap-1.5">
            {([1, 10, 20] as const).map((r) => (
              <button key={r} onClick={() => setKdvRate(r)}
                className="flex-1 py-2 rounded-xl text-[13px] font-bold btn-press focus-ring"
                style={kdvRate === r && !useCustomRate
                  ? { background: "var(--color-primary)", color: "#fff" }
                  : { background: "var(--color-glass)", color: "var(--color-text-secondary)", border: "1px solid var(--color-glass-border)" }}>
                %{r}
              </button>
            ))}
            <input value={customKdvRate} onChange={(e) => setCustomKdvRate(e.target.value)} placeholder="Özel %" inputMode="decimal"
              className="w-[86px] px-2 py-2 rounded-xl text-[13px] font-bold text-center focus-ring"
              style={{ background: useCustomRate ? "rgba(47,123,255,.14)" : "var(--color-glass)", border: useCustomRate ? "1px solid var(--color-primary)" : "1px solid var(--color-glass-border)", color: "var(--color-text-primary)" }} />
          </div>
          {(mode === "kdv" || mode === "fiyat") && (
            <div className="flex gap-1.5">
              {([{ v: false, l: "Hariçten Dahil (+KDV)" }, { v: true, l: "Dahilden Hariç (−KDV)" }] as const).map((o) => (
                <button key={o.l} onClick={() => setKdvExtract(o.v)}
                  className="flex-1 py-1.5 rounded-lg text-[11px] font-semibold btn-press"
                  style={kdvExtract === o.v
                    ? { background: "rgba(47,123,255,.16)", color: "var(--color-primary-light)", border: "1px solid var(--color-primary)" }
                    : { background: "transparent", color: "var(--color-text-ghost)", border: "1px solid var(--color-glass-border)" }}>
                  {o.l}
                </button>
              ))}
            </div>
          )}
          {mode === "tevkifat" && (
            <div className="space-y-2">
              <div className="relative">
                <Search size={13} className="absolute left-2.5 top-1/2 -translate-y-1/2" style={{ color: "var(--color-text-ghost)" }} />
                <input value={codeQuery} onChange={(e) => setCodeQuery(e.target.value)} placeholder="İş arayın: temizlik, nakliye, danışmanlık..."
                  className="w-full pl-8 pr-3 py-2 rounded-xl text-[12px] focus-ring"
                  style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-primary)" }} />
              </div>
              <div className="max-h-[132px] overflow-y-auto space-y-1 pr-0.5">
                {TEVKIFAT_CODES.filter((c) =>
                  !codeQuery ||
                  c.name.toLocaleLowerCase("tr").includes(codeQuery.toLocaleLowerCase("tr")) ||
                  c.code.includes(codeQuery)
                ).map((c) => {
                  const itemKey = `${c.code} ${c.name}`;
                  const activePay = tevkifatCode === itemKey;
                  return (
                    <button key={`${c.code}-${c.name}`} onClick={() => setTevkifatCode(itemKey, c.pay)}
                      className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left btn-press"
                      style={activePay
                        ? { background: "rgba(245,165,36,.16)", border: "1px solid var(--color-accent)" }
                        : { background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
                      <span className="text-[11px] font-bold shrink-0" style={{ color: "var(--color-accent)" }}>{c.pay}/10</span>
                      <span className="text-[11px] truncate" style={{ color: "var(--color-text-secondary)" }}>{c.code !== "—" ? `${c.code} ` : ""}{c.name}</span>
                    </button>
                  );
                })}
              </div>
              {tevkifatCode && (
                <div className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg" style={{ background: "rgba(245,165,36,.12)", border: "1px solid var(--color-accent)" }}>
                  <span className="text-[11px] font-bold truncate" style={{ color: "var(--color-accent)" }}>Seçili: {tevkifatPay}/10 — {tevkifatCode}</span>
                  <button onClick={() => setTevkifatCode("", tevkifatPay)} className="ml-auto p-0.5 rounded shrink-0" style={{ color: "var(--color-text-ghost)" }} title="Seçimi temizle">
                    <X size={11} />
                  </button>
                </div>
              )}
              {underLimit && (
                <p className="text-[11px] px-2.5 py-1.5 rounded-lg" style={{ background: "rgba(245,165,36,.12)", color: "var(--color-accent)", border: "1px solid var(--color-accent)" }}>
                  KDV dahil toplam 12.000 TL altında — kısmi tevkifat uygulanmayabilir (2026 sınırı).
                </p>
              )}
              <p className="text-[10px] px-1" style={{ color: "var(--color-text-ghost)" }}>{RATE_DISCLAIMER}</p>
            </div>
          )}
        </div>
      )}

      {mode === "stopaj" && (
        <div className="space-y-2">
          <div className="flex flex-wrap gap-1.5">
            {STOPAJ_PRESETS.map((preset) => (
              <button key={preset.label} title={preset.note} onClick={() => setStopajRate(preset.rate)}
                className="px-3 py-2 rounded-xl text-[12px] font-bold btn-press"
                style={stopajRate === preset.rate
                  ? { background: "var(--color-success)", color: "#fff" }
                  : { background: "var(--color-glass)", color: "var(--color-text-secondary)", border: "1px solid var(--color-glass-border)" }}>
                {preset.label} %{preset.rate}
              </button>
            ))}
          </div>
          <div className="flex flex-wrap gap-1.5">
            {([1, 3, 5, 7, 10, 15, 17, 20] as const).map((r) => (
              <button key={r} onClick={() => setStopajRate(r)}
                className="px-2.5 py-1.5 rounded-lg text-[11px] font-bold btn-press"
                style={stopajRate === r
                  ? { background: "var(--color-success)", color: "#fff" }
                  : { background: "transparent", color: "var(--color-text-ghost)", border: "1px solid var(--color-glass-border)" }}>
                %{r}
              </button>
            ))}
          </div>
          <p className="text-[10px] px-1" style={{ color: "var(--color-text-ghost)" }}>{RATE_DISCLAIMER}</p>
        </div>
      )}

      {isDiscount ? (
        <div className="space-y-2">
          <label className="text-[11px] font-bold uppercase tracking-wider" style={{ color: "var(--color-text-ghost)" }}>{mode === "fiyat" ? "Etiket fiyatı (₺)" : "Ana tutar (₺)"}</label>
          <input value={input} onChange={(e) => setInputDirect(e.target.value)} placeholder="0" inputMode="decimal" autoFocus
            className="num-display w-full px-4 py-3 rounded-xl text-right font-bold fluid-number-sm focus-ring"
            style={{ background: "var(--color-glass)", border: activeField === "first" ? "1px solid var(--color-primary)" : "1px solid var(--color-glass-border)", color: "var(--color-text-primary)" }}
            onFocus={() => activeField !== "first" && switchField()} />
          <div className="flex items-center justify-between">
            <label className="text-[11px] font-bold uppercase tracking-wider" style={{ color: "var(--color-text-ghost)" }}>İndirimler</label>
            <button onClick={addDiscount} className="flex items-center gap-1 px-2 py-1 rounded-lg text-[11px] font-bold btn-press" style={{ background: "var(--color-glass)", color: "var(--color-success)", border: "1px solid var(--color-glass-border)" }}>
              <Plus size={12} /> Oran ekle
            </button>
          </div>
          <div className="flex flex-wrap gap-1.5">
            {discountInputs.map((d, i) => (
              <div key={i} className="flex items-center gap-1 rounded-xl px-1 py-1"
                style={{ background: activeDiscountIndex === i && activeField === "second" ? "rgba(47,123,255,.14)" : "var(--color-glass)", border: activeDiscountIndex === i && activeField === "second" ? "1px solid var(--color-primary)" : "1px solid var(--color-glass-border)" }}>
                <input value={d} placeholder="%0" inputMode="decimal" onChange={(e) => { setActiveDiscountIndex(i); const v = e.target.value.replace(/[^0-9.]/g, ""); const cur = [...discountInputs]; cur[i] = v; useStore.setState({ discountInputs: cur }); }}
                  onFocus={() => setActiveDiscountIndex(i)}
                  className="w-[64px] px-2 py-1.5 rounded-lg text-center text-[13px] font-bold bg-transparent focus-ring" style={{ color: "var(--color-text-primary)" }} />
                {discountInputs.length > 1 && (
                  <button onClick={() => removeDiscount(i)} className="p-1 rounded" style={{ color: "var(--color-text-ghost)" }}><X size={11} /></button>
                )}
              </div>
            ))}
          </div>
        </div>
      ) : isTwo ? (
        <div className="grid grid-cols-2 gap-2">
          <div className="space-y-1 min-w-0">
            <label className="text-[11px] font-bold uppercase tracking-wider" style={{ color: "var(--color-text-ghost)" }}>{labels[0]}</label>
            <input value={input} onChange={(e) => setInputDirect(e.target.value)} placeholder="0" inputMode="decimal"
              className="num-display w-full px-3 py-2.5 rounded-xl text-right font-bold text-[18px] focus-ring"
              style={{ background: "var(--color-glass)", border: activeField === "first" ? "1px solid var(--color-primary)" : "1px solid var(--color-glass-border)", color: "var(--color-text-primary)" }} />
          </div>
          <div className="space-y-1 min-w-0">
            <label className="text-[11px] font-bold uppercase tracking-wider" style={{ color: "var(--color-text-ghost)" }}>{labels[1]}</label>
            <input value={secondInput} onChange={(e) => setSecondInputDirect(e.target.value)} placeholder="0" inputMode="decimal"
              className="num-display w-full px-3 py-2.5 rounded-xl text-right font-bold text-[18px] focus-ring"
              style={{ background: "var(--color-glass)", border: activeField === "second" ? "1px solid var(--color-primary)" : "1px solid var(--color-glass-border)", color: "var(--color-text-primary)" }} />
          </div>
          {mode === "compound" && (
            <div className="col-span-2 flex gap-2">
              <input value={compoundYears} onChange={(e) => setCompoundYears(e.target.value)} placeholder="Yıl" inputMode="decimal"
                className="w-[90px] px-3 py-2 rounded-xl text-center text-[13px] font-bold focus-ring" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-primary)" }} />
              <div className="flex flex-1 gap-1">
                {([1, 4, 12, 365] as const).map((f) => (
                  <button key={f} onClick={() => setCompoundFrequency(f)} className="flex-1 py-2 rounded-lg text-[11px] font-bold btn-press"
                    style={compoundFrequency === f ? { background: "var(--color-primary)", color: "#fff" } : { background: "var(--color-glass)", color: "var(--color-text-ghost)", border: "1px solid var(--color-glass-border)" }}>
                    {f === 1 ? "Yıllık" : f === 4 ? "Çeyrek" : f === 12 ? "Aylık" : "Günlük"}
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>
      ) : (
        <div className="space-y-1">
          <label className="text-[11px] font-bold uppercase tracking-wider" style={{ color: "var(--color-text-ghost)" }}>Tutar (₺)</label>
          <input value={input} onChange={(e) => setInputDirect(e.target.value)} placeholder="0" inputMode="decimal" autoFocus
            className="num-display w-full px-4 py-3 rounded-xl text-right font-bold fluid-number-sm focus-ring"
            style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-primary)" }} />
        </div>
      )}

      <div className="grid grid-cols-3 gap-1.5">
        <button onClick={clearAll} className="h-11 rounded-xl flex items-center justify-center gap-1.5 font-semibold text-[13px] btn-press" style={{ background: "var(--color-glass)", color: "var(--color-error)", border: "1px solid var(--color-glass-border)" }}>
          <RotateCcw size={13} /> Sıfırla
        </button>
        <button onClick={deleteLast} className="h-11 rounded-xl flex items-center justify-center btn-press" style={{ background: "var(--color-glass)", color: "var(--color-text-secondary)", border: "1px solid var(--color-glass-border)" }}>
          <Delete size={17} />
        </button>
        {(isTwo || isDiscount) ? (
          <button onClick={() => (activeField === "first" ? switchField() : confirmInput())} className="h-11 rounded-xl flex items-center justify-center gap-1.5 font-semibold text-[13px] btn-press" style={{ background: "var(--color-primary)", color: "#fff" }}>
            {activeField === "first" ? (<><ArrowRight size={13} /> Sonraki</>) : (<><CornerDownLeft size={13} /> Kaydet</>)}
          </button>
        ) : (
          <button onClick={confirmInput} disabled={!result} className="h-11 rounded-xl flex items-center justify-center gap-1.5 font-semibold text-[13px] btn-press" style={{ background: result ? "var(--color-primary)" : "var(--color-glass)", color: result ? "#fff" : "var(--color-text-ghost)", opacity: result ? 1 : 0.5 }}>
            <CornerDownLeft size={13} /> Kaydet
          </button>
        )}
      </div>

      <div className="card-soft p-2 flex-1 min-h-0 flex flex-col gap-1.5">
        {showOps && (
          <div className="grid grid-cols-4 gap-1.5">
            {ops.map((o) => (
              <button key={o.value} onClick={() => appendDigit(o.value)}
                className="h-10 rounded-xl font-bold text-[17px] btn-press focus-ring"
                style={{ background: "rgba(47,123,255,.12)", color: "var(--color-primary-light)", border: "1px solid var(--color-primary)" }}>
                {o.label}
              </button>
            ))}
          </div>
        )}
        <div className="grid grid-cols-3 grid-rows-4 gap-1.5 flex-1 min-h-0">
          {digits.map((d) => (
            <button key={d} onClick={() => (d === "⌫" ? deleteLast() : appendDigit(d))}
              className="h-full min-h-[52px] rounded-xl font-bold text-[22px] num-display btn-press focus-ring"
              style={{ background: "var(--color-bg-elevated)", color: "var(--color-text-primary)", border: "1px solid var(--color-glass-border)" }}>
              {d}
            </button>
          ))}
        </div>
      </div>
    </section>
  );
}
