const STORAGE_KEY = "finansal-hesaplayici-v2";

export interface PersistedData {
  history: unknown[];
  donemler: unknown[];
  cariler: unknown[];
  activeCari: string;
  settings: {
    showCurrencySymbol: boolean;
    precision: number;
    fontScale?: number;
  };
  theme: string;
}

export function loadState(): Partial<PersistedData> | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === "object") {
      return {
        history: Array.isArray(parsed.history) ? parsed.history.slice(0, 100) : [],
        donemler: Array.isArray(parsed.donemler) ? parsed.donemler.slice(0, 240) : [],
        cariler: Array.isArray(parsed.cariler) ? parsed.cariler.slice(0, 500) : [],
        activeCari: typeof parsed.activeCari === "string" ? parsed.activeCari : "",
        settings: parsed.settings ?? { showCurrencySymbol: true, precision: 2 },
        theme: parsed.theme ?? "dark",
      };
    }
    return null;
  } catch {
    return null;
  }
}

export function saveState(data: PersistedData): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      history: data.history.slice(0, 100),
      donemler: data.donemler.slice(0, 240),
      cariler: data.cariler.slice(0, 500),
      activeCari: data.activeCari,
      settings: data.settings,
      theme: data.theme,
    }));
  } catch { /* silently fail */ }
}
