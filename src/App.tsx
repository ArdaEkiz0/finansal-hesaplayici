import { useState, useEffect } from "react";
import { useKeyboard } from "./hooks/useKeyboard";
import { TitleBar } from "./components/TitleBar";
import { ModeSelector } from "./components/ModeSelector";
import { CalcPanel } from "./components/CalcPanel";
import { ResultPanel } from "./components/ResultPanel";
import { HistoryPanel } from "./components/HistoryPanel";
import { Settings } from "./components/Settings";
import { Help } from "./components/Help";
import { Notification } from "./components/Notification";
import { Widget } from "./components/Widget";
import { RatesInfo } from "./components/RatesInfo";
import { DonemlerModal } from "./components/DonemlerModal";
import { CarilerModal } from "./components/CarilerModal";
import { CommandPalette } from "./components/CommandPalette";
import { Welcome } from "./components/Welcome";
import { Takvim } from "./components/Takvim";
import { BeyanOzeti } from "./components/BeyanOzeti";
import { WidgetWindow } from "./components/WidgetWindow";
import { InvoicePreview } from "./components/InvoicePreview";
import { useStore } from "./store/useStore";

function MainApp() {
  useKeyboard();
  useEffect(() => {
    try {
      const saklandi = localStorage.getItem("fh-welcome-hide") === "1";
      if (!saklandi) {
        window.setTimeout(() => useStore.getState().setShowWelcome(true), 450);
      }
    } catch { /* localStorage yoksa sessiz geç */ }
  }, []);
  return (
    <div className="h-screen w-screen flex flex-col overflow-hidden select-none relative" style={{ background: "var(--color-bg-deep)", color: "var(--color-text-primary)" }}>
      {/* Ambiyans: odak duygusu veren yumusak isiklar */}
      <div aria-hidden className="ambient-glow" style={{ width: 420, height: 420, top: -220, right: -160, background: "radial-gradient(circle, rgba(47,123,255,.10), transparent 70%)" }} />
      <div aria-hidden className="ambient-glow" style={{ width: 380, height: 380, bottom: -200, left: -140, background: "radial-gradient(circle, rgba(16,185,129,.07), transparent 70%)" }} />
      <TitleBar />
      <ModeSelector />

      <div className="flex-1 flex min-h-0 overflow-hidden">
        <div className="hidden xl:flex w-[310px] shrink-0 min-h-0 flex-col" style={{ borderRight: "1px solid var(--color-glass-border)", background: "rgba(148,175,255,.02)" }}>
          <HistoryPanel />
        </div>

        <main className="flex-1 min-w-0 min-h-0 flex flex-col overflow-hidden relative z-[1]">
          <div className="flex-1 min-h-0 w-full max-w-[1400px] mx-auto px-4 sm:px-6 py-4 grid gap-5 lg:grid-cols-[minmax(0,6fr)_minmax(0,5fr)] items-stretch overflow-y-auto">
            <CalcPanel />
            <ResultPanel />
          </div>
        </main>
      </div>

      <footer className="shrink-0 px-4 py-1.5 flex items-center justify-center gap-3 text-[10px]" style={{ borderTop: "1px solid var(--color-glass-border)", color: "var(--color-text-ghost)" }}>
        <span><kbd className="px-1 py-0.5 rounded font-mono" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>0-9</kbd> yaz</span>
        <span><kbd className="px-1 py-0.5 rounded font-mono" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>Tab</kbd> alan</span>
        <span><kbd className="px-1 py-0.5 rounded font-mono" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>Enter</kbd> kaydet</span>
        <span><kbd className="px-1 py-0.5 rounded font-mono" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>Esc</kbd> temizle</span>
        <span className="hidden sm:inline">Alt+Shift+K: hızlı KDV • Developer: Arda M. Ekiz</span>
      </footer>

      <Settings />
      <Help />
      <Welcome />
      <Takvim />
      <BeyanOzeti />
      <RatesInfo />
      <DonemlerModal />
      <CarilerModal />
      <CommandPalette />
      <Notification />
      <Widget />
      <InvoicePreview />
    </div>
  );
}

export default function App() {
  const [isWidget] = useState(() => {
    try {
      return new URLSearchParams(window.location.search).has("widget");
    } catch {
      return false;
    }
  });
  const theme = useStore((s) => s.theme);
  const fontScale = useStore((s) => s.settings.fontScale);

  // Tema: "system" ise Windows ayarını izler (canlı geçiş dahil).
  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      const resolved = theme === "system" ? (mq.matches ? "dark" : "light") : theme;
      document.documentElement.setAttribute("data-theme", resolved);
    };
    apply();
    if (theme === "system") {
      mq.addEventListener("change", apply);
      return () => mq.removeEventListener("change", apply);
    }
  }, [theme]);

  // Yazı ölçeği: HTML kökünde zoom — tüm arayüzü orantılı büyütür/küçültür.
  useEffect(() => {
    const s = Math.min(1.25, Math.max(0.85, fontScale || 1));
    document.documentElement.style.zoom = s === 1 ? "" : String(s);
  }, [fontScale]);

  if (isWidget) return <WidgetWindow />;
  return <MainApp />;
}
