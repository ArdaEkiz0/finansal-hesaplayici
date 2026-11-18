/** Paylasilan CSV uretimi — enjeksiyon korumali hucreler, BOM ve tarayici indirmesi. */

/** Excel/Sheets'te formül olarak calisabilecek degerleri metne cevirir, tirnaklar kacirilir. */
export function csvHucre(v: unknown): string {
  let s = v == null ? "" : String(v);
  if (/^[=+\-@\t\r]/.test(s)) s = "'" + s;
  return `"${s.replace(/"/g, '""')}"`;
}

export function csvSatir(cells: unknown[]): string {
  return cells.map(csvHucre).join(",");
}

/** Satirlari CSV metnine cevirir; BOM Excel'de Türkçe karakterler icin sart. */
export function csvMetin(rows: unknown[][]): string {
  return "\ufeff" + rows.map(csvSatir).join("\n");
}

/** CSV metnini tarayici uzerinden dosya olarak indirir. */
export function csvIndir(dosya: string, rows: unknown[][]): void {
  const blob = new Blob([csvMetin(rows)], { type: "text/csv;charset=utf-8;" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = dosya;
  link.click();
  URL.revokeObjectURL(url);
}
