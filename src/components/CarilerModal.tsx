import { useState } from "react";
import { X, Plus, Trash2, Users, Check } from "lucide-react";
import { useStore } from "../store/useStore";

/** Cari kartları: dönem kayıtları müşteriye bağlanır. */
export function CarilerModal() {
  const show = useStore((s) => s.showCariler);
  const toggle = useStore((s) => s.toggleCariler);
  const cariler = useStore((s) => s.cariler);
  const addCari = useStore((s) => s.addCari);
  const deleteCari = useStore((s) => s.deleteCari);
  const activeCari = useStore((s) => s.activeCari);
  const setActiveCari = useStore((s) => s.setActiveCari);
  const donemler = useStore((s) => s.donemler);
  const [draft, setDraft] = useState("");

  if (!show) return null;

  const ekle = () => {
    addCari(draft);
    setDraft("");
  };

  return (
    <div className="fixed inset-0 z-[9999] flex items-center justify-center p-4" style={{ background: "rgba(0,0,0,.55)", backdropFilter: "blur(4px)" }} onClick={toggle}>
      <div className="w-[380px] max-w-full max-h-[80vh] rounded-2xl p-5 space-y-3 overflow-y-auto anim-scale-in" style={{ background: "var(--color-bg-surface)", border: "1px solid var(--color-glass-border)" }} onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Users size={16} style={{ color: "var(--color-accent)" }} />
            <span className="text-[14px] font-bold">Cariler</span>
          </div>
          <button onClick={toggle} className="p-1.5 rounded-lg btn-press" style={{ color: "var(--color-text-ghost)" }}><X size={14} /></button>
        </div>

        <div className="flex gap-1.5">
          <input autoFocus value={draft} placeholder="Müşteri / firma adı"
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => { if (e.key === "Enter") ekle(); }}
            className="flex-1 px-3 py-2 rounded-xl text-[12px] font-semibold focus-ring"
            style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-primary)" }} />
          <button onClick={ekle} className="px-3 rounded-xl btn-press" style={{ background: "var(--color-primary)", color: "#fff" }}><Plus size={15} /></button>
        </div>

        {cariler.length === 0 ? (
          <p className="text-[11px] py-4 text-center" style={{ color: "var(--color-text-ghost)" }}>
            Cari ekleyin; dönem KDV kayıtları buradan seçtiğiniz müşteriye bağlı arşivlenir.
          </p>
        ) : (
          <div className="space-y-1">
            {cariler.map((c) => {
              const adet = donemler.filter((d) => d.cari === c.name).length;
              const aktif = activeCari === c.id;
              return (
                <div key={c.id} className="flex items-center gap-2 px-3 py-2 rounded-xl group"
                  style={{ background: aktif ? "rgba(47,123,255,.12)" : "var(--color-glass)", border: aktif ? "1px solid var(--color-primary)" : "1px solid var(--color-glass-border)" }}>
                  <button onClick={() => setActiveCari(aktif ? "" : c.id)} className="flex-1 text-left min-w-0">
                    <span className="text-[12px] font-bold truncate block" style={{ color: "var(--color-text-primary)" }}>{c.name}</span>
                    <span className="text-[10px]" style={{ color: "var(--color-text-ghost)" }}>{adet} dönem kaydı</span>
                  </button>
                  {aktif ? (
                    <span className="text-[9px] font-bold px-1.5 py-0.5 rounded-full shrink-0" style={{ background: "var(--color-primary)", color: "#fff" }}>AKTİF</span>
                  ) : (
                    <button onClick={() => setActiveCari(c.id)} title="Aktif yap" className="p-1 shrink-0 opacity-0 group-hover:opacity-100" style={{ color: "var(--color-primary-light)" }}><Check size={12} /></button>
                  )}
                  <button onClick={() => deleteCari(c.id)} title="Sil" className="p-1 shrink-0 opacity-0 group-hover:opacity-100" style={{ color: "var(--color-error)" }}><Trash2 size={12} /></button>
                </div>
              );
            })}
          </div>
        )}

        <p className="text-[10px]" style={{ color: "var(--color-text-ghost)" }}>Aktif cari, Dönem KDV modunda yeni arşiv kaydına otomatik bağlanır.</p>
      </div>
    </div>
  );
}
