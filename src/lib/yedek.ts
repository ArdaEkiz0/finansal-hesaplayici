/** Yedek dosyasi (JSON) sanitizasyonu — disaridan gelen veride guven zinciri:
 *  bilinmeyen alanlari at, bozuk kayitlari filitrele, sinirlara kilitle. */

export interface TemizYedek {
  history: Record<string, unknown>[];
  donemler: Record<string, unknown>[];
  cariler: Record<string, unknown>[];
  activeCari: string;
  settings: { showCurrencySymbol: boolean; precision: number; fontScale: number };
  theme: string;
}

export const YEDEK_SINIRLAR = { history: 100, donemler: 240, cariler: 500 } as const;

const obj = (v: unknown): v is Record<string, unknown> => !!v && typeof v === "object" && !Array.isArray(v);
const arr = (v: unknown): unknown[] => Array.isArray(v) ? v : [];

const gecerliGeclmis = (e: unknown): e is Record<string, unknown> => {
  if (!obj(e)) return false;
  return typeof e.id === "string" && typeof e.result === "string" && typeof e.timestamp === "number" && typeof e.mode === "string";
};

const gecerliDonem = (e: unknown): e is Record<string, unknown> => {
  if (!obj(e)) return false;
  return typeof e.id === "string" && /^\d{4}-\d{2}$/.test(e.id)
    && typeof e.year === "number" && typeof e.month === "number"
    && typeof e.hesaplanan === "string" && typeof e.indirilecek === "string" && typeof e.net === "string"
    && typeof e.odenecek === "boolean";
};

const gecerliCari = (e: unknown): e is Record<string, unknown> =>
  obj(e) && typeof e.id === "string" && typeof e.name === "string";

/** Gecerli bir Finansal Hesaplayici yedegi ise temizlenmis halini, degilse null dondurur. */
export function temizleYedek(data: unknown): TemizYedek | null {
  if (!obj(data)) return null;
  // Yedek dosyasi oldugunun imzasi: bu uc alan dan en az biri dizi olarak bulunmali.
  const hamGecmis = arr(data.history);
  const hamDonem = arr(data.donemler);
  const hamCari = arr(data.cariler);
  const yedekIzi = ["history", "donemler", "cariler"].some((k) => Array.isArray(data[k])) || data.uygulama === "finansal-hesaplayici";
  if (!yedekIzi) return null;

  const s: Record<string, unknown> = obj(data.settings) ? data.settings : {};
  const font = Number(s.fontScale);
  return {
    history: hamGecmis.filter(gecerliGeclmis).slice(0, YEDEK_SINIRLAR.history),
    donemler: hamDonem.filter(gecerliDonem).slice(0, YEDEK_SINIRLAR.donemler),
    cariler: hamCari.filter(gecerliCari).slice(0, YEDEK_SINIRLAR.cariler),
    activeCari: typeof data.activeCari === "string" ? data.activeCari : "",
    settings: {
      showCurrencySymbol: typeof s.showCurrencySymbol === "boolean" ? s.showCurrencySymbol : true,
      precision: [0, 2, 4].includes(Number(s.precision)) ? Number(s.precision) : 2,
      fontScale: Number.isFinite(font) && font >= 0.85 && font <= 1.25 ? font : 1,
    },
    theme: ["dark", "light", "system"].includes(String(data.theme)) ? String(data.theme) : "dark",
  };
}

/** Mevcut state'ten yedek sozlugu uretir (Settings'teki disa aktarma ile ayni bicim). */
export function yedekOlustur(parca: {
  surum?: string | null;
  history?: unknown[];
  donemler?: unknown[];
  cariler?: unknown[];
  activeCari?: string;
  settings?: TemizYedek["settings"];
  theme?: string;
}): Record<string, unknown> {
  return {
    uygulama: "finansal-hesaplayici",
    surum: parca.surum ?? null,
    tarih: new Date().toISOString(),
    history: parca.history ?? [],
    donemler: parca.donemler ?? [],
    cariler: parca.cariler ?? [],
    activeCari: parca.activeCari ?? "",
    settings: parca.settings,
    theme: parca.theme ?? "dark",
  };
}
