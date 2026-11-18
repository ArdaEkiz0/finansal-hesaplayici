import { describe, it, expect } from "vitest";
import Decimal from "decimal.js";
import {
  computeDonemNet,
  kalanDevreden,
  sortByDonemId,
  donemOzet,
  donemId,
  donemAd,
  yillaraGir,
  parseDonemSayisi,
  type DonemRecord,
} from "../src/data/donem";

function rec(partial: Partial<DonemRecord>): DonemRecord {
  return {
    id: "2026-01", year: 2026, month: 1,
    satis: "0", alis: "0", devredenGiris: "0",
    hesaplanan: "0", indirilecek: "0", net: "0",
    odenecek: true, kdvRate: "20", cari: "", closedAt: 0,
    ...partial,
  };
}

describe("donem neti", () => {
  it("satış 100000, alış 60000, devreden 1000, %20 -> ödenecek 7000", () => {
    const r = computeDonemNet("100000", "60000", "1000", new Decimal(20))!;
    expect(r.net.toString()).toBe("7000");
    expect(r.odenecek).toBe(true);
  });
  it("satış 10000, alış 50000 -> devreden -8000", () => {
    const r = computeDonemNet("10000", "50000", "", new Decimal(20))!;
    expect(r.net.toString()).toBe("-8000");
    expect(r.odenecek).toBe(false);
  });
  it("iki alan boşsa null", () => {
    expect(computeDonemNet("", "", "", new Decimal(20))).toBeNull();
  });
});

describe("kalan devreden", () => {
  it("kayıt yoksa null", () => {
    expect(kalanDevreden([])).toBeNull();
  });
  it("önceki dönem ödendiyse 0", () => {
    expect(kalanDevreden([rec({ net: "7000", odenecek: true })])!.toString()).toBe("0");
  });
  it("önceki dönem devreden kaldıysa mutlak değer", () => {
    expect(kalanDevreden([rec({ net: "-8000", odenecek: false })])!.toString()).toBe("8000");
  });
});

describe("sıralama ve özet", () => {
  it("donemId ve donemAd", () => {
    expect(donemId(2026, 3)).toBe("2026-03");
    expect(donemAd({ year: 2026, month: 3 })).toBe("Mart 2026");
  });
  it("yeni dönem üstte", () => {
    const s = sortByDonemId([rec({ id: "2026-01" }), rec({ id: "2026-03" }), rec({ id: "2026-02" })]);
    expect(s.map((x) => x.id)).toEqual(["2026-03", "2026-02", "2026-01"]);
  });
  it("ozet toplamları", () => {
    const o = donemOzet([
      rec({ hesaplanan: "20000", indirilecek: "12000", net: "8000", odenecek: true }),
      rec({ hesaplanan: "5000", indirilecek: "13000", net: "-8000", odenecek: false }),
    ]);
    expect(o.hesaplanan.toString()).toBe("25000");
    expect(o.net.toString()).toBe("0");
    expect(o.odenecekNet.toString()).toBe("8000");
    expect(o.devredenNet.toString()).toBe("8000");
    expect(o.sayi).toBe(2);
  });
});

// ——— Yeni: yıllık gruplama ve güvenli sayı çözümlemesi ———

const yilRec = (yil: number, ay: number, net: string, odenecek = true): DonemRecord =>
  rec({ id: `${yil}-${String(ay).padStart(2, "0")}`, year: yil, month: ay,
    net, odenecek, hesaplanan: "200", indirilecek: "100" });

describe("yillaraGir", () => {
  it("yillari yeniden eskiye, aylari yeniden eskiye siralar", () => {
    const g = yillaraGir([yilRec(2025, 3, "100"), yilRec(2026, 1, "50"), yilRec(2026, 3, "80"), yilRec(2025, 11, "60")]);
    expect(g.map((x) => x.yil)).toEqual([2026, 2025]);
    expect(g[0].kayitlar.map((x) => x.id)).toEqual(["2026-03", "2026-01"]);
    expect(g[1].kayitlar.map((x) => x.id)).toEqual(["2025-11", "2025-03"]);
  });

  it("her yilin kendi toplamini hesaplar", () => {
    const g = yillaraGir([yilRec(2026, 1, "100"), yilRec(2026, 2, "50")]);
    expect(g[0].ozet.hesaplanan.toString()).toBe("400");
    expect(g[0].ozet.indirilecek.toString()).toBe("200");
    expect(g[0].ozet.net.toString()).toBe("150");
    expect(g[0].ozet.sayi).toBe(2);
  });

  it("bos listede bos dondurur", () => {
    expect(yillaraGir([])).toEqual([]);
  });
});

describe("parseDonemSayisi", () => {
  it("bos/bozuk/ifade iceren girdileri reddeder", () => {
    expect(parseDonemSayisi("")).toBeNull();
    expect(parseDonemSayisi("abc")).toBeNull();
    expect(parseDonemSayisi("1+1")).toBeNull();
    expect(parseDonemSayisi("1.000.000")).toBeNull();
  });

  it("sayisal degerleri kabul eder", () => {
    expect(parseDonemSayisi("1234.56")?.toString()).toBe("1234.56");
    expect(parseDonemSayisi("-30")?.toString()).toBe("-30");
    expect(parseDonemSayisi("1,5")?.toString()).toBe("1.5");
  });
});
