import { describe, it, expect } from "vitest";
import Decimal from "decimal.js";
import {
  computeDonemNet,
  kalanDevreden,
  sortByDonemId,
  donemOzet,
  donemId,
  donemAd,
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
