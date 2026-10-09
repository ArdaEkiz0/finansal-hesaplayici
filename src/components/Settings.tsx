import { useStore } from "../store/useStore";
import { X, Sun, Moon, Eye, EyeOff, RefreshCw } from "lucide-react";
import { useState, useEffect } from "react";

export function Settings() {
  const show = useStore((s) => s.showSettings);
  const toggle = useStore((s) => s.toggleSettings);
  const theme = useStore((s) => s.theme);
  const setTheme = useStore((s) => s.setTheme);
  const showWidget = useStore((s) => s.showWidget);
  const toggleWidget = useStore((s) => s.toggleWidget);
  const settings = useStore((s) => s.settings);
  const updateSettings = useStore((s) => s.updateSettings);
  const notify = useStore((s) => s.notify);
  const [checking, setChecking] = useState(false);
  const [version, setVersion] = useState("");

  useEffect(() => {
    if (show) window.electronAPI?.getAppVersion?.().then(setVersion).catch(() => {});
  }, [show]);

  const checkUpdate = async () => {
    if (!window.electronAPI?.requestUpdate) {
      notify("Masaüstü uygulamasında çalışır", "info");
      return;
    }
    setChecking(true);
    try {
      const res = await window.electronAPI.requestUpdate();
      if (res === "none") notify("Zaten güncelsiniz", "success");
      else if (res === "skipped") notify("Güncelleme atlandı", "info");
      else if (res === "failed") notify("Güncelleme başarısız", "error");
    } catch {
      notify("Kontrol edilemedi", "error");
    } finally {
      setChecking(false);
    }
  };

  if (!show) return null;

  return (
    <div className="fixed inset-0 z-[9998] flex items-center justify-center p-4" style={{ background: "rgba(0,0,0,.5)", backdropFilter: "blur(4px)" }} onClick={toggle}>
      <div className="w-[340px] rounded-2xl p-5 space-y-3 anim-scale-in" style={{ background: "var(--color-bg-surface)", border: "1px solid var(--color-glass-border)" }} onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between">
          <span className="text-[14px] font-bold">Ayarlar</span>
          <button onClick={toggle} className="p-1.5 rounded-lg btn-press" style={{ color: "var(--color-text-ghost)" }}><X size={14} /></button>
        </div>
        <div className="flex items-center justify-between py-2" style={{ borderTop: "1px solid var(--color-glass-border)" }}>
          <span className="text-[12px]" style={{ color: "var(--color-text-secondary)" }}>Tema</span>
          <button onClick={() => setTheme(theme === "dark" ? "light" : "dark")} className="flex items-center gap-2 px-3 py-1.5 rounded-xl text-[12px] font-bold btn-press" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-secondary)" }}>
            {theme === "dark" ? <Moon size={13} /> : <Sun size={13} />} {theme === "dark" ? "Koyu" : "Açık"}
          </button>
        </div>
        <div className="flex items-center justify-between py-2" style={{ borderTop: "1px solid var(--color-glass-border)" }}>
          <span className="text-[12px]" style={{ color: "var(--color-text-secondary)" }}>Hızlı widget</span>
          <button onClick={toggleWidget} className="flex items-center gap-2 px-3 py-1.5 rounded-xl text-[12px] font-bold btn-press" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: showWidget ? "var(--color-primary-light)" : "var(--color-text-secondary)" }}>
            {showWidget ? <Eye size={13} /> : <EyeOff size={13} />} {showWidget ? "Açık" : "Kapalı"}
          </button>
        </div>
        <div className="flex items-center justify-between py-2" style={{ borderTop: "1px solid var(--color-glass-border)" }}>
          <span className="text-[12px]" style={{ color: "var(--color-text-secondary)" }}>Para birimi (₺)</span>
          <button onClick={() => updateSettings({ showCurrencySymbol: !settings.showCurrencySymbol })} className="px-3 py-1.5 rounded-xl text-[12px] font-bold btn-press" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-secondary)" }}>
            {settings.showCurrencySymbol ? "Göster" : "Gizle"}
          </button>
        </div>
        <div className="flex items-center justify-between py-2" style={{ borderTop: "1px solid var(--color-glass-border)" }}>
          <span className="text-[12px]" style={{ color: "var(--color-text-secondary)" }}>Ondalık basamak</span>
          <div className="flex gap-1">
            {[0, 2, 4].map((n) => (
              <button key={n} onClick={() => updateSettings({ precision: n })} className="px-3 py-1.5 rounded-xl text-[12px] font-bold btn-press"
                style={settings.precision === n ? { background: "var(--color-primary)", color: "#fff" } : { background: "var(--color-glass)", color: "var(--color-text-ghost)", border: "1px solid var(--color-glass-border)" }}>
                {n}
              </button>
            ))}
          </div>
        </div>
        <div className="flex items-center justify-between py-2" style={{ borderTop: "1px solid var(--color-glass-border)" }}>
          <span className="text-[12px]" style={{ color: "var(--color-text-secondary)" }}>Güncelleme</span>
          <button onClick={checkUpdate} disabled={checking} className="flex items-center gap-2 px-3 py-1.5 rounded-xl text-[12px] font-bold btn-press" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-secondary)", opacity: checking ? 0.6 : 1 }}>
            <RefreshCw size={13} className={checking ? "animate-spin" : ""} /> {checking ? "Bakılıyor..." : "Denetle"}
          </button>
        </div>
        <p className="text-[10px] pt-1" style={{ color: "var(--color-text-ghost)" }}>v{version || "…"} • Developer: Arda M. Ekiz</p>
      </div>
    </div>
  );
}
