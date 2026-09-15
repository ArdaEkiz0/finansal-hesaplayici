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
    <div className="h-screen bg-bg-deep flex flex-col overflow-hidden select-none">
      <TitleBar />

      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar - History */}
        <aside className="hidden lg:flex w-72 flex-col" style={{ borderRight: "1px solid rgba(255,255,255,0.05)", background: "rgba(255,255,255,0.01)" }}>
          <div className="flex-1 overflow-y-auto p-3">
            <History />
          </div>
        </aside>

        {/* Main Content */}
        <main className="flex-1 flex flex-col min-h-0 overflow-y-auto">
          {/* Mode Tabs */}
          <div className="px-4 sm:px-6 py-3" style={{ borderBottom: "1px solid rgba(255,255,255,0.05)", background: "rgba(255,255,255,0.01)" }}>
            <ModeSelector />
          </div>

          {/* Calculator Area */}
          <div className="flex-1 px-4 sm:px-6 py-6">
            <div className="max-w-lg mx-auto space-y-6">
              <Display />
              <Keypad />
            </div>
          </div>

          {/* Footer */}
          <footer className="py-2 px-6 text-center" style={{ borderTop: "1px solid rgba(255,255,255,0.05)" }}>
            <p className="text-[10px]" style={{ color: "rgba(255,255,255,0.2)" }}>
              <kbd className="px-1 py-0.5 rounded text-[9px] font-mono font-bold" style={{ background: "rgba(255,255,255,0.06)", color: "rgba(255,255,255,0.4)" }}>0-9</kbd> rakamlar
              {" · "}
              <kbd className="px-1 py-0.5 rounded text-[9px] font-mono font-bold" style={{ background: "rgba(255,255,255,0.06)", color: "rgba(255,255,255,0.4)" }}>Tab</kbd> alan
              {" · "}
              <kbd className="px-1 py-0.5 rounded text-[9px] font-mono font-bold" style={{ background: "rgba(255,255,255,0.06)", color: "rgba(255,255,255,0.4)" }}>Enter</kbd> kaydet
              {" · "}
              <kbd className="px-1 py-0.5 rounded text-[9px] font-mono font-bold" style={{ background: "rgba(255,255,255,0.06)", color: "rgba(255,255,255,0.4)" }}>Esc</kbd> temizle
              {" · "}
              <span style={{ color: "rgba(255,255,255,0.15)" }}>Developer: Arda M. Ekiz</span>
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
