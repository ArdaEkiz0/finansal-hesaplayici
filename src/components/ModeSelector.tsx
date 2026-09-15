import { useStore, type CalcMode } from "../store/useStore";
import { Receipt, TrendingDown, BarChart3, TrendingUp, Tag, Percent, Landmark, GitCompareArrows } from "lucide-react";

const modes: { key: CalcMode; label: string; icon: React.ReactNode; activeBg: string }[] = [
  { key: "kdv", label: "KDV", icon: <Receipt size={16} />, activeBg: "bg-blue-500/15 border-blue-500/30 text-blue-400" },
  { key: "stopaj", label: "Stopaj", icon: <TrendingDown size={16} />, activeBg: "bg-emerald-500/15 border-emerald-500/30 text-emerald-400" },
  { key: "margin", label: "Marj", icon: <BarChart3 size={16} />, activeBg: "bg-violet-500/15 border-violet-500/30 text-violet-400" },
  { key: "markup", label: "Kâr Oranı", icon: <TrendingUp size={16} />, activeBg: "bg-amber-500/15 border-amber-500/30 text-amber-400" },
  { key: "discount", label: "İndirim", icon: <Tag size={16} />, activeBg: "bg-cyan-500/15 border-cyan-500/30 text-cyan-400" },
  { key: "percent", label: "Yüzde", icon: <Percent size={16} />, activeBg: "bg-rose-500/15 border-rose-500/30 text-rose-400" },
  { key: "compound", label: "Bileşik", icon: <Landmark size={16} />, activeBg: "bg-emerald-500/15 border-emerald-500/30 text-emerald-400" },
  { key: "kdvCompare", label: "KDV Karşıl.", icon: <GitCompareArrows size={16} />, activeBg: "bg-violet-500/15 border-violet-500/30 text-violet-400" },
];

export function ModeSelector() {
  const mode = useStore((s) => s.mode);
  const setMode = useStore((s) => s.setMode);

  return (
    <div className="grid grid-cols-4 gap-1.5 sm:grid-cols-8">
      {modes.map((m) => {
        const active = mode === m.key;
        return (
          <button
            key={m.key}
            onClick={() => setMode(m.key)}
            className={`group relative flex flex-col items-center gap-1 py-2.5 px-1.5 rounded-xl text-[10px] font-semibold transition-all duration-200 ${
              active
                ? `${m.activeBg} border shadow-lg scale-[1.02]`
                : "glass border-transparent text-text-secondary hover:text-text-primary hover:bg-white/[0.04]"
            }`}
          >
            {active && <div className="absolute -bottom-0.5 left-1/2 -translate-x-1/2 w-5 h-[2px] rounded-full bg-current opacity-60" />}
            <span className="transition-transform duration-200 group-hover:scale-110">{m.icon}</span>
            <span className="leading-tight">{m.label}</span>
          </button>
        );
      })}
    </div>
  );
}
