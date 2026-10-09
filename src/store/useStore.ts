import { create } from "zustand";
import Decimal from "decimal.js";
import { loadState, saveState } from "../lib/persist";
import {
  toDecimal,
  formatTurkishNumber,
  calculateKDV,
  calculateStopaj,
  calculateTevkifat,
  calculateMargin,
  calculateMarkup,
  calculateChainedDiscount,
  calculatePercentOf,
  calculatePercentChange,
  calculateReversePercent,
  calculateKDVCompare,
  calculateCompoundInterest,
  type KVDRate,
  type StopajRate,
  type TevkifatPay,
} from "../core/engine";

export type CalcMode = "kdv" | "stopaj" | "tevkifat" | "margin" | "markup" | "discount" | "percent" | "compound" | "kdvCompare";

export interface HistoryEntry {
  id: string;
  mode: CalcMode;
  inputs: string;
  result: string;
  detail?: string;
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

const TWO_INPUT_MODES: CalcMode[] = ["margin", "markup", "percent", "compound"];
export const MODE_LABELS: Record<CalcMode, string> = {
  kdv: "KDV",
  stopaj: "Stopaj",
  tevkifat: "Tevkifat",
  margin: "Marj",
  markup: "Kâr Oranı",
  discount: "İndirim",
  percent: "Yüzde",
  compound: "Bileşik",
  kdvCompare: "Karşılaştır",
};

export type Theme = "dark" | "light";

interface State {
  input: string;
  secondInput: string;
  activeField: "first" | "second";
  mode: CalcMode;
  kdvRate: KVDRate;
  stopajRate: StopajRate;
  tevkifatPay: TevkifatPay;
  kdvExtract: boolean;
  customKdvRate: string;
  useCustomRate: boolean;
  result: CalcResult | null;
  history: HistoryEntry[];
  discountInputs: string[];
  activeDiscountIndex: number;
  compoundYears: string;
  compoundFrequency: number;
  showSettings: boolean;
  showHelp: boolean;
  showWidget: boolean;
  showInvoice: boolean;
  theme: Theme;
  settings: { showCurrencySymbol: boolean; precision: number };
  notification: { id: number; message: string; type: "success" | "error" | "info" } | null;

  setMode: (m: CalcMode) => void;
  setInputDirect: (v: string) => void;
  setSecondInputDirect: (v: string) => void;
  setKdvRate: (r: KVDRate) => void;
  setStopajRate: (r: StopajRate) => void;
  setTevkifatPay: (p: TevkifatPay) => void;
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

export function effectiveKdvRate(s: { kdvRate: KVDRate; useCustomRate: boolean; customKdvRate: string }): Decimal {
  if (s.useCustomRate) {
    const v = toDecimal(s.customKdvRate || "0");
    if (v.isNeg()) return new Decimal(0);
    if (v.gt(100)) return new Decimal(100);
    return v;
  }
  return new Decimal(s.kdvRate);
}

function compute(s: State): CalcResult | null {
  if (!s.input && s.mode !== "discount") return null;
  if (s.mode === "discount" && !s.input) {
    if (s.discountInputs.some((d) => d !== "")) {
      return { display: "Tutarı girin", detail: "Önce ana tutar" };
    }
    return null;
  }
  const a = toDecimal(s.input || "0");
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
      const r = calculateStopaj(a, s.stopajRate);
      return {
        display: fmt(r.netAmount),
        detail: `%${s.stopajRate} Stopaj`,
        rows: [
          { label: "Net Ödenen", value: fmt(r.netAmount), accent: true },
          { label: `%${s.stopajRate} Kesinti`, value: fmt(r.taxAmount) },
          { label: "Brüt (girdiğiniz)", value: fmt(r.total) },
        ],
      };
    }
    case "tevkifat": {
      const r = calculateTevkifat(a, s.kdvRate, s.tevkifatPay);
      return {
        display: fmt(r.saticiyaOdenen),
        detail: `Tevkifat ${s.tevkifatPay}/10 — %${s.kdvRate} KDV`,
        rows: [
          { label: "Satıcıya Ödenen", value: fmt(r.saticiyaOdenen), accent: true },
          { label: "Matrah", value: fmt(r.matrah) },
          { label: `%${s.kdvRate} KDV`, value: fmt(r.kdv) },
          { label: `Tevkifat (${s.tevkifatPay}/10)`, value: fmt(r.tevkifat) },
          { label: "2 No'lu Beyan", value: fmt(r.beyan) },
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
  }
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
  tevkifatPay: 5,
  kdvExtract: false,
  customKdvRate: "",
  useCustomRate: false,
  result: null,
  history: (persisted?.history ?? []) as HistoryEntry[],
  discountInputs: ["", ""],
  activeDiscountIndex: 0,
  compoundYears: "1",
  compoundFrequency: 12,
  showSettings: false,
  showHelp: false,
  showWidget: false,
  showInvoice: false,
  theme: (persisted?.theme as Theme) ?? "dark",
  settings: persisted?.settings ?? { showCurrencySymbol: true, precision: 2 },
  notification: null,

  setMode: (mode) => set({ mode, input: "", secondInput: "", result: null, activeField: "first", discountInputs: ["", ""], activeDiscountIndex: 0 }),
  setInputDirect: (v) => {
    const s = get();
    const clean = v.replace(/[^0-9.,]/g, "").replace(/,/g, ".");
    set({ input: clean, result: compute({ ...s, input: clean }) });
  },
  setSecondInputDirect: (v) => {
    const s = get();
    const clean = v.replace(/[^0-9.,]/g, "").replace(/,/g, ".");
    set({ secondInput: clean, result: compute({ ...s, secondInput: clean }) });
  },
  setKdvRate: (kdvRate) => { set({ kdvRate, useCustomRate: false }); const s = get(); if (s.input || s.mode === "discount") set({ result: compute({ ...s, kdvRate, useCustomRate: false }) }); },
  setStopajRate: (stopajRate) => { set({ stopajRate }); const s = get(); if (s.input) set({ result: compute({ ...s, stopajRate }) }); },
  setTevkifatPay: (tevkifatPay) => { set({ tevkifatPay }); const s = get(); if (s.input) set({ result: compute({ ...s, tevkifatPay }) }); },
  setKdvExtract: (kdvExtract) => { set({ kdvExtract }); const s = get(); if (s.input) set({ result: compute({ ...s, kdvExtract }) }); },
  setCustomKdvRate: (customKdvRate) => {
    const clean = customKdvRate.replace(/[^0-9.,]/g, "").replace(/,/g, ".");
    set({ customKdvRate: clean, useCustomRate: clean !== "" });
    const s = get();
    if (s.input) set({ result: compute({ ...s, customKdvRate: clean, useCustomRate: clean !== "" }) });
  },
  setUseCustomRate: (useCustomRate) => { set({ useCustomRate }); const s = get(); if (s.input) set({ result: compute({ ...s, useCustomRate }) }); },

  appendDigit: (digit) => {
    const s = get();
    if (s.mode === "discount") {
      if (s.activeField === "first") {
        const v = (s.input + digit).slice(0, 15);
        if ((v.match(/\./g) || []).length > 1) return;
        set({ input: v, result: compute({ ...s, input: v }) });
        return;
      }
      const d = [...s.discountInputs];
      const cur = (d[s.activeDiscountIndex] ?? "") + digit;
      if ((cur.match(/\./g) || []).length > 1) return;
      d[s.activeDiscountIndex] = cur.slice(0, 7);
      set({ discountInputs: d, result: compute({ ...s, discountInputs: d }) });
      return;
    }
    const useSecond = TWO_INPUT_MODES.includes(s.mode) && s.activeField === "second";
    if (useSecond) {
      const v = (s.secondInput + digit).slice(0, 15);
      if ((v.match(/\./g) || []).length > 1) return;
      set({ secondInput: v, result: compute({ ...s, secondInput: v }) });
    } else {
      const v = (s.input + digit).slice(0, 15);
      if ((v.match(/\./g) || []).length > 1) return;
      set({ input: v, result: compute({ ...s, input: v }) });
    }
  },

  deleteLast: () => {
    const s = get();
    if (s.mode === "discount" && s.activeField !== "first") {
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

  clearAll: () => set({ input: "", secondInput: "", result: null, activeField: "first", discountInputs: ["", ""], activeDiscountIndex: 0 }),

  switchField: () => {
    const s = get();
    if (s.mode === "discount") {
      set({ activeField: s.activeField === "first" ? "second" : "first" });
      return;
    }
    if (!TWO_INPUT_MODES.includes(s.mode)) return;
    set({ activeField: s.activeField === "first" ? "second" : "first" });
  },

  confirmInput: () => {
    const s = get();
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
  toggleWidget: () => set((s) => ({ showWidget: !s.showWidget })),
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
      "Tarih,Mod,Girdiler,Sonuç,Açıklama",
      ...history.map((e) => `"${new Date(e.timestamp).toLocaleString("tr-TR")}","${MODE_LABELS[e.mode]}","${e.inputs}","${e.result}","${e.detail || ""}"`),
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
