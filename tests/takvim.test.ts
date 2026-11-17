import { describe, it, expect } from "vitest";
import { takvimListesi, gunKaldi } from "../src/data/takvim";

const t = (y: number, m: number, d: number) => `${y}-${String(m).padStart(2, "0")}-${String(d).padStart(2, "0")}`;

describe("vergi takvimi", () => {
  const bugun = new Date(2026, 9, 17); // 17 Ekim 2026

  it("yalnizca bugun ve sonrasi kayitlari dondurur, artan sirali", () => {
    const l = takvimListesi(bugun);
    expect(l.length).toBeGreaterThan(10);
    for (const k of l) expect(k.tarih >= "2026-10-17").toBe(true);
    for (let i = 1; i < l.length; i++) expect(l[i].tarih >= l[i - 1].tarih).toBe(true);
  });

  it("KDV beyannamesi her ayin 28'inde", () => {
    const l = takvimListesi(new Date(2026, 0, 1));
    const kdv = l.filter((k) => k.tip === "kdv" && k.tarih.startsWith("2026-"));
    expect(kdv).toHaveLength(12);
    for (const k of kdv) expect(k.tarih.endsWith("-28")).toBe(true);
  });

  it("Muhtasar ve damga ayin 26'sinda, Ba-Bs ay sonunda", () => {
    const l = takvimListesi(new Date(2026, 2, 1));
    const mpd = l.find((k) => k.tip === "mpd" && k.tarih.startsWith("2026-05"));
    expect(mpd?.tarih).toBe(t(2026, 5, 26));
    const dv = l.find((k) => k.tip === "dV" && k.tarih.startsWith("2026-05"));
    expect(dv?.tarih).toBe(t(2026, 5, 26));
    const babs = l.find((k) => k.tip === "babs" && k.tarih.startsWith("2026-05"));
    expect(babs?.tarih).toBe(t(2026, 5, 31));
  });

  it("Ba-Bs Subat ayinda 28 (2026) / 29 (2028 artik) gun", () => {
    const s26 = takvimListesi(new Date(2026, 0, 1)).find((k) => k.tip === "babs" && k.tarih.startsWith("2026-02"));
    expect(s26?.tarih).toBe(t(2026, 2, 28));
    const s28 = takvimListesi(new Date(2028, 0, 1)).find((k) => k.tip === "babs" && k.tarih.startsWith("2028-02"));
    expect(s28?.tarih).toBe(t(2028, 2, 29));
  });

  it("gecici vergi: ceyrek sonundan iki ay sonranin 15'i", () => {
    const l = takvimListesi(new Date(2026, 0, 1));
    const g1 = l.find((k) => k.tip === "gVV" && k.dönemi.startsWith("1. çeyrek"));
    expect(g1?.tarih).toBe(t(2026, 5, 15));
    const g2 = l.find((k) => k.tip === "gVV" && k.dönemi.startsWith("2. çeyrek"));
    expect(g2?.tarih).toBe(t(2026, 8, 15));
    const g3 = l.find((k) => k.tip === "gVV" && k.dönemi.startsWith("3. çeyrek"));
    expect(g3?.tarih).toBe(t(2026, 11, 15));
    const g4 = l.find((k) => k.tip === "gVV" && k.dönemi.startsWith("4. çeyrek"));
    expect(g4?.tarih).toBe(t(2026, 2, 15)); // 2025 4. çeyrek beyani
  });

  it("yil siniri: Ocak sonlari Aralik 2026 donemine isaret eder", () => {
    const l = takvimListesi(new Date(2026, 11, 1));
    const ocakKdv = l.find((k) => k.tip === "kdv" && k.tarih.startsWith("2027-01"));
    expect(ocakKdv?.dönemi).toBe("Aralık 2026");
  });

  it("kayit id'leri benzersizdir", () => {
    const l = takvimListesi(bugun);
    expect(new Set(l.map((k) => k.id)).size).toBe(l.length);
  });
});

describe("gunKaldi", () => {
  it("bugun 0, yarin 1, gecmis negatif", () => {
    const b = new Date(2026, 9, 17);
    expect(gunKaldi("2026-10-17", b)).toBe(0);
    expect(gunKaldi("2026-10-18", b)).toBe(1);
    expect(gunKaldi("2026-10-28", b)).toBe(11);
    expect(gunKaldi("2026-10-10", b)).toBe(-7);
  });

  it("ay yil atlamalarinda gun sayisi dogru", () => {
    const b = new Date(2026, 11, 30);
    expect(gunKaldi("2027-01-03", b)).toBe(4);
  });
});
