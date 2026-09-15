const STORAGE_KEY = "finansal-hesaplayici-v2";

export interface PersistedData {
  history: unknown[];
  settings: {
    showCurrencySymbol: boolean;
    precision: number;
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
      settings: data.settings,
      theme: data.theme,
    }));
  } catch { /* silently fail */ }
}
