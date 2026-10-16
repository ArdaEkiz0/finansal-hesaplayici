/**
 * GİB e-fatura / e-arşiv portalı ve Excel'den çıkan CSV'leri toplu moda çevirir.
 * Virgül, noktalı virgül, tab ve | ayraçları; tırnaklı alanlar destekli.
 * Dosyalar yalnızca yerelde okunur (offline).
 */

export interface EfaturaParse {
  /** toplu moda verilecek satırlar (matrah, nokta ondalık) */
  rows: string[];
  count: number;
  skipped: number;
  matrahSutun: string | null;
}

const MATRAH_IPUCU = /(matrah|ara\s*toplam|aratoplam|mal[\/\s]*(hizmet|ve\s*hizmet))/i;
const KDV_IPUCU = /kdv/i;

function ayracSec(text: string): string {
  const ilk = text.split(/\r?\n/).filter((l) => l.trim()).slice(0, 5).join("\n");
  const adaylar = [";", ",", "\t", "|"];
  let iyi = ",";
  let sayi = 0;
  for (const a of adaylar) {
    const n = (ilk.match(new RegExp(`\\${a}`, "g")) ?? []).length;
    if (n > sayi) {
      sayi = n;
      iyi = a;
    }
  }
  return iyi;
}

function hucreleriBol(line: string, ayrac: string): string[] {
  const out: string[] = [];
  let cur = "";
  let quoted = false;
  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (ch === '"') {
      if (quoted && line[i + 1] === '"') {
        cur += '"';
        i++;
      } else {
        quoted = !quoted;
      }
    } else if (ch === ayrac && !quoted) {
      out.push(cur);
      cur = "";
    } else {
      cur += ch;
    }
  }
  out.push(cur);
  return out.map((c) => c.trim());
}

/** "1.234,56" | "1234.56" | "1,234.56" -> "1234.56" */
function sayiTemizle(raw: string): string | null {
  let t = raw.replace(/[^\d.,\-]/g, "").trim();
  if (!t) return null;
  if (t.includes(",") && t.includes(".")) {
    t = t.replace(/\./g, "").replace(",", ".");
  } else if (t.includes(",")) {
    t = t.replace(",", ".");
  }
  if (!t.includes(",")) {
    t = t.replace(/\.(\d{3})(?=[^\d]|$)/g, "$1");
  }
  if (!/^-?\d+(\.\d+)?$/.test(t)) return null;
  return t;
}

export function parseEfaturaCsv(text: string, kaynak = ""): EfaturaParse {
  const temiz = text.replace(/^\uFEFF/, "");
  const ayrac = ayracSec(temiz);
  const satirlar = temiz.split(/\r?\n/).filter((l) => l.trim() !== "");
  if (satirlar.length === 0) return { rows: [], count: 0, skipped: 0, matrahSutun: null };

  const hucre0 = hucreleriBol(satirlar[0], ayrac);
  const matrahIdx = hucre0.findIndex((h) => MATRAH_IPUCU.test(h) && !KDV_IPUCU.test(h));
  const headerVar = matrahIdx >= 0;

  const rows: string[] = [];
  let skipped = 0;
  const ilk = headerVar ? 1 : 0;

  for (let i = ilk; i < satirlar.length; i++) {
    const hucreler = hucreleriBol(satirlar[i], ayrac);
    if (hucreler.length === 1 && headerVar === false) {
      const n = sayiTemizle(hucreler[0]);
      if (n) rows.push(n);
      else skipped++;
      continue;
    }
    const aday = headerVar ? hucreler[matrahIdx] : null;
    let matrahRaw: string | null = null;
    if (headerVar) {
      matrahRaw = sayiTemizle(aday ?? "");
    } else {
      const sayisal = hucreler.map((c) => sayiTemizle(c)).filter((n): n is string => n !== null && parseFloat(n) !== 0);
      matrahRaw = sayisal.length > 1
        ? sayisal.reduce((a, b) => (b.replace(/\D/g, "").length > a.replace(/\D/g, "").length ? b : a))
        : sayisal[0] ?? null;
    }
    if (matrahRaw && parseFloat(matrahRaw) !== 0) {
      rows.push(matrahRaw);
    } else {
      skipped++;
    }
  }

  return {
    rows,
    count: rows.length,
    skipped,
    matrahSutun: headerVar ? hucre0[matrahIdx] : null,
  };
}
