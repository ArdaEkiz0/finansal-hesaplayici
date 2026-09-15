import { useKeyboard } from "./hooks/useKeyboard";
import { TitleBar } from "./components/TitleBar";
import { ModeSelector } from "./components/ModeSelector";
import { Display } from "./components/Display";
import { Keypad } from "./components/Keypad";
import { History } from "./components/History";
import { Settings } from "./components/Settings";
import { Help } from "./components/Help";
import { Notification } from "./components/Notification";
import { Widget } from "./components/Widget";
import { useStore } from "./store/useStore";
import { useEffect } from "react";

export default function App() {
  useKeyboard();
  const showSettings = useStore((s) => s.showSettings);
  const theme = useStore((s) => s.theme);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
  }, [theme]);

  return (
    <div className="h-screen flex flex-col overflow-hidden select-none" style={{ background: "var(--color-bg-deep)" }}>
      <TitleBar />

      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar */}
        <aside className="hidden lg:flex w-64 flex-col" style={{ borderRight: "1px solid var(--color-glass-border)" }}>
          <div className="flex-1 overflow-y-auto p-2">
            <History />
          </div>
        </aside>

        {/* Main */}
        <main className="flex-1 flex flex-col min-h-0 overflow-y-auto">
          <div className="px-4 sm:px-6 py-3" style={{ borderBottom: "1px solid var(--color-glass-border)" }}>
            <ModeSelector />
          </div>

          <div className="flex-1 px-4 sm:px-6 py-5">
            <div className="max-w-lg mx-auto space-y-5">
              <Display />
              <Keypad />
            </div>
          </div>

          <footer className="py-2 px-4 text-center" style={{ borderTop: "1px solid var(--color-glass-border)" }}>
            <p className="text-[10px]" style={{ color: "var(--color-text-ghost)" }}>
              <kbd className="px-1 py-0.5 rounded text-[9px] font-mono font-bold" style={{ background: "var(--color-glass)", color: "var(--color-text-tertiary)" }}>0-9</kbd> rakamlar
              {" · "}
              <kbd className="px-1 py-0.5 rounded text-[9px] font-mono font-bold" style={{ background: "var(--color-glass)", color: "var(--color-text-tertiary)" }}>Tab</kbd> alan
              {" · "}
              <kbd className="px-1 py-0.5 rounded text-[9px] font-mono font-bold" style={{ background: "var(--color-glass)", color: "var(--color-text-tertiary)" }}>Enter</kbd> kaydet
              {" · "}
              <kbd className="px-1 py-0.5 rounded text-[9px] font-mono font-bold" style={{ background: "var(--color-glass)", color: "var(--color-text-tertiary)" }}>Esc</kbd> temizle
              {" · "}
              <span style={{ color: "var(--color-text-ghost)" }}>Developer: Arda M. Ekiz</span>
            </p>
          </footer>
        </main>
      </div>

      {showSettings && <Settings />}
      <Help />
      <Notification />
      <Widget />
    </div>
  );
}
