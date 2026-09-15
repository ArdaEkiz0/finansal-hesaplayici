import { useStore } from "../store/useStore";
import { Search, Pin, Trash2, Download } from "lucide-react";
import { useState, useMemo } from "react";
import type { HistoryEntry } from "../store/useStore";

export function History() {
  const history = useStore((s) => s.history);
  const togglePinHistory = useStore((s) => s.togglePinHistory);
  const clearHistory = useStore((s) => s.clearHistory);
  const exportHistory = useStore((s) => s.exportHistory);

  const [query, setQuery] = useState("");

  const filtered = useMemo(() => {
    if (!query) return history;
    const q = query.toLowerCase();
    return history.filter((e) =>
      e.inputs.toLowerCase().includes(q) ||
      e.result.toLowerCase().includes(q) ||
      e.mode.toLowerCase().includes(q)
    );
  }, [history, query]);

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between px-1">
        <span className="text-[11px] font-bold uppercase tracking-wider" style={{ color: "var(--color-text-ghost)" }}>Geçmiş</span>
        <div className="flex gap-1">
          <button onClick={exportHistory} className="p-1 rounded transition-colors" style={{ color: "var(--color-text-ghost)" }}
            onMouseEnter={(e) => (e.currentTarget.style.color = "var(--color-text-secondary)")}
            onMouseLeave={(e) => (e.currentTarget.style.color = "var(--color-text-ghost)")}>
            <Download size={12} />
          </button>
          <button onClick={clearHistory} className="p-1 rounded transition-colors" style={{ color: "var(--color-text-ghost)" }}
            onMouseEnter={(e) => (e.currentTarget.style.color = "#f43f5e")}
            onMouseLeave={(e) => (e.currentTarget.style.color = "var(--color-text-ghost)")}>
            <Trash2 size={12} />
          </button>
        </div>
      </div>

      <div className="relative">
        <Search size={12} className="absolute left-2.5 top-1/2 -translate-y-1/2" style={{ color: "var(--color-text-ghost)" }} />
        <input type="text" placeholder="Ara..." value={query} onChange={(e) => setQuery(e.target.value)}
          className="w-full pl-7 pr-3 py-1.5 rounded-lg text-[11px] focus:outline-none focus:ring-1 transition-all"
          style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-primary)" }}
          onFocus={(e) => (e.currentTarget.style.borderColor = "#3b82f640")}
          onBlur={(e) => (e.currentTarget.style.borderColor = "var(--color-glass-border)")}
        />
      </div>

      <div className="space-y-1">
        {filtered.length === 0 ? (
          <p className="text-center py-6 text-[11px]" style={{ color: "var(--color-text-ghost)" }}>Kayıt yok</p>
        ) : (
          filtered.slice(0, 50).map((item: HistoryEntry) => (
            <div key={item.id} className="group rounded-lg p-2 transition-colors cursor-default"
              style={{ background: item.pinned ? "#3b82f608" : "transparent" }}
              onMouseEnter={(e) => (e.currentTarget.style.background = "var(--color-glass-hover)")}
              onMouseLeave={(e) => (e.currentTarget.style.background = item.pinned ? "#3b82f608" : "transparent")}>
              <div className="flex items-start justify-between gap-2">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-1.5">
                    <span className="text-[10px] font-bold px-1.5 py-0.5 rounded"
                      style={{ background: "var(--color-glass)", color: "var(--color-text-tertiary)" }}>
                      {item.mode.toUpperCase()}
                    </span>
                    {item.pinned && <Pin size={9} style={{ color: "#3b82f6" }} />}
                  </div>
                  <p className="text-[11px] font-medium mt-1 truncate" style={{ color: "var(--color-text-secondary)" }}>{item.inputs}</p>
                  <p className="num-display text-xs font-bold mt-0.5" style={{ color: "var(--color-text-primary)" }}>{item.result}</p>
                </div>
                <button onClick={() => togglePinHistory(item.id)} className="p-1 rounded opacity-0 group-hover:opacity-100 transition-all"
                  style={{ color: item.pinned ? "#3b82f6" : "var(--color-text-ghost)" }}>
                  <Pin size={10} />
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
