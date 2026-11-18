import { describe, it, expect } from "vitest";
import { csvHucre, csvSatir, csvMetin } from "../src/lib/csv";

describe("csvHucre", () => {
  it("degerleri tirnak icine alir ve gomulu tirnaklari kacirir", () => {
    expect(csvHucre("Merhaba")).toBe('"Merhaba"');
    expect(csvHucre('Bir "çift" tirnak')).toBe('"Bir ""çift"" tirnak"');
    expect(csvHucre(42)).toBe('"42"');
    expect(csvHucre(null)).toBe('""');
    expect(csvHucre(undefined)).toBe('""');
  });

  it("formul benzeri degerleri apostrofla metne cevirir (CSV enjeksiyonu)", () => {
    for (const v of ["=1+1", "+SUM(A1)", "-2+3", "@cmd", "\tveri", "\rveri"]) {
      expect(csvHucre(v).startsWith('"\'')).toBe(true);
    }
    expect(csvHucre("=1+1")).toBe('"\'=1+1"');
    expect(csvHucre("Normal -3.5")).toBe('"Normal -3.5"');
  });
});

describe("csvSatir / csvMetin", () => {
  it("satiri virgulle birlestirir", () => {
    expect(csvSatir(["a", 2, "x,y"])).toBe('"a","2","x,y"');
  });

  it("BOM ile baslar, satirlari newlines birlestirir", () => {
    expect(csvMetin([["a"], ["b"]])).toBe('\ufeff"a"\n"b"');
  });
});
