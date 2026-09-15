import Decimal from "decimal.js";

Decimal.set({ precision: 30, rounding: Decimal.ROUND_HALF_UP });

export type KVDRate = 1 | 10 | 20;
export type StopajRate = 1 | 3 | 5 | 7 | 10 | 15 | 20;

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

export function toDecimal(value: string | number): Decimal {
  if (typeof value === "string") {
    const cleaned = value.replace(/,/g, ".");
    return new Decimal(cleaned || "0");
  }
  return new Decimal(value);
}

export function formatTurkishNumber(value: Decimal, decimals = 2): string {
  const fixed = value.toDP(decimals).toFixed(decimals);
  const [intPart, decPart] = fixed.split(".");
  const withDots = intPart.replace(/\B(?=(\d{3})+(?!\d))/g, ".");
  if (decimals === 0 || !decPart) return withDots;
  return withDots + "," + decPart;
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

export function calculateMargin(cost: Decimal, targetMargin: Decimal): MarginResult {
  if (targetMargin.gte(100)) {
    return { sellingPrice: new Decimal(Infinity), profit: new Decimal(Infinity), marginPercent: targetMargin };
  }
  const divisor = new Decimal(1).minus(targetMargin.div(100));
  if (divisor.isZero() || divisor.isNeg()) {
    return { sellingPrice: new Decimal(Infinity), profit: new Decimal(Infinity), marginPercent: targetMargin };
  }
  const sp = cost.div(divisor);
  const profit = sp.minus(cost);
  const mp = sp.isZero() ? new Decimal(0) : profit.div(sp).mul(100);
  return { sellingPrice: sp.toDP(2), profit: profit.toDP(2), marginPercent: mp.toDP(2) };
}

export function calculateMarkup(cost: Decimal, targetMarkup: Decimal): MarkupResult {
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
  return value.mul(percent).div(100);
}

export function calculatePercentChange(oldValue: Decimal, newValue: Decimal): Decimal {
  if (oldValue.isZero()) return new Decimal(0);
  return newValue.minus(oldValue).div(oldValue.abs()).mul(100);
}

export function calculateReversePercent(percent: Decimal, value: Decimal): Decimal {
  if (percent.isZero()) return value;
  return value.div(percent.div(100).plus(1));
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
  const effective = fv.div(principal).minus(1).mul(100);
  return {
    futureValue: fv.toDP(2),
    totalInterest: interest.toDP(2),
    effectiveRate: effective.toDP(4),
  };
}

export function calculateKDVCompare(amount: Decimal, rate: KVDRate): KDVCompareResult {
  const extract = calculateKDV(amount, rate, true);
  const inject = calculateKDV(amount, rate, false);
  const diff = inject.total.minus(extract.total);
  return { extract, inject, difference: diff.toDP(2) };
}
