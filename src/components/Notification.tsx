import { useStore } from "../store/useStore";
import { useEffect } from "react";

export function Notification() {
  const notification = useStore((s) => s.notification);

  if (!notification) return null;

  const colors: Record<string, string> = {
    success: "#10b981",
    error: "#f43f5e",
    info: "#3b82f6",
  };

  return (
    <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-[9999] anim-fade-up">
      <div className="px-4 py-2 rounded-lg text-xs font-semibold shadow-lg"
        style={{ background: colors[notification.type] || "#3b82f6", color: "#fff" }}>
        {notification.message}
      </div>
    </div>
  );
}
