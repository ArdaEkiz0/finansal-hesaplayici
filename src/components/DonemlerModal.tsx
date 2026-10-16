import { useState, useMemo } from "react";
import Decimal from "decimal.js";
import { X, Trash2, FolderArchive, Users } from "lucide-react";
import { useStore } from "../store/useStore";
import { formatTurkishNumber } from "../core/engine";
import { donemAd, donemOzet, type DonemRecord } from "../data/donem";

function parseNum(v: string): Decimal {
  const t = (v || "").trim();
  try {
    const d = new Decimal(t || "0");
    return d.isFinite() ? d : new Decimal(0);
  } catch {
    return new Decimal(0);
  }
}

const td = "px-2 py-1.5 text-[11px]";
const th = "px-2 py-1.5 text-[10px] font-bold uppercase tracking-wider text-left";

/** Dönem KDV arşivi: kapanan aylar devreden zinciriyle. */
export function DonemlerModal() {
  const show = useStore((s) => s.showDonemler);
  const toggle = useStore((s) => s.toggleDonemler);
  const donemler = useStore((s) => s.donemler);
  const deleteDonem = useStore((s) => s.deleteDonem);
  const toggleCariler = useStore((s) => s.toggleCariler);
  const cariler = useStore((s) => s.cariler);
  const [filter, setFilter] = useState<string>("");

  const cariAdlari = useMemo(() => {
    const set = new Set<string>();
    for (const d of donemler) if (d.cari) set.add(d.cari);
    for (const c of cariler) set.add(c.name);
    return [...set];
  }, [donemler, cariler]);

  const secili = useMemo(() => (filter ? donemler.filter((d) => d.cari === filter) : donemler), [donemler, filter]);
  const ozet = useMemo(() => donemOzet(secili), [secili]);
  const p = useStore((s) => s.settings.precision);

  if (!show) return null;
  const fmt = (v: string | Decimal) => formatTurkishNumber(typeof v === "string" ? parseNum(v) : v, p);

  return (
    <div className="fixed inset-0 z-[9998] flex items-center justify-center p-4" style={{ background: "rgba(0,0,0,.55)", backdropFilter: "blur(4px)" }} onClick={toggle}>
      <div className="w-[640px] max-w-full max-h-[84vh] rounded-2xl p-5 space-y-3 overflow-y-auto anim-scale-in" style={{ background: "var(--color-bg-surface)", border: "1px solid var(--color-glass-border)" }} onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <FolderArchive size={16} style={{ color: "var(--color-primary-light)" }} />
            <span className="text-[14px] font-bold">Dönem KDV Arşivi</span>
            <span className="text-[11px] px-1.5 rounded-full" style={{ background: "var(--color-glass)", color: "var(--color-text-ghost)" }}>{secili.length} dönem</span>
          </div>
          <div className="flex items-center gap-1">
            <button onClick={toggleCariler} title="Carileri yönet" className="p-1.5 rounded-lg btn-press" style={{ color: "var(--color-text-secondary)" }}><Users size={14} /></button>
            <button onClick={toggle} className="p-1.5 rounded-lg btn-press" style={{ color: "var(--color-text-ghost)" }}><X size={14} /></button>
          </div>
        </div>

        {cariAdlari.length > 0 && (
          <div className="flex flex-wrap gap-1">
            <FilterChip active={filter === ""} onClick={() => setFilter("")} label="Tümü" />
            {cariAdlari.map((n) => <FilterChip key={n} active={filter === n} onClick={() => setFilter(n)} label={n} />)}
          </div>
        )}

        {donemler.length === 0 ? (
          <div className="py-8 text-center">
            <p className="text-[13px] font-semibold" style={{ color: "var(--color-text-secondary)" }}>Henüz dönem arşivlenmedi</p>
            <p className="text-[11px] mt-1" style={{ color: "var(--color-text-ghost)" }}>Dönem KDV modunda değerleri girip arşivle butonuyla kapatın. Devreden KDV sonraki aya otomatik taşınır.</p>
          </div>
        ) : (
          <div className="rounded-xl overflow-hidden" style={{ border: "1px solid var(--color-glass-border)" }}>
            <table className="w-full" style={{ borderCollapse: "collapse" }}>
              <thead style={{ background: "var(--color-glass)" }}>
                <tr>
                  <th className={th}>Dönem</th>
                  <th className={th}>Cari</th>
                  <th className={th} style={{ textAlign: "right" }}>Matrah S / A</th>
                  <th className={th} style={{ textAlign: "right" }}>Hesaplanan</th>
                  <th className={th} style={{ textAlign: "right" }}>İndirilecek</th>
                  <th className={th} style={{ textAlign: "right" }}>Giren devreden</th>
                  <th className={th} style={{ textAlign: "right" }}>Net</th>
                  <th className={th}></th>
                </tr>
              </thead>
              <tbody>
                {secili.map((d: DonemRecord) => (
                  <tr key={d.id} style={{ borderTop: "1px solid var(--color-glass-border)" }}>
                    <td className={td}><span className="font-bold num-display">{donemAd(d)}</span><span className="ml-1 text-[10px]" style={{ color: "var(--color-text-ghost)" }}>%{d.kdvRate}</span></td>
                    <td className={td} style={{ color: "var(--color-accent)" }}>{d.cari || "—"}</td>
                    <td className={`${td} num-display`} style={{ textAlign: "right", color: "var(--color-text-secondary)" }}>{fmt(d.satis)} / {fmt(d.alis)}</td>
                    <td className={`${td} num-display`} style={{ textAlign: "right" }}>{fmt(d.hesaplanan)}</td>
                    <td className={`${td} num-display`} style={{ textAlign: "right" }}>{fmt(d.indirilecek)}</td>
                    <td className={`${td} num-display`} style={{ textAlign: "right", color: "var(--color-text-ghost)" }}>{fmt(d.devredenGiris)}</td>
                    <td className={`${td} num-display font-bold`} style={{ textAlign: "right", color: d.odenecek ? "var(--color-warning)" : "var(--color-success)" }}>
                      {fmt(d.net)} <span className="text-[9px] font-bold">{d.odenecek ? "ÖDENECEK" : "DEVREDEN"}</span>
                    </td>
                    <td className={td}>
                      <button onClick={() => deleteDonem(d.id)} title="Dönemi sil" className="p-1 rounded" style={{ color: "var(--color-error)" }}><Trash2 size={12} /></button>
                    </td>
                  </tr>
                ))}
              </tbody>
              <tfoot style={{ background: "var(--color-glass)" }}>
                <tr>
                  <td className={`${td} font-bold`} colSpan={3}>TOPLAM ({secili.length})</td>
                  <td className={`${td} num-display font-bold`} style={{ textAlign: "right" }}>{fmt(ozet.hesaplanan)}</td>
                  <td className={`${td} num-display font-bold`} style={{ textAlign: "right" }}>{fmt(ozet.indirilecek)}</td>
                  <td className={td}></td>
                  <td className={`${td} num-display font-bold`} style={{ textAlign: "right" }}>{fmt(ozet.net)}</td>
                  <td className={td}></td>
                </tr>
              </tfoot>
            </table>
          </div>
        )}

        {donemler.length > 0 && (
          <p className="text-[10px]" style={{ color: "var(--color-text-ghost)" }}>
            Ödenecek {fmt(ozet.odenecekNet)} ₺ • Devreden {fmt(ozet.devredenNet)} ₺ — yeni dönemde "geçen aydan doldur" ile devreden otomatik gelir.
          </p>
        )}
      </div>
    </div>
  );
}

function FilterChip({ active, onClick, label }: { active: boolean; onClick: () => void; label: string }) {
  return (
    <button onClick={onClick} className="px-2.5 py-1 rounded-full text-[11px] font-semibold btn-press"
      style={active
        ? { background: "var(--color-primary)", color: "#fff" }
        : { background: "var(--color-glass)", color: "var(--color-text-secondary)", border: "1px solid var(--color-glass-border)" }}>
      {label}
    </button>
  );
}
