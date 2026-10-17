// Dogrulanmis oranlar. Her kaydin yururluk yili var — yil degisince uygulama
// "oranlari teyit edin" uyarisi basar. GIB rehberi son soz.

export interface StopajPreset {
  label: string;
  rate: 1 | 3 | 5 | 7 | 10 | 15 | 17 | 20;
  note: string;
  /** Maddinin yururluk yili (app uyarisini buradan secer) */
  since: number;
}

export const STOPAJ_PRESETS: StopajPreset[] = [
  { label: "Kira (işyeri)", rate: 20, note: "GVK 94/5-a — brüt kiradan", since: 2019 },
  { label: "Serbest meslek", rate: 20, note: "GVK 94/2-b", since: 2019 },
  { label: "Telif (GVK 18)", rate: 17, note: "GVK 94/2-a", since: 2024 },
];

export interface TevkifatCode {
  code: string;
  name: string;
  pay: 2 | 3 | 4 | 5 | 6 | 7 | 9 | 10;
}

export const TEVKIFAT_CODES: TevkifatCode[] = [
  { code: "601", name: "Yapım işleri + mühendislik-mimarlık", pay: 4 },
  { code: "602", name: "Etüt, plan-proje, danışmanlık, denetim", pay: 9 },
  { code: "603", name: "Makine-taşıt tadil, bakım, onarım", pay: 7 },
  { code: "604", name: "Yemek servis", pay: 5 },
  { code: "605", name: "Organizasyon", pay: 5 },
  { code: "606", name: "İşgücü temin", pay: 9 },
  { code: "607", name: "Özel güvenlik", pay: 9 },
  { code: "608", name: "Yapı denetim", pay: 9 },
  { code: "609", name: "Fason tekstil-konfeksiyon, çanta-ayakkabı", pay: 7 },
  { code: "610", name: "Turistik mağaza müşteri bulma", pay: 9 },
  { code: "611", name: "Spor yayın-reklam-isim hakkı", pay: 9 },
  { code: "612", name: "Temizlik", pay: 9 },
  { code: "613", name: "Çevre-bahçe bakım", pay: 9 },
  { code: "614", name: "Servis taşımacılığı", pay: 5 },
  { code: "—", name: "Ticari reklam", pay: 3 },
  { code: "—", name: "Yük taşımacılığı (nakliye)", pay: 2 },
];

/** Kismi tevkifat alt siniri: KDV dahil bedel bu tutari asmiyorsa tevkifat uygulanmaz. */
export const TEVKIFAT_LIMIT = { value: 12000, since: 2026 };

/** Oran veritabaninin son dogrulama yili — yeni yil girince uyarı çıkar. */
export const RATES_YEAR = 2026;

export const RATE_DISCLAIMER = "2026 oranlarıdır — işleminize uygulamadan önce GİB rehberini teyit edin.";

/** Verinin doygunluğu: en düşük yürürlük yılından sonraki her 1 Ocak uyarı üretir. */
export function ratesStaleBugun(bugun: Date = new Date()): boolean {
  const eski = Math.min(RATES_YEAR, ...STOPAJ_PRESETS.map((p) => p.since), TEVKIFAT_LIMIT.since);
  return bugun.getFullYear() > Math.max(eski, RATES_YEAR);
}
