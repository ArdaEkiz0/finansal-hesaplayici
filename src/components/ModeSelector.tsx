import { useStore, type CalcMode, MODE_LABELS } from "../store/useStore";
import { Receipt, Scissors, FileWarning, Percent, TrendingUp, BadgePercent, Landmark, Scale, PiggyBank, ShoppingCart, Layers, CalendarRange, AlarmClock, Wallet, Banknote } from "lucide-react";

const ONU: { key: CalcMode; icon: React.ReactNode }[] = [
  { key: "kdv", icon: <Receipt size={15} /> },
  { key: "donem", icon: <CalendarRange size={15} /> },
  { key: "fiyat", icon: <ShoppingCart size={15} /> },
  { key: "stopaj", icon: <Scissors size={15} /> },
  { key: "tevkifat", icon: <FileWarning size={15} /> },
];

const DESTEK: { key: CalcMode; icon: React.ReactNode }[] = [
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

function Chip({ m, active, onClick }: { m: { key: CalcMode; icon: React.ReactNode }; active: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      title={MODE_LABELS[m.key]}
      className="flex items-center gap-2 h-9 px-3.5 rounded-xl text-[12.5px] font-semibold whitespace-nowrap btn-press focus-ring"
      style={active
        ? { background: "linear-gradient(135deg, var(--color-primary) 0%, #1e5fe0 100%)", color: "#fff", boxShadow: "var(--shadow-glow)", border: "1px solid transparent" }
        : { background: "var(--color-glass)", color: "var(--color-text-secondary)", border: "1px solid var(--color-glass-border)" }}
    >
      <span style={{ color: active ? "#fff" : "var(--color-primary-light)" }}>{m.icon}</span>
      {MODE_LABELS[m.key]}
    </button>
  );
}

export function ModeSelector() {
  const mode = useStore((s) => s.mode);
  const setMode = useStore((s) => s.setMode);

  return (
    <div className="shrink-0 px-3 sm:px-5 py-2.5 relative z-10">
      <div className="max-w-[1440px] mx-auto flex flex-wrap items-center gap-2">
        {ONU.map((m) => <Chip key={m.key} m={m} active={mode === m.key} onClick={() => setMode(m.key)} />)}
        <span className="hidden lg:block w-px h-6 mx-1" style={{ background: "var(--color-glass-border)" }} />
        {DESTEK.map((m) => <Chip key={m.key} m={m} active={mode === m.key} onClick={() => setMode(m.key)} />)}
      </div>
    </div>
  );
}
