// 2026 dogrulanmis oranlar. Degisebilir — GIB rehberini teyit edin.
export interface StopajPreset {
  label: string;
  rate: 1 | 3 | 5 | 7 | 10 | 15 | 17 | 20;
  note: string;
}

export const STOPAJ_PRESETS: StopajPreset[] = [
  { label: "Kira (işyeri)", rate: 20, note: "GVK 94/5-a — brüt kiradan" },
  { label: "Serbest meslek", rate: 20, note: "GVK 94/2-b" },
  { label: "Telif (GVK 18)", rate: 17, note: "GVK 94/2-a" },
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

/** 2026: KDV dahil bedeli bu tutari asmayan islemlerde kismi tevkifat uygulanmaz. */
export const TEVKIFAT_LIMIT_2026 = 12000;

export const RATE_DISCLAIMER = "2026 oranlarıdır — işleminize uygulamadan önce GİB rehberini teyit edin.";
