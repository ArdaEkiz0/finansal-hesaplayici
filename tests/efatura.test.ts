import { describe, it, expect } from "vitest";
import { parseEfaturaCsv } from "../src/data/efatura";

describe("e-fatura CSV", () => {
  it("noktalı virgüllü e-arşiv formatı: matrah sütununu bulur", () => {
    const csv = `Fatura No;Tarih;VKN;Ünvan;Matrah;Kdv Oranı;Kdv Tutarı;Toplam
ELF001;01.01.2026;1234567890;ABC LTD;1.000,00;%20;200,00;1.200,00
ELF002;02.01.2026;9876543210;XYZ AŞ;2.500,50;%20;500,10;3.000,60`;
    const p = parseEfaturaCsv(csv, "etiket.csv");
    expect(p.count).toBe(2);
    expect(p.rows).toEqual(["1000.00", "2500.50"]);
    expect(p.matrahSutun).toBe("Matrah");
    expect(p.skipped).toBe(0);
  });

  it("virgüllü İngiliz formatı", () => {
    const csv = `Number,Date,Name,Basis,VAT,VAT Amount
A1,2026-01-01,Foo,1234.56,20,246.91`;
    const p = parseEfaturaCsv(csv);
    expect(p.rows).toEqual(["1234.56"]);
  });

  it("başlıksız tek sütun", () => {
    const p = parseEfaturaCsv("1000\n2500\n3750");
    expect(p.rows).toEqual(["1000", "2500", "3750"]);
  });

  it("başlıksız çok sütunlu satırda ilk sayıyı matrah alır", () => {
    const p = parseEfaturaCsv("F001;01.01.2026;1000;200;1200\nF002;02.01.2026;2500;500;3000");
    expect(p.rows).toEqual(["1000", "2500"]);
  });

  it("geçersiz satırları atlar", () => {
    const csv = `No;Matrah;KDV
A1;abc;10
A2;500;50
A3;;0`;
    const p = parseEfaturaCsv(csv);
    expect(p.count).toBe(1);
    expect(p.rows).toEqual(["500"]);
    expect(p.skipped).toBe(2);
  });

  it("tab ayracını destekler", () => {
    const csv = "No\tMatrah\tKDV\nA1\t1200\t200";
    const p = parseEfaturaCsv(csv);
    expect(p.rows).toEqual(["1200"]);
  });

  it("boş dosya", () => {
    expect(parseEfaturaCsv("").count).toBe(0);
  });
});
