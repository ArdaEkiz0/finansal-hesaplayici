import { useStore } from "../store/useStore";
import { X, Sun, Moon, Eye, EyeOff, RefreshCw, Download, FolderOpen, Monitor } from "lucide-react";
import { useState, useEffect, useRef } from "react";

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
  const yedekRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (show) window.electronAPI?.getAppVersion?.().then(setVersion).catch(() => {});
  }, [show]);

  const yedekAl = () => {
    const s = useStore.getState();
    const yedek = {
      uygulama: "finansal-hesaplayici",
      surum: version || null,
      tarih: new Date().toISOString(),
      history: s.history,
      donemler: s.donemler,
      cariler: s.cariler,
      activeCari: s.activeCari,
      settings: s.settings,
      theme: s.theme,
    };
    const blob = new Blob([JSON.stringify(yedek, null, 2)], { type: "application/json;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `finansal-hesaplayici-yedek-${new Date().toISOString().slice(0, 10)}.json`;
    link.click();
    URL.revokeObjectURL(url);
    notify("Yedek dosyası indirildi", "success");
  };

  const yedekFile = (file: File) => {
    if (!window.confirm("Yedekten geri yükleme mevcut tüm kayıtların üzerine yazar. Devam edilsin mi?")) return;
    const okur = new FileReader();
    okur.onload = () => {
      try {
        const ok = useStore.getState().restoreYedek(JSON.parse(String(okur.result)));
        notify(ok ? "Yedek geri yüklendi — tüm kayıtlar döşendi" : "Geçersiz yedek dosyası", ok ? "success" : "error");
      } catch {
        notify("Yedek okunamadı (bozuk JSON)", "error");
      }
    };
    okur.readAsText(file);
  };

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
          <div className="flex gap-1 rounded-xl p-1" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)" }}>
            {([["dark", "Koyu", Moon], ["light", "Açık", Sun], ["system", "Sistem", Monitor]] as const).map(([val, ad, Icon]) => (
              <button key={val} onClick={() => setTheme(val)} className="px-2.5 py-1 rounded-lg text-[11px] font-bold btn-press flex items-center gap-1"
                style={theme === val
                  ? { background: "var(--color-bg-elevated)", color: "var(--color-primary-light)", boxShadow: "0 1px 4px rgba(0,0,0,.18)" }
                  : { color: "var(--color-text-ghost)" }}>
                <Icon size={11} /> {ad}
              </button>
            ))}
          </div>
        </div>
        <div className="flex items-center justify-between py-2" style={{ borderTop: "1px solid var(--color-glass-border)" }}>
          <span className="text-[12px]" style={{ color: "var(--color-text-secondary)" }}>Yazı ölçeği</span>
          <div className="flex items-center gap-1.5">
            <button title="Küçült" onClick={() => updateSettings({ fontScale: Math.max(0.85, Math.round((settings.fontScale ?? 1) * 100 - 5) / 100) })} className="w-7 h-7 rounded-lg text-[12px] font-bold btn-press" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-secondary)" }}>A−</button>
            <span className="w-10 text-center text-[11px] font-bold num-display" style={{ color: "var(--color-primary-light)" }}>{Math.round((settings.fontScale ?? 1) * 100)}%</span>
            <button title="Büyüt" onClick={() => updateSettings({ fontScale: Math.min(1.25, Math.round((settings.fontScale ?? 1) * 100 + 5) / 100) })} className="w-7 h-7 rounded-lg text-[12px] font-bold btn-press" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-secondary)" }}>A+</button>
            {(settings.fontScale ?? 1) !== 1 && (
              <button title="Sıfırla" onClick={() => updateSettings({ fontScale: 1 })} className="px-1.5 h-7 rounded-lg text-[10px] font-bold btn-press" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-ghost)" }}>↺</button>
            )}
          </div>
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
        <div className="grid grid-cols-2 gap-2 py-2" style={{ borderTop: "1px solid var(--color-glass-border)" }}>
          <button onClick={yedekAl} className="py-2 rounded-xl text-[11.5px] font-bold btn-press flex items-center justify-center gap-1.5 glow-primary" style={{ background: "linear-gradient(135deg, var(--color-primary), #1e5fe0)", color: "#fff" }}>
            <Download size={13} /> Yedek al
          </button>
          <button onClick={() => yedekRef.current?.click()} className="py-2 rounded-xl text-[11.5px] font-bold btn-press flex items-center justify-center gap-1.5" style={{ background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", color: "var(--color-text-secondary)" }}>
            <FolderOpen size={13} /> Yedekten yükle
          </button>
          <input ref={yedekRef} type="file" accept=".json,application/json" className="hidden"
            onChange={(e) => { const f = e.target.files?.[0]; if (f) yedekFile(f); e.target.value = ""; }} />
        </div>
        <p className="text-[9.5px] -mt-1" style={{ color: "var(--color-text-ghost)" }}>Yedek tüm geçmiş-dönem-cari kayıtları tek JSON'a indirir; bilgisayar değişimi ve sürüm korkusuna karşı imza gibi.</p>
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
