import { useState } from "react";
import { useStore, MODE_LABELS } from "../store/useStore";
import { X, Receipt, Archive, Search, Sparkles, TrendingUp, Settings2, Zap, Keyboard, MousePointerClick } from "lucide-react";

const KULLANIM = [
  {
    ikon: <Receipt size={16} />,
    baslik: "Hesapla",
    metin: "Tutarı yaz — sonuç canlı hesaplanır, butona gerek yok. Enter ile kayda geçer.",
  },
  {
    ikon: <Sparkles size={16} />,
    baslik: "Mod seç",
    metin: "Üst şeritten 16 hazır mod (KDV, Stopaj, Dönem…). Her yerden aramak için Ctrl+K.",
  },
  {
    ikon: <Archive size={16} />,
    baslik: "Dönem & cari",
    metin: "Dönemler panelinde ay ay arşivle; devreden KDV sonraki aya tek tikla taşınır.",
  },
  {
    ikon: <Search size={16} />,
    baslik: "E-fatura CSV",
    metin: "Toplu moda GIB e-arşiv CSV'sini yükle — matrahları otomatik tanır.",
  },
  {
    ikon: <TrendingUp size={16} />,
    baslik: "Geçmiş & kayıt",
    metin: "Sol panelde eski hesaplar; sabitle, geri yükle, CSV olarak dışa aktar.",
  },
  {
    ikon: <Zap size={16} />,
    baslik: "Hızlı KDV widget",
    metin: "Alt+Shift+K ile ekranda dönen mini hesap makinesi — her zaman görünür.",
  },
];

export function Welcome() {
  const show = useStore((s) => s.showWelcome);
  const toggle = useStore((s) => s.toggleWelcome);
  const toggleSettings = useStore((s) => s.toggleSettings);
  const historyCount = useStore((s) => s.history.length);
  const [sakla, setSakla] = useState(false);

  if (!show) return null;

  const kapat = () => {
    try {
      if (sakla) localStorage.setItem("fh-welcome-hide", "1");
      else localStorage.setItem("fh-welcome-hide", "0");
    } catch { /* yoksa sessiz */ }
    toggle();
  };

  return (
    <div className="fixed inset-0 z-[9997] flex items-center justify-center p-4 overflow-y-auto" style={{ background: "rgba(0,0,0,.55)", backdropFilter: "blur(6px)" }} onClick={kapat}>
      <div className="w-full max-w-[600px] card p-5 sm:p-6 space-y-4 anim-scale-in my-auto" onClick={(e) => e.stopPropagation()}>
        {/* Başlık */}
        <div className="flex items-start justify-between gap-3">
          <div>
            <h2 className="text-[18px] font-bold flex items-center gap-2">
              Finansal Hesaplayıcı'ya hoş geldin
              {historyCount === 0 && <span className="text-[9px] px-2 py-0.5 rounded-full font-bold" style={{ background: "rgba(47,123,255,.15)", color: "var(--color-primary-light)" }}>İLK ADIM</span>}
            </h2>
            <p className="text-[12px] mt-1" style={{ color: "var(--color-text-tertiary)" }}>
              Türk vergi hesapları için hazır — nasıl kullanılacağını 1 dakikada özetleyelim.
            </p>
          </div>
          <button onClick={kapat} className="p-1.5 rounded-lg btn-press shrink-0" style={{ color: "var(--color-text-ghost)" }}><X size={14} /></button>
        </div>

        {/* Kullanım kartları */}
        <div className="grid sm:grid-cols-2 gap-2.5">
          {KULLANIM.map((k) => (
            <div key={k.baslik} className="glass-card p-3 flex gap-2.5">
              <div className="w-8 h-8 rounded-lg flex items-center justify-center shrink-0" style={{ background: "rgba(47,123,255,.12)", color: "var(--color-primary-light)", border: "1px solid rgba(47,123,255,.25)" }}>
                {k.ikon}
              </div>
              <div className="min-w-0">
                <p className="text-[12.5px] font-bold">{k.baslik}</p>
                <p className="text-[11px] leading-relaxed mt-0.5" style={{ color: "var(--color-text-tertiary)" }}>{k.metin}</p>
              </div>
            </div>
          ))}
        </div>

        {/* Mod sayısı çip satırı */}
        <div className="flex flex-wrap gap-1.5 justify-center">
          {(["kdv", "donem", "stopaj", "tevkifat", "gecikme", "amortisman", "doviz", "margin", "batch"] as const).map((m) => (
            <span key={m} className="text-[10px] px-2 py-1 rounded-lg font-semibold" style={{ background: "var(--color-glass)", color: "var(--color-text-tertiary)", border: "1px solid var(--color-glass-border)" }}>
              {MODE_LABELS[m]}
            </span>
          ))}
          <span className="text-[10px] px-2 py-1 rounded-lg font-semibold" style={{ background: "var(--color-glass)", color: "var(--color-text-tertiary)", border: "1px solid var(--color-glass-border)" }}>+7</span>
        </div>

        {/* Kısayollar + Ayarlar */}
        <div className="glass-card p-3 flex items-center justify-between gap-3">
          <div className="flex items-center gap-2 min-w-0">
            <Keyboard size={15} style={{ color: "var(--color-primary-light)" }} className="shrink-0" />
            <p className="text-[11.5px] truncate" style={{ color: "var(--color-text-secondary)" }}>
              Klavyeyle tam kullanım:<span className="font-bold"> {`0-9`} yaz • Tab alan • Enter kaydet • Esc temizle • F1 klavye listesi</span>
            </p>
          </div>
          <button onClick={() => { kapat(); useStore.getState().toggleSettings(); }}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-[11px] font-bold shrink-0 btn-press key-cap"
            style={{ background: "var(--color-glass-active)", color: "var(--color-text-secondary)", border: "1px solid var(--color-glass-border)" }}>
            <Settings2 size={12} /> Ayarlar
          </button>
        </div>

        {/* Alt bar */}
        <div className="flex items-center justify-between gap-3 pt-1">
          <label className="flex items-center gap-2 cursor-pointer select-none" title="İşaretlersen bir daha açılmaz; başlığındaki ? simgesiyle her zaman geri çağırabilirsin">
            <input type="checkbox" checked={sakla} onChange={(e) => setSakla(e.target.checked)} className="w-3.5 h-3.5 accent-[var(--color-primary)]" />
            <span className="text-[11px]" style={{ color: "var(--color-text-tertiary)" }}>Açılışta bir daha gösterme</span>
          </label>
          <button onClick={kapat} className="px-4 py-2 rounded-xl text-[12.5px] font-bold btn-press glow-primary flex items-center gap-1.5"
            style={{ background: "linear-gradient(135deg, var(--color-primary), #1e5fe0)", color: "#fff" }}>
            <MousePointerClick size={13} /> Anladım, başlayalım
          </button>
        </div>
      </div>
    </div>
  );
}
