import Decimal from "decimal.js";
import {
  calculateChainedDiscount,
  calculateTevkifat,
  calculateStopaj,
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
import { calculateStopaj } from "C:/Users/ozel/OneDrive/Masaüstü/KDV HESAPLAYICI/finansal-hesaplayici/src/core/engine";
const s = calculateStopaj(toDecimal("50000"), 20);
check("kira net", s.netAmount.toString(), "40000");

console.log(fail === 0 ? "TUMU GECTI" : `${fail} HATA`);
process.exit(fail ? 1 : 0);
