import { useStore } from "../store/useStore";
import { X } from "lucide-react";
import { STOPAJ_PRESETS, TEVKIFAT_CODES, TEVKIFAT_LIMIT, RATE_DISCLAIMER, ratesStaleBugun, RATES_YEAR } from "../data/tax";

/** 2026 pratik oranlar rehberi (bilgi amacli). */
export function RatesInfo() {
  const show = useStore((s) => s.showRates);
  const toggle = useStore((s) => s.toggleRates);
  if (!show) return null;

  return (
    <div className="fixed inset-0 z-[9998] flex items-center justify-center p-4" style={{ background: "rgba(0,0,0,.55)", backdropFilter: "blur(4px)" }} onClick={toggle}>
      <div className="w-[420px] max-w-full max-h-[80vh] rounded-2xl p-5 space-y-4 overflow-y-auto anim-scale-in" style={{ background: "var(--color-bg-surface)", border: "1px solid var(--color-glass-border)" }} onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between">
          <span className="text-[14px] font-bold">2026 Pratik Oranlar</span>
          <button onClick={toggle} className="p-1.5 rounded-lg btn-press" style={{ color: "var(--color-text-ghost)" }}><X size={14} /></button>
        </div>

        {ratesStaleBugun() && (
          <div className="px-3 py-2 rounded-xl text-[11px] font-bold"
            style={{ background: "rgba(240,150,35,.14)", color: "var(--color-warning)", border: "1px solid var(--color-warning)" }}>
            Yeni yıl: bu oranlar {RATES_YEAR} doğrulamasıdır — güncel GİB rehberini teyit edin.
          </div>
        )}

        <div>
          <p className="text-[11px] font-bold uppercase tracking-wider mb-1.5" style={{ color: "var(--color-text-ghost)" }}>KDV</p>
          <p className="text-[12px]" style={{ color: "var(--color-text-secondary)" }}>%1 • %10 • %20 + özel oran kutusu. Haricen ekle / dahilden çıkar yönü seçilebilir.</p>
        </div>

        <div>
          <p className="text-[11px] font-bold uppercase tracking-wider mb-1.5" style={{ color: "var(--color-text-ghost)" }}>Stopaj (GVK 94)</p>
          {STOPAJ_PRESETS.map((p) => (
            <div key={p.label} className="flex items-center justify-between py-1" style={{ borderBottom: "1px solid var(--color-glass-border)" }}>
              <span className="text-[12px]" style={{ color: "var(--color-text-secondary)" }}>{p.label} <span style={{ color: "var(--color-text-ghost)" }}>({p.note})</span></span>
              <span className="num-display text-[12px] font-bold">%{p.rate}</span>
            </div>
          ))}
        </div>

        <div>
          <p className="text-[11px] font-bold uppercase tracking-wider mb-1.5" style={{ color: "var(--color-text-ghost)" }}>KDV tevkifat kodları</p>
          <div className="max-h-[180px] overflow-y-auto">
            {TEVKIFAT_CODES.map((c) => (
              <div key={`${c.code}-${c.name}`} className="flex items-center gap-2 py-1" style={{ borderBottom: "1px solid var(--color-glass-border)" }}>
                <span className="text-[11px] font-bold shrink-0" style={{ color: "var(--color-accent)" }}>{c.pay}/10</span>
                <span className="text-[12px] truncate" style={{ color: "var(--color-text-secondary)" }}>{c.code !== "—" ? `${c.code} — ` : ""}{c.name}</span>
              </div>
            ))}
          </div>
          <p className="text-[11px] mt-1.5" style={{ color: "var(--color-text-tertiary)" }}>
            KDV dahil bedeli {TEVKIFAT_LIMIT.value.toLocaleString("tr-TR")} TL'yi aşmayan işlemlerde kısmi tevkifat uygulanmaz.
          </p>
        </div>

        <p className="text-[10px]" style={{ color: "var(--color-text-ghost)" }}>{RATE_DISCLAIMER}</p>
      </div>
    </div>
  );
}
