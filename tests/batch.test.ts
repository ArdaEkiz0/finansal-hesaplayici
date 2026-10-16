import { describe, it, expect } from "vitest";
import { buildBatchRows } from "../src/core/engine";
import Decimal from "decimal.js";

const opts = {
  kdvRate: new Decimal(20),
  kdvExtract: false,
  stopajRate: 20,
  tevkifatPay: 5,
  precision: 2,
} as const;

// KDV toplu: 1000, 2500, 3750 -> toplam 7250, KDV 1450, dahil 8700
const r1 = buildBatchRows("1000\n2500\n3750", { mode: "kdv", ...opts });

// Stopaj toplu: 50000, 30000 -> net 64000
const r2 = buildBatchRows("50000\n30000", { mode: "stopaj", ...opts });

// Tevkifat toplu: 10000, 20000 -> saticiya 11000 + 22000 = 33000
const r3 = buildBatchRows("10000\n20000", { mode: "tevkifat", ...opts });

// Gecersiz satirlar atlanir
const r4 = buildBatchRows("1000\nabc\n2500", { mode: "kdv", ...opts });

describe("toplu kdv", () => {
  it("satir sayisi", () => expect(r1.rows.length).toBe(3));
  it("toplam matrah", () => expect(r1.totals.base).toBe("7.250,00"));
  it("toplam kdv", () => expect(r1.totals.tax).toBe("1.450,00"));
  it("toplam dahil", () => expect(r1.totals.main).toBe("8.700,00"));
  it("satir1", () => expect(r1.rows[0].result).toBe("1.200,00"));
});

describe("toplu stopaj", () => {
  it("net toplam", () => expect(r2.totals.main).toBe("64.000,00"));
  it("kesinti toplam", () => expect(r2.totals.tax).toBe("16.000,00"));
});

describe("toplu tevkifat", () => {
  it("saticiya toplam", () => expect(r3.totals.main).toBe("33.000,00"));
  it("kesinti toplam", () => expect(r3.totals.tax).toBe("3.000,00"));
});

describe("toplu gecersizler", () => {
  it("gecersiz atlanir", () => expect(r4.validCount).toBe(2));
});
