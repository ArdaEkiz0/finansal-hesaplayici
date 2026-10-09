export {};

declare global {
  interface Window {
    electronAPI?: {
      getAppVersion: () => Promise<string>;
      getAppName: () => Promise<string>;
      checkUpdate: () => Promise<any>;
      platform: string;
      minimize: () => void;
      maximize: () => void;
      close: () => void;
      isMaximized: () => Promise<boolean>;
      widgetToggle: () => void;
      widgetClose: () => void;
      requestUpdate: () => Promise<"updated" | "skipped" | "failed" | "no-window" | "none">;
      onMaximizeChange: (cb: (maximized: boolean) => void) => void;
    };
  }
}
