import { useState } from "react";
import { createPortal } from "react-dom";
import { X, Printer, Copy, Check, FileText } from "lucide-react";
import Decimal from "decimal.js";
import { useStore } from "../store/useStore";
import { formatTurkishNumber, numberToTurkishWords } from "../core/engine";
import { donemAd, type DonemRecord } from "../data/donem";

function num(v: string): Decimal {
  try {
    const d = new Decimal(v || "0");
    return d.isFinite() ? d : new Decimal(0);
  } catch { return new Decimal(0); }
}

/** Yazdirilabilir beyan onzeti (KDV-1 benzeri tek kart) — invoice-portal baski zinciri kullanilir. */
export function BeyanOzeti() {
  const show = useStore((s) => s.showBeyanOzeti);
  const toggle = useStore((s) => s.toggleBeyanOzeti);
  const donemId = useStore((s) => s.beyanDonemId);
  const donemler = useStore((s) => s.donemler);
  const cariler = useStore((s) => s.cariler);
  const notify = useStore((s) => s.notify);
  const [copied, setCopied] = useState(false);

  if (!show) return null;

  const d: DonemRecord | undefined = donemler.find((x) => x.id === donemId);
  if (!d) return null;

  const cariAd = d.cari || cariler.find((c) => c.id === d.cari)?.name;
  const p = useStore.getState().settings.precision;
  const f = (v: string) => formatTurkishNumber(num(v), p);
  const netN = num(d.net);
  const netYazi = numberToTurkishWords(netN.abs());

  const satirlar: { label: string; deger: string; vurgu?: boolean }[] = [
    { label: "Satış matrahı", deger: `${f(d.satis)} ₺` },
    { label: `Hesaplanan KDV (%${d.kdvRate})`, deger: `${f(d.hesaplanan)} ₺` },
    { label: "Alış matrahı", deger: `${f(d.alis)} ₺` },
    { label: `İndirilecek KDV (%${d.kdvRate})`, deger: `${f(d.indirilecek)} ₺` },
    ...(num(d.devredenGiris).gt(0) ? [{ label: "Geçen dönemden devreden", deger: `${f(d.devredenGiris)} ₺` }] : []),
  ];

  const metin = [
    `BEYAN ÖZETİ — DÖNEM: ${donemAd(d)}`,
    `KDV oranı: %${d.kdvRate}`,
    ...satirlar.map((s) => `  ${s.label}: ${s.deger}`),
    `${d.odenecek ? "ÖDENECEK" : "DEVREDEN (doldur)"}: ${f(d.net)} ₺`,
    netYazi ? `Yazıyla: ${netYazi}` : "",
    cariAd ? `Cari: ${cariAd}` : "",
    `Tarih: ${new Date(d.closedAt).toLocaleString("tr-TR")}`,
    "(Resmi belge değildir — beyanname ön kontrol içindir)",
  ].filter(Boolean).join("\n");

  return createPortal(
    <div id="invoice-portal" className="fixed inset-0 z-[9999] flex items-center justify-center p-4 anim-scale-in invoice-screen"
      style={{ background: "rgba(0,0,0,.55)", backdropFilter: "blur(4px)" }} onClick={toggle}>
      <div className="w-[460px] max-w-full rounded-2xl p-0 overflow-hidden invoice-card" style={{ background: "#fff", color: "#111" }} onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between px-5 py-3" style={{ borderBottom: "1px solid #eee" }}>
          <div>
            <div className="text-[14px] font-bold flex items-center gap-1.5"><FileText size={14} /> Beyan Özeti</div>
            <div className="text-[11px]" style={{ color: "#666" }}>{donemAd(d)} • KDV %{d.kdvRate}{cariAd ? ` • ${cariAd}` : ""}</div>
          </div>
          <button onClick={toggle} className="p-1.5 rounded-lg" style={{ color: "#666" }}><X size={15} /></button>
        </div>
        <div className="px-5 py-4 space-y-1.5" id="invoice-body">
          {satirlar.map((s, i) => (
            <div key={i} className="flex justify-between text-[12px]">
              <span style={{ color: "#666" }}>{s.label}</span>
              <span className="font-mono">{s.deger}</span>
            </div>
          ))}
          <div className="flex justify-between items-center pt-3" style={{ borderTop: "2px solid #111" }}>
            <span className="text-[13px] font-bold">{d.odenecek ? "ÖDENECEK KDV" : "SONRAKİ DÖNEME DEVREDEN"}</span>
            <span className="font-mono font-bold text-[17px]" style={{ color: d.odenecek ? "#b45309" : "#047857" }}>{f(d.net)} ₺</span>
          </div>
          <p className="text-[10.5px] italic" style={{ color: "#555" }}>{d.odenecek ? `Yazıyla: ${netYazi}` : `Sonraki döneme devreden taşı — yazıyla: ${netYazi}`}</p>
          <p className="text-[10px] pt-1" style={{ color: "#888", borderTop: "1px dashed #ddd" }}>
            Mükellef: ______________________   •   VKN: ______________
          </p>
          <p className="text-[9.5px]" style={{ color: "#aaa" }}>Resmi belge değildir; beyanname girişinde ön kontrol içindir. Belge serisi/no belgelendirmede işlev görmez.</p>
        </div>
        <div className="flex gap-2 px-5 pb-5 invoice-actions">
          <button onClick={() => window.print()} className="flex-1 py-2 rounded-xl text-[13px] font-bold flex items-center justify-center gap-1.5" style={{ background: "#111", color: "#fff" }}>
            <Printer size={14} /> Yazdır
          </button>
          <button onClick={async () => { try { await navigator.clipboard.writeText(metin); } catch { /* yok */ } setCopied(true); notify("Özet metni kopyalandı", "success"); setTimeout(() => setCopied(false), 1200); }}
            className="flex-1 py-2 rounded-xl text-[13px] font-bold flex items-center justify-center gap-1.5" style={{ background: "#f1f3f6", color: "#111" }}>
            {copied ? <Check size={14} /> : <Copy size={14} />} {copied ? "Oldu" : "Metni kopyala"}
          </button>
        </div>
      </div>
    </div>,
    document.body
  );
}
