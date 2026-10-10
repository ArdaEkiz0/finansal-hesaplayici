import { create } from "zustand";
import Decimal from "decimal.js";
import { loadState, saveState } from "../lib/persist";
import {
  toDecimal,
  parseAmount,
  formatTurkishNumber,
  calculateKDV,
  calculateStopaj,
  calculateStopajReverse,
  calculateTevkifat,
  calculateMargin,
  calculateMarkup,
  calculateChainedDiscount,
  calculatePercentOf,
  calculatePercentChange,
  calculateReversePercent,
  calculateKDVCompare,
  calculateCompoundInterest,
  calculateAmortisman,
  buildBatchRows,
  type KVDRate,
  type StopajRate,
  type TevkifatPay,
} from "../core/engine";

export type CalcMode = "kdv" | "stopaj" | "tevkifat" | "margin" | "markup" | "discount" | "fiyat" | "percent" | "compound" | "kdvCompare" | "batch" | "donem" | "gecikme" | "amortisman" | "doviz";

export interface HistoryEntry {
  id: string;
  mode: CalcMode;
  inputs: string;
  result: string;
  detail?: string;
  note?: string;
  timestamp: number;
  pinned: boolean;
}

export interface CalcResult {
  display: string;
  detail: string;
  secondary?: string;
  third?: string;
  words?: string;
  rows?: { label: string; value: string; accent?: boolean }[];
}

const TWO_INPUT_MODES: CalcMode[] = ["margin", "markup", "percent", "compound", "gecikme", "amortisman"];
export const MODE_LABELS: Record<CalcMode, string> = {
  kdv: "KDV",
  stopaj: "Stopaj",
  tevkifat: "Tevkifat",
  margin: "Marj",
  markup: "Kâr Oranı",
  discount: "İndirim",
  fiyat: "Fiyat",
  percent: "Yüzde",
  compound: "Bileşik",
  kdvCompare: "Karşılaştır",
  batch: "Toplu",
  donem: "Dönem KDV",
  gecikme: "Gecikme",
  amortisman: "Amortisman",
  doviz: "Döviz",
};

export type Theme = "dark" | "light";

interface State {
  input: string;
  secondInput: string;
  activeField: "first" | "second";
  mode: CalcMode;
  kdvRate: KVDRate;
  stopajRate: StopajRate;
  stopajDirection: "brutten" | "netten";
  tevkifatPay: TevkifatPay;
  tevkifatCode: string;
  donemSatis: string;
  donemAlis: string;
  donemDevreden: string;
  activeDonemField: "satis" | "alis" | "devreden";
  gecikmeDays: string;
  amortismanYears: string;
  amortismanMethod: "normal" | "azalan";
  fxRates: Record<string, { buy: number; sell: number }> | null;
  fxDate: string;
  fxManual: string;
  fxCurrency: "USD" | "EUR" | "GBP";
  fxDirection: "toTL" | "fromTL";
  kdvExtract: boolean;
  customKdvRate: string;
  useCustomRate: boolean;
  result: CalcResult | null;
  history: HistoryEntry[];
  discountInputs: string[];
  activeDiscountIndex: number;
  compoundYears: string;
  compoundFrequency: number;
  batchInput: string;
  batchKind: "kdv" | "stopaj" | "tevkifat" | "fiyat";
  batchResult: { rows: { inputs: string; result: string; detail?: string }[]; totals: { base?: string; tax?: string; main?: string }; validCount: number } | null;
  showSettings: boolean;
  showHelp: boolean;
  showRates: boolean;
  showWidget: boolean;
  showInvoice: boolean;
  theme: Theme;
  settings: { showCurrencySymbol: boolean; precision: number };
  notification: { id: number; message: string; type: "success" | "error" | "info" } | null;

  setBatchInput: (v: string) => void;
  setBatchKind: (k: "kdv" | "stopaj" | "tevkifat" | "fiyat") => void;
  setMode: (m: CalcMode) => void;
  setInputDirect: (v: string) => void;
  setSecondInputDirect: (v: string) => void;
  setKdvRate: (r: KVDRate) => void;
  setStopajRate: (r: StopajRate) => void;
  setStopajDirection: (d: "brutten" | "netten") => void;
  setTevkifatPay: (p: TevkifatPay) => void;
  setTevkifatCode: (code: string, pay: TevkifatPay) => void;
  setDonemField: (f: "satis" | "alis" | "devreden", v: string) => void;
  setActiveDonemField: (f: "satis" | "alis" | "devreden") => void;
  setGecikmeDays: (d: string) => void;
  setAmortismanYears: (y: string) => void;
  setAmortismanMethod: (m: "normal" | "azalan") => void;
  setFxManual: (v: string) => void;
  setFxCurrency: (c: "USD" | "EUR" | "GBP") => void;
  setFxDirection: (d: "toTL" | "fromTL") => void;
  fetchFx: () => void;
  setHistoryNote: (id: string, note: string) => void;
  setKdvExtract: (e: boolean) => void;
  setCustomKdvRate: (v: string) => void;
  setUseCustomRate: (b: boolean) => void;
  appendDigit: (d: string) => void;
  deleteLast: () => void;
  clearAll: () => void;
  confirmInput: () => void;
  switchField: () => void;
  addDiscount: () => void;
  removeDiscount: (i: number) => void;
  setActiveDiscountIndex: (i: number) => void;
  setCompoundYears: (y: string) => void;
  setCompoundFrequency: (f: number) => void;
  toggleSettings: () => void;
  toggleHelp: () => void;
  toggleRates: () => void;
  toggleWidget: () => void;
  toggleInvoice: () => void;
  setTheme: (t: Theme) => void;
  updateSettings: (p: Partial<State["settings"]>) => void;
  togglePinHistory: (id: string) => void;
  deleteHistoryEntry: (id: string) => void;
  loadHistoryEntry: (id: string) => void;
  clearHistory: () => void;
  exportHistory: () => void;
  notify: (msg: string, type?: "success" | "error" | "info") => void;
}

/** Matraha KDV uygular: extract=true ise verilen tutar KDV dahildir. */
export function applyKdv(base: Decimal, eff: Decimal, extract: boolean): { main: Decimal; tax: Decimal } {
  if (extract) {
    const net = base.div(eff.div(100).plus(1)).toDP(2);
    return { main: net, tax: base.minus(net).toDP(2) };
  }
  const tax = base.mul(eff).div(100).toDP(2);
  return { main: base.plus(tax).toDP(2), tax };
}

export function effectiveKdvRate(s: { kdvRate: KVDRate; useCustomRate: boolean; customKdvRate: string }): Decimal {
  if (s.useCustomRate) {
    const v = toDecimal(s.customKdvRate || "0");
    if (v.isNeg()) return new Decimal(0);
    if (v.gt(100)) return new Decimal(100);
    return v;
  }
  return new Decimal(s.kdvRate);
}

function computeInner(s: State): CalcResult | null {
  if (!s.input && s.mode !== "discount") return null;
  if (s.mode === "discount" && !s.input) {
    if (s.discountInputs.some((d) => d !== "")) {
      return { display: "Tutarı girin", detail: "Önce ana tutar" };
    }
    return null;
  }
  const a = parseAmount(s.input);
  if (!a) return { display: "—", detail: "Geçersiz ifade (örn: 100+50)" };
  const p = s.settings.precision;
  const fmt = s.settings.showCurrencySymbol
    ? (v: Decimal) => formatTurkishNumber(v, p) + " ₺"
    : (v: Decimal) => formatTurkishNumber(v, p);
  const pct = (v: Decimal) => formatTurkishNumber(v, p) + "%";

  switch (s.mode) {
    case "kdv": {
      const eff = effectiveKdvRate(s);
      const rNum = eff.toNumber();
      const norm = ([1, 10, 20].includes(rNum) ? (rNum as KVDRate) : 20) as KVDRate;
      const useEff = s.useCustomRate;
      const kdvAmt = useEff ? a.mul(eff).div(100).toDP(2) : null;
      if (useEff) {
        if (s.kdvExtract) {
          const net = a.div(eff.div(100).plus(1)).toDP(2);
          const tax = a.minus(net).toDP(2);
          return {
            display: fmt(net),
            detail: `Dahilden Hariç — %${eff.toString()} KDV`,
            rows: [
              { label: "Net Tutar", value: fmt(net), accent: true },
              { label: `%${eff.toString()} KDV`, value: fmt(tax) },
              { label: "Brüt (girdiğiniz)", value: fmt(a) },
            ],
          };
        }
        const total = a.plus(kdvAmt!).toDP(2);
        return {
          display: fmt(total),
          detail: `Hariciden Dahil — %${eff.toString()} KDV`,
          rows: [
            { label: "Toplam", value: fmt(total), accent: true },
            { label: "Net (girdiğiniz)", value: fmt(a) },
            { label: `%${eff.toString()} KDV`, value: fmt(kdvAmt!) },
          ],
        };
      }
      const r = calculateKDV(a, norm, s.kdvExtract);
      const dir = s.kdvExtract ? "Dahilden Hariç" : "Hariciden Dahil";
      const main = s.kdvExtract ? r.netAmount : r.total;
      return {
        display: fmt(main),
        detail: `${dir} — %${norm} KDV`,
        rows: [
          { label: s.kdvExtract ? "Net Tutar" : "Toplam", value: fmt(main), accent: true },
          { label: `%${norm} KDV`, value: fmt(r.taxAmount) },
          { label: s.kdvExtract ? "Brüt (girdiğiniz)" : "Net (girdiğiniz)", value: fmt(a) },
        ],
      };
    }
    case "stopaj": {
      if (s.stopajDirection === "netten") {
        const r = calculateStopajReverse(a, s.stopajRate);
        return {
          display: fmt(r.total),
          detail: `Netten Brüt — %${s.stopajRate} Stopaj`,
          rows: [
            { label: "Brüt", value: fmt(r.total), accent: true },
            { label: "Net (girdiğiniz)", value: fmt(a) },
            { label: `%${s.stopajRate} Kesinti`, value: fmt(r.taxAmount) },
          ],
        };
      }
      const r = calculateStopaj(a, s.stopajRate);
      return {
        display: fmt(r.netAmount),
        detail: `Brütten Net — %${s.stopajRate} Stopaj`,
        rows: [
          { label: "Net Ödenen", value: fmt(r.netAmount), accent: true },
          { label: `%${s.stopajRate} Kesinti`, value: fmt(r.taxAmount) },
          { label: "Brüt (girdiğiniz)", value: fmt(r.total) },
        ],
      };
    }
    case "tevkifat": {
      const r = calculateTevkifat(a, s.kdvRate, s.tevkifatPay);
      const codeSuffix = s.tevkifatCode ? ` • Kod ${s.tevkifatCode}` : "";
      return {
        display: fmt(r.saticiyaOdenen),
        detail: `Tevkifat ${s.tevkifatPay}/10 — %${s.kdvRate} KDV${codeSuffix}`,
        rows: [
          { label: "Satıcıya Ödenen", value: fmt(r.saticiyaOdenen), accent: true },
          { label: "Matrah", value: fmt(r.matrah) },
          { label: `%${s.kdvRate} KDV`, value: fmt(r.kdv) },
          { label: `Tevkifat (${s.tevkifatPay}/10)`, value: fmt(r.tevkifat) },
          { label: "2 No'lu Beyan", value: fmt(r.beyan) },
          ...(s.tevkifatCode ? [{ label: "İşlem kodu", value: s.tevkifatCode }] : []),
        ],
      };
    }
    case "margin": {
      if (!s.secondInput) return { display: fmt(a), detail: "Hedef marjı girin (%)" };
      const r = calculateMargin(a, toDecimal(s.secondInput));
      return {
        display: fmt(r.sellingPrice),
        detail: `Satış Fiyatı — Marj: ${pct(r.marginPercent)}`,
        rows: [
          { label: "Satış Fiyatı", value: fmt(r.sellingPrice), accent: true },
          { label: "Maliyet", value: fmt(a) },
          { label: "Kâr", value: fmt(r.profit) },
          { label: "Gerçek Marj", value: pct(r.marginPercent) },
        ],
      };
    }
    case "markup": {
      if (!s.secondInput) return { display: fmt(a), detail: "Kâr oranını girin (%)" };
      const r = calculateMarkup(a, toDecimal(s.secondInput));
      return {
        display: fmt(r.sellingPrice),
        detail: `Satış Fiyatı — Kâr Oranı: ${pct(toDecimal(s.secondInput))}`,
        rows: [
          { label: "Satış Fiyatı", value: fmt(r.sellingPrice), accent: true },
          { label: "Maliyet", value: fmt(a) },
          { label: "Kâr", value: fmt(r.profit) },
          { label: "Marj Karşılığı", value: pct(r.markupPercent) },
        ],
      };
    }
    case "discount": {
      const ds = s.discountInputs.filter((d) => d !== "").map(toDecimal);
      if (ds.length === 0) return { display: fmt(a), detail: "İndirim oranlarını girin" };
      const r = calculateChainedDiscount(a, ds);
      const list = ds.map((d) => "%" + formatTurkishNumber(d, p)).join(" → ");
      return {
        display: fmt(r.finalPrice),
        detail: `İndirimler: ${list}`,
        rows: [
          { label: "Son Fiyat", value: fmt(r.finalPrice), accent: true },
          { label: "Orijinal", value: fmt(a) },
          { label: "Tasarruf", value: fmt(r.totalSaved) },
          { label: "Etkin İndirim", value: pct(r.effectiveDiscount) },
          ...r.stepByStep.map((st, i) => ({
            label: `Adım ${i + 1} (%${formatTurkishNumber(st.rate, p)})`,
            value: fmt(st.priceAfter),
          })),
        ],
      };
    }
    case "fiyat": {
      const ds = s.discountInputs.filter((d) => d !== "").map(toDecimal);
      const disc = ds.length > 0 ? calculateChainedDiscount(a, ds) : null;
      const matrah = disc ? disc.finalPrice : a.toDP(2);
      const eff = effectiveKdvRate(s);
      const k = applyKdv(matrah, eff, s.kdvExtract);
      const dir = s.kdvExtract ? "KDV dahil etiket" : "KDV hariç matrah";
      return {
        display: fmt(k.main),
        detail: `Fiyat — ${dir}, %${eff.toString()} KDV`,
        rows: [
          { label: s.kdvExtract ? "Ödenecek" : "Toplam", value: fmt(k.main), accent: true },
          { label: "Etiket", value: fmt(a) },
          ...(disc
            ? [
                { label: "İndirimler", value: ds.map((d) => "%" + formatTurkishNumber(d, p)).join(" → ") },
                { label: "Tasarruf", value: fmt(disc.totalSaved) },
              ]
            : []),
          { label: "Matrah", value: fmt(matrah) },
          { label: `%${eff.toString()} KDV`, value: fmt(k.tax) },
        ],
      };
    }
    case "percent": {
      if (!s.secondInput) return { display: fmt(a), detail: "İkinci değeri girin" };
      const b = toDecimal(s.secondInput);
      return {
        display: fmt(calculatePercentOf(a, b)),
        detail: `%${formatTurkishNumber(a, p)} of ${formatTurkishNumber(b, p)}`,
        rows: [
          { label: "Sonuç", value: fmt(calculatePercentOf(a, b)), accent: true },
          { label: "Değişim %", value: pct(calculatePercentChange(a, b)) },
          { label: "Ters Hesap", value: fmt(calculateReversePercent(a, b)) },
        ],
      };
    }
    case "compound": {
      if (!s.secondInput) return { display: fmt(a), detail: "Yıllık faizi girin (%)" };
      const years = toDecimal(s.compoundYears || "1");
      if (years.isZero() || years.isNeg()) return { display: fmt(a), detail: "Yıl sayısını ayarlayın" };
      const r = calculateCompoundInterest(a, toDecimal(s.secondInput), years, s.compoundFrequency);
      const fl: Record<number, string> = { 1: "Yıllık", 2: "6 Aylık", 4: "3 Aylık", 12: "Aylık", 365: "Günlük" };
      return {
        display: fmt(r.futureValue),
        detail: `Vade Sonu — ${fl[s.compoundFrequency] || s.compoundFrequency} bileşik`,
        rows: [
          { label: "Vade Sonu", value: fmt(r.futureValue), accent: true },
          { label: "Ana Para", value: fmt(a) },
          { label: "Faiz", value: fmt(r.totalInterest) },
          { label: "Efektif Oran", value: pct(r.effectiveRate) },
        ],
      };
    }
    case "gecikme": {
      if (!s.secondInput) return { display: fmt(a), detail: "Aylık gecikme oranını girin (%)" };
      const rate = toDecimal(s.secondInput);
      const days = toDecimal(s.gecikmeDays || "0");
      const interest = a.mul(rate).div(100).mul(days).div(30).toDP(2);
      const total = a.plus(interest).toDP(2);
      return {
        display: fmt(total),
        detail: `Gecikme — %${formatTurkishNumber(rate, p)} aylık, ${formatTurkishNumber(days, 0)} gün`,
        rows: [
          { label: "Toplam (anapara + faiz)", value: fmt(total), accent: true },
          { label: "Anapara", value: fmt(a) },
          { label: "Gecikme faizi", value: fmt(interest) },
        ],
      };
    }
    case "amortisman": {
      if (!s.secondInput) return { display: fmt(a), detail: "Amortisman oranını girin (%)" };
      const years = Math.min(50, Math.max(1, parseInt(s.amortismanYears || "5", 10) || 5));
      const r = calculateAmortisman(a, toDecimal(s.secondInput), years, s.amortismanMethod);
      const methodLabel = s.amortismanMethod === "normal" ? "Normal" : "Azalan bakiyeler";
      return {
        display: fmt(r.toplam),
        detail: `Amortisman — ${methodLabel}, %${formatTurkishNumber(toDecimal(s.secondInput), p)}`,
        rows: [
          { label: "Toplam ayrılan", value: fmt(r.toplam), accent: true },
          { label: "Maliyet", value: fmt(a) },
          ...r.rows.map((row) => ({
            label: `Yıl ${row.year} (kalan ${fmt(row.kalan)})`,
            value: fmt(row.ayrılan),
          })),
        ],
      };
    }
    case "doviz": {
      const manual = s.fxManual ? parseAmount(s.fxManual) : null;
      const live = s.fxRates?.[s.fxCurrency];
      const liveRate = live ? (s.fxDirection === "toTL" ? live.buy : live.sell) : null;
      const rateNum = manual ? manual.toNumber() : liveRate;
      if (!rateNum || rateNum <= 0) return { display: fmt(a), detail: "Önce kuru getirin veya elle girin" };
      const rate = new Decimal(rateNum);
      const out = (s.fxDirection === "toTL" ? a.mul(rate) : a.div(rate)).toDP(2);
      const src = manual ? "Elle girilen kur" : `TCMB ${s.fxDate || ""}`.trim();
      return {
        display: s.fxDirection === "toTL" ? fmt(out) : formatTurkishNumber(out, p),
        detail: `${s.fxCurrency} → TL (${s.fxDirection === "toTL" ? "alış" : "satış"}) • ${src}`,
        rows: [
          { label: "Sonuç", value: s.fxDirection === "toTL" ? fmt(out) : formatTurkishNumber(out, p), accent: true },
          { label: "Girdi", value: formatTurkishNumber(a, p) + (s.fxDirection === "toTL" ? ` ${s.fxCurrency}` : " ₺") },
          { label: `Kur (1 ${s.fxCurrency})`, value: formatTurkishNumber(rate, 4) + " ₺" },
        ],
      };
    }
    case "kdvCompare": {
      const r = calculateKDVCompare(a, s.kdvRate);
      return {
        display: fmt(r.inject.total),
        detail: `Karşılaştırma — %${s.kdvRate}`,
        rows: [
          { label: "Hariciden Dahil Toplam", value: fmt(r.inject.total), accent: true },
          { label: "Dahilden Hariç Net", value: fmt(r.extract.netAmount) },
          { label: "KDV Farkı", value: fmt(r.difference) },
        ],
      };
    }
    default:
      return null;
  }
}

function computeDonem(s: State): CalcResult | null {
  if (!s.donemSatis && !s.donemAlis && !s.donemDevreden) return null;
  const p = s.settings.precision;
  const fmt = s.settings.showCurrencySymbol
    ? (v: Decimal) => formatTurkishNumber(v, p) + " ₺"
    : (v: Decimal) => formatTurkishNumber(v, p);
  const satis = parseAmount(s.donemSatis || "") ?? new Decimal(0);
  const alis = parseAmount(s.donemAlis || "") ?? new Decimal(0);
  const devr = parseAmount(s.donemDevreden || "") ?? new Decimal(0);
  const eff = effectiveKdvRate(s);
  const hesaplanan = satis.mul(eff).div(100).toDP(2);
  const indirilecek = alis.mul(eff).div(100).toDP(2);
  const net = hesaplanan.minus(indirilecek).minus(devr).toDP(2);
  const odenecek = !net.isNeg();
  const main = net.abs().toDP(2);
  return {
    display: fmt(main),
    detail: `Dönem — %${eff.toString()} • ${odenecek ? "Ödenecek KDV" : "Devreden KDV"}`,
    rows: [
      { label: odenecek ? "Ödenecek KDV" : "Devreden KDV", value: fmt(main), accent: true },
      { label: "Satış matrahı", value: fmt(satis) },
      { label: "Hesaplanan KDV", value: fmt(hesaplanan) },
      { label: "Alış matrahı", value: fmt(alis) },
      { label: "İndirilecek KDV", value: fmt(indirilecek) },
      { label: "Önceki devreden", value: fmt(devr) },
    ],
  };
}

const EXPR_MODES: CalcMode[] = ["kdv", "stopaj", "tevkifat", "kdvCompare", "fiyat"];

function compute(s: State): CalcResult | null {
  if (s.mode === "donem") return computeDonem(s);
  if (s.mode === "batch") return null;
  const r = computeInner(s);
  if (r?.rows && /[+\-*/()%]/.test(s.input) && EXPR_MODES.includes(s.mode)) {
    const a = parseAmount(s.input);
    if (a) {
      const v = formatTurkishNumber(a, s.settings.precision) + (s.settings.showCurrencySymbol ? " ₺" : "");
      r.rows.unshift({ label: `Girdi: ${s.input} =`, value: v });
    }
  }
  return r;
}

function refreshBatch(s: State): { batchResult: State["batchResult"] } {
  const lines = s.batchInput.split(/\r?\n/).filter((l) => l.trim() !== "");
  if (lines.length < 2) return { batchResult: null };
  const r = buildBatchRows(s.batchInput, {
    mode: s.batchKind,
    kdvRate: effectiveKdvRate(s),
    kdvExtract: s.kdvExtract,
    stopajRate: s.stopajRate,
    tevkifatPay: s.tevkifatPay,
    precision: s.settings.precision,
  });
  return { batchResult: r.validCount >= 2 ? r : null };
}

const persisted = loadState();

let notifId = 0;

export const useStore = create<State>((set, get) => ({
  input: "",
  secondInput: "",
  activeField: "first",
  mode: "kdv",
  kdvRate: 20,
  stopajRate: 15,
  stopajDirection: "brutten" as "brutten" | "netten",
  tevkifatPay: 5,
  tevkifatCode: "",
  donemSatis: "",
  donemAlis: "",
  donemDevreden: "",
  activeDonemField: "satis" as "satis" | "alis" | "devreden",
  gecikmeDays: "30",
  amortismanYears: "5",
  amortismanMethod: "normal" as "normal" | "azalan",
  fxRates: null,
  fxDate: "",
  fxManual: "",
  fxCurrency: "USD" as "USD" | "EUR" | "GBP",
  fxDirection: "toTL" as "toTL" | "fromTL",
  kdvExtract: false,
  customKdvRate: "",
  useCustomRate: false,
  result: null,
  history: (persisted?.history ?? []) as HistoryEntry[],
  discountInputs: ["", ""],
  activeDiscountIndex: 0,
  compoundYears: "1",
  compoundFrequency: 12,
  batchInput: "",
  batchKind: "kdv" as "kdv" | "stopaj" | "tevkifat" | "fiyat",
  batchResult: null,
  showSettings: false,
  showHelp: false,
  showRates: false,
  showWidget: false,
  showInvoice: false,
  theme: (persisted?.theme as Theme) ?? "dark",
  settings: persisted?.settings ?? { showCurrencySymbol: true, precision: 2 },
  notification: null,

  setBatchInput: (batchInput) => {
    const s = get();
    const clean = batchInput.replace(/[^\d.,+\-*/()\r\n\s]/g, "").slice(0, 4000);
    set({ batchInput: clean });
    set(refreshBatch({ ...get(), batchInput: clean }));
  },
  setBatchKind: (batchKind) => {
    set({ batchKind });
    const s = get();
    if (s.batchInput) set(refreshBatch({ ...s, batchKind }));
  },
  setMode: (mode) => set({ mode, input: "", secondInput: "", result: null, activeField: "first", discountInputs: ["", ""], activeDiscountIndex: 0, batchInput: "", batchResult: null, donemSatis: "", donemAlis: "", donemDevreden: "", activeDonemField: "satis" }),
  setInputDirect: (v) => {
    const s = get();
    const clean = v.replace(/[^0-9.,+\-*/()]/g, "").replace(/,/g, ".").slice(0, 30);
    set({ input: clean, result: clean ? compute({ ...s, input: clean }) : null });
  },
  setSecondInputDirect: (v) => {
    const s = get();
    const clean = v.replace(/[^0-9.,]/g, "").replace(/,/g, ".");
    set({ secondInput: clean, result: compute({ ...s, secondInput: clean }) });
  },
  setKdvRate: (kdvRate) => { set({ kdvRate, useCustomRate: false }); const s = get(); if (s.input || s.mode === "discount" || s.mode === "fiyat") set({ result: compute({ ...s, kdvRate, useCustomRate: false }) }); if (s.mode === "batch" && s.batchInput) set(refreshBatch(get())); },
  setStopajRate: (stopajRate) => { set({ stopajRate }); const s = get(); if (s.input) set({ result: compute({ ...s, stopajRate }) }); if (s.mode === "batch" && s.batchInput) set(refreshBatch(get())); },
  setStopajDirection: (stopajDirection) => { set({ stopajDirection }); const s = get(); if (s.input) set({ result: compute({ ...s, stopajDirection }) }); },
  setDonemField: (f, v) => {
    const clean = v.replace(/[^0-9.,+\-*/()]/g, "").replace(/,/g, ".").slice(0, 30);
    const key = f === "satis" ? "donemSatis" : f === "alis" ? "donemAlis" : "donemDevreden";
    set({ [key]: clean } as Partial<State>);
    const s = get();
    set({ result: compute({ ...s, [key]: clean } as State) });
  },
  setActiveDonemField: (activeDonemField) => set({ activeDonemField }),
  setGecikmeDays: (gecikmeDays) => {
    const clean = gecikmeDays.replace(/[^0-9.]/g, "").slice(0, 6);
    set({ gecikmeDays: clean });
    const s = get();
    if (s.input) set({ result: compute({ ...s, gecikmeDays: clean }) });
  },
  setAmortismanYears: (amortismanYears) => {
    const clean = amortismanYears.replace(/[^0-9]/g, "").slice(0, 2);
    set({ amortismanYears: clean });
    const s = get();
    if (s.input) set({ result: compute({ ...s, amortismanYears: clean }) });
  },
  setAmortismanMethod: (amortismanMethod) => {
    set({ amortismanMethod });
    const s = get();
    if (s.input) set({ result: compute({ ...s, amortismanMethod }) });
  },
  setFxManual: (fxManual) => {
    const clean = fxManual.replace(/[^0-9.,]/g, "").replace(/,/g, ".").slice(0, 12);
    set({ fxManual: clean });
    const s = get();
    if (s.input) set({ result: compute({ ...s, fxManual: clean }) });
  },
  setFxCurrency: (fxCurrency) => {
    set({ fxCurrency });
    const s = get();
    if (s.input) set({ result: compute({ ...s, fxCurrency }) });
  },
  setFxDirection: (fxDirection) => {
    set({ fxDirection });
    const s = get();
    if (s.input) set({ result: compute({ ...s, fxDirection }) });
  },
  fetchFx: async () => {
    const api = window.electronAPI;
    if (!api?.getFx) {
      get().notify("Kurlar masaüstü uygulamasında alınır", "info");
      return;
    }
    get().notify("TCMB kurları alınıyor...", "info");
    try {
      const fx = await api.getFx();
      if (fx?.rates) {
        set({ fxRates: fx.rates, fxDate: fx.date || "" });
        const s = get();
        if (s.input) set({ result: compute(s) });
        get().notify(`Kurlar güncellendi (${fx.date || "bugün"})`, "success");
      } else {
        get().notify("Kur alınamadı, elle girebilirsiniz", "error");
      }
    } catch {
      get().notify("Kur alınamadı, elle girebilirsiniz", "error");
    }
  },
  setHistoryNote: (id, note) => set((s) => ({ history: s.history.map((e) => (e.id === id ? { ...e, note } : e)) })),
  setTevkifatPay: (tevkifatPay) => { set({ tevkifatPay }); const s = get(); if (s.input) set({ result: compute({ ...s, tevkifatPay }) }); },
  setTevkifatCode: (tevkifatCode, pay) => { set({ tevkifatCode, tevkifatPay: pay }); const s = get(); if (s.input) set({ result: compute({ ...s, tevkifatCode, tevkifatPay: pay }) }); if (s.mode === "batch" && s.batchInput) set(refreshBatch(get())); },
  setKdvExtract: (kdvExtract) => { set({ kdvExtract }); const s = get(); if (s.input) set({ result: compute({ ...s, kdvExtract }) }); if (s.mode === "batch" && s.batchInput) set(refreshBatch(get())); },
  setCustomKdvRate: (customKdvRate) => {
    const clean = customKdvRate.replace(/[^0-9.,]/g, "").replace(/,/g, ".");
    set({ customKdvRate: clean, useCustomRate: clean !== "" });
    const s = get();
    if (s.input) set({ result: compute({ ...s, customKdvRate: clean, useCustomRate: clean !== "" }) });
  },
  setUseCustomRate: (useCustomRate) => { set({ useCustomRate }); const s = get(); if (s.input) set({ result: compute({ ...s, useCustomRate }) }); },

  appendDigit: (digit) => {
    const s = get();
    const isOp = digit === "+" || digit === "-" || digit === "*" || digit === "/";
    if (s.mode === "donem") {
      const key = s.activeDonemField === "satis" ? "donemSatis" : s.activeDonemField === "alis" ? "donemAlis" : "donemDevreden";
      const cur = (s[key] ?? "") as string;
      if (digit === ".") {
        const seg = cur.split(/[+\-*/()]/).pop() ?? "";
        if (seg.includes(".")) return;
      }
      const v = (cur + digit).slice(0, 30);
      set({ [key]: v } as Partial<State>);
      const ns = get();
      set({ result: compute({ ...ns, [key]: v } as State) });
      return;
    }
    const segHasDot = (cur: string) => {
      const seg = cur.split(/[+\-*/()]/).pop() ?? "";
      return seg.includes(".");
    };
    if (s.mode === "discount" || s.mode === "fiyat") {
      if (s.activeField === "first") {
        const v = (s.input + digit).slice(0, 30);
        if (digit === "." && segHasDot(s.input)) return;
        set({ input: v, result: compute({ ...s, input: v }) });
        return;
      }
      if (isOp) return;
      const d = [...s.discountInputs];
      const cur = (d[s.activeDiscountIndex] ?? "") + digit;
      if ((cur.match(/\./g) || []).length > 1) return;
      d[s.activeDiscountIndex] = cur.slice(0, 7);
      set({ discountInputs: d, result: compute({ ...s, discountInputs: d }) });
      return;
    }
    const useSecond = TWO_INPUT_MODES.includes(s.mode) && s.activeField === "second";
    if (useSecond) {
      if (isOp) return;
      const v = (s.secondInput + digit).slice(0, 15);
      if ((v.match(/\./g) || []).length > 1) return;
      set({ secondInput: v, result: compute({ ...s, secondInput: v }) });
    } else {
      const v = (s.input + digit).slice(0, 30);
      if (digit === "." && segHasDot(s.input)) return;
      set({ input: v, result: compute({ ...s, input: v }) });
    }
  },

  deleteLast: () => {
    const s = get();
    if (s.mode === "donem") {
      const key = s.activeDonemField === "satis" ? "donemSatis" : s.activeDonemField === "alis" ? "donemAlis" : "donemDevreden";
      const v = ((s[key] ?? "") as string).slice(0, -1);
      set({ [key]: v } as Partial<State>);
      const ns = get();
      set({ result: compute({ ...ns, [key]: v } as State) });
      return;
    }
    if ((s.mode === "discount" || s.mode === "fiyat") && s.activeField !== "first") {
      const d = [...s.discountInputs];
      d[s.activeDiscountIndex] = (d[s.activeDiscountIndex] ?? "").slice(0, -1);
      set({ discountInputs: d, result: compute({ ...s, discountInputs: d }) });
      return;
    }
    const useSecond = TWO_INPUT_MODES.includes(s.mode) && s.activeField === "second";
    if (useSecond) {
      const v = s.secondInput.slice(0, -1);
      set({ secondInput: v, result: compute({ ...s, secondInput: v }) });
    } else {
      const v = s.input.slice(0, -1);
      set({ input: v, result: s.input.length <= 1 ? null : compute({ ...s, input: v }) });
    }
  },

  clearAll: () => set({ input: "", secondInput: "", result: null, activeField: "first", discountInputs: ["", ""], activeDiscountIndex: 0, donemSatis: "", donemAlis: "", donemDevreden: "", activeDonemField: "satis" }),

  switchField: () => {
    const s = get();
    if (s.mode === "donem") {
      set({ activeDonemField: s.activeDonemField === "satis" ? "alis" : s.activeDonemField === "alis" ? "devreden" : "satis" });
      return;
    }
    if (s.mode === "discount" || s.mode === "fiyat") {
      set({ activeField: s.activeField === "first" ? "second" : "first" });
      return;
    }
    if (!TWO_INPUT_MODES.includes(s.mode)) return;
    set({ activeField: s.activeField === "first" ? "second" : "first" });
  },

  confirmInput: () => {
    const s = get();
    if (s.mode === "donem") {
      if (s.result && s.result.display !== "—" && (s.donemSatis || s.donemAlis || s.donemDevreden)) {
        const entry: HistoryEntry = {
          id: Date.now().toString(), mode: s.mode,
          inputs: `Satış ${s.donemSatis || "0"} / Alış ${s.donemAlis || "0"} / Devreden ${s.donemDevreden || "0"}`,
          result: s.result.display, detail: s.result.detail, timestamp: Date.now(), pinned: false,
        };
        set({ history: [entry, ...s.history].slice(0, 100), donemSatis: "", donemAlis: "", donemDevreden: "", activeDonemField: "satis", result: null });
        get().notify("Kaydedildi", "success");
      }
      return;
    }
    if (s.mode === "batch") {
      if (s.batchResult && s.batchResult.validCount >= 2) {
        const entry: HistoryEntry = {
          id: Date.now().toString(), mode: s.mode,
          inputs: `${s.batchResult.validCount} satır toplu`,
          result: s.batchResult.totals.main ?? "", detail: `Matrah ${s.batchResult.totals.base} • Vergi ${s.batchResult.totals.tax}`, timestamp: Date.now(), pinned: false,
        };
        set({ history: [entry, ...s.history].slice(0, 100), batchInput: "", batchResult: null });
        get().notify("Kaydedildi", "success");
      }
      return;
    }
    if (s.mode === "fiyat") {
      if (s.result && s.input && s.result.display !== "—") {
        const eff = effectiveKdvRate(s);
        const entry: HistoryEntry = {
          id: Date.now().toString(), mode: s.mode,
          inputs: `${s.input} → ` + s.discountInputs.filter((d) => d !== "").map((d) => "%" + d).join(" → ") + ` → KDV%${eff.toString()}`,
          result: s.result.display, detail: s.result.detail, timestamp: Date.now(), pinned: false,
        };
        set({ history: [entry, ...s.history].slice(0, 100), input: "", result: null, discountInputs: ["", ""], activeDiscountIndex: 0, activeField: "first" });
        get().notify("Kaydedildi", "success");
      }
      return;
    }
    if (s.mode === "discount") {
      if (s.result && s.input) {
        const entry: HistoryEntry = {
          id: Date.now().toString(), mode: s.mode,
          inputs: `${s.input} → ` + s.discountInputs.filter((d) => d !== "").map((d) => "%" + d).join(" → "),
          result: s.result.display, detail: s.result.detail, timestamp: Date.now(), pinned: false,
        };
        set({ history: [entry, ...s.history].slice(0, 100), input: "", result: null, discountInputs: ["", ""], activeDiscountIndex: 0, activeField: "first" });
        get().notify("Kaydedildi", "success");
      }
      return;
    }
    if (TWO_INPUT_MODES.includes(s.mode)) {
      if (s.activeField === "first" && s.input) { set({ activeField: "second" }); return; }
      if (s.activeField === "second" && s.input && s.secondInput && s.result) {
        const entry: HistoryEntry = {
          id: Date.now().toString(), mode: s.mode,
          inputs: `${s.input} / ${s.secondInput}`,
          result: s.result.display, detail: s.result.detail, timestamp: Date.now(), pinned: false,
        };
        set({ history: [entry, ...s.history].slice(0, 100), input: "", secondInput: "", activeField: "first", result: null });
        get().notify("Kaydedildi", "success");
        return;
      }
      return;
    }
    if (s.result && s.input) {
      const entry: HistoryEntry = {
        id: Date.now().toString(), mode: s.mode, inputs: s.input,
        result: s.result.display, detail: s.result.detail, timestamp: Date.now(), pinned: false,
      };
      set({ history: [entry, ...s.history].slice(0, 100), input: "", secondInput: "", activeField: "first", result: null, discountInputs: ["", ""], activeDiscountIndex: 0 });
      get().notify("Kaydedildi", "success");
    }
  },

  addDiscount: () => {
    const s = get();
    if (s.discountInputs.length >= 5) return;
    const d = [...s.discountInputs, ""];
    set({ discountInputs: d, activeDiscountIndex: d.length - 1, activeField: "second" });
  },

  removeDiscount: (index) => {
    const s = get();
    if (s.discountInputs.length <= 1) return;
    const d = s.discountInputs.filter((_, i) => i !== index);
    set({ discountInputs: d, activeDiscountIndex: Math.min(s.activeDiscountIndex, d.length - 1), result: compute({ ...s, discountInputs: d }) });
  },

  setActiveDiscountIndex: (i) => set({ activeDiscountIndex: i, activeField: "second" }),
  setCompoundYears: (compoundYears) => {
    const clean = compoundYears.replace(/[^0-9.,]/g, "").replace(/,/g, ".");
    set({ compoundYears: clean });
    const s = get();
    if (s.input) set({ result: compute({ ...s, compoundYears: clean }) });
  },
  setCompoundFrequency: (f) => { set({ compoundFrequency: f }); const s = get(); if (s.input) set({ result: compute({ ...s, compoundFrequency: f }) }); },

  toggleSettings: () => set((s) => ({ showSettings: !s.showSettings })),
  toggleHelp: () => set((s) => ({ showHelp: !s.showHelp })),
  toggleRates: () => set((s) => ({ showRates: !s.showRates })),
  toggleWidget: () => {
    if (window.electronAPI?.widgetToggle) {
      window.electronAPI.widgetToggle();
      return;
    }
    set((s) => ({ showWidget: !s.showWidget }));
  },
  toggleInvoice: () => set((s) => ({ showInvoice: !s.showInvoice })),
  setTheme: (theme) => { set({ theme }); document.documentElement.setAttribute("data-theme", theme); },

  updateSettings: (p) => {
    const s = get();
    const ns = { ...s.settings, ...p };
    set({ settings: ns });
    if (s.input) set({ result: compute({ ...s, settings: ns }) });
  },

  togglePinHistory: (id) => set((s) => ({ history: s.history.map((e) => (e.id === id ? { ...e, pinned: !e.pinned } : e)) })),
  deleteHistoryEntry: (id) => set((s) => ({ history: s.history.filter((e) => e.id !== id) })),

  loadHistoryEntry: (id) => {
    const s = get();
    const entry = s.history.find((e) => e.id === id);
    if (!entry) return;
    if (entry.mode === "discount") {
      const m = entry.inputs.match(/^([\d.]+) → (.*)$/);
      if (m) {
        const rates = m[2].split(" → ").map((x) => x.replace("%", ""));
        set({ mode: "discount", input: m[1], discountInputs: rates.length ? rates : [""], activeDiscountIndex: 0, activeField: "first" });
        const ns = get();
        set({ result: compute(ns) });
        return;
      }
    }
    const parts = entry.inputs.split(" / ");
    const main = parts[0]?.replace(/[^0-9.]/g, "") ?? "";
    const second = parts[1]?.replace(/[^0-9.]/g, "") ?? "";
    const ns = { ...s, mode: entry.mode, input: main, secondInput: second, activeField: "first" as const, result: null };
    set({ mode: entry.mode, input: main, secondInput: second, activeField: "first", result: compute(ns as State) });
  },

  clearHistory: () => set({ history: [] }),

  exportHistory: () => {
    const { history } = get();
    if (history.length === 0) return;
    const rows = [
      "Tarih,Mod,Girdiler,Sonuç,Açıklama,Etiket",
      ...history.map((e) => `"${new Date(e.timestamp).toLocaleString("tr-TR")}","${MODE_LABELS[e.mode]}","${e.inputs}","${e.result}","${e.detail || ""}","${e.note || ""}"`),
    ];
    const blob = new Blob(["﻿" + rows.join("\n")], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `hesaplayici-${new Date().toISOString().slice(0, 10)}.csv`;
    link.click();
    URL.revokeObjectURL(url);
  },

  notify: (message, type = "info") => {
    notifId += 1;
    const id = notifId;
    set({ notification: { id, message, type } });
    setTimeout(() => {
      const cur = get().notification;
      if (cur?.id === id) set({ notification: null });
    }, 2600);
  },
}));

useStore.subscribe((s) => saveState({ history: s.history, settings: s.settings, theme: s.theme }));
