import { useStore } from "../store/useStore";
import { CheckCircle, AlertCircle, Info, X } from "lucide-react";

const icons = {
  success: <CheckCircle size={16} className="text-emerald-400" />,
  error: <AlertCircle size={16} className="text-rose-400" />,
  info: <Info size={16} className="text-blue-400" />,
};

export function Notification() {
  const n = useStore((s) => s.notification);
  if (!n) return null;

  return (
    <div className="fixed bottom-6 right-6 z-50 anim-fade-up">
      <div className="glass-strong rounded-2xl shadow-2xl flex items-center gap-3 px-4 py-3 max-w-xs">
        {icons[n.type]}
        <span className="text-sm text-text-primary font-medium flex-1">{n.message}</span>
        <button onClick={() => useStore.setState({ notification: null })}
          className="p-1 rounded-lg text-text-ghost hover:text-text-primary transition-colors">
          <X size={14} />
        </button>
      </div>
    </div>
  );
}
