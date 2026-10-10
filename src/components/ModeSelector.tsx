import { useStore, type CalcMode, MODE_LABELS } from "../store/useStore";
import { Receipt, Scissors, FileWarning, Percent, TrendingUp, BadgePercent, Landmark, Scale, PiggyBank, ShoppingCart, Layers, CalendarRange, AlarmClock, Wallet, Banknote } from "lucide-react";

const MODES: { key: CalcMode; icon: React.ReactNode }[] = [
  { key: "kdv", icon: <Receipt size={15} /> },
  { key: "fiyat", icon: <ShoppingCart size={15} /> },
  { key: "donem", icon: <CalendarRange size={15} /> },
  { key: "stopaj", icon: <Scissors size={15} /> },
  { key: "tevkifat", icon: <FileWarning size={15} /> },
  { key: "gecikme", icon: <AlarmClock size={15} /> },
  { key: "amortisman", icon: <Wallet size={15} /> },
  { key: "doviz", icon: <Banknote size={15} /> },
  { key: "margin", icon: <TrendingUp size={15} /> },
  { key: "markup", icon: <PiggyBank size={15} /> },
  { key: "discount", icon: <BadgePercent size={15} /> },
  { key: "percent", icon: <Percent size={15} /> },
  { key: "compound", icon: <Landmark size={15} /> },
  { key: "kdvCompare", icon: <Scale size={15} /> },
  { key: "batch", icon: <Layers size={15} /> },
];

export function ModeSelector() {
  const mode = useStore((s) => s.mode);
  const setMode = useStore((s) => s.setMode);

  return (
    <div className="shrink-0 px-3 sm:px-4 py-2" style={{ background: "var(--color-bg-base)", borderBottom: "1px solid var(--color-glass-border)" }}>
      <div className="flex gap-1.5 overflow-x-auto scrollbar-hide">
        {MODES.map((m) => {
          const active = mode === m.key;
          return (
            <button
              key={m.key}
              onClick={() => setMode(m.key)}
              className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-[12px] font-semibold whitespace-nowrap btn-press focus-ring transition-all"
              style={
                active
                  ? { background: "var(--color-primary)", color: "#fff", boxShadow: "0 4px 14px rgba(47,123,255,.35)" }
                  : { background: "var(--color-glass)", color: "var(--color-text-secondary)", border: "1px solid var(--color-glass-border)" }
              }
            >
              {m.icon}
              {MODE_LABELS[m.key]}
            </button>
          );
        })}
      </div>
    </div>
  );
}
