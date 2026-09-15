import { useKeyboard } from "./hooks/useKeyboard";
import { TitleBar } from "./components/TitleBar";
import { ModeSelector } from "./components/ModeSelector";
import { Display } from "./components/Display";
import { Keypad } from "./components/Keypad";
import { History } from "./components/History";
import { Settings } from "./components/Settings";
import { Help } from "./components/Help";
import { Notification } from "./components/Notification";
import { useStore } from "./store/useStore";

export default function App() {
  useKeyboard();
  const showSettings = useStore((s) => s.showSettings);

  return (
    <div className="h-screen bg-bg-deep flex flex-col overflow-hidden select-none">
      <TitleBar />

      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar */}
        <aside className="hidden lg:flex w-72 flex-col border-r border-white/[0.06] glass">
          <div className="flex-1 overflow-y-auto p-3">
            <History />
          </div>
        </aside>

        {/* Main */}
        <main className="flex-1 flex flex-col min-h-0 overflow-y-auto">
          <div className="px-4 sm:px-6 py-4 border-b border-white/[0.06] glass">
            <ModeSelector />
          </div>

          <div className="flex-1 px-4 sm:px-6 py-6">
            <div className="max-w-lg mx-auto space-y-6">
              <Display />
              <Keypad />
            </div>
          </div>

          <footer className="border-t border-white/[0.06] py-2 px-6 text-center glass">
            <p className="text-[10px] text-text-ghost">
              <kbd className="px-1 py-0.5 rounded glass-strong text-[9px] font-mono font-bold text-text-secondary">0-9</kbd> rakamlar
              {" · "}
              <kbd className="px-1 py-0.5 rounded glass-strong text-[9px] font-mono font-bold text-text-secondary">Tab</kbd> alan
              {" · "}
              <kbd className="px-1 py-0.5 rounded glass-strong text-[9px] font-mono font-bold text-text-secondary">Enter</kbd> kaydet
              {" · "}
              <kbd className="px-1 py-0.5 rounded glass-strong text-[9px] font-mono font-bold text-text-secondary">Esc</kbd> temizle
            </p>
          </footer>
        </main>
      </div>

      {showSettings && <Settings />}
      <Help />
      <Notification />
    </div>
  );
}
