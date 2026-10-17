import { useState, useMemo, useEffect, useRef } from "react";
import { Search, CornerDownLeft } from "lucide-react";
import { useStore, MODE_LABELS, type CalcMode } from "../store/useStore";

interface Komut {
  id: string;
  label: string;
  hint: string;
  kos: () => void;
}

/** Ctrl+K komut paleti: modlar ve paneller tek yerden erişilir. */
export function CommandPalette() {
  const show = useStore((s) => s.showPalette);
  const toggle = useStore((s) => s.togglePalette);
  const setMode = useStore((s) => s.setMode);
  const [query, setQuery] = useState("");
  const [idx, setIdx] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const komutlar = useMemo<Komut[]>(() => [
    ...Object.entries(MODE_LABELS).map(([m, l]) => ({
      id: `mode:${m}`,
      label: `Mod: ${l}`,
      hint: MODE_LABELS[m as CalcMode],
      kos: () => setMode(m as CalcMode),
    })),
    { id: "pan:donemler", label: "Dönem KDV arşivini aç", hint: "Panel", kos: () => useStore.getState().toggleDonemler() },
    { id: "pan:cariler", label: "Carileri aç", hint: "Panel", kos: () => useStore.getState().toggleCariler() },
    { id: "pan:oranlar", label: "2026 pratik oranlar", hint: "Panel", kos: () => useStore.getState().toggleRates() },
    { id: "pan:fis", label: "Fiş / fatura yazdır", hint: "Panel", kos: () => useStore.getState().toggleInvoice() },
    { id: "pan:widget", label: "Widget'ı aç/kapat (Alt+Shift+K)", hint: "Panel", kos: () => useStore.getState().toggleWidget() },
    { id: "pan:ayarlar", label: "Ayarlar", hint: "Panel", kos: () => useStore.getState().toggleSettings() },
    { id: "pan:yardim", label: "Yardım", hint: "Panel", kos: () => useStore.getState().toggleHelp() },
    { id: "pan:tema", label: "Temayı değiştir", hint: "Panel", kos: () => useStore.getState().setTheme(useStore.getState().theme === "dark" ? "light" : "dark") },
  ], [setMode]);

  const filtreli = useMemo(() => {
    const q = query.toLocaleLowerCase("tr").trim();
    if (!q) return komutlar;
    return komutlar.filter((k) => k.label.toLocaleLowerCase("tr").includes(q));
  }, [komutlar, query]);

  useEffect(() => {
    if (show) {
      setQuery("");
      setIdx(0);
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }, [show]);

  if (!show) return null;
  const list = filtreli;

  const kos = (id: string) => {
    toggle();
    const k = komutlar.find((x) => x.id === id);
    if (k) k.kos();
  };

  return (
    <div className="fixed inset-0 z-[10000] flex items-start justify-center pt-[14vh] p-4" style={{ background: "rgba(0,0,0,.5)", backdropFilter: "blur(3px)" }} onClick={toggle}>
      <div className="w-[440px] max-w-full rounded-2xl overflow-hidden anim-scale-in" style={{ background: "var(--color-bg-surface)", border: "1px solid var(--color-glass-border)" }}
        onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center gap-2 px-3 py-2.5" style={{ borderBottom: "1px solid var(--color-glass-border)" }}>
          <Search size={14} style={{ color: "var(--color-text-ghost)" }} />
          <input ref={inputRef} value={query} placeholder="Mod veya panel ara… (Dönem, Cari, Oranlar)"
            onChange={(e) => { setQuery(e.target.value); setIdx(0); }}
            onKeyDown={(e) => {
              if (e.key === "ArrowDown") { e.preventDefault(); setIdx((i) => Math.min(i + 1, list.length - 1)); }
              else if (e.key === "ArrowUp") { e.preventDefault(); setIdx((i) => Math.max(i - 1, 0)); }
              else if (e.key === "Enter" && list[idx]) { e.preventDefault(); kos(list[idx].id); }
              else if (e.key === "Escape") { e.preventDefault(); toggle(); }
            }}
            className="flex-1 bg-transparent text-[13px] font-semibold outline-none"
            style={{ color: "var(--color-text-primary)" }} />
        </div>
        <div className="max-h-[46vh] overflow-y-auto py-1">
          {list.length === 0 ? (
            <p className="px-4 py-6 text-center text-[12px]" style={{ color: "var(--color-text-ghost)" }}>Eşleşme yok</p>
          ) : (
            list.map((k, i) => (
              <button key={k.id} onClick={() => kos(k.id)} onMouseEnter={() => setIdx(i)}
                className="w-full flex items-center justify-between px-3.5 py-2.5 text-left"
                style={{ background: i === idx ? "var(--color-glass)" : "transparent" }}>
                <span className="text-[13px] font-semibold" style={{ color: i === idx ? "var(--color-text-primary)" : "var(--color-text-secondary)" }}>{k.label}</span>
                <span className="flex items-center gap-1.5">
                  <span className="text-[10px] px-1.5 py-0.5 rounded-full" style={{ background: "var(--color-glass)", color: "var(--color-text-ghost)" }}>{k.hint === "Panel" ? "Panel" : "Hesap"}</span>
                  {i === idx && <CornerDownLeft size={11} style={{ color: "var(--color-primary-light)" }} />}
                </span>
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
