import { useStore } from "../store/useStore";
import { Copy, Check, ArrowRight, ArrowLeft, ChevronDown, ChevronUp, Plus, X } from "lucide-react";
import { useState, useCallback, useRef, useEffect } from "react";

const kdvRates = [1, 10, 20] as const;
const stopajRates = [1, 3, 5, 7, 10, 15, 20] as const;
const compoundFreqs = [
  { value: 1, label: "Yıllık" },
  { value: 2, label: "6 Aylık" },
  { value: 4, label: "3 Aylık" },
  { value: 12, label: "Aylık" },
  { value: 365, label: "Günlük" },
];

const modeCfg: Record<string, { title: string; desc: string; inputLabel: string; secondLabel?: string }> = {
  kdv: { title: "KDV Hesaplama", desc: "Katma Değer Vergisi hesapla veya KDV dahil tutardan çıkar", inputLabel: "Tutar" },
  stopaj: { title: "Stopaj Kesintisi", desc: "Ödeme veya gelir üzerinden stopaj kesintisini hesapla", inputLabel: "Brüt Tutar" },
  margin: { title: "Kâr Marjı", desc: "Satış fiyatı ve kârı marj üzerinden bul", inputLabel: "Maliyet", secondLabel: "Hedef Kâr Marjı (%)" },
  markup: { title: "Kâr Oranı", desc: "Maliyet üzerine kar oranı ekle", inputLabel: "Maliyet", secondLabel: "Kâr Oranı (%)" },
  discount: { title: "Zincirli İndirim", desc: "Ardışık indirim oranlarını zincirle", inputLabel: "Orijinal Fiyat" },
  percent: { title: "Yüzde İşlemleri", desc: "Yüzde oranı, değişimi ve tersini hesapla", inputLabel: "Yüzde (%)", secondLabel: "Değer" },
  compound: { title: "Bileşik Faiz", desc: "Yıllık faiz ve bileşik dönemle gelecek değeri bul", inputLabel: "Ana Para", secondLabel: "Yıllık Faiz (%)" },
  kdvCompare: { title: "KDV Karşılaştırma", desc: "KDV dahil ve hariç tutarlar arasındaki farkı gör", inputLabel: "Tutar" },
};

export function Display() {
  const input = useStore((s) => s.input);
  const secondInput = useStore((s) => s.secondInput);
  const activeField = useStore((s) => s.activeField);
  const mode = useStore((s) => s.mode);
  const result = useStore((s) => s.result);
  const kdvRate = useStore((s) => s.kdvRate);
  const stopajRate = useStore((s) => s.stopajRate);
  const kdvExtract = useStore((s) => s.kdvExtract);
  const discountInputs = useStore((s) => s.discountInputs);
  const activeDiscountIndex = useStore((s) => s.activeDiscountIndex);
  const compoundYears = useStore((s) => s.compoundYears);
  const compoundFrequency = useStore((s) => s.compoundFrequency);
  const setKdvRate = useStore((s) => s.setKdvRate);
  const setStopajRate = useStore((s) => s.setStopajRate);
  const setKdvExtract = useStore((s) => s.setKdvExtract);
  const setActiveDiscountIndex = useStore((s) => s.setActiveDiscountIndex);
  const removeDiscount = useStore((s) => s.removeDiscount);
  const addDiscount = useStore((s) => s.addDiscount);
  const setCompoundYears = useStore((s) => s.setCompoundYears);
  const setCompoundFrequency = useStore((s) => s.setCompoundFrequency);

  const [copied, setCopied] = useState(false);
  const [showDetail, setShowDetail] = useState(true);
  const [inputAnim, setInputAnim] = useState<"add" | "del" | null>(null);
  const prevLen = useRef(input.length);

  useEffect(() => {
    if (input.length > prevLen.current) setInputAnim("add");
    else if (input.length < prevLen.current) setInputAnim("del");
    prevLen.current = input.length;
    const t = setTimeout(() => setInputAnim(null), 150);
    return () => clearTimeout(t);
  }, [input]);

  const [secondAnim, setSecondAnim] = useState<"add" | "del" | null>(null);
  const prevSecond = useRef(secondInput.length);
  useEffect(() => {
    if (secondInput.length > prevSecond.current) setSecondAnim("add");
    else if (secondInput.length < prevSecond.current) setSecondAnim("del");
    prevSecond.current = secondInput.length;
    const t = setTimeout(() => setSecondAnim(null), 150);
    return () => clearTimeout(t);
  }, [secondInput]);

  const copyResult = useCallback(() => {
    if (result?.display) {
      navigator.clipboard.writeText(result.display);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    }
  }, [result]);

  const cfg = modeCfg[mode];

  return (
    <div className="space-y-5">
      <div className="space-y-1">
        <h2 className="text-xl font-bold text-text-primary tracking-tight">{cfg.title}</h2>
        <p className="text-[13px] text-text-tertiary leading-relaxed">{cfg.desc}</p>
      </div>

      {/* KDV Rate */}
      {mode === "kdv" && (
        <div className="flex flex-wrap items-center gap-2">
          <div className="flex glass rounded-2xl p-1">
            {kdvRates.map((r) => (
              <button key={r} onClick={() => setKdvRate(r)}
                className={`relative px-4 py-1.5 rounded-xl text-sm font-bold transition-all duration-200 ${kdvRate === r ? "bg-blue-500 text-white shadow-lg shadow-blue-500/25" : "text-text-secondary hover:text-text-primary"}`}>
                %{r}
              </button>
            ))}
          </div>
          <button onClick={() => setKdvExtract(!kdvExtract)}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-bold border transition-all duration-200 ${kdvExtract ? "bg-violet-500/15 text-violet-400 border-violet-500/30" : "glass text-text-secondary border-transparent hover:border-white/10"}`}>
            {kdvExtract ? <ArrowLeft size={13} /> : <ArrowRight size={13} />}
            {kdvExtract ? "Dahilden Hariç" : "Hariciden Dahil"}
          </button>
        </div>
      )}

      {/* Stopaj Rate */}
      {mode === "stopaj" && (
        <div className="flex glass rounded-2xl p-1 flex-wrap">
          {stopajRates.map((r) => (
            <button key={r} onClick={() => setStopajRate(r)}
              className={`px-3 py-1.5 rounded-xl text-sm font-bold transition-all duration-200 ${stopajRate === r ? "bg-emerald-500 text-white shadow-lg shadow-emerald-500/25" : "text-text-secondary hover:text-text-primary"}`}>
              %{r}
            </button>
          ))}
        </div>
      )}

      {/* Compound */}
      {mode === "compound" && (
        <div className="space-y-2">
          <div className="flex flex-wrap gap-1.5 items-center">
            <div className="flex items-center gap-1.5 glass rounded-xl px-2.5 py-1.5">
              <label className="text-xs text-text-tertiary font-medium">Vade:</label>
              <input type="number" value={compoundYears} onChange={(e) => setCompoundYears(e.target.value)} min="0.1" step="0.5"
                className="w-14 bg-transparent text-sm text-text-primary font-mono font-bold focus:outline-none text-center" />
              <span className="text-xs text-text-tertiary">yıl</span>
            </div>
            {["0.5", "1", "2", "3", "5", "10"].map((y) => (
              <button key={y} onClick={() => setCompoundYears(y)}
                className={`px-2.5 py-1.5 rounded-xl text-[10px] font-bold transition-all duration-200 ${compoundYears === y ? "bg-emerald-500/15 text-emerald-400 border border-emerald-500/30" : "glass text-text-secondary hover:text-text-primary"}`}>
                {y} yıl
              </button>
            ))}
          </div>
          <div className="flex glass rounded-2xl p-1">
            {compoundFreqs.map((f) => (
              <button key={f.value} onClick={() => setCompoundFrequency(f.value)}
                className={`px-2.5 py-1.5 rounded-xl text-[10px] font-bold transition-all duration-200 ${compoundFrequency === f.value ? "bg-emerald-500 text-white shadow-lg shadow-emerald-500/25" : "text-text-secondary hover:text-text-primary"}`}>
                {f.label}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Input Panel */}
      <div className="glass-strong rounded-3xl p-5 space-y-4">
        {/* Primary Input */}
        <div className="space-y-1.5">
          <label className="text-[10px] font-bold text-text-ghost uppercase tracking-[0.15em]">{cfg.inputLabel}</label>
          <div className={`flex items-baseline justify-end gap-2 py-2 px-3 -mx-3 rounded-xl transition-all duration-200 ${activeField === "first" && cfg.secondLabel ? "bg-cyan-500/[0.05] border border-cyan-500/20" : ""}`}>
            <span className={`num-display text-[2.5rem] font-black text-text-primary leading-none tracking-tight transition-all duration-150 ${inputAnim === "add" ? "anim-digit-add" : inputAnim === "del" ? "anim-digit-del" : ""}`}>
              {input || <span className="text-text-ghost/40">0</span>}
            </span>
            {mode !== "percent" && input && (
              <span className={`text-lg font-bold text-text-ghost transition-all duration-150 ${inputAnim === "add" ? "anim-lira-pulse" : ""}`}>₺</span>
            )}
          </div>
        </div>

        {/* Discount Inputs */}
        {mode === "discount" && (
          <div className="space-y-2 border-t border-white/[0.06] pt-3">
            <div className="flex items-center justify-between">
              <label className="text-[10px] font-bold text-text-ghost uppercase tracking-[0.15em]">İndirim Oranları</label>
              <button onClick={addDiscount}
                className="flex items-center gap-1 px-2.5 py-1 rounded-lg text-[10px] font-bold text-cyan-400 hover:bg-cyan-500/10 border border-cyan-500/20 transition-all">
                <Plus size={10} /> Ekle
              </button>
            </div>
            <div className="flex flex-wrap gap-1.5">
              {discountInputs.map((d, i) => (
                <div key={i} className="relative group">
                  <button onClick={() => setActiveDiscountIndex(i)}
                    className={`flex items-center gap-1 px-3 py-2 rounded-xl text-xs font-mono font-bold transition-all ${i === activeDiscountIndex ? "bg-cyan-500/15 text-cyan-400 border border-cyan-500/30" : "glass text-text-secondary border-transparent hover:border-white/10"}`}>
                    <span className="text-[9px] opacity-50">#{i + 1}</span>
                    <span>{d || "0"}</span>
                    <span className="text-[9px] opacity-50">%</span>
                  </button>
                  {discountInputs.length > 1 && (
                    <button onClick={(e) => { e.stopPropagation(); removeDiscount(i); }}
                      className="absolute -top-1 -right-1 w-4 h-4 rounded-full bg-rose-500/90 text-white flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity hover:bg-rose-500">
                      <X size={8} />
                    </button>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Second Input */}
        {cfg.secondLabel && (
          <div className="space-y-1.5 border-t border-white/[0.06] pt-3">
            <label className="text-[10px] font-bold text-text-ghost uppercase tracking-[0.15em]">{cfg.secondLabel}</label>
            <div className={`flex items-baseline justify-end gap-2 py-2 px-3 -mx-3 rounded-xl transition-all duration-200 ${activeField === "second" ? "bg-cyan-500/[0.05] border border-cyan-500/20" : ""}`}>
              <span className={`num-display text-xl font-bold text-text-secondary transition-all duration-150 ${secondAnim === "add" ? "anim-digit-add-sm" : secondAnim === "del" ? "anim-digit-del-sm" : ""}`}>
                {secondInput || <span className="text-text-ghost/40">0</span>}
              </span>
              {mode !== "percent" && <span className="text-sm font-bold text-text-ghost">%</span>}
            </div>
          </div>
        )}
      </div>

      {/* Result */}
      {result && (
        <div className="anim-fade-up space-y-3">
          <button onClick={copyResult}
            className="w-full relative overflow-hidden rounded-3xl p-[1px] group cursor-pointer transition-all duration-300 hover:scale-[1.01]">
            <div className="absolute inset-0 bg-gradient-to-r from-cyan-500/40 via-blue-500/40 to-violet-500/40 opacity-60 group-hover:opacity-100 transition-opacity duration-500" />
            <div className="relative bg-bg-deep rounded-[22px] p-6 text-center space-y-2">
              <div className="flex items-center justify-center gap-2">
                <span className="text-[10px] font-bold text-brand-400 uppercase tracking-[0.2em]">Sonuç</span>
                <span className="opacity-0 group-hover:opacity-100 transition-opacity">
                  {copied ? <Check size={12} className="text-emerald-400" /> : <Copy size={12} className="text-text-ghost" />}
                </span>
              </div>
              <div className="num-display text-[2.5rem] sm:text-[3rem] font-black gradient-text-result leading-none tracking-tight">
                {result.display}
              </div>
              {result.detail && <p className="text-sm text-text-secondary font-medium">{result.detail}</p>}
              {result.secondary && <p className="text-xs text-text-tertiary">{result.secondary}</p>}
              {result.third && <p className="text-xs text-text-tertiary">{result.third}</p>}
            </div>
          </button>

          {result.rows && result.rows.length > 0 && (
            <div className="glass-strong rounded-2xl overflow-hidden">
              <button onClick={() => setShowDetail(!showDetail)}
                className="w-full flex items-center justify-between px-4 py-2.5 text-[10px] font-bold text-text-ghost uppercase tracking-[0.15em] hover:bg-white/[0.02] transition-colors">
                <span>Detay</span>
                {showDetail ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
              </button>
              {showDetail && (
                <div className="anim-fade-in">
                  {result.rows.map((row, i) => (
                    <div key={i} className={`flex justify-between items-center px-4 py-3 ${i < result.rows!.length - 1 ? "border-t border-white/[0.04]" : ""} ${row.accent ? "bg-white/[0.02]" : ""}`}>
                      <span className={`text-xs ${row.accent ? "text-text-primary font-semibold" : "text-text-secondary"}`}>{row.label}</span>
                      <span className={`num-display text-xs ${row.accent ? "gradient-text-result font-black text-sm" : "text-text-primary font-semibold"}`}>{row.value}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
