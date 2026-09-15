import { create } from "zustand";
import Decimal from "decimal.js";
import { loadState, saveState } from "../lib/persist";
import {
  toDecimal,
  formatTurkishNumber,
  calculateKDV,
  calculateStopaj,
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
} from "../core/engine";

export type CalcMode = "kdv" | "stopaj" | "margin" | "markup" | "discount" | "percent" | "compound" | "kdvCompare";

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
  rows?: { label: string; value: string; accent?: boolean }[];
}

const TWO_INPUT_MODES: CalcMode[] = ["margin", "markup", "percent", "compound"];

export type Theme = "dark" | "light";

interface State {
  input: string;
  secondInput: string;
  activeField: "first" | "second";
  mode: CalcMode;
  kdvRate: KVDRate;
  stopajRate: StopajRate;
  kdvExtract: boolean;
  result: CalcResult | null;
  history: HistoryEntry[];
  discountInputs: string[];
  activeDiscountIndex: number;
  compoundYears: string;
  compoundFrequency: number;
  showSettings: boolean;
  showHelp: boolean;
  showWidget: boolean;
  theme: Theme;
  settings: { showCurrencySymbol: boolean; precision: number };
  notification: { message: string; type: "success" | "error" | "info" } | null;

  setMode: (m: CalcMode) => void;
  setKdvRate: (r: KVDRate) => void;
  setStopajRate: (r: StopajRate) => void;
  setKdvExtract: (e: boolean) => void;
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
  setTheme: (t: Theme) => void;
  updateSettings: (p: Partial<State["settings"]>) => void;
  togglePinHistory: (id: string) => void;
  deleteHistoryEntry: (id: string) => void;
  loadHistoryEntry: (id: string) => void;
  clearHistory: () => void;
  exportHistory: () => void;
  notify: (msg: string, type?: "success" | "error" | "info") => void;
}

function compute(s: State): CalcResult | null {
  if (!s.input) return null;
  const a = toDecimal(s.input);
  const p = s.settings.precision;
  const fmt = s.settings.showCurrencySymbol
    ? (v: Decimal) => formatTurkishNumber(v, p) + " ₺"
    : (v: Decimal) => formatTurkishNumber(v, p);
  const pct = (v: Decimal) => formatTurkishNumber(v, p) + "%";

  switch (s.mode) {
    case "kdv": {
      const r = calculateKDV(a, s.kdvRate, s.kdvExtract);
      const dir = s.kdvExtract ? "Dahilden Hariç" : "Hariciden Dahil";
      return {
        display: fmt(r.netAmount),
        detail: `${dir} — ${s.kdvRate}% KDV`,
        secondary: `Vergi: ${fmt(r.taxAmount)}`,
        rows: [
          { label: "Net Tutar", value: fmt(r.netAmount), accent: true },
          { label: `${s.kdvRate}% KDV`, value: fmt(r.taxAmount) },
          { label: "Toplam", value: fmt(r.total) },
        ],
      };
    }
    case "stopaj": {
      const r = calculateStopaj(a, s.stopajRate);
      return {
        display: fmt(r.netAmount),
        detail: `${s.stopajRate}% Stopaj`,
        secondary: `Kesinti: ${fmt(r.taxAmount)}`,
        rows: [
          { label: "Net", value: fmt(r.netAmount), accent: true },
          { label: `${s.stopajRate}% Stopaj`, value: fmt(r.taxAmount) },
          { label: "Brüt", value: fmt(r.total) },
        ],
      };
    }
    case "margin": {
      if (!s.secondInput) return null;
      const r = calculateMargin(a, toDecimal(s.secondInput));
      return {
        display: fmt(r.sellingPrice),
        detail: `Satış Fiyatı — Marj: ${pct(r.marginPercent)}`,
        secondary: `Kâr: ${fmt(r.profit)}`,
        rows: [
          { label: "Satış Fiyatı", value: fmt(r.sellingPrice), accent: true },
          { label: "Maliyet", value: fmt(a) },
          { label: "Kâr", value: fmt(r.profit) },
          { label: "Gerçek Marj", value: pct(r.marginPercent) },
        ],
      };
    }
    case "markup": {
      if (!s.secondInput) return null;
      const r = calculateMarkup(a, toDecimal(s.secondInput));
      return {
        display: fmt(r.sellingPrice),
        detail: `Satış Fiyatı — Kâr Oranı: ${pct(r.markupPercent)}`,
        secondary: `Kâr: ${fmt(r.profit)}`,
        rows: [
          { label: "Satış Fiyatı", value: fmt(r.sellingPrice), accent: true },
          { label: "Maliyet", value: fmt(a) },
          { label: "Kâr", value: fmt(r.profit) },
          { label: "Marj Oranı", value: pct(r.markupPercent) },
        ],
      };
    }
    case "discount": {
      const ds = s.discountInputs.filter((d) => d !== "").map(toDecimal);
      if (ds.length === 0) return { display: fmt(a), detail: "İndirim oranlarını girin" };
      const r = calculateChainedDiscount(a, ds);
      const list = ds.map((d) => "%" + pct(d)).join(" → ");
      return {
        display: fmt(r.finalPrice),
        detail: `İndirimler: ${list}`,
        secondary: `Tasarruf: ${fmt(r.totalSaved)}`,
        third: `Etkin: ${pct(r.effectiveDiscount)}`,
        rows: [
          { label: "Son Fiyat", value: fmt(r.finalPrice), accent: true },
          { label: "Orijinal", value: fmt(a) },
          { label: "Tasarruf", value: fmt(r.totalSaved) },
          ...r.stepByStep.map((st, i) => ({
            label: `Adım ${i + 1} (%${pct(st.rate)})`,
            value: fmt(st.priceAfter),
          })),
        ],
      };
    }
    case "percent": {
      if (!s.secondInput) return null;
      const b = toDecimal(s.secondInput);
      return {
        display: fmt(calculatePercentOf(a, b)),
        detail: `%${pct(a)}'sinden ${fmt(b)}`,
        secondary: `Değişim: ${pct(calculatePercentChange(a, b))}`,
        third: `Ters: ${fmt(calculateReversePercent(a, b))}`,
        rows: [
          { label: "Sonuç", value: fmt(calculatePercentOf(a, b)), accent: true },
          { label: "Değişim", value: pct(calculatePercentChange(a, b)) },
          { label: "Ters Hesap", value: fmt(calculateReversePercent(a, b)) },
        ],
      };
    }
    case "compound": {
      if (!s.secondInput) return null;
      const years = toDecimal(s.compoundYears || "1");
      if (years.isZero() || years.isNeg()) return { display: fmt(a), detail: "Yıl sayısını ayarlayın" };
      const r = calculateCompoundInterest(a, toDecimal(s.secondInput), years, s.compoundFrequency);
      const fl: Record<number, string> = { 1: "Yıllık", 2: "6 Aylık", 4: "3 Aylık", 12: "Aylık", 365: "Günlük" };
      return {
        display: fmt(r.futureValue),
        detail: `Vade Sonu — ${fl[s.compoundFrequency] || s.compoundFrequency} bileşik`,
        secondary: `Faiz: ${fmt(r.totalInterest)}`,
        third: `Efektif: ${pct(r.effectiveRate)}`,
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
        display: fmt(r.extract.netAmount),
        detail: `KDV Karşılaştırma — %${s.kdvRate}`,
        secondary: `Fark: ${fmt(r.difference)}`,
        rows: [
          { label: "Dahilden Hariç", value: fmt(r.extract.netAmount), accent: true },
          { label: "KDV", value: fmt(r.extract.taxAmount) },
          { label: "Hariciden Dahil", value: fmt(r.inject.total) },
          { label: "Fark", value: fmt(r.difference) },
        ],
      };
    }
  }
}

const persisted = loadState();

const INIT: State = {
  input: "",
  secondInput: "",
  activeField: "first",
  mode: "kdv",
  kdvRate: 20,
  stopajRate: 15,
  kdvExtract: false,
  result: null,
  history: (persisted?.history ?? []) as HistoryEntry[],
  discountInputs: ["", ""],
  activeDiscountIndex: 0,
  compoundYears: "1",
  compoundFrequency: 12,
  showSettings: false,
  showHelp: false,
  showWidget: false,
  theme: (persisted?.theme as Theme) ?? "dark",
  settings: persisted?.settings ?? { showCurrencySymbol: true, precision: 2 },
  notification: null,
  setMode: () => {},
  setKdvRate: () => {},
  setStopajRate: () => {},
  setKdvExtract: () => {},
  appendDigit: () => {},
  deleteLast: () => {},
  clearAll: () => {},
  confirmInput: () => {},
  switchField: () => {},
  addDiscount: () => {},
  removeDiscount: () => {},
  setActiveDiscountIndex: () => {},
  setCompoundYears: () => {},
  setCompoundFrequency: () => {},
  toggleSettings: () => {},
  toggleHelp: () => {},
  toggleWidget: () => {},
  setTheme: () => {},
  updateSettings: () => {},
  togglePinHistory: () => {},
  deleteHistoryEntry: () => {},
  loadHistoryEntry: () => {},
  clearHistory: () => {},
  exportHistory: () => {},
  notify: () => {},
};

export const useStore = create<State>((set, get) => ({
  ...INIT,

  setMode: (mode) => set({ mode, input: "", secondInput: "", result: null, activeField: "first", discountInputs: ["", ""], activeDiscountIndex: 0 }),

  setKdvRate: (kdvRate) => { set({ kdvRate }); const s = get(); if (s.input) set({ result: compute({ ...s, kdvRate }) }); },
  setStopajRate: (stopajRate) => { set({ stopajRate }); const s = get(); if (s.input) set({ result: compute({ ...s, stopajRate }) }); },
  setKdvExtract: (kdvExtract) => { set({ kdvExtract }); const s = get(); if (s.input) set({ result: compute({ ...s, kdvExtract }) }); },

  appendDigit: (digit) => {
    const s = get();
    if (s.mode === "discount") {
      const d = [...s.discountInputs];
      d[s.activeDiscountIndex] = (d[s.activeDiscountIndex] ?? "") + digit;
      set({ discountInputs: d, result: compute({ ...s, discountInputs: d }) });
      return;
    }
    const useSecond = TWO_INPUT_MODES.includes(s.mode) && s.activeField === "second";
    if (useSecond) {
      const v = s.secondInput + digit;
      set({ secondInput: v, result: compute({ ...s, secondInput: v }) });
    } else {
      const v = s.input + digit;
      set({ input: v, result: compute({ ...s, input: v }) });
    }
  },

  deleteLast: () => {
    const s = get();
    if (s.mode === "discount") {
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
      set({ input: v, result: compute({ ...s, input: v }) });
    }
  },

  clearAll: () => set({ input: "", secondInput: "", result: null, activeField: "first", discountInputs: ["", ""], activeDiscountIndex: 0 }),

  switchField: () => {
    const s = get();
    if (!TWO_INPUT_MODES.includes(s.mode)) return;
    set({ activeField: s.activeField === "first" ? "second" : "first" });
  },

  confirmInput: () => {
    const s = get();
    if (s.mode === "discount") {
      if (s.result && s.input) {
        const entry: HistoryEntry = {
          id: Date.now().toString(), mode: s.mode,
          inputs: s.discountInputs.filter((d) => d !== "").map((d) => "%" + d).join(" → "),
          result: s.result.display, detail: s.result.detail, timestamp: Date.now(), pinned: false,
        };
        set({ history: [entry, ...s.history].slice(0, 100), input: "", result: null, discountInputs: ["", ""], activeDiscountIndex: 0 });
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
        return;
      }
    }
    if (s.result && s.input) {
      const entry: HistoryEntry = {
        id: Date.now().toString(), mode: s.mode, inputs: s.input,
        result: s.result.display, detail: s.result.detail, timestamp: Date.now(), pinned: false,
      };
      set({ history: [entry, ...s.history].slice(0, 100), input: "", secondInput: "", activeField: "first", result: null, discountInputs: ["", ""], activeDiscountIndex: 0 });
    }
  },

  addDiscount: () => {
    const s = get();
    const d = [...s.discountInputs, ""];
    set({ discountInputs: d, activeDiscountIndex: d.length - 1 });
  },

  removeDiscount: (index) => {
    const s = get();
    if (s.discountInputs.length <= 1) return;
    const d = s.discountInputs.filter((_, i) => i !== index);
    set({ discountInputs: d, activeDiscountIndex: Math.min(s.activeDiscountIndex, d.length - 1), result: compute({ ...s, discountInputs: d }) });
  },

  setActiveDiscountIndex: (i) => set({ activeDiscountIndex: i }),
  setCompoundYears: (y) => { set({ compoundYears: y }); const s = get(); if (s.input) set({ result: compute({ ...s, compoundYears: y }) }); },
  setCompoundFrequency: (f) => { set({ compoundFrequency: f }); const s = get(); if (s.input) set({ result: compute({ ...s, compoundFrequency: f }) }); },

  toggleSettings: () => set((s) => ({ showSettings: !s.showSettings })),
  toggleHelp: () => set((s) => ({ showHelp: !s.showHelp })),
  toggleWidget: () => set((s) => ({ showWidget: !s.showWidget })),
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
    const parts = entry.inputs.split(" / ");
    const main = parts[0]?.replace(/[^0-9.,]/g, "").replace(",", ".") ?? "";
    const second = parts[1]?.replace(/[^0-9.,]/g, "").replace(",", ".") ?? "";
    set({ mode: entry.mode, input: main, secondInput: second, activeField: "first", result: null });
    setTimeout(() => { const ns = get(); set({ result: compute({ ...ns, input: main, secondInput: second }) }); }, 0);
  },

  clearHistory: () => set({ history: [] }),

  exportHistory: () => {
    const { history } = get();
    if (history.length === 0) return;
    const ml: Record<CalcMode, string> = {
      kdv: "KDV", stopaj: "Stopaj", margin: "Marj", markup: "Kâr Oranı",
      discount: "İndirim", percent: "Yüzde", compound: "Bileşik", kdvCompare: "KDV Karşıl.",
    };
    const rows = [
      "Tarih,Mod,Girdiler,Sonuç,Açıklama",
      ...history.map((e) => `"${new Date(e.timestamp).toLocaleString("tr-TR")}","${ml[e.mode]}","${e.inputs}","${e.result}","${e.detail || ""}"`),
    ];
    const blob = new Blob(["\uFEFF" + rows.join("\n")], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `hesaplayici-${new Date().toISOString().slice(0, 10)}.csv`;
    link.click();
    URL.revokeObjectURL(url);
  },

  notify: (message, type = "info") => {
    set({ notification: { message, type } });
    setTimeout(() => set((s) => (s.notification?.message === message ? { notification: null } : s)), 3000);
  },
}));

useStore.subscribe((s) => saveState({ history: s.history, settings: s.settings, theme: s.theme }));
