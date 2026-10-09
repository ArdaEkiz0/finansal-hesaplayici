import { buildBatchRows } from "../src/core/engine";
import Decimal from "decimal.js";

let fail = 0;
function check(name: string, got: string, want: string) {
  const ok = got === want;
  if (!ok) fail++;
  console.log(`${ok ? "OK  " : "FAIL"} ${name}: ${got} (beklenen ${want})`);
}

// KDV toplu: 1000, 2500, 3750 -> toplam 7250, KDV 1450, dahil 8700
const r1 = buildBatchRows("1000\n2500\n3750", {
  mode: "kdv",
  kdvRate: new Decimal(20),
  kdvExtract: false,
  stopajRate: 20,
  tevkifatPay: 5,
  precision: 2,
});
check("satir sayisi", String(r1.rows.length), "3");
check("toplam matrah", r1.totals.base!, "7.250,00");
check("toplam kdv", r1.totals.tax!, "1.450,00");
check("toplam dahil", r1.totals.main!, "8.700,00");
check("satir1", r1.rows[0].result, "1.200,00");

// Stopaj toplu: 50000, 30000 -> net 64000
const r2 = buildBatchRows("50000\n30000", {
  mode: "stopaj",
  kdvRate: new Decimal(20),
  kdvExtract: false,
  stopajRate: 20,
  tevkifatPay: 5,
  precision: 2,
});
check("stopaj net toplam", r2.totals.main!, "64.000,00");
check("stopaj kesinti toplam", r2.totals.tax!, "16.000,00");

// Tevkifat toplu: 10000, 20000 -> saticiya 11000 + 22000 = 33000
const r3 = buildBatchRows("10000\n20000", {
  mode: "tevkifat",
  kdvRate: new Decimal(20),
  kdvExtract: false,
  stopajRate: 20,
  tevkifatPay: 5,
  precision: 2,
});
check("tevkifat saticiya toplam", r3.totals.main!, "33.000,00");
check("tevkifat kesinti toplam", r3.totals.tax!, "3.000,00");

// Gecersiz satirlar atlanir
const r4 = buildBatchRows("1000\nabc\n2500", {
  mode: "kdv",
  kdvRate: new Decimal(20),
  kdvExtract: false,
  stopajRate: 20,
  tevkifatPay: 5,
  precision: 2,
});
check("gecersiz atlama", String(r4.validCount), "2");

console.log(fail === 0 ? "TUMU GECTI" : `${fail} HATA`);
process.exit(fail ? 1 : 0);
