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
    <div className="h-screen flex flex-col overflow-hidden select-none bg-bg-deep">
      <TitleBar />

      <div className="flex-1 flex overflow-hidden">
        <aside className="w-64 flex-col border-r border-glass-border bg-bg-surface hidden md:flex">
          <div className="flex-1 overflow-y-auto p-3">
            <History />
          </div>
        </aside>

        <main className="flex-1 flex flex-col min-h-0 overflow-y-auto">
          <ModeSelector />

          <div className="flex-1 px-4 sm:px-6 py-5">
            <div className="max-w-md sm:max-w-lg lg:max-w-xl mx-auto space-y-5">
              <Display />
              <Keypad />
            </div>
          </div>

          <footer className="py-2 px-4 text-center border-t border-glass-border bg-bg-surface">
            <p className="text-[10px] text-text-ghost">
              <kbd className="px-1 py-0.5 rounded text-[9px] font-mono font-bold bg-glass text-text-tertiary border border-glass-border">0-9</kbd> rakamlar
              {" · "}
              <kbd className="px-1 py-0.5 rounded text-[9px] font-mono font-bold bg-glass text-text-tertiary border border-glass-border">Tab</kbd> alan
              {" · "}
              <kbd className="px-1 py-0.5 rounded text-[9px] font-mono font-bold bg-glass text-text-tertiary border border-glass-border">Enter</kbd> kaydet
              {" · "}
              <kbd className="px-1 py-0.5 rounded text-[9px] font-mono font-bold bg-glass text-text-tertiary border border-glass-border">Esc</kbd> temizle
              {" · "}
              <span className="text-text-secondary">Developer: Arda M. Ekiz</span>
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