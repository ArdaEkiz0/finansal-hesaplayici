import { useStore } from "../store/useStore";

export function Notification() {
  const n = useStore((s) => s.notification);
  if (!n) return null;
  const bg = n.type === "success" ? "#16b981" : n.type === "error" ? "#f4506a" : "#2f7bff";
  return (
    <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-[9999] anim-fade-up">
      <div className="px-4 py-2 rounded-xl text-[12px] font-bold shadow-lg" style={{ background: bg, color: "#fff" }}>{n.message}</div>
    </div>
  );
}
