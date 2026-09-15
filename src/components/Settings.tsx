import { useStore } from "../store/useStore";
import { X, RefreshCw, Sun, Moon, Cloud, Monitor } from "lucide-react";
import { useState } from "react";

declare global {
  interface Window {
    electronAPI?: {
      checkUpdate: () => Promise<any>;
      getGistToken: () => Promise<string>;
      setGistToken: (t: string) => void;
    };
  }
}

export function Settings() {
  const settings = useStore((s) => s.settings);
  const updateSettings = useStore((s) => s.updateSettings);
  const toggleSettings = useStore((s) => s.toggleSettings);
  const toggleWidget = useStore((s) => s.toggleWidget);
  const showWidget = useStore((s) => s.showWidget);
  const theme = useStore((s) => s.theme);
  const setTheme = useStore((s) => s.setTheme);
  const [checking, setChecking] = useState(false);
  const [updateMsg, setUpdateMsg] = useState("");
  const [syncing, setSyncing] = useState(false);
  const [syncMsg, setSyncMsg] = useState("");
  const [token, setToken] = useState("");

  const handleCheckUpdate = async () => {
    if (!window.electronAPI) {
      setUpdateMsg("Sadece masaustu uygulamasinda calisir");
      return;
    }
    setChecking(true);
    setUpdateMsg("");
    try {
      const update = await window.electronAPI.checkUpdate();
      if (update?.hasUpdate) {
        setUpdateMsg(`v${update.version} mevcut - Indirilecek...`);
      } else {
        setUpdateMsg("En guncel surumu kullaniyorsunuz");
      }
    } catch {
      setUpdateMsg("Kontrol edilemedi");
    }
    setChecking(false);
  };

  const handleSync = async (direction: "upload" | "download") => {
    setSyncing(true);
    setSyncMsg("");
    try {
      const savedToken = token || localStorage.getItem("gh_token") || "";
      if (!savedToken) {
        setSyncMsg("GitHub token gerekli");
        setSyncing(false);
        return;
      }
      localStorage.setItem("gh_token", savedToken);

      const gistId = localStorage.getItem("gist_id") || "";

      if (direction === "upload") {
        const data = JSON.stringify({
          history: useStore.getState().history,
          settings: useStore.getState().settings,
          theme: useStore.getState().theme,
        });

        if (gistId) {
          await fetch(`https://api.github.com/gists/${gistId}`, {
            method: "PATCH",
            headers: { Authorization: `token ${savedToken}`, "Content-Type": "application/json" },
            body: JSON.stringify({ files: { "finansal-hesaplaci.json": { content: data } } }),
          });
        } else {
          const res = await fetch("https://api.github.com/gists", {
            method: "POST",
            headers: { Authorization: `token ${savedToken}`, "Content-Type": "application/json" },
            body: JSON.stringify({
              description: "Finansal Hesaplaci Sync",
              files: { "finansal-hesaplaci.json": { content: data } },
            }),
          });
          const gist = await res.json();
          localStorage.setItem("gist_id", gist.id);
        }
        setSyncMsg("Buluta yuklendi!");
      } else {
        if (!gistId) { setSyncMsg("Once yukleyin"); setSyncing(false); return; }
        const res = await fetch(`https://api.github.com/gists/${gistId}`, {
          headers: { Authorization: `token ${savedToken}` },
        });
        const gist = await res.json();
        const content = gist.files["finansal-hesaplaci.json"]?.content;
        if (content) {
          const data = JSON.parse(content);
          useStore.setState({
            history: data.history || [],
            settings: data.settings || { showCurrencySymbol: true, precision: 2 },
            theme: data.theme || "dark",
          });
          document.documentElement.setAttribute("data-theme", data.theme || "dark");
          setSyncMsg("Buluttan indirildi!");
        }
      }
    } catch {
      setSyncMsg("Senkronizasyon hatasi");
    }
    setSyncing(false);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-md anim-fade-in" onClick={toggleSettings}>
      <div className="glass-strong rounded-3xl shadow-2xl w-full max-w-sm mx-4 overflow-hidden anim-scale-in" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between p-5 border-b border-glass-border">
          <h3 className="text-sm font-bold text-text-primary">Ayarlar</h3>
          <button onClick={toggleSettings} className="p-1.5 rounded-lg glass hover:bg-glass-hover text-text-ghost">
            <X size={16} />
          </button>
        </div>

        <div className="p-5 space-y-4 max-h-[60vh] overflow-y-auto">
          {/* Tema */}
          <div className="space-y-2">
            <span className="text-xs text-text-secondary font-medium">Tema</span>
            <div className="flex gap-1.5">
              {([
                { id: "dark", icon: Moon, label: "Karanlik" },
                { id: "light", icon: Sun, label: "Acik" },
              ] as const).map(({ id, icon: Icon, label }) => (
                <button key={id} onClick={() => setTheme(id)}
                  className={`flex-1 flex items-center justify-center gap-1.5 py-2 rounded-xl text-xs font-bold transition-all ${
                    theme === id ? "bg-brand-500 text-white shadow-lg shadow-brand-500/25" : "glass text-text-secondary hover:text-text-primary"
                  }`}>
                  <Icon size={14} />
                  {label}
                </button>
              ))}
            </div>
          </div>

          {/* Widget */}
          <div className="flex items-center justify-between">
            <span className="text-xs text-text-secondary">Kucuk Pencere (Widget)</span>
            <button onClick={toggleWidget}
              className={`w-10 h-5 rounded-full transition-all duration-200 ${showWidget ? "bg-brand-500" : "bg-glass-active"}`}>
              <div className={`w-4 h-4 rounded-full bg-white shadow transition-transform duration-200 ${showWidget ? "translate-x-5.5" : "translate-x-0.5"}`} />
            </button>
          </div>

          {/* ₺ Sembolü */}
          <div className="flex items-center justify-between">
            <span className="text-xs text-text-secondary">₺ Sembolü</span>
            <button onClick={() => updateSettings({ showCurrencySymbol: !settings.showCurrencySymbol })}
              className={`w-10 h-5 rounded-full transition-all duration-200 ${settings.showCurrencySymbol ? "bg-brand-500" : "bg-glass-active"}`}>
              <div className={`w-4 h-4 rounded-full bg-white shadow transition-transform duration-200 ${settings.showCurrencySymbol ? "translate-x-5.5" : "translate-x-0.5"}`} />
            </button>
          </div>

          {/* Hassasiyet */}
          <div className="space-y-1.5">
            <span className="text-xs text-text-secondary">Basamak Hassasiyeti</span>
            <div className="flex gap-1.5">
              {[0, 2, 4, 6].map((p) => (
                <button key={p} onClick={() => updateSettings({ precision: p })}
                  className={`flex-1 py-1.5 rounded-xl text-xs font-bold transition-all ${settings.precision === p ? "bg-brand-500 text-white" : "glass text-text-secondary hover:text-text-primary"}`}>
                  {p}
                </button>
              ))}
            </div>
          </div>

          {/* Bulut Senkron */}
          <div className="space-y-2">
            <div className="flex items-center gap-1.5">
              <Cloud size={14} className="text-brand-400" />
              <span className="text-xs text-text-secondary font-medium">Bulut Senkron</span>
            </div>
            <input type="password" placeholder="GitHub Personal Access Token" value={token} onChange={(e) => setToken(e.target.value)}
              className="w-full px-3 py-2 rounded-xl glass text-xs text-text-primary placeholder:text-text-ghost border border-glass-border focus:border-brand-500 focus:outline-none font-mono" />
            <div className="flex gap-1.5">
              <button onClick={() => handleSync("upload")} disabled={syncing}
                className="flex-1 py-2 rounded-xl text-xs font-bold glass text-text-secondary hover:text-text-primary transition-all disabled:opacity-50">
                {syncing ? "..." : "Yukle"}
              </button>
              <button onClick={() => handleSync("download")} disabled={syncing}
                className="flex-1 py-2 rounded-xl text-xs font-bold glass text-text-secondary hover:text-text-primary transition-all disabled:opacity-50">
                {syncing ? "..." : "Indir"}
              </button>
            </div>
            {syncMsg && <p className="text-[10px] text-emerald-400 text-center">{syncMsg}</p>}
          </div>
        </div>

        <div className="px-5 py-3 border-t border-glass-border">
          <div className="flex items-center justify-between mb-2">
            <span className="text-[10px] text-text-ghost">Surum 2.2.0</span>
            <button onClick={handleCheckUpdate} disabled={checking}
              className="flex items-center gap-1 text-[10px] text-brand-400 hover:text-brand-300 transition-colors disabled:opacity-50">
              <RefreshCw size={10} className={checking ? "animate-spin" : ""} />
              {checking ? "Kontrol..." : "Guncelleme Kontrol Et"}
            </button>
          </div>
          {updateMsg && <p className="text-[10px] text-emerald-400 text-center">{updateMsg}</p>}
          <p className="text-[10px] text-text-ghost text-center mt-1">Developer: Arda M. Ekiz</p>
        </div>
      </div>
    </div>
  );
}
