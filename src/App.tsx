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
import { InvoicePreview } from "./components/InvoicePreview";
import { useStore } from "./store/useStore";
import { useEffect } from "react";

export default function App() {
  useKeyboard();
  const theme = useStore((s) => s.theme);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
  }, [theme]);

  return (
    <div className="h-screen w-screen flex flex-col overflow-hidden select-none" style={{ background: "var(--color-bg-deep)", color: "var(--color-text-primary)" }}>
      <TitleBar />
      <ModeSelector />

      <div className="flex-1 flex min-h-0 overflow-hidden">
        <div className="hidden xl:block w-[280px] shrink-0 min-h-0" style={{ borderRight: "1px solid var(--color-glass-border)" }}>
          <HistoryPanel />
        </div>

        <main className="flex-1 min-w-0 min-h-0 overflow-y-auto">
          <div className="h-full w-full max-w-[1100px] mx-auto px-4 sm:px-6 py-4 grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)] items-start">
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
        <span className="hidden sm:inline">Developer: Arda M. Ekiz</span>
      </footer>

      <Settings />
      <Help />
      <Notification />
      <Widget />
      <InvoicePreview />
    </div>
  );
}
