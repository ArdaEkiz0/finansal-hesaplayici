import Decimal from "decimal.js";

export interface DonemRecord {
  /** "2026-03" */
  id: string;
  year: number;
  /** 1-12 */
  month: number;
  satis: string;
  alis: string;
  /** Döneme girilen önceki devreden */
  devredenGiris: string;
  hesaplanan: string;
  indirilecek: string;
  /** pozitif: ödenecek, negatif: devreden */
  net: string;
  odenecek: boolean;
  kdvRate: string;
  cari: string;
  closedAt: number;
}

export interface Cari {
  id: string;
  name: string;
  note?: string;
  createdAt: number;
}

export const AY_AD = ["", "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"];

export function donemId(year: number, month: number): string {
  return `${year}-${String(month).padStart(2, "0")}`;
}

export function donemAd(r: Pick<DonemRecord, "year" | "month">): string {
  return `${AY_AD[r.month] ?? ""} ${r.year}`;
}

function parseAmount(v: string): Decimal | null {
  const t = (v ?? "").trim();
  if (!t) return null;
  try {
    const d = new Decimal(t.replace(/,/g, "."));
    return d.isFinite() ? d : null;
  } catch {
    return null;
  }
}

export interface DonemCompute {
  hesaplanan: Decimal;
  indirilecek: Decimal;
  net: Decimal;
  odenecek: boolean;
}

/** Dönem KDV neti: hesaplanan − indirilecek − girilen devreden. */
export function computeDonemNet(satis: string, alis: string, devredenGiris: string, rate: Decimal): DonemCompute | null {
  const s = parseAmount(satis);
  const a = parseAmount(alis);
  if (!s && !a) return null;
  const d = parseAmount(devredenGiris) ?? new Decimal(0);
  const hesaplanan = (s ?? new Decimal(0)).mul(rate).div(100).toDP(2);
  const indirilecek = (a ?? new Decimal(0)).mul(rate).div(100).toDP(2);
  const net = hesaplanan.minus(indirilecek).minus(d).toDP(2);
  return { hesaplanan, indirilecek, net, odenecek: !net.isNeg() };
}

/** Son kapanan dönemden gelen devreden KDV (mutlak değer); önceki dönem ödendiyse 0. Kayıt yoksa null. */
export function kalanDevreden(donemler: DonemRecord[]): Decimal | null {
  const son = donemler[0];
  if (!son) return null;
  if (son.odenecek) return new Decimal(0);
  const n = parseAmount(son.net);
  return n ? n.abs() : null;
}

export function sortByDonemId(list: DonemRecord[]): DonemRecord[] {
  return [...list].sort((a, b) => b.id.localeCompare(a.id));
}

/** Kayıtları cariye göre süzüp toplamları çıkar. */
export function donemOzet(list: DonemRecord[]): { hesaplanan: Decimal; indirilecek: Decimal; net: Decimal; odenecekNet: Decimal; devredenNet: Decimal; sayi: number } {
  let hesaplanan = new Decimal(0);
  let indirilecek = new Decimal(0);
  let net = new Decimal(0);
  let odenecekNet = new Decimal(0);
  let devredenNet = new Decimal(0);
  for (const r of list) {
    const h = parseAmount(r.hesaplanan) ?? new Decimal(0);
    const i = parseAmount(r.indirilecek) ?? new Decimal(0);
    const n = parseAmount(r.net) ?? new Decimal(0);
    hesaplanan = hesaplanan.plus(h);
    indirilecek = indirilecek.plus(i);
    net = net.plus(n);
    if (r.odenecek) odenecekNet = odenecekNet.plus(n);
    else devredenNet = devredenNet.plus(n.abs());
  }
  return { hesaplanan, indirilecek, net, odenecekNet, devredenNet, sayi: list.length };
}
