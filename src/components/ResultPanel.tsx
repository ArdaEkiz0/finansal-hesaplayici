import { useStore } from "../store/useStore";
import { Copy, Check, Share2, ReceiptText, Sparkles } from "lucide-react";
import { useState } from "react";
import { toDecimal, numberToTurkishWords } from "../core/engine";
import Decimal from "decimal.js";

export function ResultPanel() {
  const result = useStore((s) => s.result);
  const input = useStore((s) => s.input);
  const notify = useStore((s) => s.notify);
  const toggleInvoice = useStore((s) => s.toggleInvoice);
  const [copied, setCopied] = useState(false);

  const copyText = async (t: string, msg = "Kopyalandı") => {
    try {
      await navigator.clipboard.writeText(t);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = t;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      ta.remove();
    }
    setCopied(true);
    notify(msg, "success");
    setTimeout(() => setCopied(false), 1400);
  };

  let words = "";
  try {
    if (result) {
      const numStr = result.display.replace(/\./g, "").replace(",", ".").replace(/[^0-9.-]/g, "");
      if (numStr && numStr !== "—") words = numberToTurkishWords(new Decimal(numStr || "0"));
    } else if (input) {
      words = numberToTurkishWords(toDecimal(input || "0"));
    }
  } catch { words = ""; }

  const share = async () => {
    if (!result) return;
    const text = `Hesap: ${result.detail}\nSonuç: ${result.display}${words ? `\nYazıyla: ${words}` : ""}`;
    const nav = navigator as Navigator & { share?: (d: { title: string; text: string }) => Promise<void> };
    if (nav.share) {
      try { await nav.share({ title: "Finansal Hesaplayıcı", text }); return; } catch { /* vazgecti */ }
    }
    copyText(text, "Paylaşım metni kopyalandı");
  };

  if (!result) {
    return (
      <section className="card p-6 min-h-[320px] flex flex-col items-center justify-center text-center gap-2 min-w-0">
        <Sparkles size={22} style={{ color: "var(--color-text-ghost)" }} />
        <p className="text-[14px] font-semibold" style={{ color: "var(--color-text-secondary)" }}>Sonuç burada canlanacak</p>
        <p className="text-[12px] max-w-[280px]" style={{ color: "var(--color-text-ghost)" }}>Tutarı yazın — hesaplama anında yapılır, butona basmanıza gerek yok.</p>
        {words && <p className="text-[12px] italic mt-2" style={{ color: "var(--color-text-tertiary)" }}>{words}</p>}
      </section>
    );
  }

  return (
    <section className="card p-5 space-y-4 min-w-0 anim-fade-up" key={result.display + result.detail}>
      <div className="flex items-center justify-between gap-2">
        <span className="text-[11px] font-bold uppercase tracking-wider truncate" style={{ color: "var(--color-text-tertiary)" }}>{result.detail}</span>
        <span className="text-[10px] px-2 py-0.5 rounded-full font-bold shrink-0" style={{ background: "rgba(22,185,129,.14)", color: "var(--color-success)" }}>CANLI</span>
      </div>

      <div className="text-right">
        <span className="num-display font-bold gradient-text-result fluid-number">{result.display}</span>
      </div>

      {words && (
        <p className="text-right text-[12px] italic leading-snug" style={{ color: "var(--color-text-tertiary)" }}>{words}</p>
      )}

      <div className="grid grid-cols-3 gap-1.5">
        <button onClick={() => copyText(result.display)} className="py-2 rounded-xl text-[12px] font-bold btn-press flex items-center justify-center gap-1.5" style={{ background: "var(--color-primary)", color: "#fff" }}>
          {copied ? <Check size={13} /> : <Copy size={13} />} {copied ? "Oldu" : "Kopyala"}
        </button>
        <button onClick={share} className="py-2 rounded-xl text-[12px] font-bold btn-press flex items-center justify-center gap-1.5" style={{ background: "var(--color-glass)", color: "var(--color-text-secondary)", border: "1px solid var(--color-glass-border)" }}>
          <Share2 size={13} /> Paylaş
        </button>
        <button onClick={toggleInvoice} className="py-2 rounded-xl text-[12px] font-bold btn-press flex items-center justify-center gap-1.5" style={{ background: "var(--color-glass)", color: "var(--color-text-secondary)", border: "1px solid var(--color-glass-border)" }}>
          <ReceiptText size={13} /> Fatura
        </button>
      </div>

      {result.rows && result.rows.length > 0 && (
        <div className="card-soft p-1.5">
          {result.rows.map((d, i) => (
            <button key={i} onClick={() => copyText(d.value, `${d.label} kopyalandı`)} title="Kopyalamak için tıkla"
              className="w-full flex items-center justify-between px-2.5 py-2 rounded-lg text-left btn-press"
              style={{ borderBottom: i < result.rows!.length - 1 ? "1px solid var(--color-glass-border)" : "none" }}>
              <span className="text-[12px]" style={{ color: "var(--color-text-tertiary)" }}>{d.label}</span>
              <span className="num-display text-[13px] font-bold" style={{ color: d.accent ? "var(--color-text-primary)" : "var(--color-text-secondary)" }}>{d.value}</span>
            </button>
          ))}
        </div>
      )}

      <p className="text-[10px] text-center" style={{ color: "var(--color-text-ghost)" }}>GİB kuruş yuvarlama uyumlu • Satıra tıklayarak kopyalayabilirsiniz</p>
    </section>
  );
}
