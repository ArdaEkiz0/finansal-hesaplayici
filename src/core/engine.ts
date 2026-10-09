import Decimal from "decimal.js";

Decimal.set({ precision: 30, rounding: Decimal.ROUND_HALF_UP });

export type KVDRate = 1 | 10 | 20;
export type StopajRate = 1 | 3 | 5 | 7 | 10 | 15 | 17 | 20;
export type TevkifatPay = 2 | 3 | 4 | 5 | 6 | 7 | 9 | 10;

export interface TaxResult {
  taxAmount: Decimal;
  total: Decimal;
  netAmount: Decimal;
}

export interface MarginResult {
  sellingPrice: Decimal;
  profit: Decimal;
  marginPercent: Decimal;
}

export interface MarkupResult {
  sellingPrice: Decimal;
  profit: Decimal;
  markupPercent: Decimal;
}

export interface CompoundResult {
  futureValue: Decimal;
  totalInterest: Decimal;
  effectiveRate: Decimal;
}

export interface KDVCompareResult {
  extract: TaxResult;
  inject: TaxResult;
  difference: Decimal;
}

export interface TevkifatResult {
  matrah: Decimal;
  kdv: Decimal;
  tevkifat: Decimal;
  saticiyaOdenen: Decimal;
  beyan: Decimal;
}

export function toDecimal(value: string | number): Decimal {
  if (typeof value === "string") {
    const cleaned = value.replace(/\s/g, "").replace(/,/g, ".");
    if (!cleaned || cleaned === "." || cleaned === "-") return new Decimal(0);
    try {
      return new Decimal(cleaned);
    } catch {
      return new Decimal(0);
    }
  }
  try {
    return new Decimal(value);
  } catch {
    return new Decimal(0);
  }
}

export function formatTurkishNumber(value: Decimal, decimals = 2): string {
  if (!value.isFinite()) return "—";
  const fixed = value.toDP(decimals).toFixed(decimals);
  const [intPart, decPart] = fixed.split(".");
  const neg = intPart.startsWith("-");
  const digits = neg ? intPart.slice(1) : intPart;
  const withDots = digits.replace(/\B(?=(\d{3})+(?!\d))/g, ".");
  const head = neg ? "-" + withDots : withDots;
  if (decimals === 0 || !decPart) return head;
  return head + "," + decPart;
}

export function formatCurrency(value: Decimal, decimals = 2): string {
  return formatTurkishNumber(value, decimals);
}

export function formatCurrencyWithSymbol(value: Decimal, decimals = 2): string {
  return formatTurkishNumber(value, decimals) + " ₺";
}

export function formatPercent(value: Decimal, decimals = 2): string {
  return formatTurkishNumber(value, decimals) + "%";
}

export function calculateKDV(amount: Decimal, rate: KVDRate, extract: boolean): TaxResult {
  const r = new Decimal(rate);
  if (extract) {
    const total = amount;
    const net = total.div(r.div(100).plus(1));
    const tax = total.minus(net);
    return { taxAmount: tax.toDP(2), total: total.toDP(2), netAmount: net.toDP(2) };
  }
  const tax = amount.mul(r).div(100);
  const total = amount.plus(tax);
  return { taxAmount: tax.toDP(2), total: total.toDP(2), netAmount: amount.toDP(2) };
}

export function calculateStopaj(gross: Decimal, rate: StopajRate): TaxResult {
  const r = new Decimal(rate);
  const tax = gross.mul(r).div(100);
  const net = gross.minus(tax);
  return { taxAmount: tax.toDP(2), total: gross.toDP(2), netAmount: net.toDP(2) };
}

export function calculateTevkifat(matrah: Decimal, kdvRate: KVDRate, pay: TevkifatPay): TevkifatResult {
  const kdv = matrah.mul(kdvRate).div(100).toDP(2);
  const tevkifat = kdv.mul(pay).div(10).toDP(2);
  const saticiyaOdenen = matrah.plus(kdv).minus(teklifFix(tevkifat)).toDP(2);
  return { matrah: matrah.toDP(2), kdv, tevkifat, saticiyaOdenen, beyan: tevkifat };
}

function teklifFix(v: Decimal): Decimal {
  return v;
}

export function calculateMargin(cost: Decimal, targetMargin: Decimal): MarginResult {
  if (targetMargin.gte(100) || targetMargin.isNeg()) {
    return { sellingPrice: new Decimal(0), profit: new Decimal(0), marginPercent: targetMargin };
  }
  const divisor = new Decimal(1).minus(targetMargin.div(100));
  if (divisor.isZero() || divisor.isNeg()) {
    return { sellingPrice: new Decimal(0), profit: new Decimal(0), marginPercent: targetMargin };
  }
  const sp = cost.div(divisor);
  const profit = sp.minus(cost);
  const mp = sp.isZero() ? new Decimal(0) : profit.div(sp).mul(100);
  return { sellingPrice: sp.toDP(2), profit: profit.toDP(2), marginPercent: mp.toDP(2) };
}

export function calculateMarkup(cost: Decimal, targetMarkup: Decimal): MarkupResult {
  if (targetMarkup.isNeg()) {
    return { sellingPrice: cost.toDP(2), profit: new Decimal(0), markupPercent: new Decimal(0) };
  }
  const profit = cost.mul(targetMarkup).div(100);
  const sp = cost.plus(profit);
  const mp = sp.isZero() ? new Decimal(0) : profit.div(sp).mul(100);
  return { sellingPrice: sp.toDP(2), profit: profit.toDP(2), markupPercent: mp.toDP(2) };
}

export function calculateChainedDiscount(originalPrice: Decimal, discounts: Decimal[]) {
  let price = originalPrice;
  const stepByStep: { rate: Decimal; priceAfter: Decimal }[] = [];
  for (const d of discounts) {
    const saved = price.mul(d).div(100);
    price = price.minus(saved);
    stepByStep.push({ rate: d, priceAfter: price.toDP(2) });
  }
  const totalSaved = originalPrice.minus(price);
  const effectiveDiscount = originalPrice.isZero() ? new Decimal(0) : totalSaved.div(originalPrice).mul(100);
  return {
    finalPrice: price.toDP(2),
    totalSaved: totalSaved.toDP(2),
    effectiveDiscount: effectiveDiscount.toDP(2),
    stepByStep,
  };
}

export function calculatePercentOf(percent: Decimal, value: Decimal): Decimal {
  return value.mul(percent).div(100).toDP(4);
}

export function calculatePercentChange(oldValue: Decimal, newValue: Decimal): Decimal {
  if (oldValue.isZero()) return new Decimal(0);
  return newValue.minus(oldValue).div(oldValue.abs()).mul(100).toDP(2);
}

export function calculateReversePercent(percent: Decimal, value: Decimal): Decimal {
  if (percent.isZero()) return value.toDP(2);
  return value.div(percent.div(100).plus(1)).toDP(2);
}

export function calculateCompoundInterest(
  principal: Decimal,
  annualRate: Decimal,
  years: Decimal,
  compoundsPerYear: number = 12
): CompoundResult {
  const r = annualRate.div(100);
  const n = new Decimal(compoundsPerYear);
  const nt = n.mul(years);
  const ratePerPeriod = r.div(n);
  const base = new Decimal(1).plus(ratePerPeriod);
  const fv = principal.mul(base.pow(nt));
  const interest = fv.minus(principal);
  const effective = principal.isZero() ? new Decimal(0) : fv.div(principal).minus(1).mul(100);
  return {
    futureValue: fv.toDP(2),
    totalInterest: interest.toDP(2),
    effectiveRate: effective.toDP(4),
  };
}

export function calculateKDVCompare(amount: Decimal, rate: KVDRate): KDVCompareResult {
  const extract = calculateKDV(amount, rate, true);
  const inject = calculateKDV(amount, rate, false);
  const diff = inject.taxAmount.minus(extract.taxAmount);
  return { extract, inject, difference: diff.toDP(2) };
}

/** Guvenli dort-islem degerlendirici (numpad + - * / icin). Hata durumunda null doner, throw etmez. */
export function evaluateExpression(expr: string): Decimal | null {
  const s = expr.replace(/\s/g, "");
  if (!s || !/^[0-9.+*/()%-]+$/.test(s)) return null;
  const js = s.replace(/(\d+(?:\.\d+)?)%/g, "($1/100)");
  if (/[^0-9.+*/()-]/.test(js)) return null;
  let i = 0;
  function peek(): string {
    return i < js.length ? js[i] : "";
  }
  function parseExpr(): Decimal {
    let v = parseTerm();
    for (;;) {
      const c = peek();
      if (c === "+" || c === "-") {
        i++;
        const r = parseTerm();
        v = c === "+" ? v.plus(r) : v.minus(r);
      } else return v;
    }
  }
  function parseTerm(): Decimal {
    let v = parseFactor();
    for (;;) {
      const c = peek();
      if (c === "*" || c === "/") {
        i++;
        const r = parseFactor();
        if (c === "/") {
          if (r.isZero()) return new Decimal(NaN);
          v = v.div(r);
        } else v = v.mul(r);
      } else return v;
    }
  }
  function parseFactor(): Decimal {
    const c = peek();
    if (c === "+") {
      i++;
      return parseFactor();
    }
    if (c === "-") {
      i++;
      return parseFactor().neg();
    }
    if (c === "(") {
      i++;
      const v = parseExpr();
      if (peek() !== ")") throw new Error("paren");
      i++;
      return v;
    }
    let num = "";
    while (/[0-9.]/.test(peek())) num += js[i++];
    if (!num || num === "." || (num.match(/\./g) || []).length > 1) throw new Error("num");
    return new Decimal(num);
  }
  try {
    const v = parseExpr();
    if (i !== js.length || !v.isFinite()) return null;
    return v;
  } catch {
    return null;
  }
}

/** Tutar alanini cozer: duz sayi veya "100+50*2" gibi ifade. Bos/gecersizde null. */
export function parseAmount(raw: string): Decimal | null {
  const expr = (raw || "").replace(/\s/g, "").replace(/,/g, ".");
  if (!expr) return null;
  return evaluateExpression(expr);
}

const ONES = ["", "bir", "iki", "üç", "dört", "beş", "altı", "yedi", "sekiz", "dokuz"];
const TENS = ["", "on", "yirmi", "otuz", "kırk", "elli", "altmış", "yetmiş", "seksen", "doksan"];

function threeDigitsToWords(n: number): string {
  const h = Math.floor(n / 100);
  const t = Math.floor((n % 100) / 10);
  const o = n % 10;
  let s = "";
  if (h > 0) s += h === 1 ? "yüz" : ONES[h] + "yüz";
  if (t > 0) s += (s ? " " : "") + TENS[t];
  if (o > 0) s += (s ? " " : "") + ONES[o];
  return s;
}

/** Toplu hesap satiri ureticisi. */
export function buildBatchRows(raw: string, opts: {
  mode: "kdv" | "stopaj" | "tevkifat" | "fiyat";
  kdvRate: Decimal;
  kdvExtract: boolean;
  stopajRate: number;
  tevkifatPay: number;
  precision: number;
}): { rows: { inputs: string; result: string; detail?: string }[]; totals: { base?: string; tax?: string; main?: string }; validCount: number } {
  const p = opts.precision;
  const fmt = (v: Decimal, d = p) => formatTurkishNumber(v, d);
  const rows: { inputs: string; result: string; detail?: string }[] = [];
  let baseSum = new Decimal(0);
  let taxSum = new Decimal(0);
  let mainSum = new Decimal(0);
  let validCount = 0;
  for (const line of raw.split(/\r?\n/)) {
    const value = parseAmount(line);
    if (!value) continue;
    validCount++;
    if (opts.mode === "kdv") {
      const k = applyKdvLocal(value, opts.kdvRate, opts.kdvExtract);
      baseSum = baseSum.plus(value);
      taxSum = taxSum.plus(k.tax);
      mainSum = mainSum.plus(k.main);
      rows.push({ inputs: line, result: fmt(k.main), detail: `${fmt(k.tax)} KDV` });
    } else if (opts.mode === "stopaj") {
      const s = calculateStopaj(value, opts.stopajRate as StopajRate);
      baseSum = baseSum.plus(value);
      taxSum = taxSum.plus(s.taxAmount);
      mainSum = mainSum.plus(s.netAmount);
      rows.push({ inputs: line, result: fmt(s.netAmount), detail: `${fmt(s.taxAmount)} kesinti` });
    } else if (opts.mode === "tevkifat") {
      const t = calculateTevkifat(value, opts.kdvRate.toNumber() as KVDRate, opts.tevkifatPay as TevkifatPay);
      baseSum = baseSum.plus(value);
      taxSum = taxSum.plus(t.tevkifat);
      mainSum = mainSum.plus(t.saticiyaOdenen);
      rows.push({ inputs: line, result: fmt(t.saticiyaOdenen), detail: `${fmt(t.tevkifat)} tevkifat` });
    } else {
      const k = applyKdvLocal(value, opts.kdvRate, opts.kdvExtract);
      baseSum = baseSum.plus(value);
      taxSum = taxSum.plus(k.tax);
      mainSum = mainSum.plus(k.main);
      rows.push({ inputs: line, result: fmt(k.main), detail: `${fmt(k.tax)} KDV` });
    }
  }
  return { rows, totals: { base: fmt(baseSum), tax: fmt(taxSum), main: fmt(mainSum) }, validCount };
}

function applyKdvLocal(base: Decimal, eff: Decimal, extract: boolean): { main: Decimal; tax: Decimal } {
  if (extract) {
    const net = base.div(eff.div(100).plus(1)).toDP(2);
    return { main: net, tax: base.minus(net).toDP(2) };
  }
  const tax = base.mul(eff).div(100).toDP(2);
  return { main: base.plus(tax).toDP(2), tax };
}

export function numberToTurkishWords(value: Decimal): string {
  try {
    const fixed = value.toDP(2).toFixed(2);
    const neg = fixed.startsWith("-");
    const clean = neg ? fixed.slice(1) : fixed;
    const [intStr, decStr] = clean.split(".");
    let int = parseInt(intStr, 10);
    if (isNaN(int)) return "";
    if (int === 0 && decStr === "00") return "sıfır lira";
    const groups: string[] = [];
    const scales = ["", "bin", "milyon", "milyar", "trilyon"];
    let gi = 0;
    if (int === 0) {
      groups.push("");
    } else {
      const parts: string[] = [];
      while (int > 0 && gi < scales.length) {
        const tri = int % 1000;
        if (tri > 0) {
          let w = threeDigitsToWords(tri);
          if (gi === 1 && tri === 1) w = "";
          parts.unshift((w ? w + " " : "") + scales[gi]);
        }
        int = Math.floor(int / 1000);
        gi++;
      }
      groups.push(parts.join(" ").trim());
    }
    let out = (neg ? "eksi " : "") + groups.join(" ").trim() + " lira";
    const kurus = parseInt(decStr, 10);
    if (!isNaN(kurus) && kurus > 0) {
      out += " " + threeDigitsToWords(kurus).trim() + " kuruş";
    }
    return out.replace(/\s+/g, " ").trim();
  } catch {
    return "";
  }
}
