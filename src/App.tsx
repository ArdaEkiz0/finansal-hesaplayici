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
import { WidgetWindow } from "./components/WidgetWindow";
import { InvoicePreview } from "./components/InvoicePreview";
import { useStore } from "./store/useStore";

function MainApp() {
  useKeyboard();
  return (
    <div className="h-screen w-screen flex flex-col overflow-hidden select-none" style={{ background: "var(--color-bg-deep)", color: "var(--color-text-primary)" }}>
      <TitleBar />
      <ModeSelector />

      <div className="flex-1 flex min-h-0 overflow-hidden">
        <div className="hidden xl:block w-[300px] shrink-0 min-h-0" style={{ borderRight: "1px solid var(--color-glass-border)" }}>
          <HistoryPanel />
        </div>

        <main className="flex-1 min-w-0 min-h-0 flex flex-col overflow-hidden">
          <div className="flex-1 min-h-0 w-full max-w-[1440px] mx-auto px-4 sm:px-6 py-4 grid gap-4 lg:grid-cols-[minmax(0,5fr)_minmax(0,6fr)] items-stretch overflow-y-auto">
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

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
  }, [theme]);

  if (isWidget) return <WidgetWindow />;
  return <MainApp />;
}
