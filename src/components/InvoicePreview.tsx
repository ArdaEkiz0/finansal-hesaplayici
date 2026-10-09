import { useStore, MODE_LABELS } from "../store/useStore";
import { X, Printer, Copy, Check } from "lucide-react";
import { useState } from "react";
import { toDecimal, numberToTurkishWords } from "../core/engine";
import Decimal from "decimal.js";

export function InvoicePreview() {
  const show = useStore((s) => s.showInvoice);
  const toggle = useStore((s) => s.toggleInvoice);
  const result = useStore((s) => s.result);
  const mode = useStore((s) => s.mode);
  const input = useStore((s) => s.input);
  const notify = useStore((s) => s.notify);
  const [copied, setCopied] = useState(false);

  if (!show) return null;

  let words = "";
  try {
    if (result) {
      const n = result.display.replace(/\./g, "").replace(",", ".").replace(/[^0-9.-]/g, "");
      words = numberToTurkishWords(new Decimal(n || "0"));
    }
  } catch { words = ""; }

  const lines = result?.rows?.map((r) => `${r.label}: ${r.value}`) ?? [];
  const body = [
    `FİNANSAL HESAPLAYICI — ${MODE_LABELS[mode]}`,
    `Girdi: ${input}`,
    ...lines,
    `Sonuç: ${result?.display ?? "—"}`,
    words ? `Yazıyla: ${words}` : "",
    `Tarih: ${new Date().toLocaleString("tr-TR")}`,
  ].filter(Boolean).join("\n");

  return (
    <div className="fixed inset-0 z-[9998] flex items-center justify-center p-4 anim-scale-in"
      style={{ background: "rgba(0,0,0,.55)", backdropFilter: "blur(4px)" }} onClick={toggle}>
      <div className="w-[440px] max-w-full rounded-2xl p-0 overflow-hidden" style={{ background: "#fff", color: "#111" }} onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between px-5 py-3" style={{ borderBottom: "1px solid #eee" }}>
          <div>
            <div className="text-[14px] font-bold">Fatura Önizleme</div>
            <div className="text-[11px]" style={{ color: "#666" }}>{MODE_LABELS[mode]} • {new Date().toLocaleString("tr-TR")}</div>
          </div>
          <button onClick={toggle} className="p-1.5 rounded-lg" style={{ color: "#666" }}><X size={15} /></button>
        </div>
        <div className="px-5 py-4 space-y-1.5" id="invoice-body">
          <div className="flex justify-between text-[12px]"><span style={{ color: "#666" }}>Girdi</span><span className="font-mono font-bold">{input || "—"}</span></div>
          {result?.rows?.map((r, i) => (
            <div key={i} className="flex justify-between text-[12px]" style={i === 0 ? { borderTop: "1px dashed #ddd", paddingTop: 8 } : undefined}>
              <span style={{ color: "#666" }}>{r.label}</span>
              <span className="font-mono font-bold">{r.value}</span>
            </div>
          ))}
          <div className="flex justify-between items-center pt-2" style={{ borderTop: "2px solid #111" }}>
            <span className="text-[13px] font-bold">TOPLAM</span>
            <span className="font-mono font-bold text-[18px]">{result?.display ?? "—"}</span>
          </div>
          {words && <p className="text-[11px] italic pt-1" style={{ color: "#555" }}>Yazıyla: {words}</p>}
        </div>
        <div className="flex gap-2 px-5 pb-5">
          <button onClick={() => window.print()} className="flex-1 py-2 rounded-xl text-[13px] font-bold flex items-center justify-center gap-1.5" style={{ background: "#111", color: "#fff" }}>
            <Printer size={14} /> Yazdır
          </button>
          <button onClick={async () => { try { await navigator.clipboard.writeText(body); } catch {} setCopied(true); notify("Fatura metni kopyalandı", "success"); setTimeout(() => setCopied(false), 1200); }}
            className="flex-1 py-2 rounded-xl text-[13px] font-bold flex items-center justify-center gap-1.5" style={{ background: "#f1f3f6", color: "#111" }}>
            {copied ? <Check size={14} /> : <Copy size={14} />} {copied ? "Oldu" : "Metni kopyala"}
          </button>
        </div>
      </div>
    </div>
  );
}

export function unusedInvoiceHelper(v: string) {
  return toDecimal(v);
}
