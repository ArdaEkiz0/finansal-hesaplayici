import { useStore, type CalcMode } from "../store/useStore";
import { Clock, Trash2, Pin, Download, Search, X } from "lucide-react";
import { useState, useMemo } from "react";

const ml: Record<CalcMode, string> = {
  kdv: "KDV", stopaj: "Stopaj", margin: "Marj", markup: "Kâr Oranı",
  discount: "İndirim", percent: "Yüzde", compound: "Bileşik", kdvCompare: "KDV Karşıl.",
};
const mc: Record<CalcMode, string> = {
  kdv: "bg-blue-400", stopaj: "bg-emerald-400", margin: "bg-violet-400",
  markup: "bg-amber-400", discount: "bg-cyan-400", percent: "bg-rose-400",
  compound: "bg-emerald-400", kdvCompare: "bg-violet-400",
};

export function History() {
  const history = useStore((s) => s.history);
  const togglePin = useStore((s) => s.togglePinHistory);
  const deleteEntry = useStore((s) => s.deleteHistoryEntry);
  const loadEntry = useStore((s) => s.loadHistoryEntry);
  const clearHistory = useStore((s) => s.clearHistory);
  const exportHistory = useStore((s) => s.exportHistory);
  const [search, setSearch] = useState("");

  const filtered = useMemo(() => {
    if (!search) return history;
    const q = search.toLowerCase();
    return history.filter((e) => e.inputs.toLowerCase().includes(q) || e.result.toLowerCase().includes(q) || ml[e.mode].toLowerCase().includes(q));
  }, [history, search]);

  const pinned = filtered.filter((e) => e.pinned);
  const unpinned = filtered.filter((e) => !e.pinned);

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="w-6 h-6 rounded-lg glass flex items-center justify-center">
            <Clock size={12} className="text-text-tertiary" />
          </div>
          <div>
            <h3 className="text-[11px] font-bold text-text-primary">Geçmiş</h3>
            {history.length > 0 && <p className="text-[9px] text-text-ghost">{history.length} işlem</p>}
          </div>
        </div>
        {history.length > 0 && (
          <div className="flex items-center gap-0.5">
            <button onClick={exportHistory} className="p-1.5 rounded-lg glass text-text-ghost hover:text-emerald-400 hover:bg-emerald-500/10 transition-all" title="Dışa Aktar">
              <Download size={12} />
            </button>
            <button onClick={clearHistory} className="p-1.5 rounded-lg glass text-text-ghost hover:text-rose-400 hover:bg-rose-500/10 transition-all" title="Temizle">
              <Trash2 size={12} />
            </button>
          </div>
        )}
      </div>

      {history.length > 3 && (
        <div className="relative">
          <Search size={12} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-ghost" />
          <input type="text" value={search} onChange={(e) => setSearch(e.target.value)} placeholder="Ara..."
            className="w-full glass rounded-xl pl-7 pr-6 py-1.5 text-[11px] text-text-primary placeholder:text-text-ghost focus:outline-none focus:border-brand-500/30 transition-colors" />
          {search && (
            <button onClick={() => setSearch("")} className="absolute right-2 top-1/2 -translate-y-1/2 text-text-ghost hover:text-text-primary">
              <X size={12} />
            </button>
          )}
        </div>
      )}

      {history.length === 0 && (
        <div className="text-center py-8 space-y-2">
          <div className="w-10 h-10 mx-auto rounded-xl glass flex items-center justify-center">
            <Clock size={18} className="text-text-ghost/30" />
          </div>
          <div>
            <p className="text-[11px] font-medium text-text-tertiary">Henüz işlem yok</p>
            <p className="text-[9px] text-text-ghost mt-0.5">Hesapla → Kaydet</p>
          </div>
        </div>
      )}

      {pinned.length > 0 && (
        <div className="space-y-1">
          <div className="flex items-center gap-1 text-[9px] font-bold text-amber-400 uppercase tracking-[0.2em]">
            <Pin size={8} /> Sabitlenmiş
          </div>
          {pinned.map((e) => <Card key={e.id} entry={e} onPin={() => togglePin(e.id)} onDelete={() => deleteEntry(e.id)} onLoad={() => loadEntry(e.id)} />)}
        </div>
      )}

      {unpinned.length > 0 && (
        <div className="space-y-1">
          {pinned.length > 0 && <div className="text-[9px] font-bold text-text-ghost uppercase tracking-[0.2em]">Diğer</div>}
          <div className="space-y-1 max-h-[calc(100vh-300px)] overflow-y-auto">
            {unpinned.map((e) => <Card key={e.id} entry={e} onPin={() => togglePin(e.id)} onDelete={() => deleteEntry(e.id)} onLoad={() => loadEntry(e.id)} />)}
          </div>
        </div>
      )}
    </div>
  );
}

function Card({ entry, onPin, onDelete, onLoad }: {
  entry: { id: string; mode: CalcMode; inputs: string; result: string; detail?: string; timestamp: number; pinned: boolean };
  onPin: () => void; onDelete: () => void; onLoad: () => void;
}) {
  return (
    <div onClick={onLoad}
      className="group glass rounded-xl p-2.5 hover:bg-white/[0.04] transition-all duration-200 cursor-pointer hover:border-cyan-500/20">
      <div className="flex items-start justify-between mb-1">
        <div className="flex items-center gap-1">
          <div className={`w-1.5 h-1.5 rounded-full ${mc[entry.mode]}`} />
          <span className="text-[9px] font-bold text-text-secondary uppercase tracking-wider">{ml[entry.mode]}</span>
        </div>
        <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
          <button onClick={(e) => { e.stopPropagation(); onPin(); }}
            className={`p-0.5 rounded transition-colors ${entry.pinned ? "text-amber-400" : "text-text-ghost hover:text-amber-400"}`}>
            <Pin size={9} />
          </button>
          <button onClick={(e) => { e.stopPropagation(); onDelete(); }}
            className="p-0.5 rounded text-text-ghost hover:text-rose-400 transition-colors">
            <Trash2 size={9} />
          </button>
        </div>
      </div>
      <div className="text-[9px] text-text-ghost font-mono truncate">{entry.inputs}</div>
      <div className="num-display text-xs font-black gradient-text-result">{entry.result}</div>
      {entry.detail && <div className="text-[8px] text-text-ghost mt-0.5 truncate">{entry.detail}</div>}
      <div className="text-[8px] text-text-ghost/50 mt-1">
        {new Date(entry.timestamp).toLocaleString("tr-TR", { day: "2-digit", month: "short", hour: "2-digit", minute: "2-digit" })}
      </div>
    </div>
  );
}
