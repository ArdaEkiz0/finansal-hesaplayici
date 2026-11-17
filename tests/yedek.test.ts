import { describe, it, expect } from "vitest";
import { temizleYedek, yedekOlustur } from "../src/lib/yedek";

const gecerliGecmis = { id: "a1", mode: "kdv", result: "1.000,00 ₺", timestamp: 1760000000000, inputs: "800" };
const gecerliDonem = { id: "2026-03", year: 2026, month: 3, satis: "1000", alis: "500", devredenGiris: "0", hesaplanan: "200", indirilecek: "100", net: "100", odenecek: true, kdvRate: "20", cari: "", closedAt: 1 };
const gecerliCari = { id: "c1", name: "ABC Ltd", createdAt: 1 };

describe("temizleYedek", () => {
  it("gecerli yedegi aynen (temizlenmis) dondurur", () => {
    const y = yedekOlustur({ surum: "4.7.1", history: [gecerliGecmis], donemler: [gecerliDonem], cariler: [gecerliCari], activeCari: "c1", settings: { showCurrencySymbol: true, precision: 2, fontScale: 1.05 }, theme: "system" });
    const temiz = temizleYedek(JSON.parse(JSON.stringify(y)));
    expect(temiz).not.toBeNull();
    expect(temiz!.history).toHaveLength(1);
    expect(temiz!.donemler).toHaveLength(1);
    expect(temiz!.cariler).toHaveLength(1);
    expect(temiz!.activeCari).toBe("c1");
    expect(temiz!.settings.fontScale).toBe(1.05);
    expect(temiz!.theme).toBe("system");
  });

  it("nesne olmayan her seyi reddeder", () => {
    for (const v of [null, undefined, 5, "x", [], true]) expect(temizleYedek(v)).toBeNull();
  });

  it("yedek izi olmayan nesneyi reddeder", () => {
    expect(temizleYedek({ merhaba: "dünya" })).toBeNull();
  });

  it("bos dizili yedek gecerlidir (sifir kayit)", () => {
    const temiz = temizleYedek({ history: [], donemler: [], cariler: [] });
    expect(temiz).not.toBeNull();
    expect(temiz!.history).toHaveLength(0);
  });

  it("bozuk kayitlari filitreler, saglamlari korur", () => {
    const temiz = temizleYedek({
      history: [gecerliGecmis, null, 42, { id: 5 }, { id: "x", result: 1, timestamp: "z" }],
      donemler: [gecerliDonem, { id: "2026/3" }, { id: "2026-04", year: "2026", month: 4 }],
      cariler: [gecerliCari, { id: "c2" }],
    });
    expect(temiz!.history).toHaveLength(1);
    expect(temiz!.donemler).toHaveLength(1);
    expect(temiz!.cariler).toHaveLength(1);
  });

  it("sinirlarin ustasi kayit dilimlenir", () => {
    const temiz = temizleYedek({ history: Array.from({ length: 150 }, (_, i) => ({ ...gecerliGecmis, id: `h${i}` })) });
    expect(temiz!.history).toHaveLength(100);
  });

  it("bozuk ayarlar varsayilana duser", () => {
    const temiz = temizleYedek({ history: [], settings: { showCurrencySymbol: "evet", precision: 7, fontScale: 99 }, theme: "neon", activeCari: 12 });
    expect(temiz!.settings).toEqual({ showCurrencySymbol: true, precision: 2, fontScale: 1 });
    expect(temiz!.theme).toBe("dark");
    expect(temiz!.activeCari).toBe("");
  });

  it("fontScale aralik icindeyse korunur, disindaysa 1", () => {
    expect(temizleYedek({ history: [], settings: { fontScale: 1.2 } })!.settings.fontScale).toBe(1.2);
    expect(temizleYedek({ history: [], settings: { fontScale: 2 } })!.settings.fontScale).toBe(1);
    expect(temizleYedek({ history: [], settings: { fontScale: "koca" } })!.settings.fontScale).toBe(1);
  });

  it("uygulama imzasi tek basina da gecerli neden", () => {
    expect(temizleYedek({ uygulama: "finansal-hesaplayici" })).not.toBeNull();
  });
});
