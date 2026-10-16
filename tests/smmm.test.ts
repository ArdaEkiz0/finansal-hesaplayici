import { describe, it, expect } from "vitest";
import Decimal from "decimal.js";
import {
  calculateChainedDiscount,
  calculateTevkifat,
  calculateStopaj,
  calculateStopajReverse,
  calculateAmortisman,
  toDecimal,
} from "../src/core/engine";
import { applyKdv } from "../src/store/useStore";

// Fiyat: etiket 1000, %10 iskonto, %20 KDV haricen
const disc = calculateChainedDiscount(toDecimal("1000"), [toDecimal("10")]);

// Tevkifat 604 yemek 5/10: matrah 10000 KDV20
const t = calculateTevkifat(toDecimal("10000"), 20, 5);

// SMM: kira 50000 brut stopaj %20 -> net 40000
const s = calculateStopaj(toDecimal("50000"), 20);

// Netten brüt: net 40000 %20 -> brüt 50000, kesinti 10000
const sr = calculateStopajReverse(toDecimal("40000"), 20);

// Amortisman normal: 100000 maliyet %20, 5 yıl -> her yıl 20000, toplam 100000
const am1 = calculateAmortisman(toDecimal("100000"), toDecimal("20"), 5, "normal");

// Amortisman azalan: 100000 %20 -> yıl1 40000, kalan 60000
const am2 = calculateAmortisman(toDecimal("100000"), toDecimal("20"), 5, "azalan");

describe("fiyat", () => {
  it("iskonto matrahi", () => {
    expect(disc.finalPrice.toString()).toBe("900");
  });
  it("KDV haricen toplam", () => {
    const k = applyKdv(disc.finalPrice, new Decimal(20), false);
    expect(k.main.toString()).toBe("1080");
    expect(k.tax.toString()).toBe("180");
  });
  it("KDV dahilden net", () => {
    const k2 = applyKdv(toDecimal("1180"), new Decimal(20), true);
    expect(k2.main.toString()).toBe("983.33");
  });
});

describe("tevkifat", () => {
  it("tevkifat tutari", () => expect(t.tevkifat.toString()).toBe("1000"));
  it("saticiya odenen", () => expect(t.saticiyaOdenen.toString()).toBe("11000"));
});

describe("stopaj", () => {
  it("kira net", () => expect(s.netAmount.toString()).toBe("40000"));
  it("brutlestirme brüt", () => expect(sr.total.toString()).toBe("50000"));
  it("brutlestirme kesinti", () => expect(sr.taxAmount.toString()).toBe("10000"));
});

describe("donem", () => {
  it("ödenecek: 100000 satiş, 60000 alış, 1000 devreden, %20 -> 7000", () => {
    const hes = toDecimal("100000").mul(20).div(100);
    const ind = toDecimal("60000").mul(20).div(100);
    expect(hes.minus(ind).minus(1000).toDP(2).toString()).toBe("7000");
  });
  it("devreden: 10000 satiş, 50000 alış -> -8000", () => {
    const d = toDecimal("10000").mul(20).div(100).minus(toDecimal("50000").mul(20).div(100)).toDP(2);
    expect(d.toString()).toBe("-8000");
  });
});

describe("gecikme", () => {
  it("10000 anapara, %4.5 aylık, 30 gün -> 450 faiz", () => {
    expect(toDecimal("10000").mul(4.5).div(100).mul(30).div(30).toDP(2).toString()).toBe("450");
  });
});

describe("amortisman", () => {
  it("normal yillik ayirilan", () => expect(am1.rows[0].ayrılan.toString()).toBe("20000"));
  it("normal toplam", () => expect(am1.toplam.toString()).toBe("100000"));
  it("normal satir sayisi", () => expect(am1.rows.length).toBe(5));
  it("azalan yil1", () => expect(am2.rows[0].ayrılan.toString()).toBe("40000"));
  it("azalan kalan1", () => expect(am2.rows[0].kalan.toString()).toBe("60000"));
});
