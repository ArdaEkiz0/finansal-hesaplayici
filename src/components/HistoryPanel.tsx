import { useStore, type HistoryEntry } from "../store/useStore";
import { Search, Pin, Trash2, Download, RotateCcw } from "lucide-react";
import { useState, useMemo } from "react";

export function HistoryPanel() {
  const history = useStore((s) => s.history);
  const togglePinHistory = useStore((s) => s.togglePinHistory);
  const deleteHistoryEntry = useStore((s) => s.deleteHistoryEntry);
  const loadHistoryEntry = useStore((s) => s.loadHistoryEntry);
  const clearHistory = useStore((s) => s.clearHistory);
  const exportHistory = useStore((s) => s.exportHistory);
  const [query, setQuery] = useState("");

  const filtered = useMemo(() => {
    const pinned = history.filter((e) => e.pinned);
    const rest = history.filter((e) => !e.pinned);
    const ordered = [...pinned, ...rest];
    if (!query) return ordered;
    const q = query.toLowerCase();
    return ordered.filter((e) => e.inputs.toLowerCase().includes(q) || e.result.toLowerCase().includes(q) || e.mode.toLowerCase().includes(q));
  }, [history, query]);

  return (
    <div className="h-full flex flex-col min-h-0 p-3 gap-2.5">
      <div className="flex items-center justify-between">
        <span className="text-[11px] font-bold uppercase tracking-wider" style={{ color: "var(--color-text-ghost)" }}>Geçmiş</span>
        <div className="flex gap-0.5">
          <button title="Dışa aktar (CSV)" onClick={exportHistory} className="p-1.5 rounded-lg btn-press" style={{ color: "var(--color-text-ghost)" }}><Download size={13} /></button>
          <button title="Temizle" onClick={clearHistory} className="p-1.5 rounded-lg btn-press" style={{ color: "var(--color-text-ghost)" }}><Trash2 size={13} /></button>
        </div>
      </div>

      <div className="relative shrink-0">
        <Search size={13} className="absolute left-2.5 top-1/2 -translate-y-1/2" style={{ color: "var(--color-text-ghost)" }} />
        <input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Ara..."
          className="w-full pl-8 pr-3 py-2 rounded-xl text-[12px] focus-ring"
          style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-primary)" }} />
      </div>

      <div className="flex-1 overflow-y-auto space-y-1 min-h-0">
        {filtered.length === 0 ? (
          <p className="text-center py-8 text-[12px]" style={{ color: "var(--color-text-ghost)" }}>Henüz kayıt yok.<br />Hesap yapıp Enter'a basın.</p>
        ) : (
          filtered.slice(0, 100).map((item: HistoryEntry) => (
            <div key={item.id} className="group rounded-xl p-2.5 btn-press cursor-pointer"
              style={{ background: item.pinned ? "rgba(47,123,255,.1)" : "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
              <button className="w-full text-left" onClick={() => loadHistoryEntry(item.id)} title="Geri yüklemek için tıkla">
                <div className="flex items-center gap-1.5">
                  <span className="text-[9px] font-bold px-1.5 py-0.5 rounded" style={{ background: "var(--color-glass-active)", color: "var(--color-text-tertiary)" }}>{item.mode.toUpperCase()}</span>
                  {item.pinned && <Pin size={9} style={{ color: "var(--color-primary-light)" }} />}
                  <span className="ml-auto text-[9px]" style={{ color: "var(--color-text-ghost)" }}>{new Date(item.timestamp).toLocaleDateString("tr-TR", { day: "2-digit", month: "2-digit" })}</span>
                </div>
                <p className="text-[11px] mt-1 truncate" style={{ color: "var(--color-text-secondary)" }}>{item.inputs}</p>
                <p className="num-display text-[13px] font-bold mt-0.5" style={{ color: "var(--color-text-primary)" }}>{item.result}</p>
              </button>
              <div className="hidden group-hover:flex gap-1 mt-1">
                <button onClick={() => togglePinHistory(item.id)} className="p-1 rounded text-[10px] font-bold" style={{ color: "var(--color-primary-light)" }}><Pin size={10} /></button>
                <button onClick={() => loadHistoryEntry(item.id)} className="p-1 rounded text-[10px] font-bold" style={{ color: "var(--color-text-tertiary)" }}><RotateCcw size={10} /></button>
                <button onClick={() => deleteHistoryEntry(item.id)} className="p-1 rounded text-[10px] font-bold" style={{ color: "var(--color-error)" }}><Trash2 size={10} /></button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
