/** Statik vergi takvimi — masaüstü offline. Kural: özel duyuru yoksa standart VUK süreleri. */

export type TakvimTip = "kdv" | "mpd" | "dV" | "babs" | "gVV";

export interface TakvimKayit {
  id: string;
  ad: string;
  tip: TakvimTip;
  /** ISO "2026-05-28" */
  tarih: string;
  dönemi: string;
  açıklama: string;
}

export const TAKVIM_TIPLERI: Record<TakvimTip, { ad: string; renk: string }> = {
  kdv: { ad: "KDV", renk: "var(--color-primary-light)" },
  mpd: { ad: "Stopaj/Prim", renk: "var(--color-error)" },
  dV: { ad: "Damga", renk: "var(--color-accent)" },
  babs: { ad: "Ba-Bs", renk: "var(--color-success)" },
  gVV: { ad: "Geçici Vergi", renk: "var(--color-warning)" },
};

function iso(y: number, m1: number, gun: number): string {
  return `${y}-${String(m1).padStart(2, "0")}-${String(gun).padStart(2, "0")}`;
}

/** Bir yılın tüm son gün kayıtları: her ay için takip eden dönemin beyanları. */
function kelimeleriOluştur(y: number): TakvimKayit[] {
  const out: TakvimKayit[] = [];
  const donemAd = (m1: number, yy = y) => `${AY_KISA[m1]} ${yy}`;
  for (let m1 = 1; m1 <= 12; m1++) {
    // Önceki dönem: m1-1 (Ocak → Aralık/y-1)
    const pm = m1 - 1;
    const py = pm === 0 ? y - 1 : y;
    const pmAd = donemAd(pm === 0 ? 12 : pm, py);

    // KDV beyannamesi: takip eden ayın 28'i (beyan + ödeme)
    out.push({ id: `kdv-${y}-${m1}`, ad: "KDV Beyannamesi", tip: "kdv",
      tarih: iso(y, m1, 28), dönemi: pmAd,
      açıklama: `${pmAd} KDV beyannamesi verilir ve KDV ödemesi yapılır` });

    // Muhtasar & Prim Hizmet Beyannamesi: takip eden ayın 26'sı
    out.push({ id: `mpd-${y}-${m1}`, ad: "Muhtasar & Prim Hizmet", tip: "mpd",
      tarih: iso(y, m1, 26), dönemi: pmAd,
      açıklama: `${pmAd} gelir tevkifatı ve SGK prim beyannamesi` });

    // Damga Vergisi beyanı: takip eden ayın 26'sı
    out.push({ id: `dv-${y}-${m1}`, ad: "Damga Vergisi", tip: "dV",
      tarih: iso(y, m1, 26), dönemi: pmAd,
      açıklama: `${pmAd} damga vergisi beyannamesi` });

    // Ba-Bs: dönem sonunu izleyen ayın son günü
    const son = new Date(y, m1, 0).getDate();
    out.push({ id: `babs-${y}-${m1}`, ad: "Ba-Bs Formları", tip: "babs",
      tarih: iso(y, m1, son), dönemi: pmAd,
      açıklama: `${pmAd} Ba/Bs formu bilgileri` });

    // Geçici vergi: 3 aylık dönem sonu Mart/Haziran/Eylül/Aralık → beyan: 2 ay sonranın 15'i
    if (pm === 3 || pm === 6 || pm === 9 || pm === 12) {
      const beyanAy = m1 + 1 <= 12 ? m1 + 1 : 1;
      const beyanYil = beyanAy === 1 ? y + 1 : y;
      const ceyrekAd: Record<number, string> = { 3: "1. çeyrek (Ocak–Mart) " + py, 6: "2. çeyrek (Nisan–Haziran) " + py, 9: "3. çeyrek (Temmuz–Eylül) " + py, 12: `4. çeyrek (Ekim–Aralık) ${py}` };
      out.push({ id: `gvv-${beyanYil}-${beyanAy}`, ad: "Geçici Vergi", tip: "gVV",
        tarih: iso(beyanYil, beyanAy, 15), dönemi: ceyrekAd[pm],
        açıklama: "Geçici vergi beyannamesi; vergi beyandan itibaren 7 gün içinde ödenir" });
    }
  }
  return out;
}

const AY_KISA = ["Aralık", "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"];

/** Bugünden sonrası: sıralı yaklaşan takvim (gelecek ~4 ay). */
export function takvimListesi(bugun: Date): TakvimKayit[] {
  const sablon = [...kelimeleriOluştur(bugun.getFullYear()), ...kelimeleriOluştur(bugun.getFullYear() + 1)];
  const bugunKey = `${bugun.getFullYear()}-${String(bugun.getMonth() + 1).padStart(2, "0")}-${String(bugun.getDate()).padStart(2, "0")}`;
  return sablon.filter((k) => k.tarih >= bugunKey).sort((a, b) => a.tarih.localeCompare(b.tarih));
}

/** Bugün-tAdı difference in days (positive). */
export function gunKaldi(tarih: string, bugun: Date): number {
  const t = new Date(tarih + "T00:00:00");
  const b = new Date(bugun.getFullYear(), bugun.getMonth(), bugun.getDate());
  return Math.round((t.getTime() - b.getTime()) / 86400000);
}

export const TAKVIM_UYARI = "Son günler standart sürelerdir; resmi tatile çakışan tarihler kanunen uzatılır — GİB duyurusunu esas alın. Resmi belge değildir.";
