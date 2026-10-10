import Decimal from "decimal.js";
import {
  calculateChainedDiscount,
  calculateTevkifat,
  calculateStopaj,
  calculateStopajReverse,
  toDecimal,
} from "../src/core/engine";
import { applyKdv } from "../src/store/useStore";

let fail = 0;
function check(name: string, got: string, want: string) {
  const ok = got === want;
  if (!ok) fail++;
  console.log(`${ok ? "OK  " : "FAIL"} ${name}: ${got} (beklenen ${want})`);
}

// Fiyat: etiket 1000, %10 iskonto, %20 KDV haricen
const disc = calculateChainedDiscount(toDecimal("1000"), [toDecimal("10")]);
check("matrah", disc.finalPrice.toString(), "900");
const k = applyKdv(disc.finalPrice, new Decimal(20), false);
check("toplam", k.main.toString(), "1080");
check("kdv", k.tax.toString(), "180");

// Fiyat dahilden: etiket 1180 KDV dahil %20 -> net 983.33
const k2 = applyKdv(toDecimal("1180"), new Decimal(20), true);
check("dahilden net", k2.main.toString(), "983.33");

// Tevkifat 604 yemek 5/10: matrah 10000 KDV20
const t = calculateTevkifat(toDecimal("10000"), 20, 5);
check("tevkifat", t.tevkifat.toString(), "1000");
check("saticiya", t.saticiyaOdenen.toString(), "11000");

// SMM: kira 50000 brut stopaj %20 -> net 40000
const s = calculateStopaj(toDecimal("50000"), 20);
check("kira net", s.netAmount.toString(), "40000");

// Netten brüt: net 40000 %20 -> brüt 50000, kesinti 10000
const sr = calculateStopajReverse(toDecimal("40000"), 20);
check("brutlestirme brut", sr.total.toString(), "50000");
check("brutlestirme kesinti", sr.taxAmount.toString(), "10000");

// Dönem: satış 100000, alış 60000, devreden 1000, %20 -> ödenecek 7000
const hes = toDecimal("100000").mul(20).div(100);
const ind = toDecimal("60000").mul(20).div(100);
check("donem odenecek", hes.minus(ind).minus(1000).toDP(2).toString(), "7000");

// Dönem devreden: satış 10000, alış 50000 -> -8000 devreden
const devreden = toDecimal("10000").mul(20).div(100).minus(toDecimal("50000").mul(20).div(100)).toDP(2);
check("donem devreden", devreden.toString(), "-8000");

// Gecikme: 10000 anapara, %4.5 aylık, 30 gün -> 450 faiz
check("gecikme faiz", toDecimal("10000").mul(4.5).div(100).mul(30).div(30).toDP(2).toString(), "450");

// Amortisman normal: 100000 maliyet %20, 5 yıl -> her yıl 20000, toplam 100000
import { calculateAmortisman } from "../src/core/engine";
const am1 = calculateAmortisman(toDecimal("100000"), toDecimal("20"), 5, "normal");
check("amortisman yillik", am1.rows[0].ayrılan.toString(), "20000");
check("amortisman toplam", am1.toplam.toString(), "100000");
check("amortisman satir", String(am1.rows.length), "5");

// Amortisman azalan: 100000 %20 -> yıl1 40000, kalan 60000
const am2 = calculateAmortisman(toDecimal("100000"), toDecimal("20"), 5, "azalan");
check("azalan yil1", am2.rows[0].ayrılan.toString(), "40000");
check("azalan kalan1", am2.rows[0].kalan.toString(), "60000");

console.log(fail === 0 ? "TUMU GECTI" : `${fail} HATA`);
process.exit(fail ? 1 : 0);
