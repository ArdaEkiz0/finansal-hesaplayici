import { useStore, type CalcMode } from "../store/useStore";
import { Receipt, TrendingDown, BarChart3, TrendingUp, Tag, Percent, Landmark, GitCompareArrows } from "lucide-react";

const modes: { key: CalcMode; label: string; icon: React.ReactNode; color: string }[] = [
  { key: "kdv", label: "KDV", icon: <Receipt size={14} />, color: "#3b82f6" },
  { key: "stopaj", label: "Stopaj", icon: <TrendingDown size={14} />, color: "#10b981" },
  { key: "margin", label: "Marj", icon: <BarChart3 size={14} />, color: "#8b5cf6" },
  { key: "markup", label: "Kâr Oranı", icon: <TrendingUp size={14} />, color: "#f59e0b" },
  { key: "discount", label: "İndirim", icon: <Tag size={14} />, color: "#06b6d4" },
  { key: "percent", label: "Yüzde", icon: <Percent size={14} />, color: "#f43f5e" },
  { key: "compound", label: "Bileşik", icon: <Landmark size={14} />, color: "#10b981" },
  { key: "kdvCompare", label: "KDV Karşıl.", icon: <GitCompareArrows size={14} />, color: "#8b5cf6" },
];

export function ModeSelector() {
  const mode = useStore((s) => s.mode);
  const setMode = useStore((s) => s.setMode);

  return (
    <div className="flex gap-1 overflow-x-auto pb-1 scrollbar-hide">
      {modes.map((m) => {
        const active = mode === m.key;
        return (
          <button key={m.key} onClick={() => setMode(m.key)}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[11px] font-semibold whitespace-nowrap transition-all duration-150 shrink-0"
            style={{
              background: active ? `${m.color}15` : "transparent",
              color: active ? m.color : "var(--color-text-tertiary)",
              border: active ? `1px solid ${m.color}25` : "1px solid transparent",
            }}>
            {m.icon}
            {m.label}
          </button>
        );
      })}
    </div>
  );
}
