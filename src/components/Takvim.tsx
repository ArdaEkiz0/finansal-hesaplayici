import { useMemo } from "react";
import { X, CalendarDays, TriangleAlert } from "lucide-react";
import { useStore } from "../store/useStore";
import { TAKVIM_TIPLERI, takvimListesi, gunKaldi, TAKVIM_UYARI } from "../data/takvim";

const GUN_AD = ["Pazar", "Pazartesi", "Salı", "Çarşamba", "Perşembe", "Cuma", "Cumartesi"];

export function Takvim() {
  const show = useStore((s) => s.showTakvim);
  const toggle = useStore((s) => s.toggleTakvim);
  const bugun = useMemo(() => new Date(), []);

  const kayitlar = useMemo(() => takvimListesi(bugun).slice(0, 14), [bugun]);

  if (!show) return null;

  return (
    <div className="fixed inset-0 z-[9998] flex items-center justify-center p-4" style={{ background: "rgba(0,0,0,.55)", backdropFilter: "blur(4px)" }} onClick={toggle}>
      <div className="w-[520px] max-w-full max-h-[84vh] rounded-2xl p-5 space-y-3 overflow-y-auto anim-scale-in" style={{ background: "var(--color-bg-surface)", border: "1px solid var(--color-glass-border)" }} onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <CalendarDays size={16} style={{ color: "var(--color-primary-light)" }} />
            <span className="text-[14px] font-bold">Vergi Takvimi</span>
            <span className="text-[11px]" style={{ color: "var(--color-text-ghost)" }}>{bugun.toLocaleDateString("tr-TR", { day: "2-digit", month: "long", year: "numeric" })}</span>
          </div>
          <button onClick={toggle} className="p-1.5 rounded-lg btn-press" style={{ color: "var(--color-text-ghost)" }}><X size={14} /></button>
        </div>

        <div className="space-y-1.5">
          {kayitlar.map((k) => {
            const d = gunKaldi(k.tarih, bugun);
            const tp = TAKVIM_TIPLERI[k.tip];
            const t = new Date(k.tarih + "T00:00:00");
            return (
              <div key={k.id} className="flex items-center gap-3 rounded-xl px-3 py-2.5 glass-card"
                style={d <= 2 ? { borderColor: "rgba(245,166,46,.45)" } : undefined}>
                <div className="w-11 shrink-0 text-center">
                  <span className="num-display text-[14px] font-bold block" style={{ color: tp.renk }}>{t.toLocaleDateString("tr-TR", { day: "2-digit", month: "2-digit" })}</span>
                  <span className="text-[9px]" style={{ color: "var(--color-text-ghost)" }}>{GUN_AD[t.getDay()]}</span>
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-1.5 flex-wrap">
                    <span className="text-[12.5px] font-bold">{k.ad}</span>
                    <span className="text-[9px] px-1.5 py-0.5 rounded font-bold" style={{ background: "var(--color-glass)", color: tp.renk }}>{tp.ad}</span>
                  </div>
                  <p className="text-[10.5px] leading-snug mt-0.5" style={{ color: "var(--color-text-tertiary)" }}>{k.açıklama} ({k.dönemi})</p>
                </div>
                <span className="shrink-0 text-[10px] px-2 py-1 rounded-full font-bold"
                  style={d === 0
                    ? { background: "rgba(244,83,106,.14)", color: "var(--color-error)" }
                    : d <= 2
                      ? { background: "rgba(245,166,46,.16)", color: "var(--color-warning)" }
                      : d <= 7
                        ? { background: "rgba(47,123,255,.12)", color: "var(--color-primary-light)" }
                        : { background: "var(--color-glass)", color: "var(--color-text-ghost)" }}>
                  {d === 0 ? "BUGÜN" : `${d} gün`}
                </span>
              </div>
            );
          })}
        </div>

        <div className="flex items-start gap-2 rounded-xl px-3 py-2" style={{ background: "rgba(245,166,46,.08)", border: "1px solid rgba(245,166,46,.25)" }}>
          <TriangleAlert size={13} className="shrink-0 mt-0.5" style={{ color: "var(--color-warning)" }} />
          <p className="text-[10.5px] leading-relaxed" style={{ color: "var(--color-text-tertiary)" }}>{TAKVIM_UYARI}</p>
        </div>
      </div>
    </div>
  );
}
